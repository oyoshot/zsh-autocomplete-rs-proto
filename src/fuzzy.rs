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
}
