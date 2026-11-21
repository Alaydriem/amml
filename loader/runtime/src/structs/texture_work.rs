use crate::{TextureJob, TextureRelease};

/// What the texture pump does at one UI draw point: first every release, then every job.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TextureWork {
    pub releases: Vec<TextureRelease>,
    pub jobs: Vec<TextureJob>,
}
