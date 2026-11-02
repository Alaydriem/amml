use super::GameVersion;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegisteredPackage {
    pub family_name: String,
    pub full_name: String,
    pub version: GameVersion,
    pub status_ok: bool,
    pub store_signed: bool,
    pub install_location: PathBuf,
    // `install_location` with junctions resolved, for example `C:\XboxGames\Minecraft for Windows\Content`
    pub content_location: PathBuf,
}
