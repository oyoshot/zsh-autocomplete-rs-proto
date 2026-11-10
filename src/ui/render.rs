use crate::app::App;
use crate::candidate::Candidate;
use crate::fuzzy::FuzzyMatcher;
use crate::ui::popup::Popup;
use crate::ui::theme::Theme;
use crossterm::cursor;
use crossterm::style::{
    Attribute, Color, Print, ResetColor, SetAttribute, SetBackgroundColor, SetForegroundColor,
};
use crossterm::terminal;
use std::io::Write;
use unicode_width::UnicodeWidthStr;

pub struct CandidateLayout {
    pub text: String,
    pub gap: usize,
    pub description: String,
}

#[inline]
pub fn layout_candidate(candidate: &Candidate, inner: usize) -> CandidateLayout {
    let text = truncate_to_width(&candidate.text, inner);
    let text_w = UnicodeWidthStr::width(text.as_str());

    if candidate.description.is_empty() || text_w + 2 >= inner {
        return CandidateLayout {
            text,
            gap: inner.saturating_sub(text_w),
            description: String::new(),
        };
    }

    let desc_max = inner - text_w - 2;
    let desc = truncate_to_width(&candidate.description, desc_max);
    let desc_w = UnicodeWidthStr::width(desc.as_str());
    let gap = inner - text_w - desc_w;

    CandidateLayout {
        text,
        gap,
        description: desc,
    }
}

/// Scroll the terminal to ensure enough blank space below the cursor for the popup.
/// Updates `app.cursor_row` and `app.max_visible` to reflect the new state.
pub fn ensure_space(tty: &mut std::fs::File, app: &mut App) -> std::io::Result<()> {
    let term_rows = app.term_rows;
    app.sync_max_visible();

    let popup_height = app.max_visible as u16 + 2;
    let space_below = term_rows.saturating_sub(app.cursor_row + 1);

    if space_below < popup_height {
        let scroll_amount = (popup_height - space_below).min(app.cursor_row);

        if scroll_amount > 0 {
            // Scroll terminal content up without moving cursor position
            crossterm::execute!(tty, terminal::ScrollUp(scroll_amount))?;
            app.cursor_row -= scroll_amount;
        }
    }

    Ok(())
}

#[inline]
fn print_colored(
    buf: &mut impl Write,
    text: impl std::fmt::Display,
    color: Option<Color>,
) -> std::io::Result<()> {
    if let Some(c) = color {
        crossterm::queue!(buf, SetForegroundColor(c), Print(text), ResetColor)
    } else {
        crossterm::queue!(buf, Print(text))
    }
}

fn print_candidate(
    buf: &mut impl Write,
    text: &str,
    original: &str,
    ranges: &[std::ops::Range<usize>],
    theme: &Theme,
    selected: bool,
) -> std::io::Result<()> {
    let base = theme.candidate.unwrap_or(Color::Reset);
    if !selected {
        crossterm::queue!(buf, SetForegroundColor(base))?;
    }
    let original_end = if text == original {
        text.len()
    } else {
        text.strip_suffix('…').unwrap_or(text).len()
    };
    let mut offset = 0;
    for range in ranges.iter().filter(|r| r.end <= original_end) {
        crossterm::queue!(buf, Print(&text[offset..range.start]))?;
        if !selected {
            crossterm::queue!(buf, SetForegroundColor(theme.match_fg))?;
        }
        crossterm::queue!(
            buf,
            // NO_COLOR makes crossterm's color commands reset all attributes.
            // Set bold after the foreground so non-color emphasis survives.
            SetAttribute(Attribute::Bold),
            Print(&text[range.clone()]),
            SetAttribute(Attribute::NormalIntensity)
        )?;
        if !selected {
            crossterm::queue!(buf, SetForegroundColor(base))?;
        }
        offset = range.end;
    }
    crossterm::queue!(buf, Print(&text[offset..]))?;
    if !selected {
        crossterm::queue!(buf, ResetColor)?;
    }
    Ok(())
}

fn render_popup(buf: &mut impl Write, app: &App, theme: &Theme) -> std::io::Result<Popup> {
    let popup = Popup::compute(app);
    let inner = popup.width.saturating_sub(2) as usize;

    // Pre-compute padding strings (reused via slicing, avoids per-row allocations)
    let spaces = " ".repeat(inner);
    let dashes = "─".repeat(inner);
    let dash_byte_len = '─'.len_utf8(); // 3

    crossterm::queue!(buf, cursor::Hide)?;

    // Top border with filter text
    let filter_label = truncate_to_width(&format!(" {} ", app.filter_text), inner);
    let filter_w = UnicodeWidthStr::width(filter_label.as_str());
    let remaining = inner.saturating_sub(filter_w);

    crossterm::queue!(buf, cursor::MoveTo(popup.col, popup.row))?;
    print_colored(buf, "┌", theme.border)?;
    print_colored(buf, &filter_label, theme.filter)?;
    print_colored(
        buf,
        format_args!("{}┐", &dashes[..remaining * dash_byte_len]),
        theme.border,
    )?;
    crossterm::queue!(buf, terminal::Clear(terminal::ClearType::UntilNewLine))?;

    // Candidate rows
    let visible = app.visible_candidate_indices();
    let highlight_idx = app.visible_selected_index();
    let padding = usize::from(inner >= 2);
    let gutter = if inner >= 4 { 2 } else { padding };
    let content_width = inner.saturating_sub(gutter + padding);
    let mut matcher = FuzzyMatcher::new();
    for (i, &candidate_idx) in visible.iter().enumerate() {
        let candidate = &app.all_candidates[candidate_idx];
        let layout = layout_candidate(candidate, content_width);
        let ranges = matcher.highlight_ranges(candidate, &app.filter_text, app.match_mode());

        crossterm::queue!(buf, cursor::MoveTo(popup.col, popup.row + 1 + i as u16))?;

        if Some(i) == highlight_idx {
            print_colored(buf, "│", theme.border)?;
            if gutter == 2 {
                print_colored(buf, "▸ ", Some(theme.selected_marker))?;
            } else {
                crossterm::queue!(buf, Print(&spaces[..gutter]))?;
            }
            let use_explicit = theme.selected_fg.is_some() || theme.selected_bg.is_some();
            if use_explicit {
                if let Some(c) = theme.selected_fg {
                    crossterm::queue!(buf, SetForegroundColor(c))?;
                }
                if let Some(c) = theme.selected_bg {
                    crossterm::queue!(buf, SetBackgroundColor(c))?;
                }
            } else {
                crossterm::queue!(buf, SetAttribute(Attribute::Reverse))?;
            }
            print_candidate(buf, &layout.text, &candidate.text, &ranges, theme, true)?;
            crossterm::queue!(
                buf,
                Print(&spaces[..layout.gap]),
                Print(&layout.description),
                Print(&spaces[..padding]),
            )?;
            if use_explicit {
                crossterm::queue!(buf, ResetColor)?;
            } else {
                crossterm::queue!(buf, SetAttribute(Attribute::NoReverse), ResetColor)?;
            }
            print_colored(buf, "│", theme.border)?;
        } else {
            print_colored(buf, "│", theme.border)?;
            crossterm::queue!(buf, Print(&spaces[..gutter]))?;
            print_candidate(buf, &layout.text, &candidate.text, &ranges, theme, false)?;

            if !layout.description.is_empty() {
                crossterm::queue!(
                    buf,
                    Print(&spaces[..layout.gap]),
                    SetForegroundColor(theme.description),
                    Print(&layout.description),
                    ResetColor,
                )?;
            } else {
                crossterm::queue!(buf, Print(&spaces[..layout.gap]))?;
            }

            crossterm::queue!(buf, Print(&spaces[..padding]))?;
            print_colored(buf, "│", theme.border)?;
        }
        crossterm::queue!(buf, terminal::Clear(terminal::ClearType::UntilNewLine))?;
    }

    // Bottom border
    crossterm::queue!(
        buf,
        cursor::MoveTo(popup.col, popup.row + 1 + visible.len() as u16),
    )?;
    let range_label = Popup::range_label(app);
    let range_width = UnicodeWidthStr::width(range_label.as_str());
    let range_label = if range_width <= inner {
        range_label.as_str()
    } else {
        ""
    };
    let remaining = inner - UnicodeWidthStr::width(range_label);
    print_colored(
        buf,
        format_args!("└{}", &dashes[..remaining * dash_byte_len]),
        theme.border,
    )?;
    print_colored(buf, range_label, Some(theme.description))?;
    print_colored(buf, "┘", theme.border)?;
    crossterm::queue!(buf, terminal::Clear(terminal::ClearType::UntilNewLine))?;

    Ok(popup)
}

fn queue_filter_line(buf: &mut impl Write, app: &App) -> std::io::Result<()> {
    let prefix_w = UnicodeWidthStr::width(app.prefix.as_str()) as u16;
    let prefix_start_col = app.cursor_col.saturating_sub(prefix_w);
    let filter_display = &app.filter_text;
    let filter_w = UnicodeWidthStr::width(filter_display.as_str()) as u16;

    crossterm::queue!(
        buf,
        cursor::MoveTo(prefix_start_col, app.cursor_row),
        Print(filter_display),
    )?;

    let clear_count = prefix_w.saturating_sub(filter_w);
    if clear_count > 0 {
        crossterm::queue!(buf, Print(" ".repeat(clear_count as usize)))?;
    }

    let cursor_end_col = prefix_start_col + filter_w;
    crossterm::queue!(
        buf,
        cursor::MoveTo(cursor_end_col, app.cursor_row),
        cursor::Show
    )?;

    Ok(())
}

pub fn filter_line_to_bytes(app: &App) -> std::io::Result<Vec<u8>> {
    let mut buf = Vec::with_capacity(128);
    queue_filter_line(&mut buf, app)?;
    Ok(buf)
}

pub fn draw_to_bytes(app: &App, theme: &Theme) -> std::io::Result<(Vec<u8>, Popup)> {
    let mut buf = Vec::with_capacity(2048);
    let popup = render_popup(&mut buf, app, theme)?;
    queue_filter_line(&mut buf, app)?;

    Ok((buf, popup))
}

pub fn draw(tty: &mut std::fs::File, app: &App, theme: &Theme) -> std::io::Result<()> {
    let (bytes, _) = draw_to_bytes(app, theme)?;
    tty.write_all(&bytes)?;
    tty.flush()?;
    Ok(())
}

pub fn draw_popup_only(tty: &mut std::fs::File, app: &App, theme: &Theme) -> std::io::Result<()> {
    let mut buf = std::io::BufWriter::new(&mut *tty);
    let _ = render_popup(&mut buf, app, theme)?;

    // Restore cursor to original position (zsh manages cursor)
    crossterm::queue!(
        &mut buf,
        cursor::MoveTo(app.cursor_col, app.cursor_row),
        cursor::Show,
    )?;
    buf.flush()?;

    Ok(())
}

pub fn clear_rect(
    tty: &mut std::fs::File,
    popup_row: u16,
    popup_height: u16,
    _cursor_row: u16,
) -> std::io::Result<()> {
    let mut buf = std::io::BufWriter::new(&mut *tty);
    crossterm::queue!(&mut buf, cursor::SavePosition)?;
    for i in 0..popup_height {
        crossterm::queue!(
            &mut buf,
            cursor::MoveTo(0, popup_row + i),
            terminal::Clear(terminal::ClearType::CurrentLine),
        )?;
    }

    crossterm::queue!(&mut buf, cursor::RestorePosition)?;
    buf.flush()?;

    Ok(())
}

pub fn clear_to_bytes(app: &App) -> std::io::Result<Vec<u8>> {
    let mut buf = Vec::with_capacity(512);
    let popup = Popup::compute(app);

    crossterm::queue!(&mut buf, cursor::SavePosition)?;

    for row in popup.row..popup.row + popup.height {
        crossterm::queue!(
            &mut buf,
            cursor::MoveTo(popup.col, row),
            Print(" ".repeat(popup.width as usize)),
        )?;
    }

    let prefix_w = UnicodeWidthStr::width(app.prefix.as_str()) as u16;
    let filter_w = UnicodeWidthStr::width(app.filter_text.as_str()) as u16;
    let prefix_start_col = app.cursor_col.saturating_sub(prefix_w);
    let max_w = prefix_w.max(filter_w);

    crossterm::queue!(
        &mut buf,
        cursor::MoveTo(prefix_start_col, app.cursor_row),
        Print(&app.prefix),
        Print(" ".repeat((max_w - prefix_w) as usize)),
    )?;

    crossterm::queue!(&mut buf, cursor::RestorePosition)?;

    Ok(buf)
}

pub fn clear(tty: &mut std::fs::File, app: &App) -> std::io::Result<()> {
    let bytes = clear_to_bytes(app)?;
    tty.write_all(&bytes)?;
    tty.flush()?;
    Ok(())
}

pub fn render_popup_to_bytes(app: &App, theme: &Theme) -> std::io::Result<(Vec<u8>, Popup)> {
    let mut buf = Vec::with_capacity(2048);
    let popup = render_popup(&mut buf, app, theme)?;
    crossterm::queue!(
        &mut buf,
        cursor::MoveTo(app.cursor_col, app.cursor_row),
        cursor::Show,
    )?;
    Ok((buf, popup))
}

pub fn clear_rect_to_bytes(
    popup_row: u16,
    popup_height: u16,
    _cursor_row: u16,
) -> std::io::Result<Vec<u8>> {
    let mut buf = Vec::with_capacity(256);
    crossterm::queue!(&mut buf, cursor::SavePosition)?;
    for i in 0..popup_height {
        crossterm::queue!(
            &mut buf,
            cursor::MoveTo(0, popup_row + i),
            terminal::Clear(terminal::ClearType::CurrentLine),
        )?;
    }
    crossterm::queue!(&mut buf, cursor::RestorePosition)?;
    Ok(buf)
}

pub fn clear_stale_rows_to_bytes(
    prev_popup_row: u16,
    prev_popup_height: u16,
    new_popup_row: u16,
    new_popup_height: u16,
) -> std::io::Result<Vec<u8>> {
    let mut buf = Vec::with_capacity(128);

    if prev_popup_height == 0 {
        return Ok(buf);
    }

    let prev_end = prev_popup_row.saturating_add(prev_popup_height);
    let new_end = new_popup_row.saturating_add(new_popup_height);
    let mut wrote_any = false;

    for row in prev_popup_row..prev_end {
        if row < new_popup_row || row >= new_end {
            if !wrote_any {
                crossterm::queue!(&mut buf, cursor::SavePosition)?;
                wrote_any = true;
            }
            crossterm::queue!(
                &mut buf,
                cursor::MoveTo(0, row),
                terminal::Clear(terminal::ClearType::CurrentLine),
            )?;
        }
    }

    if wrote_any {
        crossterm::queue!(&mut buf, cursor::RestorePosition)?;
    }

    Ok(buf)
}

#[inline]
pub fn truncate_to_width(s: &str, max_width: usize) -> String {
    // Fast path: most candidates fit without truncation
    if UnicodeWidthStr::width(s) <= max_width {
        return s.to_string();
    }
    if max_width == 0 {
        return String::new();
    }
    let mut width = 0;
    let mut result = String::with_capacity(s.len());
    for c in s.chars() {
        let cw = unicode_width::UnicodeWidthChar::width(c).unwrap_or(0);
        if width + cw > max_width - 1 {
            result.push('…');
            break;
        }
        width += cw;
        result.push(c);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::candidate::Candidate;

    // --- truncate_to_width ---

    fn printed_text(bytes: &[u8]) -> String {
        use termwiz::escape::{Action, parser::Parser};
        let mut text = String::new();
        Parser::new().parse(bytes, |action| match action {
            Action::Print(c) => text.push(c),
            Action::PrintString(s) => text.push_str(&s),
            _ => {}
        });
        text
    }

    fn emphasized_text(app: &App) -> String {
        use termwiz::{
            cell::Intensity,
            escape::{
                Action,
                csi::{CSI, Sgr},
                parser::Parser,
            },
        };
        let (bytes, _) = render_popup_to_bytes(app, &Theme::default()).unwrap();
        let mut bold = false;
        let mut text = String::new();
        Parser::new().parse(&bytes, |action| match action {
            Action::CSI(CSI::Sgr(Sgr::Reset)) => bold = false,
            Action::CSI(CSI::Sgr(Sgr::Intensity(intensity))) => bold = intensity == Intensity::Bold,
            Action::Print(c) if bold => text.push(c),
            Action::PrintString(s) if bold => text.push_str(&s),
            _ => {}
        });
        text
    }

    fn rescue_abbreviation_app(prefix: &str) -> App {
        App::new_with_term_size(
            vec![
                Candidate::abbreviation("gti-σΣ".into(), "git status".into(), String::new()),
                Candidate::parse_line("git\t\tcommand_rescue"),
            ],
            prefix.into(),
            1,
            0,
            80,
            24,
        )
    }

    #[test]
    fn unicode_popup_emphasizes_only_complete_matching_characters() {
        let app = App::new_with_term_size(
            vec![
                Candidate::parse_line("日本語入力.txt\t\tfile"),
                Candidate::parse_line("本早入.txt\t\tfile"),
            ],
            "日.t".into(),
            1,
            0,
            80,
            24,
        );
        // Keep byte-score filtering, including its byte-only match, while
        // refusing to emphasize unrelated characters in either candidate.
        assert_eq!(app.filtered_indices.len(), 2);
        assert_eq!(emphasized_text(&app), "日.t");
    }

    #[test]
    fn narrowing_and_cached_results_keep_strict_highlighting_after_rescue() {
        let mut app = rescue_abbreviation_app("gti");
        assert_eq!(app.filtered_indices.len(), 2);
        app.type_char('-');
        assert_eq!(app.filtered_indices, vec![0]);
        app.type_char('Σ');
        assert_eq!(emphasized_text(&app), "gti-Σ");

        // Restore the rescue query, then a different strict query, before
        // revisiting the cached uppercase result.
        assert!(app.backspace());
        app.type_char('σ');
        assert_eq!(emphasized_text(&app), "gti-σ");
        assert!(app.backspace());
        app.type_char('Σ');
        assert_eq!(emphasized_text(&app), "gti-Σ");

        // Backspace also restores the mode of a cached strict result.
        app.type_char('x');
        assert!(app.filtered_indices.is_empty());
        assert!(app.backspace());
        assert_eq!(emphasized_text(&app), "gti-Σ");
    }

    #[test]
    fn full_rescan_keeps_rescue_highlighting_without_a_rescued_result() {
        let mut app = rescue_abbreviation_app("gti-Σ");
        assert_eq!(app.filtered_indices, vec![0]);
        assert_eq!(emphasized_text(&app), "gti-σ");
        app.type_char('x');
        assert!(app.backspace());
        assert_eq!(emphasized_text(&app), "gti-σ");
    }

    #[test]
    fn popup_rows_fit_width_with_padding_and_truncated_unicode() {
        for cols in [4, 8, 20, 80] {
            let candidates = vec![Candidate {
                text: "日本語の長いファイル名.txt".into(),
                description: "長い説明文です".into(),
                kind: String::new(),
                insert_text: None,
                cursor_offset: None,
            }];
            let mut app = App::new_with_term_size(candidates, "日".into(), 1, 0, cols, 24);
            for selected in [false, true] {
                if selected {
                    app.select_first();
                }
                let (bytes, popup) = render_popup_to_bytes(&app, &Theme::default()).unwrap();
                let text = printed_text(&bytes);
                let row = text.split('│').nth(1).unwrap();
                assert_eq!(UnicodeWidthStr::width(row), popup.width as usize - 2);
                if selected && popup.width >= 6 {
                    assert!(row.starts_with("▸ "));
                } else {
                    assert!(row.starts_with(' '));
                }
                assert!(row.ends_with(' '));
            }
        }
    }

    #[test]
    fn popup_range_tracks_scrolling_and_filtering_without_overflow() {
        for cols in [8, 20, 80] {
            let candidates = (0..42)
                .map(|i| Candidate {
                    text: format!("item-{i:02}"),
                    description: String::new(),
                    kind: String::new(),
                    insert_text: None,
                    cursor_offset: None,
                })
                .collect();
            let mut app = App::new_with_term_size(candidates, "item-".into(), 1, 0, cols, 24);
            app.max_visible = 8;
            for (selected, label) in [(0, " 1–8 / 42 "), (41, " 35–42 / 42 ")] {
                app.set_selected(selected);
                let (bytes, popup) = render_popup_to_bytes(&app, &Theme::default()).unwrap();
                let text = printed_text(&bytes);
                let footer = text.rsplit_once('└').unwrap().1;
                assert_eq!(UnicodeWidthStr::width(footer), popup.width as usize - 1);
                assert_eq!(footer.contains(label), cols >= 20);
                assert_eq!(text.matches('▸').count(), 1);
            }
            app.filter_text = "item-00".into();
            app.update_filter();
            let (bytes, _) = render_popup_to_bytes(&app, &Theme::default()).unwrap();
            let text = printed_text(&bytes);
            assert!(!text.rsplit_once('└').unwrap().1.contains('/'));
        }
    }

    #[test]
    fn no_color_preserves_bold_matches() {
        // Crossterm caches NO_COLOR globally. Use a fresh process instead of
        // mutating the environment of other concurrently running tests.
        if std::env::var_os("ZACRS_TEST_NO_COLOR").is_none() {
            let output = std::process::Command::new(std::env::current_exe().unwrap())
                .args([
                    "--exact",
                    "ui::render::tests::no_color_preserves_bold_matches",
                    "--nocapture",
                ])
                .env("NO_COLOR", "1")
                .env("ZACRS_TEST_NO_COLOR", "1")
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{}\n{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            return;
        }
        use termwiz::{
            cell::Intensity,
            escape::{
                Action,
                csi::{CSI, Sgr},
                parser::Parser,
            },
        };
        let mut buf = Vec::new();
        print_candidate(
            &mut buf,
            "git",
            "git",
            std::slice::from_ref(&(0..1)),
            &Theme::default(),
            false,
        )
        .unwrap();
        let mut bold = false;
        let mut printed = Vec::new();
        Parser::new().parse(&buf, |action| match action {
            Action::CSI(CSI::Sgr(Sgr::Reset)) => bold = false,
            Action::CSI(CSI::Sgr(Sgr::Intensity(intensity))) => bold = intensity == Intensity::Bold,
            Action::Print(c) => printed.push((c, bold)),
            _ => {}
        });
        assert_eq!(printed, vec![('g', true), ('i', false), ('t', false)]);
    }

    #[test]
    fn selected_matches_keep_selection_colors_and_reset_bold() {
        let mut buf = Vec::new();
        print_candidate(
            &mut buf,
            "git",
            "git",
            std::slice::from_ref(&(0..1)),
            &Theme::default(),
            true,
        )
        .unwrap();
        assert_eq!(printed_text(&buf), "git");
        assert_eq!(String::from_utf8(buf).unwrap(), "\x1b[1mg\x1b[22mit");
    }

    #[test]
    fn ascii_within_limit() {
        assert_eq!(truncate_to_width("hello", 10), "hello");
    }

    #[test]
    fn ascii_exact_width() {
        assert_eq!(truncate_to_width("hello", 5), "hello");
    }

    #[test]
    fn ascii_exceeds() {
        assert_eq!(truncate_to_width("hello world", 5), "hell…");
    }

    #[test]
    fn empty_string() {
        assert_eq!(truncate_to_width("", 5), "");
    }

    #[test]
    fn zero_width() {
        assert_eq!(truncate_to_width("hello", 0), "");
    }

    #[test]
    fn cjk_boundary() {
        // CJK chars are width 2; "あいう" with max_width=3 → "あ" fits (w=2), "い" would be 4 > 3
        assert_eq!(truncate_to_width("あいう", 3), "あ…");
    }

    // --- layout_candidate ---

    #[test]
    fn layout_no_description() {
        let c = Candidate {
            text: "git".to_string(),
            description: String::new(),
            kind: String::new(),
            insert_text: None,
            cursor_offset: None,
        };
        let layout = layout_candidate(&c, 20);
        assert_eq!(layout.text, "git");
        assert_eq!(layout.gap, 17);
        assert!(layout.description.is_empty());
    }

    #[test]
    fn layout_with_description() {
        let c = Candidate {
            text: "git".to_string(),
            description: "command".to_string(),
            kind: String::new(),
            insert_text: None,
            cursor_offset: None,
        };
        let layout = layout_candidate(&c, 20);
        assert_eq!(layout.text, "git");
        assert_eq!(layout.description, "command");
        let text_w = UnicodeWidthStr::width(layout.text.as_str());
        let desc_w = UnicodeWidthStr::width(layout.description.as_str());
        assert_eq!(text_w + layout.gap + desc_w, 20);
    }

    #[test]
    fn draw_to_bytes_produces_output() {
        let candidates = vec![
            Candidate {
                text: "git".to_string(),
                description: "command".to_string(),
                kind: "command".to_string(),
                insert_text: None,
                cursor_offset: None,
            },
            Candidate {
                text: "grep".to_string(),
                description: "command".to_string(),
                kind: "command".to_string(),
                insert_text: None,
                cursor_offset: None,
            },
        ];
        let app = App::new_with_term_size(candidates, "g".to_string(), 5, 2, 80, 24);

        let (bytes, popup) = draw_to_bytes(&app, &Theme::default()).unwrap();

        assert!(!bytes.is_empty());
        assert!(popup.height > 0);
        assert!(popup.width > 0);
    }

    #[test]
    fn clear_to_bytes_produces_output() {
        let candidates = vec![Candidate {
            text: "git".to_string(),
            description: String::new(),
            kind: String::new(),
            insert_text: None,
            cursor_offset: None,
        }];
        let app = App::new_with_term_size(candidates, "g".to_string(), 5, 2, 80, 24);

        let bytes = clear_to_bytes(&app).unwrap();

        assert!(!bytes.is_empty());
    }

    #[test]
    fn render_popup_to_bytes_handles_zero_sized_terminal_input() {
        let candidates = vec![Candidate {
            text: "git".to_string(),
            description: String::new(),
            kind: String::new(),
            insert_text: None,
            cursor_offset: None,
        }];
        let app = App::new_with_term_size(candidates, "".to_string(), 4, 8, 0, 0);

        let (bytes, popup) = render_popup_to_bytes(&app, &Theme::default()).unwrap();

        assert!(!bytes.is_empty());
        assert_eq!(popup.width, 1);
        assert_eq!(popup.row, 0);
        assert_eq!(popup.col, 0);
    }

    #[test]
    fn clear_stale_rows_to_bytes_is_empty_when_ranges_match() {
        let bytes = clear_stale_rows_to_bytes(6, 4, 6, 4).unwrap();
        assert!(bytes.is_empty());
    }

    #[test]
    fn clear_stale_rows_to_bytes_clears_non_overlapping_rows() {
        let bytes = clear_stale_rows_to_bytes(6, 4, 7, 2).unwrap();
        let ansi = String::from_utf8_lossy(&bytes);
        assert!(ansi.contains("\u{1b}7") || ansi.contains("\u{1b}[s"));
        assert!(ansi.contains("[7;1H"));
        assert!(ansi.contains("[10;1H"));
    }

    #[test]
    fn clear_rect_to_bytes_restores_cursor_position() {
        let bytes = clear_rect_to_bytes(6, 4, 5).unwrap();
        let ansi = String::from_utf8_lossy(&bytes);
        assert!(ansi.contains("\u{1b}7") || ansi.contains("\u{1b}[s"));
        assert!(ansi.contains("\u{1b}8") || ansi.contains("\u{1b}[u"));
    }
}
