use super::{AssetFetcher, FileHash, MsiHold, ZipGuard};
use crate::{Asset, CoreProgress, GameInputPackage, LauncherError};
use amml_gdk::{CancelFlag, GdkError, PowerShellText};
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use zip::ZipArchive;

// Finds out whether the GameInput service is there and installs Microsoft's redistributable when it is not.
pub struct GameInput {
    fetcher: AssetFetcher,
    cancel: CancelFlag,
    package: GameInputPackage,
}

impl GameInput {
    const SERVICE: &'static str = "GameInputSvc";
    const MSI_NAME: &'static str = "GameInputRedist-3.5.283.msi";
    const UNPACK_DIR: &'static str = "gameinput-unpack";
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    // msiexec: done, or done and a restart is wanted
    const MSI_DONE: [i32; 2] = [0, 3010];

    pub fn new(fetcher: AssetFetcher, cancel: CancelFlag, package: GameInputPackage) -> GameInput {
        GameInput { fetcher, cancel, package }
    }

    pub fn is_installed(&self) -> Result<bool, LauncherError> {
        let output =
            Command::new("sc.exe").args(["query", Self::SERVICE]).creation_flags(Self::CREATE_NO_WINDOW).output()?;
        // sc.exe ends with 1060 for a service that does not exist; its text is localised, so the code decides
        Ok(output.status.success())
    }

    // reads the text `sc.exe query` prints: a service that exists has a STATE line
    pub fn is_installed_in(query_output: &str) -> bool {
        query_output.lines().any(|line| line.trim_start().starts_with("STATE"))
    }

    // Downloads the pinned package, checks it and the MSI inside it, and runs the MSI through one UAC prompt.
    // Returns whether the service is there afterwards.
    pub fn install(&self, progress: &mut impl FnMut(CoreProgress)) -> Result<bool, LauncherError> {
        if self.cancel.is_cancelled() {
            return Err(GdkError::Cancelled.into());
        }
        let asset = Asset { url: self.package.url.clone(), sha256: Some(self.package.sha256.clone()), size: None };
        let label = "microsoft.gameinput.nupkg".to_string();
        let file = self.fetcher.fetch(&asset, &mut |download| {
            progress(CoreProgress::Downloading { file: label.clone(), progress: download });
        })?;
        progress(CoreProgress::Verifying { file: label });
        let folder = file.path.parent().map(Path::to_path_buf).unwrap_or_default();
        let unpacked = self.take_out_msi(&file.path, &folder);
        self.fetcher.discard(&file);
        let msi = unpacked?;
        let ran = self.run_elevated(&msi);
        let _ = std::fs::remove_file(&msi);
        ran?;
        self.is_installed()
    }

    // the MSI, taken out through ZipGuard and checked against its own hash
    fn take_out_msi(&self, package: &Path, folder: &Path) -> Result<PathBuf, LauncherError> {
        let scratch = folder.join(Self::UNPACK_DIR);
        let _ = std::fs::remove_dir_all(&scratch);
        let result = self.unpack(package, folder, &scratch);
        let _ = std::fs::remove_dir_all(&scratch);
        result
    }

    fn unpack(&self, package: &Path, folder: &Path, scratch: &Path) -> Result<PathBuf, LauncherError> {
        let mut archive = ZipArchive::new(std::fs::File::open(package)?)
            .map_err(|error| ZipGuard::bad(package.to_path_buf(), error.to_string()))?;
        let entries = ZipGuard::entries(&mut archive)?;
        let entry_path = Path::new(&self.package.msi_entry);
        let Some(found) = entries.iter().find(|(_, path)| path == entry_path) else {
            return Err(ZipGuard::bad(package.to_path_buf(), format!("has no {}", self.package.msi_entry)));
        };
        let prefix = entry_path.parent().unwrap_or(Path::new(""));
        ZipGuard::extract(&mut archive, std::slice::from_ref(found), prefix, scratch)?;
        let name = entry_path.file_name().unwrap_or_default();
        let extracted = scratch.join(name);
        let sha256 = FileHash::sha256(&extracted)?;
        if !sha256.eq_ignore_ascii_case(&self.package.msi_sha256) {
            return Err(LauncherError::ChecksumMismatch {
                file: self.package.msi_entry.clone(),
                expected: self.package.msi_sha256.clone(),
                found: sha256,
            });
        }
        let target = folder.join(Self::MSI_NAME);
        std::fs::rename(&extracted, &target)?;
        Ok(target)
    }

    // Holds the MSI against writes, replacing and deleting, then checks its hash through that handle. The hold lasts
    // until the returned value drops, so a swap cannot follow the check.
    pub fn hold_checked(&self, msi: &Path) -> Result<MsiHold, LauncherError> {
        let mut hold = MsiHold::open(msi)?;
        let sha256 = hold.sha256()?;
        if !sha256.eq_ignore_ascii_case(&self.package.msi_sha256) {
            return Err(LauncherError::ChecksumMismatch {
                file: self.package.msi_entry.clone(),
                expected: self.package.msi_sha256.clone(),
                found: sha256,
            });
        }
        Ok(hold)
    }

    fn run_elevated(&self, msi: &Path) -> Result<(), LauncherError> {
        if self.cancel.is_cancelled() {
            return Err(GdkError::Cancelled.into());
        }
        let hold = self.hold_checked(msi)?;
        let arguments = format!("/i \"{}\" /qn /norestart", msi.display());
        // a refused UAC prompt makes Start-Process fail, which ends powershell with 1; otherwise msiexec's own code
        let command = format!(
            "$ErrorActionPreference = 'Stop'; $run = Start-Process msiexec.exe -Verb RunAs -Wait -PassThru -ArgumentList {}; exit $run.ExitCode",
            PowerShellText::quote(&arguments)
        );
        let status = Command::new("powershell.exe")
            .args(["-NoProfile", "-NonInteractive", "-Command", &command])
            .creation_flags(Self::CREATE_NO_WINDOW)
            .status()?;
        drop(hold);
        match status.code() {
            Some(code) if Self::MSI_DONE.contains(&code) => Ok(()),
            code => Err(LauncherError::InstallerFailed { code }),
        }
    }
}
