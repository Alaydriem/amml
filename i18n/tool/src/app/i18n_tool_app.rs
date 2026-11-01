use std::path::{Path, PathBuf};
use std::process::ExitCode;

use amml_i18n::pack::LanguagePack;

use crate::error::ToolError;
use crate::pack::PackCompiler;
use crate::po::PoReader;
use crate::pot::PotWriter;
use crate::pseudo::PseudoLocale;
use crate::report::{Coverage, CoverageReport};

use super::ToolCommand;

/// The command-line tool: runs one `ToolCommand` and turns an error into exit code 1.
pub struct I18nToolApp;

impl I18nToolApp {
    pub fn run() -> ExitCode {
        match ToolCommand::parse(std::env::args().skip(1)).and_then(|command| Self::execute(&command)) {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("amml-i18n-tool: {error}");
                ExitCode::FAILURE
            }
        }
    }

    pub fn execute(command: &ToolCommand) -> Result<(), ToolError> {
        match command {
            ToolCommand::Build { po_dir, out_dir } => Self::build(po_dir, out_dir),
            ToolCommand::Pseudo { pot, out } => {
                let pack = PseudoLocale::from_pot(&Self::read(pot)?)?;
                Self::write_pack(out, &pack)
            }
            ToolCommand::Report { pot, po_dir } => {
                print!("{}", CoverageReport::render(&Self::coverage(pot, po_dir)?));
                Ok(())
            }
            ToolCommand::CheckPot { pot, fresh } => {
                if PotWriter::same_text(&Self::read(pot)?, &Self::read(fresh)?) {
                    Ok(())
                } else {
                    Err(ToolError::StalePot { path: pot.clone() })
                }
            }
        }
    }

    // A locale under the coverage gate does not ship; its old pack is removed so a stale
    // translation cannot outlive the gate. Coverage is measured against the .pot in the same
    // folder, so a .po that was not merged after the .pot grew does not pass on its own count.
    fn build(po_dir: &Path, out_dir: &Path) -> Result<(), ToolError> {
        let template = Self::pot_in(po_dir)?;
        for (locale, path) in Self::po_files(po_dir)? {
            let catalog = PoReader::read(&path)?;
            let coverage = match &template {
                Some(pot) => CoverageReport::for_catalog(pot, &catalog)?,
                None => PackCompiler::coverage(&catalog),
            };
            let out = out_dir.join(format!("{locale}.json"));
            if coverage.ships() {
                Self::write_pack(&out, &PackCompiler::compile(&catalog, &locale)?)?;
                println!("{locale}: {}% — built", coverage.percent);
            } else {
                if out.exists() {
                    std::fs::remove_file(&out).map_err(|source| ToolError::Io { path: out.clone(), source })?;
                }
                println!("{locale}: {}% — below {}%, not shipped", coverage.percent, Coverage::SHIP_PERCENT);
            }
        }
        Ok(())
    }

    fn coverage(pot: &Path, po_dir: &Path) -> Result<Vec<(String, Coverage)>, ToolError> {
        let template = Self::read(pot)?;
        Self::po_files(po_dir)?
            .into_iter()
            .map(|(locale, path)| {
                let pack = PackCompiler::compile(&PoReader::read(&path)?, &locale)?;
                Ok((locale, CoverageReport::for_pack(&template, &pack)?))
            })
            .collect()
    }

    /// The text of the first `.pot` in `directory`, if it has one.
    fn pot_in(directory: &Path) -> Result<Option<String>, ToolError> {
        let Ok(entries) = std::fs::read_dir(directory) else { return Ok(None) };
        let mut pots: Vec<PathBuf> = entries
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| path.extension().is_some_and(|extension| extension == "pot"))
            .collect();
        pots.sort();
        pots.first().map(|pot| Self::read(pot)).transpose()
    }

    /// `(locale, path)` for every `<locale>.po` in `directory`, sorted. A missing folder has none.
    fn po_files(directory: &Path) -> Result<Vec<(String, PathBuf)>, ToolError> {
        let entries = match std::fs::read_dir(directory) {
            Ok(entries) => entries,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(source) => return Err(ToolError::Io { path: directory.to_path_buf(), source }),
        };
        let mut files: Vec<(String, PathBuf)> = entries
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| path.extension().is_some_and(|extension| extension == "po"))
            .filter_map(|path| Some((path.file_stem()?.to_str()?.to_owned(), path)))
            .collect();
        files.sort();
        Ok(files)
    }

    fn read(path: &Path) -> Result<String, ToolError> {
        std::fs::read_to_string(path).map_err(|source| ToolError::Io { path: path.to_path_buf(), source })
    }

    // serde_json's Value keeps object keys sorted, so the same pack always writes the same bytes.
    fn write_pack(path: &Path, pack: &LanguagePack) -> Result<(), ToolError> {
        let json = serde_json::to_string(&serde_json::to_value(pack)?)?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|source| ToolError::Io { path: parent.to_path_buf(), source })?;
        }
        std::fs::write(path, json + "\n").map_err(|source| ToolError::Io { path: path.to_path_buf(), source })
    }
}
