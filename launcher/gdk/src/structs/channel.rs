use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Channel {
    Release,
    Preview,
}

impl Channel {
    pub fn family_name(&self) -> &'static str {
        match self {
            Self::Release => "Microsoft.MinecraftUWP_8wekyb3d8bbwe",
            Self::Preview => "Microsoft.MinecraftWindowsBeta_8wekyb3d8bbwe",
        }
    }
}
