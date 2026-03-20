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

        let mut results: Vec<ScoredCandidate> = candidates
            .iter()
            .filter_map(|candidate| {
                let mut buf = Vec::new();
                let haystack = Utf32Str::new(&candidate.text, &mut buf);
                pattern
                    .score(haystack, &mut self.matcher)
                    .map(|score| ScoredCandidate {
                        candidate: candidate.clone(),
                        score,
                    })
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

        if results.is_empty() && query.len() >= 2 {
            results = damerau_levenshtein_fallback(candidates, query);
        }

        results
    }
}

fn damerau_levenshtein_fallback(
    candidates: &[Candidate],
    query: &str,
) -> Vec<ScoredCandidate> {
    let max_dist = if query.len() <= 4 { 1 } else { 2 };

    let mut results: Vec<ScoredCandidate> = candidates
        .iter()
        .filter_map(|candidate| {
            if query.len().abs_diff(candidate.text.len()) > max_dist {
                return None;
            }
            let dist = damerau_levenshtein(query, &candidate.text);
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

fn damerau_levenshtein(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let len_a = a.len();
    let len_b = b.len();

    let mut d = vec![vec![0usize; len_b + 1]; len_a + 1];

    for i in 0..=len_a {
        d[i][0] = i;
    }
    for j in 0..=len_b {
        d[0][j] = j;
    }

    for i in 1..=len_a {
        for j in 1..=len_b {
            let cost = if a[i - 1] == b[j - 1] { 0 } else { 1 };
            d[i][j] = (d[i - 1][j] + 1)
                .min(d[i][j - 1] + 1)
                .min(d[i - 1][j - 1] + cost);
            if i > 1 && j > 1 && a[i - 1] == b[j - 2] && a[i - 2] == b[j - 1] {
                d[i][j] = d[i][j].min(d[i - 2][j - 2] + cost);
            }
        }
    }

    d[len_a][len_b]
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
}
