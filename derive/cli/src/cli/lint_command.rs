use std::fs;

use amml_data::RecipeLoader;

use super::LintArgs;
use crate::errors::CliError;
use crate::services::{ImageLoader, RecipeLint};
use crate::structs::LintReport;

pub struct LintCommand;

impl LintCommand {
    /// Lints every recipe against the image and writes the Markdown report to `--report`, or to standard output.
    pub fn run(args: &LintArgs) -> Result<LintReport, CliError> {
        let set = RecipeLoader::load_dir(&args.recipes)?;
        let image = ImageLoader::from_path(&args.exe)?;
        let report = RecipeLint::new(&image, &set).lint()?;
        let text = RecipeLint::render(&report);
        match &args.report {
            Some(path) => {
                if let Some(parent) = path.parent().filter(|parent| !parent.as_os_str().is_empty()) {
                    fs::create_dir_all(parent)
                        .map_err(|source| CliError::Write { path: parent.to_path_buf(), source })?;
                }
                fs::write(path, text).map_err(|source| CliError::Write { path: path.clone(), source })?;
            }
            None => print!("{text}"),
        }
        Ok(report)
    }
}
