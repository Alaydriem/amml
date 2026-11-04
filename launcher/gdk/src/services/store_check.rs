use super::{DiskSpace, PackageInventory, ProcessInspector};
use crate::GdkError;
use crate::structs::{Channel, RegisteredPackage, SpaceNeed, StoreState};
use std::path::{Path, PathBuf};

pub struct StoreCheck<I, P> {
    inventory: I,
    processes: P,
    disk: DiskSpace,
}

impl<I: PackageInventory, P: ProcessInspector> StoreCheck<I, P> {
    pub const GAMING_SERVICES_FAMILY: &'static str = "Microsoft.GamingServices_8wekyb3d8bbwe";
    pub const GAME_EXE: &'static str = "Minecraft.Windows.exe";

    pub fn new(inventory: I, processes: P, disk: DiskSpace) -> StoreCheck<I, P> {
        StoreCheck { inventory, processes, disk }
    }

    // refusals come in a fixed order: Gaming Services, the game, health, signature, running, then space
    pub fn check(
        &self,
        channel: Channel,
        need: SpaceNeed,
        work_dir: &Path,
        block_running: bool,
    ) -> Result<StoreState, GdkError> {
        if self.inventory.registered(Self::GAMING_SERVICES_FAMILY)?.is_none() {
            return Ok(StoreState::GamingServicesMissing);
        }
        let Some(package) = self.inventory.registered(channel.family_name())? else {
            return Ok(match channel {
                Channel::Release => StoreState::BedrockMissing,
                Channel::Preview => StoreState::PreviewMissing,
            });
        };
        if !package.status_ok {
            return Ok(StoreState::NotHealthy { full_name: package.full_name });
        }
        if !package.store_signed {
            return Ok(StoreState::NotStoreSigned { full_name: package.full_name });
        }
        if block_running {
            let pids = self.running_pids(&package)?;
            if !pids.is_empty() {
                return Ok(StoreState::MinecraftRunning { pids });
            }
        }
        if let Some(state) = self.space_refusal(work_dir, need.work_drive)? {
            return Ok(state);
        }
        if need.game_drive > 0
            && let Some(state) = self.space_refusal(&package.content_location, need.game_drive)?
        {
            return Ok(state);
        }
        Ok(StoreState::Ready(package))
    }

    fn running_pids(&self, package: &RegisteredPackage) -> Result<Vec<u32>, GdkError> {
        let roots = [Self::normal(&package.install_location), Self::normal(&package.content_location)];
        let mut pids = Vec::new();
        for (pid, image) in self.processes.image_paths(Self::GAME_EXE)? {
            let unreadable = image.as_os_str().is_empty();
            let image = Self::normal(&image);
            // a game process with an unreadable path may be the registered game: treat it as running
            if unreadable || roots.iter().any(|root| !root.is_empty() && image.starts_with(&format!("{root}\\"))) {
                pids.push(pid);
            }
        }
        Ok(pids)
    }

    fn space_refusal(&self, path: &Path, needed: u64) -> Result<Option<StoreState>, GdkError> {
        let free = self.disk.free_bytes(path)?;
        if free >= needed {
            return Ok(None);
        }
        let drive: PathBuf = path.components().take(2).collect();
        Ok(Some(StoreState::NotEnoughSpace { drive, needed, free }))
    }

    // lower case, no `\\?\` prefix, no trailing separator
    fn normal(path: &Path) -> String {
        let text = path.to_string_lossy().to_lowercase();
        let text = text.strip_prefix(r"\\?\").unwrap_or(&text);
        text.trim_end_matches(['\\', '/']).to_string()
    }
}
