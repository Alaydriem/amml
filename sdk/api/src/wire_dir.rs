use amml_abi::AmmlWireDir;

/// The direction of a `PacketWire` packet.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum WireDir {
    /// A packet the client sends (serverbound).
    Out,
    /// A packet the client receives (clientbound).
    In,
}

impl From<AmmlWireDir> for WireDir {
    /// `IN` is `In`; any other value, which the loader never writes, reads as `Out`.
    fn from(dir: AmmlWireDir) -> Self {
        if dir == AmmlWireDir::IN { Self::In } else { Self::Out }
    }
}

impl From<WireDir> for AmmlWireDir {
    fn from(dir: WireDir) -> Self {
        match dir {
            WireDir::Out => AmmlWireDir::OUT,
            WireDir::In => AmmlWireDir::IN,
        }
    }
}
