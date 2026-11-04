use super::RegisteredPackage;
use std::path::PathBuf;

// The answer to "can the launcher touch the Store install now?". Every refusal is a state, not a failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StoreState {
    Ready(RegisteredPackage),
    BedrockMissing,
    PreviewMissing,
    GamingServicesMissing,
    NotHealthy { full_name: String },
    NotStoreSigned { full_name: String },
    MinecraftRunning { pids: Vec<u32> },
    NotEnoughSpace { drive: PathBuf, needed: u64, free: u64 },
}

impl StoreState {
    // a sentence for the player
    pub fn describe(&self) -> String {
        match self {
            Self::Ready(package) => format!("{} is ready", package.full_name),
            Self::BedrockMissing => "Minecraft Bedrock is not installed from the Store".to_string(),
            Self::PreviewMissing => "Minecraft Preview is not installed from the Store".to_string(),
            Self::GamingServicesMissing => "Gaming Services is not installed".to_string(),
            Self::NotHealthy { full_name } => format!("{full_name} is not healthy; repair it in the Store first"),
            Self::NotStoreSigned { full_name } => format!("{full_name} was not installed from the Store"),
            Self::MinecraftRunning { pids } => {
                let ids: Vec<String> = pids.iter().map(u32::to_string).collect();
                format!("Minecraft is running (process {}); close it and run this again", ids.join(", "))
            }
            Self::NotEnoughSpace { drive, needed, free } => format!(
                "not enough free space on {}: {} MiB needed, {} MiB free",
                drive.display(),
                needed / (1024 * 1024),
                free / (1024 * 1024)
            ),
        }
    }
}
