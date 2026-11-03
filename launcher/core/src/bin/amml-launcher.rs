use amml_launcher_core::{CliArgs, LauncherCli};
use clap::Parser;
use std::io::Write;

fn main() {
    let args = CliArgs::parse();
    let mut stdout = std::io::stdout();
    let code = LauncherCli::run(args, &mut stdout);
    let _ = stdout.flush();
    std::process::exit(code);
}
