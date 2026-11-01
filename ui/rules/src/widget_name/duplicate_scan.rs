use std::fs;
use std::io;
use std::path::Path;

use super::{DuplicateHit, TypeDeclaration, WidgetNames};
use crate::source::SourceFiles;

/// Finds a `struct` or `enum` of any visibility whose name is an amml-ui widget name (rule 4).
pub struct DuplicateScan;

impl DuplicateScan {
    pub fn scan_text(names: &WidgetNames, file: &Path, text: &str) -> Vec<DuplicateHit> {
        text.lines()
            .enumerate()
            .filter_map(|(index, line)| TypeDeclaration::read(line).map(|declaration| (index, declaration)))
            .filter(|(_, declaration)| names.contains(&declaration.name))
            .map(|(index, declaration)| DuplicateHit {
                file: file.to_path_buf(),
                line: index + 1,
                name: declaration.name,
            })
            .collect()
    }

    pub fn scan(names: &WidgetNames, files: &SourceFiles) -> io::Result<Vec<DuplicateHit>> {
        let mut hits = Vec::new();
        for file in files.rust_files()? {
            let text = fs::read_to_string(&file)?;
            hits.extend(Self::scan_text(names, &file, &text));
        }
        Ok(hits)
    }
}
