use std::process::ExitCode;

use amml_patch::cli::Cli;
use clap::Parser;

fn main() -> ExitCode {
    Cli::parse().execute()
}
