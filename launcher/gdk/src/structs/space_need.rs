// Free space an operation needs, per drive.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpaceNeed {
    // downloads, the loose copy and a margin, on the launcher's drive
    pub work_drive: u64,
    // the registered package, on the drive that holds the game
    pub game_drive: u64,
}

impl SpaceNeed {
    const MARGIN: u64 = 1024 * 1024 * 1024;

    // downloads that are not cached yet, the loose copy (about the wanted size) and a margin
    pub fn for_install(wanted: u64, current: u64, wanted_cached: bool, current_cached: bool) -> SpaceNeed {
        let wanted_download = if wanted_cached { 0 } else { wanted };
        let current_download = if current_cached { 0 } else { current };
        let work_drive =
            wanted_download.saturating_add(current_download).saturating_add(wanted).saturating_add(Self::MARGIN);
        SpaceNeed { work_drive, game_drive: wanted }
    }

    pub fn for_copy_out(folder_bytes: u64) -> SpaceNeed {
        SpaceNeed { work_drive: folder_bytes.saturating_add(Self::MARGIN), game_drive: 0 }
    }
}
