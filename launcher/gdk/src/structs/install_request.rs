use super::{Channel, GameVersion};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InstallRequest {
    pub version: GameVersion,
    pub channel: Channel,
}
