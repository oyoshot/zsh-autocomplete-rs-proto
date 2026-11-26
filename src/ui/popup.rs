use crate::app::App;
use unicode_width::UnicodeWidthStr;

const MAX_POPUP_WIDTH: u16 = 60;
const PADDING: usize = 3; // Two cells for the selection gutter, one on the right.
/// Rows a popup needs: the cursor line, the frame and one candidate.
const MIN_TERM_ROWS: u16 = 4;
/// Columns a popup needs: the frame and one content cell.
const MIN_TERM_COLS: u16 = 3;

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
    /// Whether a terminal of this size can show a popup below the cursor
    /// line without drawing outside the terminal.
    pub fn fits_terminal(term_cols: u16, term_rows: u16) -> bool {
        term_cols >= MIN_TERM_COLS && term_rows >= MIN_TERM_ROWS
    }

    /// Candidate rows that fit below the cursor line and inside the frame.
    pub fn rows_available(term_rows: u16) -> usize {
        usize::from(term_rows.saturating_sub(MIN_TERM_ROWS - 1))
    }

    /// Lines to scroll the terminal up so that a popup with `max_visible`
    /// candidate rows fits below the cursor. The cursor moves up by the same
    /// amount, so the result never exceeds `cursor_row`.
    pub fn scroll_to_fit(cursor_row: u16, term_rows: u16, max_visible: usize) -> u16 {
        let popup_height = u16::try_from(max_visible)
            .unwrap_or(u16::MAX)
            .saturating_add(2);
        let space_below = term_rows.saturating_sub(cursor_row.saturating_add(1));
        popup_height.saturating_sub(space_below).min(cursor_row)
    }

    pub(crate) fn range_label(app: &App) -> String {
        let visible = app.visible_candidate_indices().len();
        let total = app.filtered_indices().len();
        if visible == 0 || visible == total {
            String::new()
        } else {
            format!(
                " {}–{} / {} ",
                app.scroll_offset() + 1,
                app.scroll_offset() + visible,
                total
            )
        }
    }

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
        let filter_display = format!(" {} ", app.filter_text);

        let max_content_width = visible
            .iter()
            .map(|&candidate_idx| {
                let c = &app.all_candidates[candidate_idx];
                let text_w = UnicodeWidthStr::width(c.text.as_str());
                let desc_w = if c.description.is_empty() {
                    0
                } else {
                    UnicodeWidthStr::width(c.description.as_str()).saturating_add(2)
                };
                text_w.saturating_add(desc_w)
            })
            .max()
            .unwrap_or(0);

        let inner_width = max_content_width
            .saturating_add(PADDING)
            .max(UnicodeWidthStr::width(filter_display.as_str()))
            .max(UnicodeWidthStr::width(Self::range_label(app).as_str()));

        Self::place(
            app.cursor_row,
            app.cursor_col,
            term_cols,
            term_rows,
            inner_width,
            visible.len(),
        )
    }

    /// Places a popup with `inner_width` content cells and `num_visible` rows.
    ///
    /// Total for every input: oversized content saturates instead of wrapping,
    /// so the popup never extends past the right edge of the terminal.
    fn place(
        cursor_row: u16,
        cursor_col: u16,
        term_cols: u16,
        term_rows: u16,
        inner_width: usize,
        num_visible: usize,
    ) -> Self {
        let width = u16::try_from(inner_width)
            .unwrap_or(u16::MAX)
            .saturating_add(2)
            .min(MAX_POPUP_WIDTH)
            .min(term_cols);
        let height = u16::try_from(num_visible)
            .unwrap_or(u16::MAX)
            .saturating_add(2);

        let col = cursor_col.min(term_cols.saturating_sub(width));

        let row_below = cursor_row.saturating_add(1);
        let space_below = term_rows.saturating_sub(row_below);
        let row = if space_below >= height {
            row_below
        } else {
            cursor_row.saturating_sub(height)
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
                insert_text: None,
                cursor_offset: None,
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

    /// Runs as a randomized test under `cargo test` and as a proof over every
    /// input under `cargo kani --tests`.
    #[test]
    #[cfg_attr(kani, kani::proof)]
    fn place_stays_on_screen_for_any_geometry() {
        bolero::check!()
            .with_type::<(u16, u16, u16, u16, usize, usize)>()
            .cloned()
            .for_each(
                |(cursor_row, cursor_col, term_cols, term_rows, inner_width, num_visible)| {
                    let popup = Popup::place(
                        cursor_row,
                        cursor_col,
                        term_cols,
                        term_rows,
                        inner_width,
                        num_visible,
                    );
                    let bottom = u32::from(popup.row) + u32::from(popup.height);

                    let framed =
                        |cells: usize| u16::try_from(cells.saturating_add(2)).unwrap_or(u16::MAX);
                    assert_eq!(
                        popup.width,
                        framed(inner_width).min(MAX_POPUP_WIDTH).min(term_cols)
                    );
                    assert_eq!(popup.height, framed(num_visible));
                    assert!(popup.col <= cursor_col);
                    assert!(u32::from(popup.col) + u32::from(popup.width) <= u32::from(term_cols));

                    if popup.row > cursor_row {
                        assert_eq!(u32::from(popup.row), u32::from(cursor_row) + 1);
                        assert!(bottom <= u32::from(term_rows));
                    } else if cursor_row >= popup.height {
                        assert_eq!(bottom, u32::from(cursor_row));
                    } else {
                        assert_eq!(popup.row, 0);
                    }
                    if cursor_row < term_rows && popup.height <= term_rows {
                        assert!(bottom <= u32::from(term_rows));
                    }
                },
            );
    }

    #[test]
    fn popup_width_saturates_for_oversized_candidates() {
        let wide = "a".repeat(usize::from(u16::MAX));
        let mut app = make_app(&[&wide], "", 5, 0, 80, 24);
        app.all_candidates[0].description = wide.clone();
        let popup = Popup::compute(&app);
        assert_eq!(popup.width, MAX_POPUP_WIDTH);
    }

    #[test]
    fn fits_terminal_needs_room_for_the_frame_and_one_row() {
        assert!(Popup::fits_terminal(3, 4));
        assert!(!Popup::fits_terminal(2, 4));
        assert!(!Popup::fits_terminal(3, 3));
        assert_eq!(Popup::rows_available(4), 1);
        assert_eq!(Popup::rows_available(3), 0);
        assert_eq!(Popup::rows_available(0), 0);
    }

    /// In any terminal a popup fits, scrolling by `scroll_to_fit` leaves room
    /// to place the popup directly below the cursor and inside the terminal,
    /// so it never covers the cursor line. Runs as a randomized test under
    /// `cargo test` and as a proof over every input under `cargo kani --tests`.
    #[test]
    #[cfg_attr(kani, kani::proof)]
    fn popup_fits_below_the_cursor_after_scrolling() {
        bolero::check!()
            .with_type::<(u16, u16, u16, u16, usize, usize, usize)>()
            .cloned()
            .for_each(
                |(
                    cursor_row,
                    cursor_col,
                    term_cols,
                    term_rows,
                    inner_width,
                    max_visible,
                    num_visible,
                )| {
                    if !Popup::fits_terminal(term_cols, term_rows) {
                        return;
                    }
                    // Few random draws satisfy the conditions below, so the
                    // randomized run repairs its draw first. Kani keeps the
                    // raw draw: there the conditions alone decide coverage.
                    #[cfg(not(kani))]
                    let (cursor_row, max_visible, num_visible) = {
                        let max_visible = 1 + max_visible % Popup::rows_available(term_rows);
                        (
                            cursor_row % term_rows,
                            max_visible,
                            num_visible % (max_visible + 1),
                        )
                    };
                    if cursor_row >= term_rows
                        || max_visible == 0
                        || max_visible > Popup::rows_available(term_rows)
                        || num_visible > max_visible
                    {
                        return;
                    }

                    let scroll = Popup::scroll_to_fit(cursor_row, term_rows, max_visible);
                    assert!(scroll <= cursor_row);
                    let cursor_row = cursor_row - scroll;
                    let popup = Popup::place(
                        cursor_row,
                        cursor_col,
                        term_cols,
                        term_rows,
                        inner_width,
                        num_visible,
                    );

                    assert_eq!(popup.row, cursor_row + 1);
                    assert!(u32::from(popup.row) + u32::from(popup.height) <= u32::from(term_rows));
                    assert!(u32::from(popup.col) + u32::from(popup.width) <= u32::from(term_cols));
                    if inner_width >= 1 {
                        assert!(popup.width >= MIN_TERM_COLS);
                    }
                },
            );
    }
}
