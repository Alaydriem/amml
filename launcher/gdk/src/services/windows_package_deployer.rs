use super::PackageDeployer;
use crate::GdkError;
use crate::structs::FileUri;
use std::path::Path;
use windows::Foundation::Uri;
use windows::Management::Deployment::{DeploymentOptions, DeploymentResult, PackageManager};
use windows::core::HSTRING;

pub struct WindowsPackageDeployer;

impl WindowsPackageDeployer {
    pub fn new() -> WindowsPackageDeployer {
        WindowsPackageDeployer
    }

    fn deploy_error(action: &'static str, error: windows::core::Error) -> GdkError {
        GdkError::Deploy { action, hresult: error.code().0, text: error.message() }
    }

    fn check(action: &'static str, result: &DeploymentResult) -> Result<(), GdkError> {
        let code = result.ExtendedErrorCode().map_err(|error| Self::deploy_error(action, error))?;
        if code.is_err() {
            let text = result.ErrorText().map(|text| text.to_string()).unwrap_or_default();
            return Err(GdkError::Deploy { action, hresult: code.0, text });
        }
        Ok(())
    }
}

impl Default for WindowsPackageDeployer {
    fn default() -> Self {
        Self::new()
    }
}

impl PackageDeployer for WindowsPackageDeployer {
    fn add(&self, package: &Path) -> Result<(), GdkError> {
        let uri = Uri::CreateUri(&HSTRING::from(FileUri::from_path(package)))
            .map_err(|error| Self::deploy_error("add", error))?;
        let manager = PackageManager::new().map_err(|error| Self::deploy_error("add", error))?;
        let options = DeploymentOptions::ForceUpdateFromAnyVersion | DeploymentOptions::ForceApplicationShutdown;
        let operation =
            manager.AddPackageAsync(&uri, None, options).map_err(|error| Self::deploy_error("add", error))?;
        let result = operation.join().map_err(|error| Self::deploy_error("add", error))?;
        Self::check("add", &result)
    }

    fn remove(&self, full_name: &str, protected: &[String]) -> Result<(), GdkError> {
        if protected.iter().any(|name| name.eq_ignore_ascii_case(full_name)) {
            return Err(GdkError::Protected(full_name.to_string()));
        }
        let manager = PackageManager::new().map_err(|error| Self::deploy_error("remove", error))?;
        let operation = manager
            .RemovePackageAsync(&HSTRING::from(full_name))
            .map_err(|error| Self::deploy_error("remove", error))?;
        let result = operation.join().map_err(|error| Self::deploy_error("remove", error))?;
        Self::check("remove", &result)
    }
}
