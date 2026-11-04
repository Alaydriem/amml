use crate::{CheckActionView, CheckIdView, CheckStateView, CheckView};
use amml_gdk::{Channel, GdkError, PackageInventory, RegisteredPackage};
use std::path::{Path, PathBuf};

// The first-run checks: the Store game, Gaming Services, GameInput, the preview app and free disk space.
pub struct SystemChecks;

impl SystemChecks {
    // free space one new game version needs
    pub const VERSION_SPACE: u64 = 6_000_000_000;
    const GAMING_SERVICES_FAMILY: &'static str = "Microsoft.GamingServices_8wekyb3d8bbwe";

    pub fn run<I: PackageInventory>(
        inventory: &I,
        game_input_installed: bool,
        free_bytes: u64,
        data_root: &Path,
    ) -> Result<Vec<CheckView>, GdkError> {
        let bedrock = inventory.registered(Channel::Release.family_name())?;
        let services = inventory.registered(Self::GAMING_SERVICES_FAMILY)?;
        let preview = inventory.registered(Channel::Preview.family_name())?;
        Ok(vec![
            Self::package(CheckIdView::Store, bedrock.as_ref(), true),
            Self::package(CheckIdView::GamingServices, services.as_ref(), false),
            Self::game_input(game_input_installed),
            Self::package(CheckIdView::Preview, preview.as_ref(), true),
            Self::disk(free_bytes, data_root),
        ])
    }

    fn package(id: CheckIdView, found: Option<&RegisteredPackage>, show_version: bool) -> CheckView {
        let (state, action) = match found {
            Some(package) if package.status_ok && package.store_signed => (CheckStateView::Ok, None),
            Some(_) => (CheckStateView::Warn, None),
            None => (CheckStateView::Missing, Some(CheckActionView::OpenStore)),
        };
        CheckView {
            id,
            state,
            package: found.map(|package| package.full_name.clone()),
            version: found.filter(|_| show_version).map(|package| package.version.to_string()),
            free_bytes: None,
            drive: None,
            action,
            elevated: false,
        }
    }

    fn game_input(installed: bool) -> CheckView {
        CheckView {
            id: CheckIdView::Gameinput,
            state: if installed { CheckStateView::Ok } else { CheckStateView::Missing },
            package: None,
            version: None,
            free_bytes: None,
            drive: None,
            action: if installed { None } else { Some(CheckActionView::Install) },
            elevated: !installed,
        }
    }

    fn disk(free_bytes: u64, data_root: &Path) -> CheckView {
        let drive: PathBuf = data_root.components().take(2).collect();
        CheckView {
            id: CheckIdView::Disk,
            state: if free_bytes < Self::VERSION_SPACE { CheckStateView::Warn } else { CheckStateView::Ok },
            package: None,
            version: None,
            free_bytes: Some(free_bytes),
            drive: Some(drive.display().to_string()),
            action: None,
            elevated: false,
        }
    }
}
