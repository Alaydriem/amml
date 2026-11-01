use clap::Args;

use crate::structs::Address;

#[derive(Args, Clone, Debug)]
pub struct DisasmArgs {
    /// The first instruction, as an RVA or a VA in hex
    #[arg(value_parser = Address::parse)]
    pub rva: Address,
    /// The number of instructions; without it the listing runs to the end of the function, at most 400
    #[arg(long)]
    pub count: Option<usize>,
}
