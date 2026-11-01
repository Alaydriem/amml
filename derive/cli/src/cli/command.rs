use clap::Subcommand;

use super::{DeriveArgs, InspectArgs, LintArgs, SignatureArgs};

#[derive(Subcommand, Clone, Debug)]
pub enum Command {
    /// Derive the addresses of one game build
    Derive(DeriveArgs),
    /// Read-only views of one game executable: scan, function, disasm, vtable, xrefs, callers, draft, explain, similar, build-id
    Inspect(InspectArgs),
    /// List fixed bytes in the recipes' patterns that sit in fields which change between builds
    Lint(LintArgs),
    /// Report the argument registers, stack arguments and return kind of every hooked function
    Signature(SignatureArgs),
}
