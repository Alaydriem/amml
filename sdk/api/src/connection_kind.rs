use amml_abi::AmmlConnectionKind;

/// How the client is connected to the world it plays.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConnectionKind {
    /// The world runs on this machine.
    Local,
    /// The world runs on another machine, named by a host.
    Remote,
    /// Any other connection, or one the loader cannot tell.
    Other,
}

impl ConnectionKind {
    // A kind the loader gives that this crate does not know is `Other`.
    pub(crate) fn from_abi(kind: AmmlConnectionKind) -> Self {
        match kind {
            AmmlConnectionKind::LOCAL => Self::Local,
            AmmlConnectionKind::REMOTE => Self::Remote,
            _ => Self::Other,
        }
    }

    // The value of the kind in the ABI, for the fake loader.
    #[cfg(feature = "test")]
    pub(crate) fn to_abi(self) -> AmmlConnectionKind {
        match self {
            Self::Local => AmmlConnectionKind::LOCAL,
            Self::Remote => AmmlConnectionKind::REMOTE,
            Self::Other => AmmlConnectionKind::OTHER,
        }
    }
}
