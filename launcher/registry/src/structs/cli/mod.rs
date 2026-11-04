//! Command line shapes of `amml-registry`.

pub mod add_amml_args;
pub mod add_mod_args;
pub mod build_tree_args;
pub mod cli_args;
pub mod command;
pub mod lint_manifest_args;
pub mod next_tag_args;

pub use add_amml_args::AddAmmlArgs;
pub use add_mod_args::AddModArgs;
pub use build_tree_args::BuildTreeArgs;
pub use cli_args::CliArgs;
pub use command::Command;
pub use lint_manifest_args::LintManifestArgs;
pub use next_tag_args::NextTagArgs;
