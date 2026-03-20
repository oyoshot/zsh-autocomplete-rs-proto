use crate::app::App;
use crate::candidate::Candidate;
use crate::ui::popup::Popup;
use crate::ui::theme;
use crossterm::cursor;
use crossterm::style::{Attribute, Print, ResetColor, SetAttribute, SetForegroundColor};
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
            // Move to bottom and print newlines to scroll terminal content up
            crossterm::execute!(tty, cursor::MoveTo(0, term_rows - 1))?;
            for _ in 0..scroll_amount {
                tty.write_all(b"\n")?;
            }
            app.cursor_row -= scroll_amount;
        }
    }

    Ok(())
}

pub fn draw(tty: &mut std::fs::File, app: &App) -> std::io::Result<()> {
    let popup = Popup::compute(app);
    let inner = (popup.width - 2) as usize;

    crossterm::execute!(tty, cursor::Hide)?;

    // Top border with filter text
    let filter_label = format!(" {} ", &app.filter_text);
    let filter_w = UnicodeWidthStr::width(filter_label.as_str());
    let remaining = inner.saturating_sub(filter_w);

    crossterm::execute!(
        tty,
        cursor::MoveTo(popup.col, popup.row),
        Print("┌"),
        Print(&filter_label),
        Print("─".repeat(remaining)),
        Print("┐"),
    )?;

    // Candidate rows
    let visible = app.visible_candidates();
    let highlight_idx = app.visible_selected_index();

    for (i, candidate) in visible.iter().enumerate() {
        let layout = layout_candidate(candidate, inner);

        crossterm::execute!(tty, cursor::MoveTo(popup.col, popup.row + 1 + i as u16))?;

        if Some(i) == highlight_idx {
            crossterm::execute!(
                tty,
                Print("│"),
                SetAttribute(Attribute::Reverse),
                Print(&layout.text),
                Print(" ".repeat(layout.gap)),
                Print(&layout.description),
                SetAttribute(Attribute::NoReverse),
                ResetColor,
                Print("│"),
            )?;
        } else {
            crossterm::execute!(tty, Print("│"), Print(&layout.text))?;

            if !layout.description.is_empty() {
                crossterm::execute!(
                    tty,
                    Print(" ".repeat(layout.gap)),
                    SetForegroundColor(theme::DESCRIPTION_COLOR),
                    Print(&layout.description),
                    ResetColor,
                )?;
            } else {
                crossterm::execute!(tty, Print(" ".repeat(layout.gap)))?;
            }

            crossterm::execute!(tty, Print("│"))?;
        }
    }

    // Bottom border
    crossterm::execute!(
        tty,
        cursor::MoveTo(popup.col, popup.row + 1 + visible.len() as u16),
        Print("└"),
        Print("─".repeat(inner)),
        Print("┘"),
    )?;

    // Update filter_text on the prompt line
    let prefix_w = UnicodeWidthStr::width(app.prefix.as_str()) as u16;
    let prefix_start_col = app.cursor_col.saturating_sub(prefix_w);
    let filter_display = &app.filter_text;
    let filter_w = UnicodeWidthStr::width(filter_display.as_str()) as u16;

    crossterm::execute!(
        tty,
        cursor::MoveTo(prefix_start_col, app.cursor_row),
        Print(filter_display),
    )?;

    let clear_count = prefix_w.saturating_sub(filter_w);
    if clear_count > 0 {
        crossterm::execute!(tty, Print(" ".repeat(clear_count as usize)))?;
    }

    let cursor_end_col = prefix_start_col + filter_w;
    crossterm::execute!(
        tty,
        cursor::MoveTo(cursor_end_col, app.cursor_row),
        cursor::Show,
    )?;
    tty.flush()?;

    Ok(())
}

/// Cap max_visible to fit available space without scrolling (for render mode).
pub fn cap_visible_for_render(app: &mut App) {
    let (_, term_rows) = terminal::size().unwrap_or((80, 24));
    let space_below = term_rows.saturating_sub(app.cursor_row + 1);

    if space_below < 3 {
        app.max_visible = 0;
        return;
    }

    let max_items = (space_below - 2) as usize;
    if app.max_visible > max_items {
        app.max_visible = max_items;
    }
}

pub fn draw_popup_only(tty: &mut std::fs::File, app: &App) -> std::io::Result<()> {
    let popup = Popup::compute(app);
    let inner = (popup.width - 2) as usize;

    crossterm::execute!(tty, cursor::Hide)?;

    // Top border with filter text
    let filter_label = format!(" {} ", &app.filter_text);
    let filter_w = UnicodeWidthStr::width(filter_label.as_str());
    let remaining = inner.saturating_sub(filter_w);

    crossterm::execute!(
        tty,
        cursor::MoveTo(popup.col, popup.row),
        Print("┌"),
        Print(&filter_label),
        Print("─".repeat(remaining)),
        Print("┐"),
    )?;

    // Candidate rows
    let visible = app.visible_candidates();
    let highlight_idx = app.visible_selected_index();

    for (i, candidate) in visible.iter().enumerate() {
        let layout = layout_candidate(candidate, inner);

        crossterm::execute!(tty, cursor::MoveTo(popup.col, popup.row + 1 + i as u16))?;

        if Some(i) == highlight_idx {
            crossterm::execute!(
                tty,
                Print("│"),
                SetAttribute(Attribute::Reverse),
                Print(&layout.text),
                Print(" ".repeat(layout.gap)),
                Print(&layout.description),
                SetAttribute(Attribute::NoReverse),
                ResetColor,
                Print("│"),
            )?;
        } else {
            crossterm::execute!(tty, Print("│"), Print(&layout.text))?;

            if !layout.description.is_empty() {
                crossterm::execute!(
                    tty,
                    Print(" ".repeat(layout.gap)),
                    SetForegroundColor(theme::DESCRIPTION_COLOR),
                    Print(&layout.description),
                    ResetColor,
                )?;
            } else {
                crossterm::execute!(tty, Print(" ".repeat(layout.gap)))?;
            }

            crossterm::execute!(tty, Print("│"))?;
        }
    }

    // Bottom border
    crossterm::execute!(
        tty,
        cursor::MoveTo(popup.col, popup.row + 1 + visible.len() as u16),
        Print("└"),
        Print("─".repeat(inner)),
        Print("┘"),
    )?;

    // Restore cursor to original position (zsh manages cursor)
    crossterm::execute!(
        tty,
        cursor::MoveTo(app.cursor_col, app.cursor_row),
        cursor::Show,
    )?;
    tty.flush()?;

    Ok(())
}

pub fn clear_rect(tty: &mut std::fs::File, popup_row: u16, popup_height: u16, cursor_row: u16) -> std::io::Result<()> {
    for i in 0..popup_height {
        crossterm::execute!(
            tty,
            cursor::MoveTo(0, popup_row + i),
            terminal::Clear(terminal::ClearType::CurrentLine),
        )?;
    }

    crossterm::execute!(tty, cursor::MoveTo(0, cursor_row))?;
    tty.flush()?;

    Ok(())
}

pub fn clear(tty: &mut std::fs::File, app: &App) -> std::io::Result<()> {
    let popup = Popup::compute(app);

    crossterm::execute!(tty, cursor::SavePosition)?;

    for row in popup.row..popup.row + popup.height {
        crossterm::execute!(
            tty,
            cursor::MoveTo(popup.col, row),
            Print(" ".repeat(popup.width as usize)),
        )?;
    }

    let prefix_w = UnicodeWidthStr::width(app.prefix.as_str()) as u16;
    let filter_w = UnicodeWidthStr::width(app.filter_text.as_str()) as u16;
    let prefix_start_col = app.cursor_col.saturating_sub(prefix_w);
    let max_w = prefix_w.max(filter_w);

    crossterm::execute!(
        tty,
        cursor::MoveTo(prefix_start_col, app.cursor_row),
        Print(&app.prefix),
        Print(" ".repeat((max_w - prefix_w) as usize)),
    )?;

    crossterm::execute!(tty, cursor::RestorePosition)?;
    tty.flush()?;

    Ok(())
}

fn truncate_to_width(s: &str, max_width: usize) -> String {
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
