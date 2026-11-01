use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::Parser;

use crate::errors::PatchError;
use crate::services::ImportPatcher;

#[derive(Parser, Clone, Debug)]
#[command(name = "amml-patch", about = "Writes a copy of a PE file with one extra import")]
pub struct Cli {
    /// The file to read; it is never modified.
    #[arg(long)]
    pub input: PathBuf,
    /// The patched copy to write.
    #[arg(long)]
    pub output: PathBuf,
    /// The DLL the copy imports.
    #[arg(long, default_value = "amml.dll")]
    pub dll: String,
    /// The symbol the copy imports from the DLL.
    #[arg(long, default_value = "amml_anchor")]
    pub symbol: String,
}

impl Cli {
    /// Exit code 0 when the copy was written, 1 otherwise.
    pub fn execute(&self) -> ExitCode {
        match self.run() {
            Ok(()) => {
                println!("wrote {} with an import of {}!{}", self.output.display(), self.dll, self.symbol);
                ExitCode::SUCCESS
            }
            Err(error) => {
                eprintln!("error: {error}");
                ExitCode::FAILURE
            }
        }
    }

    fn run(&self) -> Result<(), PatchError> {
        if Self::same_file(&self.input, &self.output) {
            return Err(PatchError::Malformed("the output must not be the input file".to_string()));
        }
        let input = fs::read(&self.input).map_err(|source| PatchError::Io { path: self.input.clone(), source })?;
        let output = ImportPatcher::add_import(&input, &self.dll, &self.symbol)?;
        fs::write(&self.output, output).map_err(|source| PatchError::Io { path: self.output.clone(), source })
    }

    fn same_file(left: &Path, right: &Path) -> bool {
        match (fs::canonicalize(left), fs::canonicalize(right)) {
            (Ok(left), Ok(right)) => left == right,
            _ => false,
        }
    }
}
