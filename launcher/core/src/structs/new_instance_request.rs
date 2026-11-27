use crate::DataChoice;
use amml_gdk::GameVersion;
use std::path::PathBuf;

// What the player asks for when making an instance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewInstanceRequest {
    pub game: GameVersion,
    pub preview: bool,
    pub name: String,
    // an AMML version to use instead of the newest one that fits the game
    pub amml: Option<String>,
    // A local AMML release zip (the layout of a release zip, plus `amml/amml-api.txt`) used instead of the registry's
    // AMML. The instance is then a local build: its version ends in `+local`, so every mod minimum is met. It is the
    // way to make an instance for a game version that no registry release lists.
    pub amml_zip: Option<PathBuf>,
    pub data: DataChoice,
}
