use amml_gdk::{GameVersion, GdkError, PackageInventory, RegisteredPackage};
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;

// A registry of packages by family. Clones share it, so a fake deployer and several services see one state.
#[derive(Clone, Default)]
pub struct FakeInventory {
    packages: Rc<RefCell<HashMap<String, RegisteredPackage>>>,
    failing_reads: Rc<Cell<usize>>,
}

impl FakeInventory {
    pub fn new() -> FakeInventory {
        FakeInventory::default()
    }

    pub fn set(&self, package: RegisteredPackage) {
        self.packages.borrow_mut().insert(package.family_name.clone(), package);
    }

    // the registered release package, if any
    pub fn current(&self) -> Option<RegisteredPackage> {
        self.packages.borrow().get(Packages::RELEASE).cloned()
    }

    // the next `reads` calls to `registered` fail
    pub fn fail_next_reads(&self, reads: usize) {
        self.failing_reads.set(reads);
    }

    pub fn clear(&self, family: &str) {
        self.packages.borrow_mut().remove(family);
    }
}

impl PackageInventory for FakeInventory {
    fn registered(&self, family: &str) -> Result<Option<RegisteredPackage>, GdkError> {
        if self.failing_reads.get() > 0 {
            self.failing_reads.set(self.failing_reads.get() - 1);
            return Err(GdkError::Windows { call: "fake inventory", text: "the read fails".to_string() });
        }
        Ok(self.packages.borrow().get(family).cloned())
    }
}

pub struct Packages;

impl Packages {
    pub const RELEASE: &'static str = "Microsoft.MinecraftUWP_8wekyb3d8bbwe";
    pub const PREVIEW: &'static str = "Microsoft.MinecraftWindowsBeta_8wekyb3d8bbwe";
    pub const GAMING_SERVICES: &'static str = "Microsoft.GamingServices_8wekyb3d8bbwe";

    // the full name of a Bedrock package: `packed` is the third version part, for example "6005"
    pub fn full_name(packed: &str) -> String {
        format!("Microsoft.MinecraftUWP_1.26.{packed}.0_x64__8wekyb3d8bbwe")
    }

    // the full name of a Preview package
    pub fn preview_full_name(packed: &str) -> String {
        format!("Microsoft.MinecraftWindowsBeta_1.26.{packed}.0_x64__8wekyb3d8bbwe")
    }

    // a healthy Store-signed Bedrock package under `root`
    pub fn bedrock(packed: &str, root: &std::path::Path) -> RegisteredPackage {
        Self::from_full_name(&Self::full_name(packed), root)
    }

    pub fn from_full_name(full_name: &str, root: &std::path::Path) -> RegisteredPackage {
        let version = full_name.split('_').nth(1).unwrap();
        let location: PathBuf = root.join(full_name);
        RegisteredPackage {
            family_name: if full_name.starts_with("Microsoft.MinecraftWindowsBeta") {
                Self::PREVIEW.to_string()
            } else {
                Self::RELEASE.to_string()
            },
            full_name: full_name.to_string(),
            version: GameVersion::from_package_version(version).unwrap(),
            status_ok: true,
            store_signed: true,
            install_location: location.clone(),
            content_location: location,
        }
    }

    pub fn gaming_services(root: &std::path::Path) -> RegisteredPackage {
        RegisteredPackage {
            family_name: Self::GAMING_SERVICES.to_string(),
            full_name: "Microsoft.GamingServices_1.0.0.0_x64__8wekyb3d8bbwe".to_string(),
            version: GameVersion::parse("1.0.0.0").unwrap(),
            status_ok: true,
            store_signed: true,
            install_location: root.join("gs"),
            content_location: root.join("gs"),
        }
    }
}
