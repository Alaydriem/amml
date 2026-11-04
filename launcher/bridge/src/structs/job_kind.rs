// What a job does, with everything needed to run it again.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JobKind {
    CreateInstance { name: String, game: String, preview: bool, own_data: bool },
    UpdateAmml { name: String },
    InstallMods { name: String, ids: Vec<String> },
    InstallUrl { name: String, url: String, consent: bool },
    UpdateMods { name: String, ids: Vec<String> },
    CopyWorlds { source: String },
    InstallGameInput,
    // copies the chosen worlds into the shared data folder (none for a clean start), then saves the settings
    SetupData { source: String },
}

impl JobKind {
    pub fn slug(&self) -> &'static str {
        match self {
            Self::CreateInstance { .. } => "create-instance",
            Self::UpdateAmml { .. } => "update-amml",
            Self::InstallMods { .. } => "install-mods",
            Self::InstallUrl { .. } => "install-url",
            Self::UpdateMods { .. } => "update-mods",
            Self::CopyWorlds { .. } => "copy-worlds",
            Self::InstallGameInput => "install-game-input",
            Self::SetupData { .. } => "setup-data",
        }
    }

    // the game version a job makes
    pub fn version(&self) -> Option<String> {
        match self {
            Self::CreateInstance { game, .. } => Some(game.clone()),
            _ => None,
        }
    }

    // The one instance a job works in; none for a job that is about the whole app: a new instance (it may install a
    // game version), worlds, GameInput and the first-run data.
    pub fn instance(&self) -> Option<&str> {
        match self {
            Self::UpdateAmml { name }
            | Self::InstallMods { name, .. }
            | Self::InstallUrl { name, .. }
            | Self::UpdateMods { name, .. } => Some(name),
            Self::CreateInstance { .. } | Self::CopyWorlds { .. } | Self::InstallGameInput | Self::SetupData { .. } => {
                None
            }
        }
    }

    // the instance or the source the job is about; empty when it is about none
    pub fn title_name(&self) -> String {
        match self {
            Self::CreateInstance { name, .. }
            | Self::UpdateAmml { name }
            | Self::InstallMods { name, .. }
            | Self::InstallUrl { name, .. }
            | Self::UpdateMods { name, .. } => name.clone(),
            Self::CopyWorlds { source } => source.clone(),
            Self::SetupData { source } => source.clone(),
            Self::InstallGameInput => String::new(),
        }
    }
}
