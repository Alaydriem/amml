use std::path::PathBuf;

use clap::Args;

use crate::structs::Address;

#[derive(Args, Clone, Debug)]
pub struct SimilarArgs {
    /// The function in the old build, as an RVA or a VA in hex
    #[arg(value_parser = Address::parse)]
    pub rva: Address,
    /// The old game executable; `--exe` is the new build to search
    #[arg(long)]
    pub old_exe: PathBuf,
    /// How many candidates to list
    #[arg(long, default_value_t = 10)]
    pub top: usize,
}
