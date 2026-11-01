use amml_data::PatternText;

use crate::errors::DeriveError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pattern {
    // None is a wildcard
    bytes: Vec<Option<u8>>,
}

impl Pattern {
    pub fn parse(text: &str) -> Result<Self, DeriveError> {
        let bytes = PatternText::parse(text).map_err(|error| match error {
            amml_data::DataError::BadPatternToken { token } => DeriveError::BadPattern { token },
            other => DeriveError::BadPattern { token: other.to_string() },
        })?;
        Ok(Self { bytes })
    }

    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }

    pub fn matches_at(&self, haystack: &[u8], at: usize) -> bool {
        let Some(window) = haystack.get(at..at + self.bytes.len()) else {
            return false;
        };
        self.bytes.iter().zip(window).all(|(p, b)| p.is_none_or(|p| p == *b))
    }

    /// Offset and bytes of the longest run of fixed bytes.
    pub fn anchor(&self) -> (usize, Vec<u8>) {
        let mut best = (0, 0);
        let mut run_start = 0;
        for i in 0..=self.bytes.len() {
            if self.bytes.get(i).is_some_and(Option::is_some) {
                continue;
            }
            if i - run_start > best.1 {
                best = (run_start, i - run_start);
            }
            run_start = i + 1;
        }
        let fixed = self.bytes[best.0..best.0 + best.1].iter().flatten().copied().collect();
        (best.0, fixed)
    }
}
