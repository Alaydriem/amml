use clap::Subcommand;

use super::{CompatArgs, DeriveArgs, InspectArgs, LintArgs, SignatureArgs};

#[derive(Subcommand, Clone, Debug)]
pub enum Command {
    /// Derive the addresses of one game build
    Derive(DeriveArgs),
    /// Read-only views of one game executable: scan, function, disasm, vtable, xrefs, callers, draft, explain, similar, build-id
    Inspect(InspectArgs),
    /// Check which events the mods under --mods use are available on the build of --exe, and list forbidden imports of their DLLs
    Compat(CompatArgs),
    /// List fixed bytes in the recipes' patterns that sit in fields which change between builds
    Lint(LintArgs),
    /// Report the argument registers, stack arguments and return kind of every hooked function
    Signature(SignatureArgs),
}
