use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;

pub struct FuzzySearcher {
    matcher: SkimMatcherV2,
    pub threshold: i64,
}

impl FuzzySearcher {
    pub fn new(threshold: i64) -> Self {
        FuzzySearcher {
            matcher: SkimMatcherV2::default(),
            threshold,
        }
    }

    pub fn match_line(&self, pattern: &str, line: &str) -> Option<i64> {
        let score = self.matcher.fuzzy_match(line, pattern)?;
        if score >= self.threshold {
            Some(score)
        } else {
            None
        }
    }

    pub fn match_indices(&self, pattern: &str, line: &str) -> Option<(i64, Vec<usize>)> {
        self.matcher.fuzzy_indices(line, pattern).and_then(|(score, indices)| {
            if score >= self.threshold {
                Some((score, indices))
            } else {
                None
            }
        })
    }
}