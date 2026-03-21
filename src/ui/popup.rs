use crate::app::App;
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
        let term_cols = app.term_cols;
        let term_rows = app.term_rows;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::candidate::Candidate;

    fn make_app(
        items: &[&str],
        prefix: &str,
        cursor_row: u16,
        cursor_col: u16,
        term_cols: u16,
        term_rows: u16,
    ) -> App {
        let candidates: Vec<Candidate> = items
            .iter()
            .map(|s| Candidate {
                text: s.to_string(),
                description: String::new(),
                kind: String::new(),
            })
            .collect();
        App::new_with_term_size(
            candidates,
            prefix.to_string(),
            cursor_row,
            cursor_col,
            term_cols,
            term_rows,
        )
    }

    // terminal::size() returns real size in a terminal, fallback (80,24) otherwise.
    // Tests use dynamic cursor positions relative to terminal size for robustness.

    #[test]
    fn popup_below_cursor() {
        let app = make_app(&["alpha", "beta", "gamma"], "", 5, 10, 80, 24);
        let popup = Popup::compute(&app);
        assert_eq!(popup.row, app.cursor_row + 1);
    }

    #[test]
    fn popup_above_when_no_space() {
        let (_, term_rows) = crossterm::terminal::size().unwrap_or((80, 24));
        let cursor_row = term_rows.saturating_sub(2);
        let app = make_app(
            &["alpha", "beta", "gamma"],
            "",
            cursor_row,
            10,
            80,
            term_rows,
        );
        let popup = Popup::compute(&app);
        assert!(popup.row < app.cursor_row);
    }

    #[test]
    fn popup_width_clamped() {
        let long_text = "a".repeat(70);
        let app = make_app(&[&long_text], "", 5, 0, 80, 24);
        let popup = Popup::compute(&app);
        assert_eq!(popup.width, MAX_POPUP_WIDTH);
    }

    #[test]
    fn popup_col_near_right_edge() {
        let (term_cols, _) = crossterm::terminal::size().unwrap_or((80, 24));
        let cursor_col = term_cols.saturating_sub(5);
        let app = make_app(&["alpha", "beta"], "", 5, cursor_col, term_cols, 24);
        let popup = Popup::compute(&app);
        assert!(popup.col + popup.width <= term_cols);
    }

    #[test]
    fn popup_recomputes_against_updated_term_size() {
        let mut app = make_app(&["alpha", "beta", "gamma"], "", 5, 38, 40, 24);
        let initial = Popup::compute(&app);

        app.set_term_size(20, 8);
        let resized = Popup::compute(&app);

        assert_ne!(initial.col, resized.col);
        assert!(resized.col + resized.width <= app.term_cols);
        assert!(resized.row + resized.height <= app.term_rows);
    }
}
