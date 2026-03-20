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
        // Clamp cursor position to terminal bounds (safety net)
        let (term_cols, term_rows) = crossterm::terminal::size().unwrap_or((80, 24));
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
            fuzzy: FuzzyMatcher::new(),
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

    pub fn backspace(&mut self) {
        self.filter_text.pop();
        self.update_filter();
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
