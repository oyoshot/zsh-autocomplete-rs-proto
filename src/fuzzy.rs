use std::cmp::Ordering;
use std::collections::HashSet;

use crate::candidate::Candidate;
use nucleo_matcher::pattern::{AtomKind, CaseMatching, Normalization, Pattern};
use nucleo_matcher::{Config, Matcher, Utf32Str};

pub struct FuzzyMatcher {
    matcher: Matcher,
    pattern: Pattern,
    exact_pattern: Pattern,
    last_query: String,
    utf32_buf: Vec<char>,
    dl_scratch: DamerauScratch,
}

pub struct ScoredCandidate {
    pub candidate: Candidate,
    pub score: u32,
}

pub struct ScoredMatch {
    pub candidate_idx: usize,
    pub score: u32,
}

#[derive(Default)]
struct DamerauScratch {
    query_chars: Vec<char>,
    candidate_chars: Vec<char>,
    prev_prev: Vec<usize>,
    prev: Vec<usize>,
    curr: Vec<usize>,
}

impl Default for FuzzyMatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl FuzzyMatcher {
    pub fn new() -> Self {
        Self {
            matcher: Matcher::new(Config::DEFAULT),
            pattern: Pattern::new(
                "",
                CaseMatching::Smart,
                Normalization::Smart,
                AtomKind::Fuzzy,
            ),
            exact_pattern: Pattern::new(
                "",
                CaseMatching::Smart,
                Normalization::Smart,
                AtomKind::Exact,
            ),
            last_query: String::new(),
            utf32_buf: Vec::new(),
            dl_scratch: DamerauScratch::default(),
        }
    }

    fn ensure_pattern(&mut self, query: &str) {
        if self.last_query != query {
            self.pattern = Pattern::new(
                query,
                CaseMatching::Smart,
                Normalization::Smart,
                AtomKind::Fuzzy,
            );
            self.exact_pattern = Pattern::new(
                query,
                CaseMatching::Smart,
                Normalization::Smart,
                AtomKind::Exact,
            );
            self.last_query.clear();
            self.last_query.push_str(query);
        }
    }

    pub fn filter_matches(
        &mut self,
        candidates: &[Candidate],
        query: &str,
        fuzzy_scope: Option<&[usize]>,
    ) -> Vec<ScoredMatch> {
        if query.is_empty() {
            let mut results: Vec<ScoredMatch> = candidates
                .iter()
                .enumerate()
                .map(|(candidate_idx, _)| ScoredMatch {
                    candidate_idx,
                    score: 0,
                })
                .collect();
            sort_empty_query_matches(&mut results, candidates);
            return results;
        }

        self.ensure_pattern(query);

        let pattern = &self.pattern;
        let exact_pattern = &self.exact_pattern;
        let matcher = &mut self.matcher;
        let utf32_buf = &mut self.utf32_buf;
        let mut results: Vec<ScoredMatch> =
            Vec::with_capacity(fuzzy_scope.map_or(candidates.len(), <[usize]>::len));
        let mut has_exact_match = false;

        if let Some(scope) = fuzzy_scope {
            for &candidate_idx in scope {
                let candidate = &candidates[candidate_idx];
                utf32_buf.clear();
                let haystack = Utf32Str::new(&candidate.text, utf32_buf);
                has_exact_match |= exact_pattern.score(haystack, matcher).is_some();
                if let Some(score) = pattern.score(haystack, matcher) {
                    results.push(ScoredMatch {
                        candidate_idx,
                        score,
                    });
                }
            }
        } else {
            for (candidate_idx, candidate) in candidates.iter().enumerate() {
                utf32_buf.clear();
                let haystack = Utf32Str::new(&candidate.text, utf32_buf);
                has_exact_match |= exact_pattern.score(haystack, matcher).is_some();
                if let Some(score) = pattern.score(haystack, matcher) {
                    results.push(ScoredMatch {
                        candidate_idx,
                        score,
                    });
                }
            }
        }

        if query.len() >= 2 && !has_exact_match {
            let dl_results = self.damerau_levenshtein_fallback_matches(candidates, query);
            if !dl_results.is_empty() {
                let seen: HashSet<usize> = results.iter().map(|r| r.candidate_idx).collect();
                let novel: Vec<ScoredMatch> = dl_results
                    .into_iter()
                    .filter(|r| !seen.contains(&r.candidate_idx))
                    .collect();
                results.extend(novel);
            }
        }

        sort_scored_matches(&mut results, candidates);
        results
    }

    pub fn filter(&mut self, candidates: &[Candidate], query: &str) -> Vec<ScoredCandidate> {
        if query.is_empty() {
            let mut results: Vec<ScoredCandidate> = candidates
                .iter()
                .map(|candidate| ScoredCandidate {
                    candidate: candidate.clone(),
                    score: 0,
                })
                .collect();
            sort_empty_query_results(&mut results);
            return results;
        }

        self.ensure_pattern(query);

        let pattern = &self.pattern;
        let exact_pattern = &self.exact_pattern;
        let matcher = &mut self.matcher;
        let utf32_buf = &mut self.utf32_buf;
        let mut results = Vec::with_capacity(candidates.len());
        let mut has_exact_match = false;
        for candidate in candidates {
            utf32_buf.clear();
            let haystack = Utf32Str::new(&candidate.text, utf32_buf);
            has_exact_match |= exact_pattern.score(haystack, matcher).is_some();
            if let Some(score) = pattern.score(haystack, matcher) {
                results.push(ScoredCandidate {
                    candidate: candidate.clone(),
                    score,
                });
            }
        }

        if query.len() >= 2 && !has_exact_match {
            let dl_results = self.damerau_levenshtein_fallback_candidates(candidates, query);
            if !dl_results.is_empty() {
                let seen: HashSet<&str> = results
                    .iter()
                    .map(|result| result.candidate.text.as_str())
                    .collect();
                let novel: Vec<ScoredCandidate> = dl_results
                    .into_iter()
                    .filter(|result| !seen.contains(result.candidate.text.as_str()))
                    .collect();
                results.extend(novel);
            }
        }

        sort_scored_results(&mut results);
        results
    }

    fn damerau_levenshtein_fallback_matches(
        &mut self,
        candidates: &[Candidate],
        query: &str,
    ) -> Vec<ScoredMatch> {
        let max_dist = if query.len() <= 4 { 1 } else { 2 };
        let case_insensitive = !query.chars().any(|c| c.is_uppercase());
        self.dl_scratch.set_query(query);

        let mut results = Vec::new();
        for (candidate_idx, candidate) in candidates.iter().enumerate() {
            if query.len().abs_diff(candidate.text.len()) > max_dist {
                continue;
            }

            self.dl_scratch.set_candidate(&candidate.text);
            let dist = self.dl_scratch.distance(case_insensitive, Some(max_dist));
            if dist <= max_dist {
                let score = dl_match_score(query.len(), candidate.text.len(), dist);
                results.push(ScoredMatch {
                    candidate_idx,
                    score,
                });
            }
        }

        results
    }

    fn damerau_levenshtein_fallback_candidates(
        &mut self,
        candidates: &[Candidate],
        query: &str,
    ) -> Vec<ScoredCandidate> {
        let max_dist = if query.len() <= 4 { 1 } else { 2 };
        let case_insensitive = !query.chars().any(|c| c.is_uppercase());
        self.dl_scratch.set_query(query);

        let mut results = Vec::new();
        for candidate in candidates {
            if query.len().abs_diff(candidate.text.len()) > max_dist {
                continue;
            }

            self.dl_scratch.set_candidate(&candidate.text);
            let dist = self.dl_scratch.distance(case_insensitive, Some(max_dist));
            if dist <= max_dist {
                let score = dl_match_score(query.len(), candidate.text.len(), dist);
                results.push(ScoredCandidate {
                    candidate: candidate.clone(),
                    score,
                });
            }
        }

        results
    }
}

fn dl_match_score(query_len: usize, candidate_len: usize, dist: usize) -> u32 {
    let len_gap = query_len.abs_diff(candidate_len) as u32;
    200u32.saturating_sub(dist as u32 * 30 + len_gap * 10)
}

impl DamerauScratch {
    fn set_query(&mut self, query: &str) {
        self.query_chars.clear();
        self.query_chars.extend(query.chars());
    }

    fn set_candidate(&mut self, candidate: &str) {
        self.candidate_chars.clear();
        self.candidate_chars.extend(candidate.chars());
    }

    fn distance(&mut self, case_insensitive: bool, max_dist: Option<usize>) -> usize {
        let query_chars = &self.query_chars;
        let candidate_chars = &self.candidate_chars;
        let prev_prev = &mut self.prev_prev;
        let prev = &mut self.prev;
        let curr = &mut self.curr;
        damerau_levenshtein_chars(
            query_chars,
            candidate_chars,
            case_insensitive,
            max_dist,
            prev_prev,
            prev,
            curr,
        )
    }
}

fn compare_empty_candidates(a: &Candidate, b: &Candidate) -> Ordering {
    a.kind_priority()
        .cmp(&b.kind_priority())
        .then_with(|| a.text.len().cmp(&b.text.len()))
        .then_with(|| a.text.cmp(&b.text))
}

fn compare_scored_candidates(a: &Candidate, a_score: u32, b: &Candidate, b_score: u32) -> Ordering {
    b_score
        .cmp(&a_score)
        .then_with(|| a.text.len().cmp(&b.text.len()))
        .then_with(|| a.kind_priority().cmp(&b.kind_priority()))
        .then_with(|| a.text.cmp(&b.text))
}

fn sort_empty_query_results(results: &mut [ScoredCandidate]) {
    results.sort_unstable_by(|a, b| compare_empty_candidates(&a.candidate, &b.candidate));
}

fn sort_scored_results(results: &mut [ScoredCandidate]) {
    results.sort_unstable_by(|a, b| {
        compare_scored_candidates(&a.candidate, a.score, &b.candidate, b.score)
    });
}

fn sort_empty_query_matches(results: &mut [ScoredMatch], candidates: &[Candidate]) {
    results.sort_unstable_by(|a, b| {
        compare_empty_candidates(&candidates[a.candidate_idx], &candidates[b.candidate_idx])
    });
}

fn sort_scored_matches(results: &mut [ScoredMatch], candidates: &[Candidate]) {
    results.sort_unstable_by(|a, b| {
        compare_scored_candidates(
            &candidates[a.candidate_idx],
            a.score,
            &candidates[b.candidate_idx],
            b.score,
        )
    });
}

fn damerau_levenshtein_chars(
    a: &[char],
    b: &[char],
    case_insensitive: bool,
    max_dist: Option<usize>,
    prev_prev: &mut Vec<usize>,
    prev: &mut Vec<usize>,
    curr: &mut Vec<usize>,
) -> usize {
    let len_a = a.len();
    let len_b = b.len();
    let row_len = len_b + 1;
    let unreachable = max_dist.unwrap_or(len_a + len_b).saturating_add(1);

    ensure_row_capacity(prev_prev, row_len);
    ensure_row_capacity(prev, row_len);
    ensure_row_capacity(curr, row_len);

    prev_prev[..row_len].fill(unreachable);
    prev[..row_len].fill(unreachable);
    curr[..row_len].fill(unreachable);

    prev[0] = 0;
    let initial_end = max_dist.map_or(len_b, |dist| len_b.min(dist));
    for (j, slot) in prev.iter_mut().enumerate().take(initial_end + 1).skip(1) {
        *slot = j;
    }

    let eq = |x: char, y: char| -> bool {
        if case_insensitive {
            x.eq_ignore_ascii_case(&y)
        } else {
            x == y
        }
    };

    for i in 1..=len_a {
        curr[..row_len].fill(unreachable);
        if max_dist.is_none_or(|dist| i <= dist) {
            curr[0] = i;
        }

        let start = max_dist.map_or(1, |dist| i.saturating_sub(dist).max(1));
        let end = max_dist.map_or(len_b, |dist| len_b.min(i + dist));
        if start > end {
            return unreachable;
        }

        let mut row_min = unreachable;
        for j in start..=end {
            let cost = if eq(a[i - 1], b[j - 1]) { 0 } else { 1 };
            let mut value = prev[j]
                .saturating_add(1)
                .min(curr[j - 1].saturating_add(1))
                .min(prev[j - 1].saturating_add(cost));
            if i > 1 && j > 1 && eq(a[i - 1], b[j - 2]) && eq(a[i - 2], b[j - 1]) {
                value = value.min(prev_prev[j - 2].saturating_add(cost));
            }
            curr[j] = value;
            row_min = row_min.min(value);
        }

        if max_dist.is_some_and(|dist| row_min > dist) {
            return unreachable;
        }

        std::mem::swap(prev_prev, prev);
        std::mem::swap(prev, curr);
    }

    let dist = prev[len_b];
    if max_dist.is_some_and(|limit| dist > limit) {
        unreachable
    } else {
        dist
    }
}

fn ensure_row_capacity(row: &mut Vec<usize>, len: usize) {
    if row.len() < len {
        row.resize(len, 0);
    }
}

pub fn damerau_levenshtein(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut prev_prev = Vec::new();
    let mut prev = Vec::new();
    let mut curr = Vec::new();
    damerau_levenshtein_chars(&a, &b, false, None, &mut prev_prev, &mut prev, &mut curr)
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn empty_query_returns_all() {
        let mut m = FuzzyMatcher::new();
        let candidates = make_candidates(&["foo", "bar", "baz"]);
        let results = m.filter(&candidates, "");
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn fuzzy_match_filters() {
        let mut m = FuzzyMatcher::new();
        let candidates = make_candidates(&["foo", "foobar", "bar", "baz"]);
        let results = m.filter(&candidates, "foo");
        assert!(results.iter().all(|r| r.candidate.text.contains("foo")));
        assert!(!results.is_empty());
    }

    #[test]
    fn no_match_returns_empty() {
        let mut m = FuzzyMatcher::new();
        let candidates = make_candidates(&["foo", "bar"]);
        let results = m.filter(&candidates, "zzz");
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn damerau_levenshtein_transposition() {
        let mut m = FuzzyMatcher::new();
        let candidates = make_candidates(&["git", "grep", "gzip"]);
        let results = m.filter(&candidates, "gti");
        let texts: Vec<&str> = results.iter().map(|r| r.candidate.text.as_str()).collect();
        assert!(texts.contains(&"git"), "gti should match git: {texts:?}");
    }

    #[test]
    fn damerau_levenshtein_substitution() {
        let mut m = FuzzyMatcher::new();
        let candidates = make_candidates(&["cargo", "cat", "curl"]);
        let results = m.filter(&candidates, "carog");
        let texts: Vec<&str> = results.iter().map(|r| r.candidate.text.as_str()).collect();
        assert!(
            texts.contains(&"cargo"),
            "carog should match cargo: {texts:?}"
        );
    }

    #[test]
    fn damerau_levenshtein_respects_threshold() {
        let mut m = FuzzyMatcher::new();
        let candidates = make_candidates(&["git", "grep", "gzip"]);
        // "xyz" is too far from any candidate
        let results = m.filter(&candidates, "xyz");
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn damerau_levenshtein_distance_basic() {
        assert_eq!(damerau_levenshtein("git", "gti"), 1);
        assert_eq!(damerau_levenshtein("cargo", "carog"), 1);
        assert_eq!(damerau_levenshtein("cargo", "garco"), 2);
        assert_eq!(damerau_levenshtein("git", "git"), 0);
    }

    fn make_candidates_with_kind(items: &[(&str, &str)]) -> Vec<Candidate> {
        items
            .iter()
            .map(|(text, kind)| Candidate {
                text: text.to_string(),
                description: String::new(),
                kind: kind.to_string(),
            })
            .collect()
    }

    #[test]
    fn short_options_before_long_options() {
        let mut m = FuzzyMatcher::new();
        let candidates = make_candidates(&["--release", "--verbose", "-j", "-v"]);
        let results = m.filter(&candidates, "-");
        let texts: Vec<&str> = results.iter().map(|r| r.candidate.text.as_str()).collect();
        // Short options (-j, -v) should come before long options (--release, --verbose)
        let first_long = texts.iter().position(|t| t.starts_with("--")).unwrap();
        let last_short = texts.iter().rposition(|t| !t.starts_with("--")).unwrap();
        assert!(
            last_short < first_long,
            "short options should precede long: {texts:?}"
        );
    }

    #[test]
    fn shorter_command_preferred() {
        let mut m = FuzzyMatcher::new();
        let candidates = make_candidates(&["gi-compile-repository", "git", "gitk"]);
        let results = m.filter(&candidates, "gi");
        let texts: Vec<&str> = results.iter().map(|r| r.candidate.text.as_str()).collect();
        assert_eq!(texts[0], "git", "git should be first: {texts:?}");
    }

    #[test]
    fn empty_query_kind_then_len_sort() {
        let mut m = FuzzyMatcher::new();
        let candidates = make_candidates_with_kind(&[
            ("longcmd", "command"),
            ("ab", "command"),
            ("z", "file"),
            ("abc", "command"),
        ]);
        let results = m.filter(&candidates, "");
        let texts: Vec<&str> = results.iter().map(|r| r.candidate.text.as_str()).collect();
        // file (priority 1) before command (priority 2), then by len, then alphabetical
        assert_eq!(texts[0], "z");
        assert_eq!(texts[1], "ab");
        assert_eq!(texts[2], "abc");
        assert_eq!(texts[3], "longcmd");
    }

    #[test]
    fn typo_found_alongside_nucleo_matches() {
        let mut m = FuzzyMatcher::new();
        let candidates = make_candidates(&["claude", "cat", "curl", "clear", "clone"]);
        let results = m.filter(&candidates, "calude");
        let texts: Vec<&str> = results.iter().map(|r| r.candidate.text.as_str()).collect();
        assert!(
            texts.contains(&"claude"),
            "calude should match claude via DL: {texts:?}"
        );
    }

    #[test]
    fn dl_results_appear_after_nucleo() {
        let mut m = FuzzyMatcher::new();
        // "calculated" matches nucleo subsequence c-a-l-u-...-d; "claude" only matches DL
        let candidates = make_candidates(&["calculated", "claude"]);
        let results = m.filter(&candidates, "calude");
        let texts: Vec<&str> = results.iter().map(|r| r.candidate.text.as_str()).collect();
        if texts.contains(&"calculated") && texts.contains(&"claude") {
            let pos_calc = texts.iter().position(|t| *t == "calculated").unwrap();
            let pos_claude = texts.iter().position(|t| *t == "claude").unwrap();
            assert!(
                pos_calc < pos_claude,
                "nucleo match should precede DL match: {texts:?}"
            );
        }
    }

    #[test]
    fn dl_results_are_globally_resorted_with_nucleo() {
        let mut m = FuzzyMatcher::new();
        let candidates = make_candidates(&["ca-l-u-d-e-helper", "claude"]);
        let results = m.filter(&candidates, "calude");
        let texts: Vec<&str> = results.iter().map(|r| r.candidate.text.as_str()).collect();

        assert_eq!(
            texts.first().copied(),
            Some("claude"),
            "DL typo candidate should be re-ranked ahead of weaker fuzzy matches: {texts:?}"
        );
    }

    #[test]
    fn dl_smart_case_insensitive() {
        let mut m = FuzzyMatcher::new();
        let candidates = make_candidates(&["Claude", "cat"]);
        let results = m.filter(&candidates, "calude");
        let texts: Vec<&str> = results.iter().map(|r| r.candidate.text.as_str()).collect();
        assert!(
            texts.contains(&"Claude"),
            "lowercase query should match Claude case-insensitively: {texts:?}"
        );
    }

    #[test]
    fn dl_smart_case_sensitive() {
        let mut m = FuzzyMatcher::new();
        // Short query (len<=4) → max_dist=1, so case diff + transposition = distance 2 is rejected
        let candidates = make_candidates(&["Git", "git"]);
        let results = m.filter(&candidates, "Gti");
        let texts: Vec<&str> = results.iter().map(|r| r.candidate.text.as_str()).collect();
        assert!(
            texts.contains(&"Git"),
            "uppercase query should match Git: {texts:?}"
        );
        assert!(
            !texts.contains(&"git"),
            "uppercase query should not match lowercase git (distance 2 > max_dist 1): {texts:?}"
        );
    }

    #[test]
    fn dl_deduplication() {
        let mut m = FuzzyMatcher::new();
        // "git" is matched by both nucleo (subsequence) and DL (distance 1 from "gti")
        let candidates = make_candidates(&["git", "grep", "gzip"]);
        let results = m.filter(&candidates, "gti");
        let git_count = results.iter().filter(|r| r.candidate.text == "git").count();
        assert_eq!(git_count, 1, "git should appear exactly once");
    }

    #[test]
    fn exact_match_skips_dl_near_miss_candidates() {
        let mut m = FuzzyMatcher::new();
        let candidates = make_candidates(&["cargo-build", "cargo-builc"]);
        let results = m.filter(&candidates, "cargo-build");
        let texts: Vec<&str> = results.iter().map(|r| r.candidate.text.as_str()).collect();

        assert_eq!(
            texts,
            vec!["cargo-build"],
            "exact match should not pull in DL near-miss candidates: {texts:?}"
        );
    }

    #[test]
    fn smart_case_exact_match_skips_dl_near_miss_candidates() {
        let mut m = FuzzyMatcher::new();
        let candidates = make_candidates(&["Git", "gat", "gib"]);
        let results = m.filter(&candidates, "git");
        let texts: Vec<&str> = results.iter().map(|r| r.candidate.text.as_str()).collect();

        assert_eq!(
            texts,
            vec!["Git"],
            "smart-case exact match should not pull in DL near-miss candidates: {texts:?}"
        );
    }

    #[test]
    fn normalized_exact_match_skips_dl_near_miss_candidates() {
        let mut m = FuzzyMatcher::new();
        let candidates = make_candidates(&["café", "cafg"]);
        let results = m.filter(&candidates, "cafe");
        let texts: Vec<&str> = results.iter().map(|r| r.candidate.text.as_str()).collect();

        assert_eq!(
            texts,
            vec!["café"],
            "normalization-equivalent exact match should not pull in DL near-miss candidates: {texts:?}"
        );
    }

    #[test]
    fn filter_matches_smart_case_exact_match_skips_dl_near_miss_candidates() {
        let mut m = FuzzyMatcher::new();
        let candidates = make_candidates(&["Git", "gat", "gib"]);
        let results = m.filter_matches(&candidates, "git", None);
        let texts: Vec<&str> = results
            .iter()
            .map(|r| candidates[r.candidate_idx].text.as_str())
            .collect();

        assert_eq!(
            texts,
            vec!["Git"],
            "filter_matches should also skip DL near-miss candidates when smart-case exact exists: {texts:?}"
        );
    }

    #[test]
    fn filter_matches_scope_matches_full_rescan_for_append_query() {
        let mut m = FuzzyMatcher::new();
        let candidates = make_candidates(&["alpha", "alpine", "beta", "zzz"]);

        let scoped_a = m.filter_matches(&candidates, "a", None);
        let scoped_a_idx: Vec<usize> = scoped_a.iter().map(|r| r.candidate_idx).collect();
        let scoped = m.filter_matches(&candidates, "alp", Some(&scoped_a_idx));
        let full = m.filter_matches(&candidates, "alp", None);

        let scoped_texts: Vec<&str> = scoped
            .iter()
            .map(|r| candidates[r.candidate_idx].text.as_str())
            .collect();
        let full_texts: Vec<&str> = full
            .iter()
            .map(|r| candidates[r.candidate_idx].text.as_str())
            .collect();
        assert_eq!(scoped_texts, full_texts);
    }

    #[test]
    fn filter_matches_scope_keeps_dl_result_from_full_candidate_set() {
        let mut m = FuzzyMatcher::new();
        let candidates = make_candidates(&["claude", "calculated", "cat"]);

        let prior = m.filter_matches(&candidates, "calud", None);
        let prior_idx: Vec<usize> = prior.iter().map(|r| r.candidate_idx).collect();
        let results = m.filter_matches(&candidates, "calude", Some(&prior_idx));
        let texts: Vec<&str> = results
            .iter()
            .map(|r| candidates[r.candidate_idx].text.as_str())
            .collect();

        assert!(
            texts.contains(&"claude"),
            "calude should still include claude via DL: {texts:?}"
        );
    }
}
