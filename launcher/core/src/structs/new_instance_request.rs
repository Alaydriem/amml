use crate::DataChoice;
use amml_gdk::GameVersion;

// What the player asks for when making an instance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewInstanceRequest {
    pub game: GameVersion,
    pub preview: bool,
    pub name: String,
    // an AMML version to use instead of the newest one that fits the game
    pub amml: Option<String>,
    pub data: DataChoice,
}
