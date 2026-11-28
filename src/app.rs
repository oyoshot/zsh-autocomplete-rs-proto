use crate::candidate::Candidate;
use crate::config::{Config, KindOrder};
use crate::fuzzy::{FuzzyMatcher, MatchMode};
use crate::ui::popup::Popup;
use std::collections::HashMap;

const DEFAULT_MAX_VISIBLE: usize = 10;

struct CachedFilter {
    indices: Vec<usize>,
    mode: MatchMode,
}

/// Selection and scroll position over a list of `len` rows, of which at most
/// `max_visible` are shown.
///
/// Every operation keeps `is_valid` true. Changing `len` requires `reset`
/// and changing `max_visible` requires `clamp`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Viewport {
    selected: Option<usize>,
    scroll_offset: usize,
}

impl Viewport {
    /// The window shows at least one row and never scrolls past the last
    /// full page. A selected row exists and lies inside the window; without
    /// one the window is at the top.
    #[cfg(test)]
    fn is_valid(self, len: usize, max_visible: usize) -> bool {
        max_visible >= 1
            && self.scroll_offset <= len.saturating_sub(max_visible)
            && match self.selected {
                None => self.scroll_offset == 0,
                Some(sel) => {
                    sel < len && sel >= self.scroll_offset && sel - self.scroll_offset < max_visible
                }
            }
    }

    fn reset(&mut self) {
        *self = Self::default();
    }

    fn move_down(&mut self, len: usize, max_visible: usize) {
        if len == 0 {
            return;
        }
        let sel = match self.selected {
            None => 0,
            Some(s) if s + 1 < len => s + 1,
            Some(_) => {
                self.scroll_offset = 0;
                0
            }
        };
        self.selected = Some(sel);
        if sel >= self.scroll_offset + max_visible {
            self.scroll_offset = sel + 1 - max_visible;
        }
    }

    fn move_up(&mut self, len: usize, max_visible: usize) {
        if len == 0 {
            return;
        }
        let sel = match self.selected {
            None | Some(0) => {
                let last = len - 1;
                self.scroll_offset = last.saturating_sub(max_visible.saturating_sub(1));
                last
            }
            Some(s) => s - 1,
        };
        self.selected = Some(sel);
        if sel < self.scroll_offset {
            self.scroll_offset = sel;
        }
    }

    fn page_down(&mut self, len: usize, max_visible: usize) {
        if len == 0 {
            return;
        }
        let cur = self.selected.unwrap_or(0);
        let sel = cur.saturating_add(max_visible).min(len - 1);
        self.selected = Some(sel);
        if sel >= self.scroll_offset + max_visible {
            self.scroll_offset = sel + 1 - max_visible;
        }
    }

    fn page_up(&mut self, len: usize, max_visible: usize) {
        if len == 0 {
            return;
        }
        let cur = self.selected.unwrap_or(0);
        let sel = cur.saturating_sub(max_visible);
        self.selected = Some(sel);
        if sel < self.scroll_offset {
            self.scroll_offset = sel;
        }
    }

    fn select_first(&mut self, len: usize) {
        self.selected = (len > 0).then_some(0);
        self.scroll_offset = 0;
    }

    fn select(&mut self, idx: usize, len: usize, max_visible: usize) {
        if idx >= len {
            return;
        }
        self.selected = Some(idx);
        if idx < self.scroll_offset {
            self.scroll_offset = idx;
        } else if idx >= self.scroll_offset + max_visible {
            self.scroll_offset = idx + 1 - max_visible;
        }
    }

    /// Restores validity after `len` or `max_visible` changed.
    fn clamp(&mut self, len: usize, max_visible: usize) {
        if len == 0 {
            self.reset();
            return;
        }

        self.scroll_offset = self.scroll_offset.min(len.saturating_sub(max_visible));

        if let Some(sel) = self.selected {
            let sel = sel.min(len - 1);
            self.selected = Some(sel);

            if sel < self.scroll_offset {
                self.scroll_offset = sel;
            }

            if sel >= self.scroll_offset + max_visible {
                self.scroll_offset = sel + 1 - max_visible;
            }
        }
    }

    fn visible_range(self, len: usize, max_visible: usize) -> std::ops::Range<usize> {
        self.scroll_offset..(self.scroll_offset + max_visible).min(len)
    }
}

/// The candidates for one completion request: native candidates plus the
/// abbreviations eligible at the cursor, ranked with the kind order for the
/// same position.
pub struct CompletionSet {
    candidates: Vec<Candidate>,
    kind_order: KindOrder,
}

impl CompletionSet {
    pub fn new(
        config: &Config,
        native_tsv: &str,
        command_position: bool,
        command_context: Option<&str>,
    ) -> Self {
        let mut candidates = Candidate::parse_lines_dedup(native_tsv);
        candidates.extend(
            config
                .abbreviations
                .iter()
                .filter(|abbr| abbr.is_available_at(command_position, command_context))
                .map(Candidate::from),
        );
        Self {
            candidates,
            kind_order: config.kind_order.at_position(command_position),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.candidates.is_empty()
    }

    /// Builds the popup state. `fuzzy` is reused for its cached matcher; its
    /// kind order is replaced by this set's position-resolved order.
    pub fn into_app(
        self,
        prefix: String,
        cursor_row: u16,
        cursor_col: u16,
        term_cols: u16,
        term_rows: u16,
        fuzzy: FuzzyMatcher,
    ) -> App {
        App::new_with_matcher(
            self.candidates,
            prefix,
            cursor_row,
            cursor_col,
            term_cols,
            term_rows,
            fuzzy.with_kind_order(self.kind_order),
        )
    }
}

pub struct App {
    pub all_candidates: Vec<Candidate>,
    filtered_indices: Vec<usize>,
    // Unbounded; typical session produces few unique queries so no eviction needed
    cached_filters: HashMap<String, CachedFilter>,
    match_mode: MatchMode,
    pub filter_text: String,
    viewport: Viewport,
    max_visible: usize,
    pub cursor_row: u16,
    pub cursor_col: u16,
    pub term_cols: u16,
    pub term_rows: u16,
    pub prefix: String,
    fuzzy: FuzzyMatcher,
}

impl App {
    pub(crate) fn kind_order(&self) -> crate::config::KindOrder {
        self.fuzzy.kind_order()
    }

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
            match_mode: MatchMode::default(),
            filter_text: lcp,
            viewport: Viewport::default(),
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
        let result =
            self.fuzzy
                .filter_with_mode(&self.all_candidates, &self.filter_text, fuzzy_scope);
        self.filtered_indices = result
            .matches
            .into_iter()
            .map(|s| s.candidate_idx)
            .collect();
        self.match_mode = result.mode;
        self.viewport.reset();
        self.cache_current_filter();
    }

    fn cache_current_filter(&mut self) {
        self.cached_filters.insert(
            self.filter_text.clone(),
            CachedFilter {
                indices: self.filtered_indices.clone(),
                mode: self.match_mode,
            },
        );
    }

    fn restore_cached_filter(&mut self) -> bool {
        let Some(cached) = self.cached_filters.get(&self.filter_text) else {
            return false;
        };
        self.filtered_indices = cached.indices.clone();
        self.match_mode = cached.mode;
        self.viewport.reset();
        true
    }

    pub(crate) fn match_mode(&self) -> MatchMode {
        self.match_mode
    }

    pub fn move_down(&mut self) {
        self.viewport
            .move_down(self.filtered_indices.len(), self.max_visible);
    }

    pub fn move_up(&mut self) {
        self.viewport
            .move_up(self.filtered_indices.len(), self.max_visible);
    }

    pub fn page_down(&mut self) {
        self.viewport
            .page_down(self.filtered_indices.len(), self.max_visible);
    }

    pub fn page_up(&mut self) {
        self.viewport
            .page_up(self.filtered_indices.len(), self.max_visible);
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
        self.viewport.select_first(self.filtered_indices.len());
    }

    pub fn selected(&self) -> Option<usize> {
        self.viewport.selected
    }

    /// Indices into `all_candidates` that match the filter, best first.
    pub fn filtered_indices(&self) -> &[usize] {
        &self.filtered_indices
    }

    pub fn scroll_offset(&self) -> usize {
        self.viewport.scroll_offset
    }

    pub fn max_visible(&self) -> usize {
        self.max_visible
    }

    pub fn set_selected(&mut self, idx: usize) {
        self.viewport
            .select(idx, self.filtered_indices.len(), self.max_visible);
    }

    /// Returns the index into `all_candidates` for the currently selected item.
    pub fn selected_original_idx(&self) -> Option<usize> {
        let sel = self.viewport.selected?;
        self.filtered_indices.get(sel).copied()
    }

    pub fn selected_candidate(&self) -> Option<&Candidate> {
        let sel = self.viewport.selected?;
        self.filtered_indices
            .get(sel)
            .and_then(|&candidate_idx| self.all_candidates.get(candidate_idx))
    }

    pub fn visible_candidate_indices(&self) -> &[usize] {
        let range = self
            .viewport
            .visible_range(self.filtered_indices.len(), self.max_visible);
        &self.filtered_indices[range]
    }

    pub fn visible_selected_index(&self) -> Option<usize> {
        let sel = self.viewport.selected?;
        Some(sel - self.viewport.scroll_offset)
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

    pub fn set_terminal_state(
        &mut self,
        cursor_row: u16,
        cursor_col: u16,
        term_cols: u16,
        term_rows: u16,
    ) {
        self.term_cols = term_cols.max(1);
        self.term_rows = term_rows.max(1);
        self.cursor_row = cursor_row.min(self.term_rows.saturating_sub(1));
        self.cursor_col = cursor_col.min(self.term_cols.saturating_sub(1));
        self.sync_max_visible();
    }

    pub fn sync_max_visible(&mut self) {
        // Stays at one row even when no popup fits; callers check
        // `popup_fits` before drawing.
        let max_visible = Popup::rows_available(self.term_rows).max(1);
        self.max_visible = DEFAULT_MAX_VISIBLE.min(max_visible);
        self.clamp_viewport();
    }

    /// Whether the terminal is large enough to show the popup at all.
    pub fn popup_fits(&self) -> bool {
        Popup::fits_terminal(self.term_cols, self.term_rows)
    }

    /// Lowers `max_visible` to at most `limit` rows, never below one, and
    /// keeps the selection visible.
    #[cfg(test)]
    pub(crate) fn cap_max_visible(&mut self, limit: usize) {
        self.max_visible = self.max_visible.min(limit.max(1));
        self.clamp_viewport();
    }

    fn clamp_viewport(&mut self) {
        self.viewport
            .clamp(self.filtered_indices.len(), self.max_visible);
    }

    /// Returns the extended common prefix if it is longer than the original prefix, else `None`.
    /// `filter_text` is initialized to the common prefix of all candidates at construction time.
    pub fn unambiguous_prefix(&self) -> Option<&str> {
        if self.filter_text.len() > self.prefix.len() {
            Some(&self.filter_text)
        } else {
            None
        }
    }

    /// Resets `filter_text` to the original typed prefix and re-runs filtering.
    /// No-op if `filter_text` already equals `prefix`.
    pub fn reset_filter_to_prefix(&mut self) {
        if self.filter_text != self.prefix {
            self.filter_text = self.prefix.clone();
            self.update_filter();
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

    #[test]
    fn completion_set_adds_eligible_abbreviations_ranked_for_that_position() {
        use crate::config::{Abbreviation, AbbreviationPosition, AbbreviationWhen};
        let abbreviation = |trigger: &str, position| Abbreviation {
            trigger: trigger.to_string(),
            expansion: format!("{trigger} expanded"),
            description: String::new(),
            when: AbbreviationWhen::at_position(position),
        };
        let mut config = Config::default();
        config.abbreviations = vec![
            abbreviation("cmd", AbbreviationPosition::Command),
            abbreviation("arg", AbbreviationPosition::Argument),
            abbreviation("any", AbbreviationPosition::Any),
        ];

        // Native subcommands arrive without a kind. At argument position they
        // rank before abbreviations; at command position abbreviations lead.
        for (command_position, expected) in [
            (true, ["any", "cmd", "status"]),
            (false, ["status", "any", "arg"]),
        ] {
            let app = CompletionSet::new(&config, "status\t\t\n", command_position, None).into_app(
                String::new(),
                5,
                0,
                80,
                24,
                FuzzyMatcher::new(),
            );
            let texts: Vec<&str> = app
                .filtered_indices()
                .iter()
                .map(|&i| app.all_candidates[i].text.as_str())
                .collect();
            assert_eq!(texts, expected, "command_position={command_position}");
        }
        assert!(CompletionSet::new(&Config::default(), "", false, None).is_empty());
    }

    fn make_candidates(items: &[&str]) -> Vec<Candidate> {
        items
            .iter()
            .map(|s| Candidate {
                text: s.to_string(),
                description: String::new(),
                kind: String::new(),
                insert_text: None,
                cursor_offset: None,
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
        app.viewport.scroll_offset = 1;
        assert!(app.backspace());

        let restored: Vec<String> = filtered_texts(&app)
            .into_iter()
            .map(str::to_string)
            .collect();
        assert_eq!(restored, a_results);
        assert_eq!(app.selected(), None);
        assert_eq!(app.scroll_offset(), 0);
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
        app.viewport.scroll_offset = 1;
        assert!(app.backspace());

        assert_eq!(app.filter_text, "");
        let restored: Vec<String> = filtered_texts(&app)
            .into_iter()
            .map(str::to_string)
            .collect();
        assert_eq!(restored, initial);
        assert_eq!(app.selected(), None);
        assert_eq!(app.scroll_offset(), 0);
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
        assert_eq!(app.scroll_offset(), 0);
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
        app.viewport.scroll_offset = 5;
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
        app.viewport.scroll_offset = 1;
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
        app.viewport.scroll_offset = 0;

        app.set_term_size(80, 6);

        assert_eq!(app.max_visible, 3);
        let sel = app.selected().unwrap();
        assert!(sel >= app.scroll_offset());
        assert!(sel < app.scroll_offset() + app.max_visible);
    }

    #[test]
    fn set_terminal_state_updates_cursor_for_resized_wrap() {
        let candidates = make_candidates(&["alpha", "beta", "gamma"]);
        let mut app = App::new_with_term_size(candidates, "".to_string(), 5, 35, 40, 24);

        app.set_terminal_state(2, 18, 80, 24);

        assert_eq!(app.term_cols, 80);
        assert_eq!(app.term_rows, 24);
        assert_eq!(app.cursor_row, 2);
        assert_eq!(app.cursor_col, 18);
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
        assert!(app.scroll_offset() <= 5);
        assert!(5 < app.scroll_offset() + app.max_visible);
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

    // --- unambiguous_prefix ---

    #[test]
    fn unambiguous_prefix_returns_some_when_filter_text_extends_prefix() {
        // "gi" typed; candidates share "git-" prefix → filter_text extended to "git-"
        let app = make_test_app("gi", &["git-log", "git-status", "git-diff"]);
        assert_eq!(app.unambiguous_prefix(), Some("git-"));
    }

    #[test]
    fn unambiguous_prefix_returns_none_when_no_extension() {
        // candidates only share the typed prefix itself
        let app = make_test_app("git", &["git-log", "git-status"]);
        assert_eq!(app.unambiguous_prefix(), Some("git-"));
        // when filter_text == prefix exactly
        let app2 = make_test_app("abc", &["abcx", "abcy"]);
        // common prefix is "abc" (== typed prefix), so None
        assert_eq!(app2.unambiguous_prefix(), None);
    }

    #[test]
    fn unambiguous_prefix_returns_none_for_empty_candidates() {
        let app = make_test_app("gi", &[]);
        assert_eq!(app.unambiguous_prefix(), None);
    }

    #[test]
    fn unambiguous_prefix_returns_none_for_single_candidate_equal_prefix() {
        let app = make_test_app("git-log", &["git-log"]);
        assert_eq!(app.unambiguous_prefix(), None);
    }

    #[test]
    fn unambiguous_prefix_returns_some_for_single_candidate_longer_than_prefix() {
        let app = make_test_app("gi", &["git-log"]);
        assert_eq!(app.unambiguous_prefix(), Some("git-log"));
    }

    // --- reset_filter_to_prefix ---

    #[test]
    fn reset_filter_to_prefix_noop_when_already_equal() {
        let mut app = make_test_app("abc", &["abcx", "abcy"]);
        // filter_text == prefix ("abc"), so reset is a no-op
        let before = app.filtered_indices.clone();
        app.reset_filter_to_prefix();
        assert_eq!(app.filter_text, "abc");
        assert_eq!(app.filtered_indices, before);
    }

    #[test]
    fn reset_filter_to_prefix_restores_and_refilters() {
        // prefix "g"; "grep" prevents common prefix from extending past "g".
        // type_char('i') narrows to 2 matches ("git-log", "gist").
        // reset_filter_to_prefix() must restore filter_text to "g" and re-expand to 3 matches.
        let mut app = make_test_app("g", &["git-log", "gist", "grep"]);
        app.type_char('i');
        assert_eq!(app.filter_text, "gi");
        assert_eq!(app.filtered_indices.len(), 2); // git-log, gist
        app.reset_filter_to_prefix();
        assert_eq!(app.filter_text, "g");
        assert_eq!(app.filtered_indices.len(), 3); // git-log, gist, grep
    }

    // Kani checks every `usize`. The randomized run draws small values so
    // that short lists and windows near the list length come up.
    #[cfg(kani)]
    type Dim = usize;
    #[cfg(not(kani))]
    type Dim = u8;

    const VIEWPORT_OPS: u8 = 8;

    fn apply_viewport_op(
        viewport: &mut Viewport,
        op: u8,
        idx: usize,
        len: usize,
        max_visible: usize,
    ) {
        match op % VIEWPORT_OPS {
            0 => viewport.move_down(len, max_visible),
            1 => viewport.move_up(len, max_visible),
            2 => viewport.page_down(len, max_visible),
            3 => viewport.page_up(len, max_visible),
            4 => viewport.select_first(len),
            5 => viewport.select(idx, len, max_visible),
            6 => viewport.clamp(len, max_visible),
            _ => viewport.reset(),
        }
    }

    /// Inductive step: from any valid state, every operation yields a valid
    /// state. Runs as a randomized test under `cargo test` and as a proof
    /// over every `usize` under `cargo kani --tests`.
    #[test]
    #[cfg_attr(kani, kani::proof)]
    fn viewport_operations_preserve_validity() {
        bolero::check!()
            .with_type::<(Option<Dim>, Dim, Dim, Dim, u8, Dim)>()
            .cloned()
            .for_each(|(selected, scroll_offset, len, max_visible, op, idx)| {
                let (len, max_visible) = (usize::from(len), usize::from(max_visible));
                let mut viewport = Viewport {
                    selected: selected.map(usize::from),
                    scroll_offset: usize::from(scroll_offset),
                };
                // Few random draws are valid states, so the randomized run
                // repairs its draw first. Kani keeps the raw draw: there the
                // check below alone decides which states are covered.
                #[cfg(not(kani))]
                {
                    if viewport.selected.is_none() {
                        viewport.scroll_offset = 0;
                    }
                    viewport.clamp(len, max_visible.max(1));
                }
                if !viewport.is_valid(len, max_visible) {
                    return;
                }

                apply_viewport_op(&mut viewport, op, usize::from(idx), len, max_visible);

                assert!(viewport.is_valid(len, max_visible));
                let visible = viewport.visible_range(len, max_visible);
                assert_eq!(visible.len(), len.min(max_visible));
                assert!(visible.end <= len);
            });
    }

    /// `clamp` repairs a state for any new list length and window size,
    /// keeps the selection on the same row or the last row if that row is
    /// gone, and leaves an already valid state untouched. The window must
    /// show at least one row. The state may be arbitrary
    /// except that an unselected window is at the top, which `clamp` keeps
    /// but does not establish.
    #[test]
    #[cfg_attr(kani, kani::proof)]
    fn viewport_clamp_restores_validity_after_any_resize() {
        bolero::check!()
            .with_type::<(Option<Dim>, Dim, Dim, Dim)>()
            .cloned()
            .for_each(|(selected, scroll_offset, len, max_visible)| {
                let (len, max_visible) = (usize::from(len), usize::from(max_visible));
                let before = Viewport {
                    selected: selected.map(usize::from),
                    scroll_offset: usize::from(scroll_offset),
                };
                if max_visible == 0 || (before.selected.is_none() && before.scroll_offset != 0) {
                    return;
                }

                let mut after = before;
                after.clamp(len, max_visible);

                assert!(after.is_valid(len, max_visible));
                let kept = before.selected.filter(|_| len > 0);
                assert_eq!(after.selected, kept.map(|sel| sel.min(len - 1)));
                if before.is_valid(len, max_visible) {
                    assert_eq!(after, before);
                }
            });
    }

    /// Randomized only: drives one viewport through a sequence of
    /// operations, list changes and resizes, as `App` does.
    #[test]
    fn viewport_stays_valid_across_operation_sequences() {
        bolero::check!()
            .with_type::<(u8, u8, Vec<(u8, u8)>)>()
            .for_each(|(len, max_visible, steps)| {
                let mut len = usize::from(*len);
                let mut max_visible = usize::from(*max_visible).max(1);
                let mut viewport = Viewport::default();

                for &(op, arg) in steps {
                    let arg = usize::from(arg);
                    match op % (VIEWPORT_OPS + 2) {
                        op if op < VIEWPORT_OPS => {
                            apply_viewport_op(&mut viewport, op, arg, len, max_visible);
                        }
                        VIEWPORT_OPS => {
                            len = arg;
                            viewport.reset();
                        }
                        _ => {
                            max_visible = arg.max(1);
                            viewport.clamp(len, max_visible);
                        }
                    }
                    assert!(viewport.is_valid(len, max_visible));
                }
            });
    }
}
