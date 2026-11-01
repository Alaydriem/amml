use clap::Args;

use crate::structs::Address;

#[derive(Args, Clone, Debug)]
pub struct FunctionArgs {
    /// An address inside the function, as an RVA or a VA in hex
    #[arg(value_parser = Address::parse)]
    pub rva: Address,
}
