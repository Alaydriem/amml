use clap::Subcommand;

use super::{AddAmmlArgs, AddModArgs, BuildTreeArgs, LintManifestArgs, NextTagArgs};

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Checks a mod release and adds it to the tree. Exit 0 changed, 10 unchanged, 1 error
    AddMod(AddModArgs),
    /// Checks an AMML release and adds it to the tree. Exit 0 changed, 10 unchanged, 1 error
    AddAmml(AddAmmlArgs),
    /// Prints the next free registry release tag for a date
    NextTag(NextTagArgs),
    /// Checks the listing in a mod amml.hcl, as the release workflow does
    LintManifest(LintManifestArgs),
    /// Regenerates registry.json, names.json and the index pages from the mod documents
    BuildTree(BuildTreeArgs),
}
