use crate::game::GameConnectionInfoLayout;

/// A `Social::GameConnectionInfo` read through its layout head.
pub struct GameConnectionInfo<'a> {
    layout: &'a GameConnectionInfoLayout,
}

impl<'a> GameConnectionInfo<'a> {
    pub fn new(layout: &'a GameConnectionInfoLayout) -> Self {
        Self { layout }
    }

    /// The host address, empty when the game holds none. Bytes that are not UTF-8 are replaced.
    pub fn host(&self) -> String {
        String::from_utf8_lossy(self.layout.host.as_bytes()).into_owned()
    }

    /// The port; 0 when the game's `int` does not fit 16 bits.
    pub fn port(&self) -> u16 {
        u16::try_from(self.layout.port).unwrap_or(0)
    }
}
