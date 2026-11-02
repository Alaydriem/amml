/// Which folder function a game call went through; logged once per kind when it is redirected.
///
/// The last six kinds are only traced by the `folder-trace` build; they are never redirected.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FolderCall {
    KnownFolder,
    FolderPath,
    Environment,
    TempPath,
    EnvironmentA,
    TempPathA,
    Getenv,
    GetenvS,
    DupenvS,
    WdupenvS,
}

impl FolderCall {
    /// How many kinds there are.
    pub const COUNT: usize = 10;

    /// The Windows or CRT function behind the kind.
    pub fn function(self) -> &'static str {
        match self {
            Self::KnownFolder => "SHGetKnownFolderPath",
            Self::FolderPath => "SHGetFolderPathW",
            Self::Environment => "GetEnvironmentVariableW",
            Self::TempPath => "GetTempPathW",
            Self::EnvironmentA => "GetEnvironmentVariableA",
            Self::TempPathA => "GetTempPathA",
            Self::Getenv => "getenv",
            Self::GetenvS => "getenv_s",
            Self::DupenvS => "_dupenv_s",
            Self::WdupenvS => "_wdupenv_s",
        }
    }

    /// Position of the kind, below [`FolderCall::COUNT`].
    pub fn index(self) -> usize {
        self as usize
    }
}
