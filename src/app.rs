use crate::candidate::Candidate;
use crate::fuzzy::FuzzyMatcher;
use std::collections::HashMap;

const DEFAULT_MAX_VISIBLE: usize = 10;

pub struct App {
    pub all_candidates: Vec<Candidate>,
    pub filtered_indices: Vec<usize>,
    // Unbounded; typical session produces few unique queries so no eviction needed
    cached_filters: HashMap<String, Vec<usize>>,
    pub filter_text: String,
    selected: Option<usize>,
    pub scroll_offset: usize,
    pub max_visible: usize,
    pub cursor_row: u16,
    pub cursor_col: u16,
    pub term_cols: u16,
    pub term_rows: u16,
    pub prefix: String,
    fuzzy: FuzzyMatcher,
}

impl App {
    pub fn new(
        candidates: Vec<Candidate>,
        prefix: String,
        cursor_row: u16,
        cursor_col: u16,
    ) -> Self {
        let (term_cols, term_rows) = crossterm::terminal::size().unwrap_or((80, 24));
        Self::new_with_term_size(
            candidates, prefix, cursor_row, cursor_col, term_cols, term_rows,
        )
    }

    pub fn new_with_term_size(
        candidates: Vec<Candidate>,
        prefix: String,
        cursor_row: u16,
        cursor_col: u16,
        term_cols: u16,
        term_rows: u16,
    ) -> Self {
        Self::new_with_matcher(
            candidates,
            prefix,
            cursor_row,
            cursor_col,
            term_cols,
            term_rows,
            FuzzyMatcher::new(),
        )
    }

    pub fn new_with_matcher(
        candidates: Vec<Candidate>,
        prefix: String,
        cursor_row: u16,
        cursor_col: u16,
        term_cols: u16,
        term_rows: u16,
        fuzzy: FuzzyMatcher,
    ) -> Self {
        let term_cols = term_cols.max(1);
        let term_rows = term_rows.max(1);
        let cursor_row = cursor_row.min(term_rows.saturating_sub(1));
        let cursor_col = cursor_col.min(term_cols.saturating_sub(1));

        let lcp = compute_common_prefix(&candidates, &prefix);
        let mut app = App {
            all_candidates: candidates,
            filtered_indices: Vec::new(),
            cached_filters: HashMap::new(),
            filter_text: lcp,
            selected: None,
            scroll_offset: 0,
            max_visible: DEFAULT_MAX_VISIBLE,
            cursor_row,
            cursor_col,
            term_cols,
            term_rows,
            prefix,
            fuzzy,
        };
        app.sync_max_visible();
        app.update_filter();
        app
    }

    pub fn update_filter(&mut self) {
        self.update_filter_with_scope(None);
    }

    fn update_filter_with_scope(&mut self, fuzzy_scope: Option<&[usize]>) {
        let scored =
            self.fuzzy
                .filter_matches(&self.all_candidates, &self.filter_text, fuzzy_scope);
        self.filtered_indices = scored.into_iter().map(|s| s.candidate_idx).collect();
        self.selected = None;
        self.scroll_offset = 0;
        self.cache_current_filter();
    }

    fn cache_current_filter(&mut self) {
        self.cached_filters
            .insert(self.filter_text.clone(), self.filtered_indices.clone());
    }

    fn restore_cached_filter(&mut self) -> bool {
        let Some(filtered_indices) = self.cached_filters.get(&self.filter_text).cloned() else {
            return false;
        };
        self.filtered_indices = filtered_indices;
        self.selected = None;
        self.scroll_offset = 0;
        true
    }

    pub fn move_down(&mut self) {
        if self.filtered_indices.is_empty() {
            return;
        }
        let sel = match self.selected {
            None => 0,
            Some(s) if s + 1 < self.filtered_indices.len() => s + 1,
            Some(_) => {
                self.scroll_offset = 0;
                0
            }
        };
        self.selected = Some(sel);
        if sel >= self.scroll_offset + self.max_visible {
            self.scroll_offset = sel + 1 - self.max_visible;
        }
    }

    pub fn move_up(&mut self) {
        if self.filtered_indices.is_empty() {
            return;
        }
        let sel = match self.selected {
            None | Some(0) => {
                let last = self.filtered_indices.len() - 1;
                self.scroll_offset = last.saturating_sub(self.max_visible.saturating_sub(1));
                last
            }
            Some(s) => s - 1,
        };
        self.selected = Some(sel);
        if sel < self.scroll_offset {
            self.scroll_offset = sel;
        }
    }

    pub fn page_down(&mut self) {
        if self.filtered_indices.is_empty() {
            return;
        }
        let cur = self.selected.unwrap_or(0);
        let sel = (cur + self.max_visible).min(self.filtered_indices.len() - 1);
        self.selected = Some(sel);
        if sel >= self.scroll_offset + self.max_visible {
            self.scroll_offset = sel + 1 - self.max_visible;
        }
    }

    pub fn page_up(&mut self) {
        if self.filtered_indices.is_empty() {
            return;
        }
        let cur = self.selected.unwrap_or(0);
        let sel = cur.saturating_sub(self.max_visible);
        self.selected = Some(sel);
        if sel < self.scroll_offset {
            self.scroll_offset = sel;
        }
    }

    pub fn type_char(&mut self, c: char) {
        self.filter_text.push(c);
        if self.restore_cached_filter() {
            return;
        }
        let previous = std::mem::take(&mut self.filtered_indices);
        self.update_filter_with_scope(Some(&previous));
    }

    pub fn backspace(&mut self) -> bool {
        if self.filter_text.is_empty() {
            return false;
        }
        self.filter_text.pop();
        if self.restore_cached_filter() {
            return true;
        }
        self.update_filter();
        true
    }

    pub fn select_first(&mut self) {
        if self.filtered_indices.is_empty() {
            self.selected = None;
            self.scroll_offset = 0;
            return;
        }
        self.selected = Some(0);
        self.scroll_offset = 0;
    }

    pub fn selected(&self) -> Option<usize> {
        self.selected
    }

    pub fn set_selected(&mut self, idx: usize) {
        if idx >= self.filtered_indices.len() {
            return;
        }
        self.selected = Some(idx);
        // Adjust viewport to keep selected visible
        if idx < self.scroll_offset {
            self.scroll_offset = idx;
        } else if idx >= self.scroll_offset + self.max_visible {
            self.scroll_offset = idx + 1 - self.max_visible;
        }
    }

    /// Returns the index into `all_candidates` for the currently selected item.
    pub fn selected_original_idx(&self) -> Option<usize> {
        let sel = self.selected?;
        self.filtered_indices.get(sel).copied()
    }

    pub fn selected_candidate(&self) -> Option<&Candidate> {
        let sel = self.selected?;
        self.filtered_indices
            .get(sel)
            .and_then(|&candidate_idx| self.all_candidates.get(candidate_idx))
    }

    pub fn visible_candidate_indices(&self) -> &[usize] {
        let end = (self.scroll_offset + self.max_visible).min(self.filtered_indices.len());
        &self.filtered_indices[self.scroll_offset..end]
    }

    pub fn visible_selected_index(&self) -> Option<usize> {
        let sel = self.selected?;
        Some(sel - self.scroll_offset)
    }

    pub fn take_fuzzy(self) -> FuzzyMatcher {
        self.fuzzy
    }

    pub fn set_term_size(&mut self, term_cols: u16, term_rows: u16) {
        self.term_cols = term_cols.max(1);
        self.term_rows = term_rows.max(1);
        self.cursor_row = self.cursor_row.min(self.term_rows.saturating_sub(1));
        self.cursor_col = self.cursor_col.min(self.term_cols.saturating_sub(1));
        self.sync_max_visible();
    }

    pub fn sync_max_visible(&mut self) {
        let max_popup_height = self.term_rows.saturating_sub(1);
        let max_visible = max_popup_height.saturating_sub(2).max(1) as usize;
        self.max_visible = DEFAULT_MAX_VISIBLE.min(max_visible);
        self.clamp_viewport();
    }

    fn clamp_viewport(&mut self) {
        if self.filtered_indices.is_empty() {
            self.selected = None;
            self.scroll_offset = 0;
            return;
        }

        let max_scroll = self.filtered_indices.len().saturating_sub(self.max_visible);
        self.scroll_offset = self.scroll_offset.min(max_scroll);

        if let Some(sel) = self.selected {
            let sel = sel.min(self.filtered_indices.len() - 1);
            self.selected = Some(sel);

            if sel < self.scroll_offset {
                self.scroll_offset = sel;
            }

            if sel >= self.scroll_offset + self.max_visible {
                self.scroll_offset = sel + 1 - self.max_visible;
            }
        }
    }
}

pub fn compute_common_prefix(candidates: &[Candidate], prefix: &str) -> String {
    if candidates.is_empty() {
        return prefix.to_string();
    }
    let first = &candidates[0].text;
    let mut len = first.len();
    for c in &candidates[1..] {
        len = first
            .bytes()
            .zip(c.text.bytes())
            .take(len)
            .take_while(|(a, b)| a.eq_ignore_ascii_case(b))
            .count();
    }
    while len > 0 && !first.is_char_boundary(len) {
        len -= 1;
    }
    let lcp = &first[..len];
    if lcp.len() > prefix.len() {
        lcp.to_string()
    } else {
        prefix.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::candidate::Candidate;
    use crate::fuzzy::FuzzyMatcher;

    fn make_candidates(items: &[&str]) -> Vec<Candidate> {
        items
            .iter()
            .map(|s| Candidate {
                text: s.to_string(),
                description: String::new(),
                kind: String::new(),
            })
            .collect()
    }

    fn filtered_texts(app: &App) -> Vec<&str> {
        app.filtered_indices
            .iter()
            .map(|&candidate_idx| app.all_candidates[candidate_idx].text.as_str())
            .collect()
    }

    // --- compute_common_prefix ---

    #[test]
    fn common_prefix_empty_candidates() {
        let result = compute_common_prefix(&[], "foo");
        assert_eq!(result, "foo");
    }

    #[test]
    fn common_prefix_single_candidate() {
        let candidates = make_candidates(&["foobar"]);
        let result = compute_common_prefix(&candidates, "fo");
        assert_eq!(result, "foobar");
    }

    #[test]
    fn common_prefix_shared() {
        let candidates = make_candidates(&["foobar", "foobaz"]);
        let result = compute_common_prefix(&candidates, "fo");
        assert_eq!(result, "fooba");
    }

    #[test]
    fn common_prefix_no_shared_beyond_prefix() {
        let candidates = make_candidates(&["foo", "bar"]);
        let result = compute_common_prefix(&candidates, "");
        assert_eq!(result, "");
    }

    #[test]
    fn common_prefix_case_insensitive() {
        let candidates = make_candidates(&["Foo", "foo"]);
        let result = compute_common_prefix(&candidates, "f");
        assert_eq!(result, "Foo");
    }

    #[test]
    fn common_prefix_multibyte_shared() {
        let candidates = make_candidates(&["【配布】演習用RFP.pdf", "【配布】提案書.pdf"]);
        let result = compute_common_prefix(&candidates, "~/");
        assert_eq!(result, "【配布】");
    }

    #[test]
    fn common_prefix_multibyte_no_panic() {
        let candidates = make_candidates(&["あいう", "あえお"]);
        let result = compute_common_prefix(&candidates, "");
        assert_eq!(result, "あ");
    }

    #[test]
    fn common_prefix_emoji() {
        let candidates = make_candidates(&["🎉abc", "🎊def"]);
        let result = compute_common_prefix(&candidates, "");
        assert_eq!(result, "");
    }

    #[test]
    fn common_prefix_latin_accented() {
        let candidates = make_candidates(&["café", "cafè"]);
        let result = compute_common_prefix(&candidates, "");
        assert_eq!(result, "caf");
    }

    #[test]
    fn common_prefix_ascii_then_multibyte() {
        let candidates = make_candidates(&["~/Downloads/【配布】演", "~/Downloads/【配布】提"]);
        let result = compute_common_prefix(&candidates, "~/");
        assert_eq!(result, "~/Downloads/【配布】");
    }

    // --- App::new ---

    #[test]
    fn new_sets_filter_to_common_prefix() {
        let candidates = make_candidates(&["foobar", "foobaz"]);
        let app = App::new(candidates, "fo".to_string(), 5, 10);
        assert_eq!(app.filter_text, "fooba");
    }

    #[test]
    fn new_filters_candidates() {
        let candidates = make_candidates(&["foobar", "foobaz", "bar"]);
        let app = App::new(candidates, "fo".to_string(), 5, 10);
        assert_eq!(app.filtered_indices.len(), 2);
        assert!(!filtered_texts(&app).contains(&"bar"));
    }

    #[test]
    fn new_empty_candidates() {
        let app = App::new(Vec::new(), "fo".to_string(), 5, 10);
        assert!(app.filtered_indices.is_empty());
        assert_eq!(app.selected(), None);
    }

    #[test]
    fn initial_selected_is_none() {
        let candidates = make_candidates(&["alpha", "beta", "gamma"]);
        let app = App::new(candidates, "".to_string(), 5, 10);
        assert_eq!(app.selected(), None);
        assert!(app.selected_candidate().is_none());
        assert_eq!(app.visible_selected_index(), None);
    }

    // --- move_down ---

    #[test]
    fn move_down_from_none_selects_first() {
        let candidates = make_candidates(&["a", "b", "c"]);
        let mut app = App::new(candidates, "".to_string(), 5, 10);
        assert_eq!(app.selected(), None);
        app.move_down();
        assert_eq!(app.selected(), Some(0));
    }

    #[test]
    fn move_down_increments() {
        let candidates = make_candidates(&["a", "b", "c"]);
        let mut app = App::new(candidates, "".to_string(), 5, 10);
        app.move_down();
        app.move_down();
        assert_eq!(app.selected(), Some(1));
    }

    #[test]
    fn move_down_wraps() {
        let candidates = make_candidates(&["a", "b", "c"]);
        let mut app = App::new(candidates, "".to_string(), 5, 10);
        app.set_selected(app.filtered_indices.len() - 1);
        app.move_down();
        assert_eq!(app.selected(), Some(0));
    }

    #[test]
    fn move_down_empty_noop() {
        let mut app = App::new(Vec::new(), "".to_string(), 5, 10);
        app.move_down();
        assert_eq!(app.selected(), None);
    }

    // --- move_up ---

    #[test]
    fn move_up_from_none_selects_last() {
        let candidates = make_candidates(&["a", "b", "c"]);
        let mut app = App::new(candidates, "".to_string(), 5, 10);
        assert_eq!(app.selected(), None);
        app.move_up();
        assert_eq!(app.selected(), Some(app.filtered_indices.len() - 1));
    }

    #[test]
    fn move_up_decrements() {
        let candidates = make_candidates(&["a", "b", "c"]);
        let mut app = App::new(candidates, "".to_string(), 5, 10);
        app.set_selected(2);
        app.move_up();
        assert_eq!(app.selected(), Some(1));
    }

    #[test]
    fn move_up_wraps() {
        let candidates = make_candidates(&["a", "b", "c"]);
        let mut app = App::new(candidates, "".to_string(), 5, 10);
        app.move_up();
        assert_eq!(app.selected(), Some(app.filtered_indices.len() - 1));
    }

    #[test]
    fn move_up_empty_noop() {
        let mut app = App::new(Vec::new(), "".to_string(), 5, 10);
        app.move_up();
        assert_eq!(app.selected(), None);
    }

    // --- page_down / page_up ---

    const FIFTEEN_ITEMS: &[&str] = &[
        "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o",
    ];

    #[test]
    fn page_down_by_max_visible() {
        let mut app = App::new(make_candidates(FIFTEEN_ITEMS), "".to_string(), 5, 10);
        app.page_down();
        assert_eq!(app.selected(), Some(10));
    }

    #[test]
    fn page_down_clamps() {
        let mut app = App::new(make_candidates(FIFTEEN_ITEMS), "".to_string(), 5, 10);
        app.set_selected(10);
        app.page_down();
        assert_eq!(app.selected(), Some(14));
    }

    #[test]
    fn page_up_by_max_visible() {
        let mut app = App::new(make_candidates(FIFTEEN_ITEMS), "".to_string(), 5, 10);
        app.set_selected(14);
        app.page_up();
        assert_eq!(app.selected(), Some(4));
    }

    #[test]
    fn page_up_clamps() {
        let mut app = App::new(make_candidates(FIFTEEN_ITEMS), "".to_string(), 5, 10);
        app.set_selected(3);
        app.page_up();
        assert_eq!(app.selected(), Some(0));
    }

    // --- type_char / backspace ---

    #[test]
    fn type_char_narrows() {
        let candidates = make_candidates(&["alpha", "alpine", "zzz"]);
        let mut app = App::new(candidates, "".to_string(), 5, 10);
        let before = app.filtered_indices.len();
        app.type_char('a');
        app.type_char('l');
        app.type_char('p');
        app.type_char('i');
        assert!(app.filtered_indices.len() < before);
    }

    #[test]
    fn backspace_widens() {
        let candidates = make_candidates(&["alpha", "alpine", "zzz"]);
        let mut app = App::new(candidates, "".to_string(), 5, 10);
        app.type_char('a');
        app.type_char('l');
        app.type_char('p');
        app.type_char('i');
        let narrow = app.filtered_indices.len();
        app.backspace();
        assert!(app.filtered_indices.len() > narrow);
    }

    #[test]
    fn backspace_restores_cached_query_results() {
        let candidates = make_candidates(&["alpha", "alpine", "beta", "zzz"]);
        let mut app = App::new(candidates, "".to_string(), 5, 10);

        app.type_char('a');
        let a_results: Vec<String> = filtered_texts(&app)
            .into_iter()
            .map(str::to_string)
            .collect();
        app.type_char('l');
        assert!(app.cached_filters.contains_key("a"));

        app.set_selected(1);
        app.scroll_offset = 1;
        assert!(app.backspace());

        let restored: Vec<String> = filtered_texts(&app)
            .into_iter()
            .map(str::to_string)
            .collect();
        assert_eq!(restored, a_results);
        assert_eq!(app.selected(), None);
        assert_eq!(app.scroll_offset, 0);
    }

    #[test]
    fn backspace_cache_miss_falls_back_to_full_rescan() {
        let candidates = make_candidates(&["foobar", "foobaz"]);
        let mut app = App::new(candidates, "fo".to_string(), 5, 10);
        assert_eq!(app.filter_text, "fooba");
        assert!(!app.cached_filters.contains_key("foob"));

        assert!(app.backspace());
        assert_eq!(app.filter_text, "foob");
        assert!(app.cached_filters.contains_key("foob"));

        let mut matcher = FuzzyMatcher::new();
        let expected = matcher.filter(&app.all_candidates, &app.filter_text);
        let expected_texts: Vec<&str> =
            expected.iter().map(|r| r.candidate.text.as_str()).collect();
        assert_eq!(filtered_texts(&app), expected_texts);
    }

    #[test]
    fn backspace_restores_empty_query_from_cache() {
        let candidates = make_candidates(&["add", "bisect", "clone"]);
        let mut app = App::new(candidates, "".to_string(), 5, 10);
        let initial: Vec<String> = filtered_texts(&app)
            .into_iter()
            .map(str::to_string)
            .collect();
        assert!(app.cached_filters.contains_key(""));

        app.type_char('a');
        app.set_selected(1);
        app.scroll_offset = 1;
        assert!(app.backspace());

        assert_eq!(app.filter_text, "");
        let restored: Vec<String> = filtered_texts(&app)
            .into_iter()
            .map(str::to_string)
            .collect();
        assert_eq!(restored, initial);
        assert_eq!(app.selected(), None);
        assert_eq!(app.scroll_offset, 0);
    }

    #[test]
    fn type_char_incremental_matches_full_rescan() {
        let candidates = make_candidates(&["alpha", "alpine", "beta", "zzz"]);
        let mut app = App::new(candidates, "".to_string(), 5, 10);

        app.type_char('a');
        app.type_char('l');
        app.type_char('p');

        let mut matcher = FuzzyMatcher::new();
        let expected = matcher.filter(&app.all_candidates, &app.filter_text);
        let expected_texts: Vec<&str> =
            expected.iter().map(|r| r.candidate.text.as_str()).collect();

        assert_eq!(filtered_texts(&app), expected_texts);
    }

    #[test]
    fn type_char_resets_selection() {
        let candidates = make_candidates(&["alpha", "alpine", "zzz"]);
        let mut app = App::new(candidates, "".to_string(), 5, 10);
        app.move_down();
        app.move_down();
        assert!(app.selected().is_some());
        app.type_char('a');
        assert_eq!(app.selected(), None);
        assert_eq!(app.scroll_offset, 0);
    }

    #[test]
    fn move_down_after_filter_selects_top_filtered_match() {
        let candidates = make_candidates(&["ab", "ax", "b"]);
        let mut app = App::new(candidates, "".to_string(), 5, 10);

        app.type_char('a');
        assert_eq!(app.selected(), None);

        app.move_down();
        assert_eq!(app.selected(), Some(0));
        assert_eq!(app.selected_candidate().unwrap().text, "ab");
    }

    // --- accessors ---

    #[test]
    fn selected_candidate_correct() {
        let candidates = make_candidates(&["alpha", "beta", "gamma"]);
        let mut app = App::new(candidates, "".to_string(), 5, 10);
        app.move_down();
        app.move_down();
        let selected = app.selected_candidate().unwrap();
        assert_eq!(selected.text, filtered_texts(&app)[1]);
    }

    #[test]
    fn selected_candidate_empty_none() {
        let app = App::new(Vec::new(), "".to_string(), 5, 10);
        assert!(app.selected_candidate().is_none());
    }

    #[test]
    fn visible_candidates_respects_scroll() {
        let mut app = App::new(make_candidates(FIFTEEN_ITEMS), "".to_string(), 5, 10);
        app.scroll_offset = 5;
        let visible: Vec<&str> = app
            .visible_candidate_indices()
            .iter()
            .map(|&candidate_idx| app.all_candidates[candidate_idx].text.as_str())
            .collect();
        assert_eq!(visible.len(), 10);
        assert_eq!(visible[0], "f");
    }

    #[test]
    fn visible_selected_index_offset() {
        let candidates = make_candidates(&["alpha", "beta", "gamma"]);
        let mut app = App::new(candidates, "".to_string(), 5, 10);
        app.set_selected(2);
        app.scroll_offset = 1;
        assert_eq!(app.visible_selected_index(), Some(1));
    }

    #[test]
    fn visible_selected_index_none_when_no_selection() {
        let candidates = make_candidates(&["alpha", "beta", "gamma"]);
        let app = App::new(candidates, "".to_string(), 5, 10);
        assert_eq!(app.visible_selected_index(), None);
    }

    // --- Backspace on empty filter (Tab-triggered popup regression) ---

    #[test]
    fn backspace_on_empty_filter_is_noop() {
        let candidates = make_candidates(&["add", "bisect", "clone"]);
        let mut app = App::new(candidates, "".to_string(), 5, 10);
        assert_eq!(app.filter_text, "");
        assert!(!app.backspace());
        assert_eq!(app.filter_text, "");
        assert_eq!(app.filtered_indices.len(), 3);
    }

    #[test]
    fn empty_prefix_empty_filter_break_conditions() {
        let candidates = make_candidates(&["add", "bisect", "clone"]);
        let app = App::new(candidates, "".to_string(), 5, 10);
        // Both break conditions in the old Backspace handler are false
        assert!(!app.filtered_indices.is_empty());
        assert!(app.filter_text.len() >= app.prefix.len());
    }

    #[test]
    fn backspace_below_prefix_triggers_break_condition() {
        let candidates = make_candidates(&["log", "ls"]);
        let mut app = App::new(candidates, "l".to_string(), 5, 10);
        assert_eq!(app.filter_text, "l");
        assert!(app.backspace());
        assert_eq!(app.filter_text, "");
        assert!(app.filter_text.len() < app.prefix.len());
    }

    #[test]
    fn set_term_size_clamps_cursor_to_new_bounds() {
        let candidates = make_candidates(&["alpha", "beta", "gamma"]);
        let mut app = App::new_with_term_size(candidates, "".to_string(), 20, 40, 80, 24);

        app.set_term_size(12, 6);

        assert_eq!(app.term_cols, 12);
        assert_eq!(app.term_rows, 6);
        assert_eq!(app.cursor_col, 11);
        assert_eq!(app.cursor_row, 5);
    }

    #[test]
    fn set_term_size_keeps_selection_visible_after_shrink() {
        let mut app = App::new_with_term_size(
            make_candidates(FIFTEEN_ITEMS),
            "".to_string(),
            5,
            10,
            80,
            24,
        );
        app.set_selected(9);
        app.scroll_offset = 0;

        app.set_term_size(80, 6);

        assert_eq!(app.max_visible, 3);
        let sel = app.selected().unwrap();
        assert!(sel >= app.scroll_offset);
        assert!(sel < app.scroll_offset + app.max_visible);
    }

    #[test]
    fn new_with_term_size_clamps_zero_dimensions() {
        let app = App::new_with_term_size(
            make_candidates(&["alpha", "beta"]),
            "".to_string(),
            7,
            9,
            0,
            0,
        );

        assert_eq!(app.term_cols, 1);
        assert_eq!(app.term_rows, 1);
        assert_eq!(app.cursor_col, 0);
        assert_eq!(app.cursor_row, 0);
        assert_eq!(app.max_visible, 1);
    }

    fn make_test_app(prefix: &str, items: &[&str]) -> App {
        let candidates = make_candidates(items);
        App::new_with_term_size(candidates, prefix.to_string(), 5, 2, 80, 24)
    }

    #[test]
    fn set_selected_clamps_out_of_range() {
        let mut app = make_test_app("gi", &["git", "gist", "gizmo"]);
        app.set_selected(99);
        assert_eq!(app.selected(), None);
    }

    #[test]
    fn set_selected_adjusts_viewport() {
        let mut app = make_test_app("", &["a", "b", "c", "d", "e", "f", "g", "h"]);
        app.max_visible = 3;
        app.set_selected(5);
        assert_eq!(app.selected(), Some(5));
        assert!(app.scroll_offset <= 5);
        assert!(5 < app.scroll_offset + app.max_visible);
    }

    #[test]
    fn selected_original_idx_maps_to_all_candidates() {
        let mut app = make_test_app("gi", &["git", "gist", "gizmo"]);
        // filtered_indices[0] is "git" (shortest match), original index 0
        app.set_selected(0);
        let orig = app.selected_original_idx().unwrap();
        assert_eq!(app.all_candidates[orig].text, "git");
        // filtered_indices[1] is "gist", original index 1
        app.set_selected(1);
        let orig = app.selected_original_idx().unwrap();
        assert_eq!(app.all_candidates[orig].text, "gist");
    }

    #[test]
    fn selected_original_idx_none_when_no_selection() {
        let app = make_test_app("gi", &["git", "gist"]);
        assert_eq!(app.selected_original_idx(), None);
    }
}
