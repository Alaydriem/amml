// The Microsoft GameInput redistributable: where the NuGet package is, and the hashes the launcher checks twice.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameInputPackage {
    pub url: String,
    // SHA-256 of the whole package file
    pub sha256: String,
    // the MSI inside the package
    pub msi_entry: String,
    // SHA-256 of the MSI after it is taken out of the package
    pub msi_sha256: String,
}

impl GameInputPackage {
    pub fn official() -> GameInputPackage {
        GameInputPackage {
            url: "https://api.nuget.org/v3-flatcontainer/microsoft.gameinput/3.5.283/microsoft.gameinput.3.5.283.nupkg"
                .to_string(),
            sha256: "b5988cb8ff9d7009b6ddf6ad4e3ff87e91b00cc17ffcd5d628dabc21c208f100".to_string(),
            msi_entry: "redist/GameInputRedist.msi".to_string(),
            msi_sha256: "f8fceb2d75e8dbb080088930a673c43b86d3e602329bd54309611e2eb78bf575".to_string(),
        }
    }
}
