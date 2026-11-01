use std::process::ExitCode;

use crate::error::GenError;

use super::{CliArgs, Generator};

/// The command line program: parses the arguments, runs the generator and sets the exit code.
pub struct GeneratorApp;

impl GeneratorApp {
    /// Exit code 0 on success, 1 when a check finds changes or the generator fails, 2 on bad usage.
    pub fn run() -> ExitCode {
        let args = match CliArgs::parse(std::env::args().skip(1)) {
            Ok(args) => args,
            Err(error) => {
                eprintln!("{error}");
                return ExitCode::from(2);
            }
        };
        match Generator::generate(&args) {
            Ok(changed) if args.check && !changed.is_empty() => {
                eprintln!("generated tokens are out of date; run `mise run design:generate`:");
                for path in changed {
                    eprintln!("  {}", path.display());
                }
                ExitCode::FAILURE
            }
            Ok(changed) => {
                if !args.check {
                    println!("amml-ui-gen: {} file(s) changed", changed.len());
                }
                ExitCode::SUCCESS
            }
            Err(error @ GenError::Usage { .. }) => {
                eprintln!("{error}");
                ExitCode::from(2)
            }
            Err(error) => {
                eprintln!("amml-ui-gen: {error}");
                ExitCode::FAILURE
            }
        }
    }
}
