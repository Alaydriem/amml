use std::path::PathBuf;

use clap::Args;

use crate::structs::Address;

#[derive(Args, Clone, Debug)]
pub struct FieldsArgs {
    /// The function in `--exe` (a constructor, destructor or copy works best), as an RVA or a VA in hex
    #[arg(value_parser = Address::parse)]
    pub rva: Address,
    /// The old game executable, to line its function up with the new one
    #[arg(long, requires = "old_rva")]
    pub old_exe: Option<PathBuf>,
    /// The same function in the old executable, as an RVA or a VA in hex
    #[arg(long, value_parser = Address::parse, requires = "old_exe")]
    pub old_rva: Option<Address>,
}
