use std::io::Write;
use std::process::ExitCode;

use amml_registry::{HttpReleaseSource, RegistryCli};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    let output = RegistryCli::new(HttpReleaseSource::github()).run(&args);
    let _ = std::io::stdout().write_all(output.stdout.as_bytes());
    let _ = std::io::stderr().write_all(output.stderr.as_bytes());
    ExitCode::from(u8::try_from(output.code).unwrap_or(1))
}
