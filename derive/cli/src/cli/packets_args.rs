use std::path::PathBuf;

use clap::Args;

use crate::structs::Address;

#[derive(Args, Clone, Debug)]
pub struct PacketsArgs {
    /// `MinecraftPackets::createPacket` in `--exe` (the `MinecraftPackets.createPacket` value in data/), as an RVA or
    /// a VA in hex
    #[arg(long, value_parser = Address::parse)]
    pub create_packet: Address,
    /// The old game executable, to compare its packets by name
    #[arg(long, requires = "old_create_packet")]
    pub old_exe: Option<PathBuf>,
    /// `createPacket` in the old executable, as an RVA or a VA in hex
    #[arg(long, value_parser = Address::parse, requires = "old_exe")]
    pub old_create_packet: Option<Address>,
}
