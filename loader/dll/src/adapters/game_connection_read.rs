/// What a read of a `Social::GameConnectionInfo` gives: the connection type, the host when it is UTF-8 and the port.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GameConnectionRead {
    /// `mType`: 0 for a local game, above 0 for a network connection, below 0 for none.
    pub connection_type: i16,
    /// `None` when the host text is not UTF-8.
    pub host: Option<String>,
    pub port: i32,
}
