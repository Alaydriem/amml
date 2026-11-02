use super::PackageInventory;
use crate::GdkError;
use crate::structs::{GameVersion, RegisteredPackage};
use std::path::PathBuf;
use windows::ApplicationModel::{Package, PackageSignatureKind};
use windows::Management::Deployment::PackageManager;
use windows::core::HSTRING;

pub struct WindowsPackageInventory;

impl WindowsPackageInventory {
    pub fn new() -> WindowsPackageInventory {
        WindowsPackageInventory
    }

    fn describe(package: &Package, family: &str) -> windows::core::Result<Option<RegisteredPackage>> {
        let id = package.Id()?;
        let raw = id.Version()?;
        let package_version = format!("{}.{}.{}.{}", raw.Major, raw.Minor, raw.Build, raw.Revision);
        let Ok(version) = GameVersion::from_package_version(&package_version) else {
            return Ok(None);
        };
        let install_location = PathBuf::from(package.InstalledPath()?.to_string());
        let content_location = Self::resolve(&install_location);
        Ok(Some(RegisteredPackage {
            family_name: family.to_string(),
            full_name: id.FullName()?.to_string(),
            version,
            status_ok: package.Status()?.VerifyIsOK()?,
            store_signed: package.SignatureKind()? == PackageSignatureKind::Store,
            install_location,
            content_location,
        }))
    }

    // the real folder behind a package location, without the `\\?\` prefix
    fn resolve(location: &std::path::Path) -> PathBuf {
        let resolved = std::fs::canonicalize(location).unwrap_or_else(|_| location.to_path_buf());
        let text = resolved.to_string_lossy();
        match text.strip_prefix(r"\\?\") {
            Some(plain) => PathBuf::from(plain),
            None => resolved,
        }
    }
}

impl Default for WindowsPackageInventory {
    fn default() -> Self {
        Self::new()
    }
}

impl PackageInventory for WindowsPackageInventory {
    fn registered(&self, family: &str) -> Result<Option<RegisteredPackage>, GdkError> {
        let windows_error =
            |call: &'static str| move |error: windows::core::Error| GdkError::Windows { call, text: error.to_string() };
        let manager = PackageManager::new().map_err(windows_error("PackageManager"))?;
        let packages = manager
            .FindPackagesByUserSecurityIdPackageFamilyName(&HSTRING::new(), &HSTRING::from(family))
            .map_err(windows_error("FindPackagesByUserSecurityIdPackageFamilyName"))?;
        for package in packages {
            if let Some(found) = Self::describe(&package, family).map_err(windows_error("Package"))? {
                return Ok(Some(found));
            }
        }
        Ok(None)
    }
}
