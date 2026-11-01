#![forbid(unsafe_code)]

use amml_ui_gen::app::GeneratorApp;

fn main() -> std::process::ExitCode {
    GeneratorApp::run()
}
