use std::cmp::Ordering;

use crate::candidate::Candidate;
use nucleo_matcher::pattern::{AtomKind, CaseMatching, Normalization, Pattern};
use nucleo_matcher::{Config, Matcher, Utf32Str};

pub struct FuzzyMatcher {
    matcher: Matcher,
    pattern: Pattern,
    last_query: String,
    utf32_buf: Vec<char>,
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
        Self {
            matcher: Matcher::new(Config::DEFAULT),
            pattern: Pattern::new(
                "",
                CaseMatching::Smart,
                Normalization::Smart,
                AtomKind::Fuzzy,
            ),
            last_query: String::new(),
            utf32_buf: Vec::new(),
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
        let matcher = &mut self.matcher;
        let utf32_buf = &mut self.utf32_buf;
        let mut results: Vec<ScoredMatch> =
            Vec::with_capacity(fuzzy_scope.map_or(candidates.len(), <[usize]>::len));

        if let Some(scope) = fuzzy_scope {
            for &candidate_idx in scope {
                let candidate = &candidates[candidate_idx];
                utf32_buf.clear();
                let haystack = Utf32Str::new(&candidate.text, utf32_buf);
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
                if let Some(score) = pattern.score(haystack, matcher) {
                    results.push(ScoredMatch {
                        candidate_idx,
                        score,
                    });
                }
            }
        }

        sort_scored_matches(&mut results, candidates);
        results
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
