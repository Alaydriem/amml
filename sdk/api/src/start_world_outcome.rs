use amml_abi::AmmlStartWorldOutcome;

/// How the game answered `Client::start_local_world`. The game's "not ready" answer is `ApiError::NotReady`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StartWorldOutcome {
    /// The game took the start; `WorldJoined` follows when the world is loaded.
    Started,
    /// The world is not in the game's world list.
    NotInLevelList,
    /// Another world start has not finished.
    StartupInProgress,
    /// A newer game version saved the world.
    SavedInNewerVersion,
    /// The world has platform locked content that the player did not confirm.
    PlatformLockedContent,
    /// The world folder is not on disk.
    NotFoundInFileSystem,
    /// The world has slim cache data.
    SlimCacheData,
    /// Any other refusal of the game.
    OtherRefusal,
}

impl From<AmmlStartWorldOutcome> for StartWorldOutcome {
    fn from(outcome: AmmlStartWorldOutcome) -> Self {
        match outcome {
            AmmlStartWorldOutcome::STARTED => Self::Started,
            AmmlStartWorldOutcome::NOT_IN_LEVEL_LIST => Self::NotInLevelList,
            AmmlStartWorldOutcome::STARTUP_IN_PROGRESS => Self::StartupInProgress,
            AmmlStartWorldOutcome::SAVED_IN_NEWER_VERSION => Self::SavedInNewerVersion,
            AmmlStartWorldOutcome::PLATFORM_LOCKED_CONTENT => Self::PlatformLockedContent,
            AmmlStartWorldOutcome::NOT_FOUND_IN_FILE_SYSTEM => Self::NotFoundInFileSystem,
            AmmlStartWorldOutcome::SLIM_CACHE_DATA => Self::SlimCacheData,
            _ => Self::OtherRefusal,
        }
    }
}
