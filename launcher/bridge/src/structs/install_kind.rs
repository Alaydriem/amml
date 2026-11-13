use super::BundleKind;

// How the launcher got onto the machine. Only the NSIS and MSI installs update themselves.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallKind {
    Nsis,
    Msi,
    Msix,
    Standalone,
}

impl InstallKind {
    // A packaged process is always MSIX, whatever the bundle type says.
    pub fn detect(packaged: bool, bundle: Option<BundleKind>) -> InstallKind {
        if packaged {
            return InstallKind::Msix;
        }
        match bundle {
            Some(BundleKind::Nsis) => InstallKind::Nsis,
            Some(BundleKind::Msi) => InstallKind::Msi,
            None => InstallKind::Standalone,
        }
    }

    // A Store install is updated by the Store; every other kind looks at the GitHub release.
    pub fn checks_github(self) -> bool {
        !matches!(self, InstallKind::Msix)
    }

    pub fn self_updates(self) -> bool {
        matches!(self, InstallKind::Nsis | InstallKind::Msi)
    }
}
