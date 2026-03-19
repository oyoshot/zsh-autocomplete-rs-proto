use crate::app::App;
use crossterm::terminal;
use unicode_width::UnicodeWidthStr;

const MAX_POPUP_WIDTH: u16 = 60;
const PADDING: u16 = 2;

pub struct Popup {
    pub row: u16,
    pub col: u16,
    pub width: u16,
    pub height: u16,
}

impl Popup {
    pub fn compute(app: &App) -> Self {
        let (term_cols, term_rows) = terminal::size().unwrap_or((80, 24));

        let visible = app.visible_candidates();
        let filter_display = format!(" {} ", &app.filter_text);

        let max_content_width = visible
            .iter()
            .map(|c| {
                let text_w = UnicodeWidthStr::width(c.text.as_str()) as u16;
                let desc_w = if c.description.is_empty() {
                    0
                } else {
                    UnicodeWidthStr::width(c.description.as_str()) as u16 + 2
                };
                text_w + desc_w
            })
            .max()
            .unwrap_or(0)
            .max(UnicodeWidthStr::width(filter_display.as_str()) as u16);

        let inner_width = max_content_width + PADDING;
        let width = (inner_width + 2).min(MAX_POPUP_WIDTH).min(term_cols);

        let num_visible = visible.len() as u16;
        let height = num_visible + 2;

        let col = app.cursor_col.min(term_cols.saturating_sub(width));

        let space_below = term_rows.saturating_sub(app.cursor_row + 1);
        let row = if space_below >= height {
            app.cursor_row + 1
        } else {
            app.cursor_row.saturating_sub(height)
        };

        Popup {
            row,
            col,
            width,
            height,
        }
    }
}
