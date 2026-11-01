pub mod build_id_args;
pub mod callers_args;
pub mod command;
pub mod derive_args;
pub mod derive_command;
pub mod disasm_args;
pub mod draft_args;
pub mod explain_args;
pub mod fields_args;
pub mod function_args;
pub mod inspect_args;
pub mod inspect_command;
pub mod inspect_op;
pub mod lint_args;
pub mod lint_command;
pub mod packets_args;
pub mod scan_args;
pub mod signature_args;
pub mod signature_command;
pub mod similar_args;
pub mod vtable_args;
pub mod xrefs_args;

use std::process::ExitCode;

use clap::Parser;

pub use build_id_args::BuildIdArgs;
pub use callers_args::CallersArgs;
pub use command::Command;
pub use derive_args::DeriveArgs;
pub use derive_command::DeriveCommand;
pub use disasm_args::DisasmArgs;
pub use draft_args::DraftArgs;
pub use explain_args::ExplainArgs;
pub use fields_args::FieldsArgs;
pub use function_args::FunctionArgs;
pub use inspect_args::InspectArgs;
pub use inspect_command::InspectCommand;
pub use inspect_op::InspectOp;
pub use lint_args::LintArgs;
pub use lint_command::LintCommand;
pub use packets_args::PacketsArgs;
pub use scan_args::ScanArgs;
pub use signature_args::SignatureArgs;
pub use signature_command::SignatureCommand;
pub use similar_args::SimilarArgs;
pub use vtable_args::VtableArgs;
pub use xrefs_args::XrefsArgs;

#[derive(Parser, Clone, Debug)]
#[command(name = "amml-derive", about = "Derives per-build addresses for the AMML client mod loader")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

impl Cli {
    /// `derive`: exit code 0 when the run finished, even if entries failed; 2 when the inputs are invalid; 3 when the two
    /// derives differ (the report is written, the data is not); 4 when an output cannot be written.
    /// `lint`: 0 without open findings, 1 with them, 2 and 4 as for `derive`; its summary goes to standard error so
    /// the report can use standard output.
    /// `inspect`: 0 with the result on standard output, 2 when the executable, address, pattern or class is unusable.
    /// `signature`: 0 when every entry is known and, with `--compare`, the same; 1 when any entry changed or is
    /// unknown, 2 and 4 as for `derive`; its summary goes to standard error.
    pub fn execute(&self) -> ExitCode {
        match &self.command {
            Command::Derive(args) => match DeriveCommand::run(args) {
                Ok(report) => {
                    println!("{}", report.summary_line());
                    if !report.deterministic {
                        eprintln!("error: the two derive runs differ, so no data file was written");
                    }
                    ExitCode::from(report.exit_code())
                }
                Err(error) => {
                    eprintln!("error: {error}");
                    ExitCode::from(error.exit_code())
                }
            },
            Command::Signature(args) => match SignatureCommand::run(args) {
                Ok((file, None)) => {
                    eprintln!("{}", file.summary_line());
                    ExitCode::from(file.exit_code())
                }
                Ok((_, Some(report))) => {
                    print!("{}", report.render());
                    eprintln!("{}", report.summary_line());
                    ExitCode::from(report.exit_code())
                }
                Err(error) => {
                    eprintln!("error: {error}");
                    ExitCode::from(error.exit_code())
                }
            },
            Command::Inspect(args) => match InspectCommand::run(args) {
                Ok(text) => {
                    println!("{text}");
                    ExitCode::SUCCESS
                }
                Err(error) => {
                    eprintln!("error: {error}");
                    ExitCode::from(error.exit_code())
                }
            },
            Command::Lint(args) => match LintCommand::run(args) {
                Ok(report) => {
                    eprintln!("{}", report.summary_line());
                    ExitCode::from(report.exit_code())
                }
                Err(error) => {
                    eprintln!("error: {error}");
                    ExitCode::from(error.exit_code())
                }
            },
        }
    }
}
