use std::process::ExitCode;

use amml_derive::cli::Cli;
use clap::Parser;

fn main() -> ExitCode {
    Cli::parse().execute()
}
