use std::path::PathBuf;

use crate::Connection;

/// The facts of the joined world, read by `Client::world_info`. A fact the game does not give, or gives as text that is
/// not UTF-8, is `None`; the others stay.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WorldInfo {
    /// The folder of the world on disk.
    pub storage_folder: Option<PathBuf>,
    pub level_name: Option<String>,
    /// The world seed.
    pub seed: Option<u64>,
    /// Whether the world is a realm.
    pub realm: Option<bool>,
    /// Whether commands are enabled in the world.
    pub commands_enabled: Option<bool>,
    pub connection: Option<Connection>,
}
