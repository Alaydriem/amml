//! Writes `i18n/amml-ui.pot` from `AmmlUiMessage`. With `--check`, writes nothing and fails when
//! the file on disk is out of date.

use std::path::PathBuf;
use std::process::ExitCode;

use amml_i18n::message::Message;
use amml_i18n_tool::error::ToolError;
use amml_i18n_tool::pot::PotWriter;
use amml_ui::AmmlUiMessage;

struct I18nExtract;

impl I18nExtract {
    fn run() -> ExitCode {
        let check = std::env::args().any(|arg| arg == "--check");
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("i18n")
            .join(format!("{}.pot", AmmlUiMessage::DOMAIN.name()));
        match PotWriter::write::<AmmlUiMessage>(&path, check) {
            Ok(true) if check => {
                eprintln!("{}", ToolError::StalePot { path });
                ExitCode::FAILURE
            }
            Ok(_) => ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("{error}");
                ExitCode::FAILURE
            }
        }
    }
}

fn main() -> ExitCode {
    I18nExtract::run()
}
