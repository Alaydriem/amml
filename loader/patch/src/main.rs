use std::process::ExitCode;

use amml_patch::cli::Cli;

fn main() -> ExitCode {
    Cli::from_args()
}
