use clap::{ArgGroup, Args};

use crate::structs::Address;

#[derive(Args, Clone, Debug)]
#[command(group(ArgGroup::new("source").required(true).multiple(false).args(["rva", "class"])))]
pub struct VtableArgs {
    /// Slot 0 of the vtable, as an RVA or a VA in hex
    #[arg(long, value_parser = Address::parse)]
    pub rva: Option<Address>,
    /// The raw RTTI class name, for example `.?AVClientInstance@@`
    #[arg(long)]
    pub class: Option<String>,
    /// The first slot to list
    #[arg(long, default_value_t = 0)]
    pub from: u32,
    /// The slot after the last one to list
    #[arg(long, default_value_t = 40)]
    pub to: u32,
    /// Adds the name of each slot, from the `__FUNCSIG__` or scoped string its function loads
    #[arg(long)]
    pub names: bool,
}
