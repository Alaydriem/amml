use std::path::PathBuf;
use std::time::SystemTime;

// A `Minecraft Bedrock` folder that holds worlds the player may copy into the shared data folder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorldSource {
    pub path: PathBuf,
    // `appdata` for `%APPDATA%\Minecraft Bedrock`, else the name of the instance that holds the folder
    pub label: String,
    pub world_count: usize,
    // the newest `level.dat` change among the worlds
    pub modified: Option<SystemTime>,
}
