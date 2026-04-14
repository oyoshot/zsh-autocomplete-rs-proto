use crate::app::App;
use unicode_width::UnicodeWidthStr;

const MAX_POPUP_WIDTH: u16 = 60;
const PADDING: u16 = 2;

/// Returns `true` if `s` is safe to embed as a space-delimited header token.
/// Rejects empty strings, spaces, and ASCII control characters.
pub fn is_safe_prefix(s: &str) -> bool {
    !s.is_empty() && !s.contains(|c: char| c == ' ' || c.is_ascii_control())
}

pub struct Popup {
    pub row: u16,
    pub col: u16,
    pub width: u16,
    pub height: u16,
}

impl Popup {
    /// Format metadata string for shell consumption (used by both daemon and subprocess paths).
    pub fn format_metadata(
        &self,
        cursor_row: u16,
        reuse_token: u64,
        filtered_count: usize,
        selected_original_idx: Option<usize>,
        common_prefix: Option<&str>,
    ) -> String {
        let mut meta = format!(
            "popup_row={} popup_height={} cursor_row={} reuse_token={} filtered_count={}",
            self.row, self.height, cursor_row, reuse_token, filtered_count
        );
        if let Some(orig_idx) = selected_original_idx {
            meta.push_str(&format!(" selected_original_idx={}", orig_idx));
        }
        if let Some(cp) = common_prefix
            && is_safe_prefix(cp)
        {
            meta.push_str(&format!(" common_prefix={}", cp));
        }
        meta
    }

    pub fn compute(app: &App) -> Self {
        let term_cols = app.term_cols;
        let term_rows = app.term_rows;

        let visible = app.visible_candidate_indices();
        let filter_display = format!(" {} ", &app.filter_text);

        let max_content_width = visible
            .iter()
            .map(|&candidate_idx| {
                let c = &app.all_candidates[candidate_idx];
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

    #[test]
    fn format_metadata_without_selection() {
        let popup = Popup {
            row: 6,
            col: 0,
            width: 30,
            height: 5,
        };
        let meta = popup.format_metadata(5, 12345, 10, None, None);
        assert_eq!(
            meta,
            "popup_row=6 popup_height=5 cursor_row=5 reuse_token=12345 filtered_count=10"
        );
    }

    #[test]
    fn format_metadata_with_selection() {
        let popup = Popup {
            row: 6,
            col: 0,
            width: 30,
            height: 5,
        };
        let meta = popup.format_metadata(5, 99, 3, Some(2), None);
        assert_eq!(
            meta,
            "popup_row=6 popup_height=5 cursor_row=5 reuse_token=99 filtered_count=3 selected_original_idx=2"
        );
    }

    #[test]
    fn format_metadata_with_common_prefix() {
        let popup = Popup {
            row: 6,
            col: 0,
            width: 30,
            height: 5,
        };
        let meta = popup.format_metadata(5, 42, 3, None, Some("git-"));
        assert_eq!(
            meta,
            "popup_row=6 popup_height=5 cursor_row=5 reuse_token=42 filtered_count=3 common_prefix=git-"
        );
    }

    #[test]
    fn format_metadata_none_common_prefix_omitted() {
        let popup = Popup {
            row: 6,
            col: 0,
            width: 30,
            height: 5,
        };
        let meta = popup.format_metadata(5, 42, 3, None, None);
        assert!(!meta.contains("common_prefix"));
    }

    #[test]
    fn format_metadata_space_in_common_prefix_omitted() {
        let popup = Popup {
            row: 6,
            col: 0,
            width: 30,
            height: 5,
        };
        // Space-containing prefix would break the space-delimited header protocol
        let meta = popup.format_metadata(5, 42, 3, None, Some("foo bar"));
        assert!(!meta.contains("common_prefix"));
    }

    #[test]
    fn format_metadata_control_char_in_common_prefix_omitted() {
        let popup = Popup {
            row: 6,
            col: 0,
            width: 30,
            height: 5,
        };
        for ctrl in ["\t", "\r", "\n", "\x1b", "\x7f"] {
            let prefix = format!("foo{ctrl}bar");
            let meta = popup.format_metadata(5, 42, 3, None, Some(prefix.as_str()));
            assert!(
                !meta.contains("common_prefix"),
                "control char {ctrl:?} should suppress common_prefix in: {meta}"
            );
        }
    }
}
