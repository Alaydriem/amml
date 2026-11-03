use std::path::PathBuf;

// Where an instance keeps the game's roaming data: a shared folder, or the instance folder itself.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataChoice {
    Shared(PathBuf),
    Own,
}
