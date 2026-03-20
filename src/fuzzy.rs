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
                .then_with(|| a.candidate.kind_priority().cmp(&b.candidate.kind_priority()))
                .then_with(|| a.candidate.text.cmp(&b.candidate.text))
        });
        results
    }
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
        assert!(last_short < first_long, "short options should precede long: {texts:?}");
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
