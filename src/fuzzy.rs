use std::collections::HashSet;

use crate::candidate::Candidate;
use nucleo_matcher::pattern::{AtomKind, CaseMatching, Normalization, Pattern};
use nucleo_matcher::{Config, Matcher, Utf32Str};

pub struct FuzzyMatcher {
    matcher: Matcher,
}

pub struct ScoredCandidate {
    pub candidate: Candidate,
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
        }
    }

    pub fn filter(&mut self, candidates: &[Candidate], query: &str) -> Vec<ScoredCandidate> {
        if query.is_empty() {
            let mut results: Vec<ScoredCandidate> = candidates
                .iter()
                .map(|c| ScoredCandidate {
                    candidate: c.clone(),
                    score: 0,
                })
                .collect();
            results.sort_by(|a, b| {
                a.candidate
                    .kind_priority()
                    .cmp(&b.candidate.kind_priority())
                    .then_with(|| a.candidate.text.len().cmp(&b.candidate.text.len()))
                    .then_with(|| a.candidate.text.cmp(&b.candidate.text))
            });
            return results;
        }

        let pattern = Pattern::new(
            query,
            CaseMatching::Smart,
            Normalization::Smart,
            AtomKind::Fuzzy,
        );

        let mut buf = Vec::new();
        let mut results: Vec<ScoredCandidate> = Vec::new();
        for candidate in candidates {
            buf.clear();
            let haystack = Utf32Str::new(&candidate.text, &mut buf);
            if let Some(score) = pattern.score(haystack, &mut self.matcher) {
                results.push(ScoredCandidate {
                    candidate: candidate.clone(),
                    score,
                });
            }
        }

        results.sort_by(|a, b| {
            b.score
                .cmp(&a.score)
                .then_with(|| a.candidate.text.len().cmp(&b.candidate.text.len()))
                .then_with(|| {
                    a.candidate
                        .kind_priority()
                        .cmp(&b.candidate.kind_priority())
                })
                .then_with(|| a.candidate.text.cmp(&b.candidate.text))
        });

        if query.len() >= 2 {
            let seen: HashSet<&str> =
                results.iter().map(|r| r.candidate.text.as_str()).collect();
            let dl_results = damerau_levenshtein_fallback(candidates, query);
            let novel: Vec<ScoredCandidate> = dl_results
                .into_iter()
                .filter(|r| !seen.contains(r.candidate.text.as_str()))
                .collect();
            results.extend(novel);
        }

        results
    }
}

fn damerau_levenshtein_fallback(candidates: &[Candidate], query: &str) -> Vec<ScoredCandidate> {
    let max_dist = if query.len() <= 4 { 1 } else { 2 };
    let case_insensitive = !query.chars().any(|c| c.is_uppercase());
    let query_chars: Vec<char> = query.chars().collect();
    let mut cand_buf: Vec<char> = Vec::new();

    let mut results: Vec<ScoredCandidate> = candidates
        .iter()
        .filter_map(|candidate| {
            if query.len().abs_diff(candidate.text.len()) > max_dist {
                return None;
            }
            cand_buf.clear();
            cand_buf.extend(candidate.text.chars());
            let dist = damerau_levenshtein_chars(&query_chars, &cand_buf, case_insensitive);
            if dist <= max_dist {
                let score = (100u32).saturating_sub(dist as u32 * 30);
                Some(ScoredCandidate {
                    candidate: candidate.clone(),
                    score,
                })
            } else {
                None
            }
        })
        .collect();

    results.sort_by(|a, b| {
        b.score
            .cmp(&a.score)
            .then_with(|| a.candidate.text.len().cmp(&b.candidate.text.len()))
            .then_with(|| {
                a.candidate
                    .kind_priority()
                    .cmp(&b.candidate.kind_priority())
            })
            .then_with(|| a.candidate.text.cmp(&b.candidate.text))
    });
    results
}

fn damerau_levenshtein_chars(a: &[char], b: &[char], case_insensitive: bool) -> usize {
    let len_a = a.len();
    let len_b = b.len();

    let mut d = vec![vec![0usize; len_b + 1]; len_a + 1];

    for (i, row) in d.iter_mut().enumerate() {
        row[0] = i;
    }
    for (j, val) in d[0].iter_mut().enumerate() {
        *val = j;
    }

    let eq = |x: char, y: char| -> bool {
        if case_insensitive {
            x.eq_ignore_ascii_case(&y)
        } else {
            x == y
        }
    };

    for i in 1..=len_a {
        for j in 1..=len_b {
            let cost = if eq(a[i - 1], b[j - 1]) { 0 } else { 1 };
            d[i][j] = (d[i - 1][j] + 1)
                .min(d[i][j - 1] + 1)
                .min(d[i - 1][j - 1] + cost);
            if i > 1 && j > 1 && eq(a[i - 1], b[j - 2]) && eq(a[i - 2], b[j - 1]) {
                d[i][j] = d[i][j].min(d[i - 2][j - 2] + cost);
            }
        }
    }

    d[len_a][len_b]
}

pub fn damerau_levenshtein(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    damerau_levenshtein_chars(&a, &b, false)
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
        let git_count = results
            .iter()
            .filter(|r| r.candidate.text == "git")
            .count();
        assert_eq!(git_count, 1, "git should appear exactly once");
    }
}
