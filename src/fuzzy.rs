use std::{borrow::Cow, cmp::Ordering};

use crate::candidate::Candidate;
use frizbee::{Config as MatchConfig, Matcher};
use unicode_normalization::{UnicodeNormalization, char::is_combining_mark};

pub struct FuzzyMatcher {
    matcher: Matcher,
    config: MatchConfig,
    last_query: String,
    last_max_typos: u16,
}

pub struct ScoredCandidate {
    pub candidate: Candidate,
    pub score: u32,
}

pub struct ScoredMatch {
    pub candidate_idx: usize,
    pub score: u32,
}

impl Default for FuzzyMatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl FuzzyMatcher {
    pub fn new() -> Self {
        let config = MatchConfig {
            max_typos: Some(0),
            sort: false,
            ..MatchConfig::default()
        };
        Self {
            matcher: Matcher::new("", &config),
            config,
            last_query: String::new(),
            last_max_typos: 0,
        }
    }

    fn ensure_matcher(&mut self, query: &str, max_typos: u16) {
        if self.last_max_typos != max_typos {
            self.config.max_typos = Some(max_typos);
            self.matcher.set_config(&self.config);
            self.last_max_typos = max_typos;
        }

        if self.last_query != query {
            self.matcher.set_needle(query);
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

        if query.chars().count() >= 3 && has_typo_rescue_candidates(candidates, fuzzy_scope) {
            return self.filter_typo_rescue_matches(candidates, query, fuzzy_scope);
        }

        self.filter_frizbee_matches(candidates, query, fuzzy_scope, 0)
    }

    pub fn filter(&mut self, candidates: &[Candidate], query: &str) -> Vec<ScoredCandidate> {
        self.filter_matches(candidates, query, None)
            .into_iter()
            .map(|m| ScoredCandidate {
                candidate: candidates[m.candidate_idx].clone(),
                score: m.score,
            })
            .collect()
    }
}

fn has_typo_rescue_candidates(candidates: &[Candidate], fuzzy_scope: Option<&[usize]>) -> bool {
    match fuzzy_scope {
        Some(scope) => scope
            .iter()
            .any(|&candidate_idx| candidates[candidate_idx].is_typo_rescue()),
        None => candidates.iter().any(Candidate::is_typo_rescue),
    }
}

fn typo_rescue_max_typos(query: &str) -> u16 {
    if query.chars().count() >= 5 { 2 } else { 1 }
}

impl FuzzyMatcher {
    fn filter_typo_rescue_matches(
        &mut self,
        candidates: &[Candidate],
        query: &str,
        fuzzy_scope: Option<&[usize]>,
    ) -> Vec<ScoredMatch> {
        let max_typos = typo_rescue_max_typos(query);
        self.filter_frizbee_matches(candidates, query, fuzzy_scope, max_typos)
    }

    fn filter_frizbee_matches(
        &mut self,
        candidates: &[Candidate],
        query: &str,
        fuzzy_scope: Option<&[usize]>,
        max_typos: u16,
    ) -> Vec<ScoredMatch> {
        let normalized_query = normalize_for_matching(query);
        let query_len = normalized_query.chars().count();
        let smart_case = max_typos == 0 && query.chars().any(char::is_uppercase);
        let mut candidate_indices = Vec::new();
        let mut haystacks = Vec::new();
        let rescue_only = max_typos > 0;
        let max_typos_usize = usize::from(max_typos);

        if let Some(scope) = fuzzy_scope {
            for &candidate_idx in scope {
                let candidate = &candidates[candidate_idx];
                let normalized_text = normalize_for_matching(&candidate.text);
                if should_match_candidate(
                    candidate,
                    &normalized_text,
                    &normalized_query,
                    query_len,
                    rescue_only,
                    max_typos_usize,
                    smart_case,
                ) {
                    candidate_indices.push(candidate_idx);
                    haystacks.push(normalized_text);
                }
            }
        } else {
            for (candidate_idx, candidate) in candidates.iter().enumerate() {
                let normalized_text = normalize_for_matching(&candidate.text);
                if should_match_candidate(
                    candidate,
                    &normalized_text,
                    &normalized_query,
                    query_len,
                    rescue_only,
                    max_typos_usize,
                    smart_case,
                ) {
                    candidate_indices.push(candidate_idx);
                    haystacks.push(normalized_text);
                }
            }
        }

        if haystacks.is_empty() {
            return Vec::new();
        }

        self.ensure_matcher(&normalized_query, max_typos);
        let mut results: Vec<ScoredMatch> = self
            .matcher
            .match_iter(&haystacks)
            .filter_map(|m| {
                candidate_indices
                    .get(m.index as usize)
                    .copied()
                    .map(|candidate_idx| ScoredMatch {
                        candidate_idx,
                        score: u32::from(m.score),
                    })
            })
            .collect();

        sort_scored_matches(&mut results, candidates);
        results
    }
}

fn should_match_candidate(
    candidate: &Candidate,
    normalized_text: &str,
    normalized_query: &str,
    query_len: usize,
    rescue_only: bool,
    max_typos: usize,
    smart_case: bool,
) -> bool {
    if rescue_only
        && (!candidate.is_typo_rescue()
            || normalized_text.chars().count().abs_diff(query_len) > max_typos)
    {
        return false;
    }

    if smart_case && !case_sensitive_subsequence_matches(normalized_query, normalized_text) {
        return false;
    }

    true
}

fn normalize_for_matching(text: &str) -> Cow<'_, str> {
    if text.is_ascii() {
        Cow::Borrowed(text)
    } else {
        Cow::Owned(text.nfd().filter(|c| !is_combining_mark(*c)).collect())
    }
}

fn case_sensitive_subsequence_matches(query: &str, haystack: &str) -> bool {
    let mut chars = haystack.chars();
    query
        .chars()
        .all(|query_char| chars.any(|c| c == query_char))
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
    fn normal_matching_strips_accents() {
        let mut m = FuzzyMatcher::new();
        let candidates = make_candidates(&["café", "cacao", "São-Paulo"]);

        let cafe_results = m.filter(&candidates, "cafe");
        let cafe_texts: Vec<&str> = cafe_results
            .iter()
            .map(|r| r.candidate.text.as_str())
            .collect();
        assert!(cafe_texts.contains(&"café"), "results: {cafe_texts:?}");

        let sao_results = m.filter(&candidates, "sao-paulo");
        let sao_texts: Vec<&str> = sao_results
            .iter()
            .map(|r| r.candidate.text.as_str())
            .collect();
        assert_eq!(
            sao_texts.first(),
            Some(&"São-Paulo"),
            "results: {sao_texts:?}"
        );
    }

    #[test]
    fn uppercase_query_filters_case_sensitively() {
        let mut m = FuzzyMatcher::new();
        let candidates = make_candidates(&["foo", "Foo", "FooBar", "fool"]);

        let uppercase_results = m.filter(&candidates, "Foo");
        let uppercase_texts: Vec<&str> = uppercase_results
            .iter()
            .map(|r| r.candidate.text.as_str())
            .collect();
        assert!(
            !uppercase_texts.contains(&"foo"),
            "results: {uppercase_texts:?}"
        );
        assert!(
            !uppercase_texts.contains(&"fool"),
            "results: {uppercase_texts:?}"
        );
        assert!(
            uppercase_texts.contains(&"Foo"),
            "results: {uppercase_texts:?}"
        );
        assert!(
            uppercase_texts.contains(&"FooBar"),
            "results: {uppercase_texts:?}"
        );

        let lowercase_results = m.filter(&candidates, "foo");
        let lowercase_texts: Vec<&str> = lowercase_results
            .iter()
            .map(|r| r.candidate.text.as_str())
            .collect();
        assert!(
            lowercase_texts.contains(&"foo"),
            "results: {lowercase_texts:?}"
        );
        assert!(
            lowercase_texts.contains(&"Foo"),
            "results: {lowercase_texts:?}"
        );
    }

    #[test]
    fn typo_rescue_surfaces_transposed_command() {
        let mut m = FuzzyMatcher::new();
        let candidates = make_candidates_with_kind(&[
            ("clang-include-fixer", "command_rescue"),
            ("clang-include-cleaner", "command_rescue"),
            ("cargo-install-update", "command_rescue"),
            ("claude", "command_rescue"),
        ]);

        let results = m.filter(&candidates, "calude");
        let texts: Vec<&str> = results.iter().map(|r| r.candidate.text.as_str()).collect();

        assert_eq!(texts.first(), Some(&"claude"), "results: {texts:?}");
    }

    #[test]
    fn typo_rescue_requires_rescue_candidate_kind() {
        let mut m = FuzzyMatcher::new();
        let candidates = make_candidates_with_kind(&[
            ("clang-include-fixer", "command"),
            ("clang-include-cleaner", "command"),
            ("cargo-install-update", "command"),
            ("claude", "command"),
        ]);

        let results = m.filter(&candidates, "calude");
        let texts: Vec<&str> = results.iter().map(|r| r.candidate.text.as_str()).collect();

        assert_ne!(texts.first(), Some(&"claude"), "results: {texts:?}");
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
}
