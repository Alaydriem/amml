use std::fs;
use std::io;
use std::path::Path;

use super::{AllowedLiteral, LiteralHit, LiteralPattern};
use crate::source::SourceFiles;

/// Finds design literals (rule 3) in a crate's source: colours, font sizes, radii, shadows and size vectors written as
/// numbers outside the generated tokens.
///
/// It reads one line at a time. A call split over lines (`vec2(\n 280.0, 40.0)`) is not seen, and a comment line is
/// skipped.
pub struct DesignLiteralScan {
    files: SourceFiles,
    allowed: Vec<AllowedLiteral>,
}

impl DesignLiteralScan {
    pub fn new(files: SourceFiles) -> Self {
        Self { files, allowed: Vec::new() }
    }

    pub fn allow(mut self, allowed: AllowedLiteral) -> Self {
        self.allowed.push(allowed);
        self
    }

    /// Every literal in `text`, with no allow-list. `file` only names the hits.
    pub fn scan_text(file: &Path, text: &str) -> Vec<LiteralHit> {
        let mut hits = Vec::new();
        for (index, line) in text.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("//") {
                continue;
            }
            for pattern in LiteralPattern::ALL {
                if pattern.matches(line) {
                    hits.push(LiteralHit {
                        file: file.to_path_buf(),
                        line: index + 1,
                        pattern,
                        text: trimmed.to_owned(),
                    });
                }
            }
        }
        hits
    }

    /// Every literal in the source files that the allow-list does not cover.
    pub fn scan(&self) -> io::Result<Vec<LiteralHit>> {
        let hits = self.all_hits()?;
        Ok(hits.into_iter().filter(|hit| !self.allowed.iter().any(|allowed| allowed.covers(hit))).collect())
    }

    /// The allow-list entries that cover no line, so the list can only shrink.
    pub fn stale_allowances(&self) -> io::Result<Vec<&AllowedLiteral>> {
        let hits = self.all_hits()?;
        Ok(self.allowed.iter().filter(|allowed| !hits.iter().any(|hit| allowed.covers(hit))).collect())
    }

    fn all_hits(&self) -> io::Result<Vec<LiteralHit>> {
        let mut hits = Vec::new();
        for file in self.files.rust_files()? {
            let text = fs::read_to_string(&file)?;
            hits.extend(Self::scan_text(&file, &text));
        }
        Ok(hits)
    }
}
