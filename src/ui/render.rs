use crate::app::App;
use crate::candidate::Candidate;
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

fn render_popup(buf: &mut impl Write, app: &App, theme: &Theme) -> std::io::Result<Popup> {
    let popup = Popup::compute(app);
    let inner = popup.width.saturating_sub(2) as usize;

    // Pre-compute padding strings (reused via slicing, avoids per-row allocations)
    let spaces = " ".repeat(inner);
    let dashes = "─".repeat(inner);
    let dash_byte_len = '─'.len_utf8(); // 3

    crossterm::queue!(buf, cursor::Hide)?;

    // Top border with filter text
    let filter_label = format!(" {} ", &app.filter_text);
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

    for (i, &candidate_idx) in visible.iter().enumerate() {
        let candidate = &app.all_candidates[candidate_idx];
        let layout = layout_candidate(candidate, inner);

        crossterm::queue!(buf, cursor::MoveTo(popup.col, popup.row + 1 + i as u16))?;

        if Some(i) == highlight_idx {
            print_colored(buf, "│", theme.border)?;
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
            crossterm::queue!(
                buf,
                Print(&layout.text),
                Print(&spaces[..layout.gap]),
                Print(&layout.description),
            )?;
            if use_explicit {
                crossterm::queue!(buf, ResetColor)?;
            } else {
                crossterm::queue!(buf, SetAttribute(Attribute::NoReverse), ResetColor)?;
            }
            print_colored(buf, "│", theme.border)?;
        } else {
            print_colored(buf, "│", theme.border)?;
            print_colored(buf, &layout.text, theme.candidate)?;

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

            print_colored(buf, "│", theme.border)?;
        }
        crossterm::queue!(buf, terminal::Clear(terminal::ClearType::UntilNewLine))?;
    }

    // Bottom border
    crossterm::queue!(
        buf,
        cursor::MoveTo(popup.col, popup.row + 1 + visible.len() as u16),
    )?;
    print_colored(
        buf,
        format_args!("└{}┘", &dashes[..inner * dash_byte_len]),
        theme.border,
    )?;
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
    let mut width = 0;
    let mut result = String::with_capacity(s.len());
    for c in s.chars() {
        let cw = unicode_width::UnicodeWidthChar::width(c).unwrap_or(0);
        if width + cw > max_width {
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
        assert_eq!(truncate_to_width("hello world", 5), "hello…");
    }

    #[test]
    fn empty_string() {
        assert_eq!(truncate_to_width("", 5), "");
    }

    #[test]
    fn zero_width() {
        assert_eq!(truncate_to_width("hello", 0), "…");
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
            },
            Candidate {
                text: "grep".to_string(),
                description: "command".to_string(),
                kind: "command".to_string(),
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
