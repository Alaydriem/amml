use std::fmt;
use std::path::PathBuf;

use super::LiteralPattern;

/// One design literal in a source file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LiteralHit {
    pub file: PathBuf,
    /// The line number, from 1.
    pub line: usize,
    pub pattern: LiteralPattern,
    /// The line, trimmed.
    pub text: String,
}

impl fmt::Display for LiteralHit {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let file = self.file.to_string_lossy().replace('\\', "/");
        write!(formatter, "{file}:{}: {}: {}", self.line, self.pattern, self.text)
    }
}
