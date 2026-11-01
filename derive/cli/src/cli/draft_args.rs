use clap::Args;

use crate::structs::Address;

#[derive(Args, Clone, Debug)]
pub struct DraftArgs {
    /// The first instruction, as an RVA or a VA in hex
    #[arg(value_parser = Address::parse)]
    pub rva: Address,
    /// The least number of bytes the pattern covers; it ends at the end of the function
    #[arg(long, default_value_t = 24)]
    pub len: usize,
}
