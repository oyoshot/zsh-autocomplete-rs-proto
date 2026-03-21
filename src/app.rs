use crate::candidate::Candidate;
use crate::fuzzy::FuzzyMatcher;

pub struct App {
    pub all_candidates: Vec<Candidate>,
    pub filtered: Vec<Candidate>,
    pub filter_text: String,
    pub selected: usize,
    pub scroll_offset: usize,
    pub max_visible: usize,
    pub cursor_row: u16,
    pub cursor_col: u16,
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
        let cursor_row = cursor_row.min(term_rows.saturating_sub(1));
        let cursor_col = cursor_col.min(term_cols.saturating_sub(1));

        let lcp = compute_common_prefix(&candidates, &prefix);
        let mut app = App {
            all_candidates: candidates,
            filtered: Vec::new(),
            filter_text: lcp,
            selected: 0,
            scroll_offset: 0,
            max_visible: 10,
            cursor_row,
            cursor_col,
            prefix,
            fuzzy,
        };
        app.update_filter();
        app
    }

    pub fn update_filter(&mut self) {
        let scored = self.fuzzy.filter(&self.all_candidates, &self.filter_text);
        self.filtered = scored.into_iter().map(|s| s.candidate).collect();
        self.selected = 0;
        self.scroll_offset = 0;
    }

    pub fn move_down(&mut self) {
        if self.filtered.is_empty() {
            return;
        }
        if self.selected + 1 < self.filtered.len() {
            self.selected += 1;
        } else {
            self.selected = 0;
            self.scroll_offset = 0;
        }
        if self.selected >= self.scroll_offset + self.max_visible {
            self.scroll_offset = self.selected + 1 - self.max_visible;
        }
    }

    pub fn move_up(&mut self) {
        if self.filtered.is_empty() {
            return;
        }
        if self.selected > 0 {
            self.selected -= 1;
        } else {
            self.selected = self.filtered.len() - 1;
            self.scroll_offset = self.selected.saturating_sub(self.max_visible - 1);
        }
        if self.selected < self.scroll_offset {
            self.scroll_offset = self.selected;
        }
    }

    pub fn page_down(&mut self) {
        if self.filtered.is_empty() {
            return;
        }
        self.selected = (self.selected + self.max_visible).min(self.filtered.len() - 1);
        if self.selected >= self.scroll_offset + self.max_visible {
            self.scroll_offset = self.selected + 1 - self.max_visible;
        }
    }

    pub fn page_up(&mut self) {
        if self.filtered.is_empty() {
            return;
        }
        self.selected = self.selected.saturating_sub(self.max_visible);
        if self.selected < self.scroll_offset {
            self.scroll_offset = self.selected;
        }
    }

    pub fn type_char(&mut self, c: char) {
        self.filter_text.push(c);
        self.update_filter();
    }

    pub fn backspace(&mut self) -> bool {
        if self.filter_text.is_empty() {
            return false;
        }
        self.filter_text.pop();
        self.update_filter();
        true
    }

    pub fn selected_candidate(&self) -> Option<&Candidate> {
        self.filtered.get(self.selected)
    }

    pub fn visible_candidates(&self) -> &[Candidate] {
        let end = (self.scroll_offset + self.max_visible).min(self.filtered.len());
        &self.filtered[self.scroll_offset..end]
    }

    pub fn visible_selected_index(&self) -> Option<usize> {
        if self.filtered.is_empty() {
            return None;
        }
        Some(self.selected - self.scroll_offset)
    }

    pub fn take_fuzzy(self) -> FuzzyMatcher {
        self.fuzzy
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
        assert_eq!(app.filtered.len(), 2);
        assert!(!app.filtered.iter().any(|c| c.text == "bar"));
    }

    #[test]
    fn new_empty_candidates() {
        let app = App::new(Vec::new(), "fo".to_string(), 5, 10);
        assert!(app.filtered.is_empty());
        assert_eq!(app.selected, 0);
    }

    // --- move_down ---

    #[test]
    fn move_down_increments() {
        let candidates = make_candidates(&["a", "b", "c"]);
        let mut app = App::new(candidates, "".to_string(), 5, 10);
        app.move_down();
        assert_eq!(app.selected, 1);
    }

    #[test]
    fn move_down_wraps() {
        let candidates = make_candidates(&["a", "b", "c"]);
        let mut app = App::new(candidates, "".to_string(), 5, 10);
        app.selected = app.filtered.len() - 1;
        app.move_down();
        assert_eq!(app.selected, 0);
    }

    #[test]
    fn move_down_empty_noop() {
        let mut app = App::new(Vec::new(), "".to_string(), 5, 10);
        app.move_down();
        assert_eq!(app.selected, 0);
    }

    // --- move_up ---

    #[test]
    fn move_up_decrements() {
        let candidates = make_candidates(&["a", "b", "c"]);
        let mut app = App::new(candidates, "".to_string(), 5, 10);
        app.selected = 2;
        app.move_up();
        assert_eq!(app.selected, 1);
    }

    #[test]
    fn move_up_wraps() {
        let candidates = make_candidates(&["a", "b", "c"]);
        let mut app = App::new(candidates, "".to_string(), 5, 10);
        app.move_up();
        assert_eq!(app.selected, app.filtered.len() - 1);
    }

    #[test]
    fn move_up_empty_noop() {
        let mut app = App::new(Vec::new(), "".to_string(), 5, 10);
        app.move_up();
        assert_eq!(app.selected, 0);
    }

    // --- page_down / page_up ---

    const FIFTEEN_ITEMS: &[&str] = &[
        "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o",
    ];

    #[test]
    fn page_down_by_max_visible() {
        let mut app = App::new(make_candidates(FIFTEEN_ITEMS), "".to_string(), 5, 10);
        app.page_down();
        assert_eq!(app.selected, 10);
    }

    #[test]
    fn page_down_clamps() {
        let mut app = App::new(make_candidates(FIFTEEN_ITEMS), "".to_string(), 5, 10);
        app.selected = 10;
        app.page_down();
        assert_eq!(app.selected, 14);
    }

    #[test]
    fn page_up_by_max_visible() {
        let mut app = App::new(make_candidates(FIFTEEN_ITEMS), "".to_string(), 5, 10);
        app.selected = 14;
        app.page_up();
        assert_eq!(app.selected, 4);
    }

    #[test]
    fn page_up_clamps() {
        let mut app = App::new(make_candidates(FIFTEEN_ITEMS), "".to_string(), 5, 10);
        app.selected = 3;
        app.page_up();
        assert_eq!(app.selected, 0);
    }

    // --- type_char / backspace ---

    #[test]
    fn type_char_narrows() {
        let candidates = make_candidates(&["alpha", "alpine", "zzz"]);
        let mut app = App::new(candidates, "".to_string(), 5, 10);
        let before = app.filtered.len();
        app.type_char('a');
        app.type_char('l');
        app.type_char('p');
        app.type_char('i');
        assert!(app.filtered.len() < before);
    }

    #[test]
    fn backspace_widens() {
        let candidates = make_candidates(&["alpha", "alpine", "zzz"]);
        let mut app = App::new(candidates, "".to_string(), 5, 10);
        app.type_char('a');
        app.type_char('l');
        app.type_char('p');
        app.type_char('i');
        let narrow = app.filtered.len();
        app.backspace();
        assert!(app.filtered.len() > narrow);
    }

    #[test]
    fn type_char_resets_selection() {
        let candidates = make_candidates(&["alpha", "alpine", "zzz"]);
        let mut app = App::new(candidates, "".to_string(), 5, 10);
        app.move_down();
        app.move_down();
        assert!(app.selected > 0);
        app.type_char('a');
        assert_eq!(app.selected, 0);
        assert_eq!(app.scroll_offset, 0);
    }

    // --- accessors ---

    #[test]
    fn selected_candidate_correct() {
        let candidates = make_candidates(&["alpha", "beta", "gamma"]);
        let mut app = App::new(candidates, "".to_string(), 5, 10);
        app.move_down();
        let selected = app.selected_candidate().unwrap();
        assert_eq!(selected.text, app.filtered[1].text);
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
        let visible = app.visible_candidates();
        assert_eq!(visible.len(), 10);
        assert_eq!(visible[0].text, "f");
    }

    #[test]
    fn visible_selected_index_offset() {
        let candidates = make_candidates(&["alpha", "beta", "gamma"]);
        let mut app = App::new(candidates, "".to_string(), 5, 10);
        app.selected = 2;
        app.scroll_offset = 1;
        assert_eq!(app.visible_selected_index(), Some(1));
    }

    // --- Backspace on empty filter (Tab-triggered popup regression) ---

    #[test]
    fn backspace_on_empty_filter_is_noop() {
        let candidates = make_candidates(&["add", "bisect", "clone"]);
        let mut app = App::new(candidates, "".to_string(), 5, 10);
        assert_eq!(app.filter_text, "");
        assert!(!app.backspace());
        assert_eq!(app.filter_text, "");
        assert_eq!(app.filtered.len(), 3);
    }

    #[test]
    fn empty_prefix_empty_filter_break_conditions() {
        let candidates = make_candidates(&["add", "bisect", "clone"]);
        let app = App::new(candidates, "".to_string(), 5, 10);
        // Both break conditions in the old Backspace handler are false
        assert!(!app.filtered.is_empty());
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
}
