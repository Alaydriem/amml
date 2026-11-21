use crate::ConnectionKind;

/// The connection the client plays over (`WorldInfo::connection`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Connection {
    pub kind: ConnectionKind,
    /// The host address; empty when the game holds none (a local game).
    pub host: String,
    /// The port; 0 when the game's value does not fit 16 bits.
    pub port: u16,
}
