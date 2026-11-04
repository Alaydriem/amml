use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::Parser;

use crate::errors::PatchError;
use crate::services::ImportPatcher;

#[derive(Parser, Clone, Debug)]
#[command(name = "amml-patch", version, about = "Writes a copy of a PE file with one extra import")]
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
    /// Parses the process arguments and runs. A bad argument gives one `error:` line on stderr
    /// and exit code 1; `--help` and `--version` print to stdout and give exit code 0.
    pub fn from_args() -> ExitCode {
        match Self::try_parse() {
            Ok(cli) => cli.execute(),
            Err(error) if error.use_stderr() => {
                eprintln!("error: {}", Self::one_line(&error.to_string()));
                ExitCode::FAILURE
            }
            Err(error) => {
                let _ = error.print();
                ExitCode::SUCCESS
            }
        }
    }

    // The message of a parse error up to the first blank line (before the usage text), on one line.
    fn one_line(message: &str) -> String {
        let lines: Vec<&str> = message.lines().take_while(|line| !line.trim().is_empty()).map(str::trim).collect();
        let joined = lines.join(" ");
        joined.strip_prefix("error: ").unwrap_or(&joined).to_string()
    }

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
        // The copy is written beside the output and renamed over it, so a failure never leaves a
        // half-written output or damages an existing one.
        let temp = Self::temp_path(&self.output);
        let written = fs::write(&temp, output).and_then(|()| fs::rename(&temp, &self.output));
        if let Err(source) = written {
            let _ = fs::remove_file(&temp);
            return Err(PatchError::Io { path: self.output.clone(), source });
        }
        Ok(())
    }

    fn temp_path(output: &Path) -> PathBuf {
        let mut name = output.as_os_str().to_os_string();
        name.push(".tmp");
        PathBuf::from(name)
    }

    fn same_file(left: &Path, right: &Path) -> bool {
        match (fs::canonicalize(left), fs::canonicalize(right)) {
            (Ok(left), Ok(right)) => left == right,
            _ => false,
        }
    }
}
