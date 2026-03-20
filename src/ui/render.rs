use crate::app::App;
use crate::candidate::Candidate;
use crate::ui::popup::Popup;
use crate::ui::theme::Theme;
use crossterm::cursor;
use crossterm::style::{
    Attribute, Print, ResetColor, SetAttribute, SetBackgroundColor, SetForegroundColor,
};
use crossterm::terminal;
use std::io::Write;
use unicode_width::UnicodeWidthStr;

struct CandidateLayout {
    text: String,
    gap: usize,
    description: String,
}

fn layout_candidate(candidate: &Candidate, inner: usize) -> CandidateLayout {
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
    let (_, term_rows) = terminal::size().unwrap_or((80, 24));

    // Cap max_visible if terminal is too short
    let max_popup_height = term_rows.saturating_sub(1); // 1 row for prompt
    if app.max_visible as u16 + 2 > max_popup_height {
        app.max_visible = max_popup_height.saturating_sub(2).max(1) as usize;
    }

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

pub fn draw(tty: &mut std::fs::File, app: &App, theme: &Theme) -> std::io::Result<()> {
    let mut buf = std::io::BufWriter::new(&mut *tty);
    let popup = Popup::compute(app);
    let inner = (popup.width - 2) as usize;

    crossterm::queue!(&mut buf, cursor::Hide)?;

    // Top border with filter text
    let filter_label = format!(" {} ", &app.filter_text);
    let filter_w = UnicodeWidthStr::width(filter_label.as_str());
    let remaining = inner.saturating_sub(filter_w);

    crossterm::queue!(&mut buf, cursor::MoveTo(popup.col, popup.row))?;
    if let Some(c) = theme.border {
        crossterm::queue!(&mut buf, SetForegroundColor(c), Print("┌"), ResetColor)?;
    } else {
        crossterm::queue!(&mut buf, Print("┌"))?;
    }
    if let Some(c) = theme.filter {
        crossterm::queue!(
            &mut buf,
            SetForegroundColor(c),
            Print(&filter_label),
            ResetColor
        )?;
    } else {
        crossterm::queue!(&mut buf, Print(&filter_label))?;
    }
    if let Some(c) = theme.border {
        crossterm::queue!(
            &mut buf,
            SetForegroundColor(c),
            Print("─".repeat(remaining)),
            Print("┐"),
            ResetColor,
        )?;
    } else {
        crossterm::queue!(&mut buf, Print("─".repeat(remaining)), Print("┐"))?;
    }
    crossterm::queue!(&mut buf, terminal::Clear(terminal::ClearType::UntilNewLine))?;

    // Candidate rows
    let visible = app.visible_candidates();
    let highlight_idx = app.visible_selected_index();

    for (i, candidate) in visible.iter().enumerate() {
        let layout = layout_candidate(candidate, inner);

        crossterm::queue!(
            &mut buf,
            cursor::MoveTo(popup.col, popup.row + 1 + i as u16)
        )?;

        if Some(i) == highlight_idx {
            if let Some(c) = theme.border {
                crossterm::queue!(&mut buf, SetForegroundColor(c), Print("│"), ResetColor)?;
            } else {
                crossterm::queue!(&mut buf, Print("│"))?;
            }
            let use_explicit = theme.selected_fg.is_some() || theme.selected_bg.is_some();
            if use_explicit {
                if let Some(c) = theme.selected_fg {
                    crossterm::queue!(&mut buf, SetForegroundColor(c))?;
                }
                if let Some(c) = theme.selected_bg {
                    crossterm::queue!(&mut buf, SetBackgroundColor(c))?;
                }
            } else {
                crossterm::queue!(&mut buf, SetAttribute(Attribute::Reverse))?;
            }
            crossterm::queue!(
                &mut buf,
                Print(&layout.text),
                Print(" ".repeat(layout.gap)),
                Print(&layout.description),
            )?;
            if use_explicit {
                crossterm::queue!(&mut buf, ResetColor)?;
            } else {
                crossterm::queue!(&mut buf, SetAttribute(Attribute::NoReverse), ResetColor)?;
            }
            if let Some(c) = theme.border {
                crossterm::queue!(&mut buf, SetForegroundColor(c), Print("│"), ResetColor)?;
            } else {
                crossterm::queue!(&mut buf, Print("│"))?;
            }
            crossterm::queue!(&mut buf, terminal::Clear(terminal::ClearType::UntilNewLine))?;
        } else {
            if let Some(c) = theme.border {
                crossterm::queue!(&mut buf, SetForegroundColor(c), Print("│"), ResetColor)?;
            } else {
                crossterm::queue!(&mut buf, Print("│"))?;
            }
            if let Some(c) = theme.candidate {
                crossterm::queue!(
                    &mut buf,
                    SetForegroundColor(c),
                    Print(&layout.text),
                    ResetColor,
                )?;
            } else {
                crossterm::queue!(&mut buf, Print(&layout.text))?;
            }

            if !layout.description.is_empty() {
                crossterm::queue!(
                    &mut buf,
                    Print(" ".repeat(layout.gap)),
                    SetForegroundColor(theme.description),
                    Print(&layout.description),
                    ResetColor,
                )?;
            } else {
                crossterm::queue!(&mut buf, Print(" ".repeat(layout.gap)))?;
            }

            if let Some(c) = theme.border {
                crossterm::queue!(&mut buf, SetForegroundColor(c), Print("│"), ResetColor)?;
            } else {
                crossterm::queue!(&mut buf, Print("│"))?;
            }
            crossterm::queue!(&mut buf, terminal::Clear(terminal::ClearType::UntilNewLine))?;
        }
    }

    // Bottom border
    crossterm::queue!(
        &mut buf,
        cursor::MoveTo(popup.col, popup.row + 1 + visible.len() as u16),
    )?;
    if let Some(c) = theme.border {
        crossterm::queue!(
            &mut buf,
            SetForegroundColor(c),
            Print("└"),
            Print("─".repeat(inner)),
            Print("┘"),
            ResetColor,
        )?;
    } else {
        crossterm::queue!(&mut buf, Print("└"), Print("─".repeat(inner)), Print("┘"),)?;
    }
    crossterm::queue!(&mut buf, terminal::Clear(terminal::ClearType::UntilNewLine))?;

    // Update filter_text on the prompt line
    let prefix_w = UnicodeWidthStr::width(app.prefix.as_str()) as u16;
    let prefix_start_col = app.cursor_col.saturating_sub(prefix_w);
    let filter_display = &app.filter_text;
    let filter_w = UnicodeWidthStr::width(filter_display.as_str()) as u16;

    crossterm::queue!(
        &mut buf,
        cursor::MoveTo(prefix_start_col, app.cursor_row),
        Print(filter_display),
    )?;

    let clear_count = prefix_w.saturating_sub(filter_w);
    if clear_count > 0 {
        crossterm::queue!(&mut buf, Print(" ".repeat(clear_count as usize)))?;
    }

    let cursor_end_col = prefix_start_col + filter_w;
    crossterm::queue!(
        &mut buf,
        cursor::MoveTo(cursor_end_col, app.cursor_row),
        cursor::Show,
    )?;
    buf.flush()?;

    Ok(())
}

pub fn draw_popup_only(tty: &mut std::fs::File, app: &App, theme: &Theme) -> std::io::Result<()> {
    let mut buf = std::io::BufWriter::new(&mut *tty);
    let popup = Popup::compute(app);
    let inner = (popup.width - 2) as usize;

    crossterm::queue!(&mut buf, cursor::Hide)?;

    // Top border with filter text
    let filter_label = format!(" {} ", &app.filter_text);
    let filter_w = UnicodeWidthStr::width(filter_label.as_str());
    let remaining = inner.saturating_sub(filter_w);

    crossterm::queue!(&mut buf, cursor::MoveTo(popup.col, popup.row))?;
    if let Some(c) = theme.border {
        crossterm::queue!(&mut buf, SetForegroundColor(c), Print("┌"), ResetColor)?;
    } else {
        crossterm::queue!(&mut buf, Print("┌"))?;
    }
    if let Some(c) = theme.filter {
        crossterm::queue!(
            &mut buf,
            SetForegroundColor(c),
            Print(&filter_label),
            ResetColor
        )?;
    } else {
        crossterm::queue!(&mut buf, Print(&filter_label))?;
    }
    if let Some(c) = theme.border {
        crossterm::queue!(
            &mut buf,
            SetForegroundColor(c),
            Print("─".repeat(remaining)),
            Print("┐"),
            ResetColor,
        )?;
    } else {
        crossterm::queue!(&mut buf, Print("─".repeat(remaining)), Print("┐"))?;
    }
    crossterm::queue!(&mut buf, terminal::Clear(terminal::ClearType::UntilNewLine))?;

    // Candidate rows
    let visible = app.visible_candidates();
    let highlight_idx = app.visible_selected_index();

    for (i, candidate) in visible.iter().enumerate() {
        let layout = layout_candidate(candidate, inner);

        crossterm::queue!(
            &mut buf,
            cursor::MoveTo(popup.col, popup.row + 1 + i as u16)
        )?;

        if Some(i) == highlight_idx {
            if let Some(c) = theme.border {
                crossterm::queue!(&mut buf, SetForegroundColor(c), Print("│"), ResetColor)?;
            } else {
                crossterm::queue!(&mut buf, Print("│"))?;
            }
            let use_explicit = theme.selected_fg.is_some() || theme.selected_bg.is_some();
            if use_explicit {
                if let Some(c) = theme.selected_fg {
                    crossterm::queue!(&mut buf, SetForegroundColor(c))?;
                }
                if let Some(c) = theme.selected_bg {
                    crossterm::queue!(&mut buf, SetBackgroundColor(c))?;
                }
            } else {
                crossterm::queue!(&mut buf, SetAttribute(Attribute::Reverse))?;
            }
            crossterm::queue!(
                &mut buf,
                Print(&layout.text),
                Print(" ".repeat(layout.gap)),
                Print(&layout.description),
            )?;
            if use_explicit {
                crossterm::queue!(&mut buf, ResetColor)?;
            } else {
                crossterm::queue!(&mut buf, SetAttribute(Attribute::NoReverse), ResetColor)?;
            }
            if let Some(c) = theme.border {
                crossterm::queue!(&mut buf, SetForegroundColor(c), Print("│"), ResetColor)?;
            } else {
                crossterm::queue!(&mut buf, Print("│"))?;
            }
            crossterm::queue!(&mut buf, terminal::Clear(terminal::ClearType::UntilNewLine))?;
        } else {
            if let Some(c) = theme.border {
                crossterm::queue!(&mut buf, SetForegroundColor(c), Print("│"), ResetColor)?;
            } else {
                crossterm::queue!(&mut buf, Print("│"))?;
            }
            if let Some(c) = theme.candidate {
                crossterm::queue!(
                    &mut buf,
                    SetForegroundColor(c),
                    Print(&layout.text),
                    ResetColor,
                )?;
            } else {
                crossterm::queue!(&mut buf, Print(&layout.text))?;
            }

            if !layout.description.is_empty() {
                crossterm::queue!(
                    &mut buf,
                    Print(" ".repeat(layout.gap)),
                    SetForegroundColor(theme.description),
                    Print(&layout.description),
                    ResetColor,
                )?;
            } else {
                crossterm::queue!(&mut buf, Print(" ".repeat(layout.gap)))?;
            }

            if let Some(c) = theme.border {
                crossterm::queue!(&mut buf, SetForegroundColor(c), Print("│"), ResetColor)?;
            } else {
                crossterm::queue!(&mut buf, Print("│"))?;
            }
            crossterm::queue!(&mut buf, terminal::Clear(terminal::ClearType::UntilNewLine))?;
        }
    }

    // Bottom border
    crossterm::queue!(
        &mut buf,
        cursor::MoveTo(popup.col, popup.row + 1 + visible.len() as u16),
    )?;
    if let Some(c) = theme.border {
        crossterm::queue!(
            &mut buf,
            SetForegroundColor(c),
            Print("└"),
            Print("─".repeat(inner)),
            Print("┘"),
            ResetColor,
        )?;
    } else {
        crossterm::queue!(&mut buf, Print("└"), Print("─".repeat(inner)), Print("┘"),)?;
    }
    crossterm::queue!(&mut buf, terminal::Clear(terminal::ClearType::UntilNewLine))?;

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
    cursor_row: u16,
) -> std::io::Result<()> {
    let mut buf = std::io::BufWriter::new(&mut *tty);
    for i in 0..popup_height {
        crossterm::queue!(
            &mut buf,
            cursor::MoveTo(0, popup_row + i),
            terminal::Clear(terminal::ClearType::CurrentLine),
        )?;
    }

    crossterm::queue!(&mut buf, cursor::MoveTo(0, cursor_row))?;
    buf.flush()?;

    Ok(())
}

pub fn clear(tty: &mut std::fs::File, app: &App) -> std::io::Result<()> {
    let mut buf = std::io::BufWriter::new(&mut *tty);
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
    buf.flush()?;

    Ok(())
}

pub fn truncate_to_width(s: &str, max_width: usize) -> String {
    let mut width = 0;
    let mut result = String::new();
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
}
