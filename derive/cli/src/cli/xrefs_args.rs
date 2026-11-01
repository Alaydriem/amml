use clap::{ArgGroup, Args};

use crate::structs::Address;

#[derive(Args, Clone, Debug)]
#[command(group(ArgGroup::new("source").required(true).multiple(false).args(["rva", "string"])))]
pub struct XrefsArgs {
    /// List every call, jump, RIP-relative operand and data pointer that names this address
    #[arg(long, value_parser = Address::parse)]
    pub rva: Option<Address>,
    /// List the functions that load this NUL-terminated string
    #[arg(long)]
    pub string: Option<String>,
}
