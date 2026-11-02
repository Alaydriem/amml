use super::{FakeInventory, GameFolder, Packages};
use amml_gdk::{GdkError, PackageDeployer};
use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FakeCall {
    Add(String),
    Remove(String),
}

// Registers and removes packages in a shared `FakeInventory`. `add` registers the package whose full name is the
// file stem of the path (or the real name set with `real_name`) and writes a synthetic game folder for it.
#[derive(Clone)]
pub struct FakeDeployer {
    inventory: FakeInventory,
    packages_dir: PathBuf,
    downloads_dir: PathBuf,
    calls: Rc<RefCell<Vec<FakeCall>>>,
    failing: Rc<RefCell<HashSet<usize>>>,
    damaging: Rc<RefCell<HashSet<usize>>>,
    real_names: Rc<RefCell<HashMap<String, String>>>,
    after_remove: Rc<RefCell<Option<String>>>,
    downloads_at_add: Rc<RefCell<Vec<BTreeMap<String, u64>>>>,
    protected_at_remove: Rc<RefCell<Vec<Vec<String>>>>,
}

impl FakeDeployer {
    pub fn new(inventory: FakeInventory, packages_dir: PathBuf, downloads_dir: PathBuf) -> FakeDeployer {
        FakeDeployer {
            inventory,
            packages_dir,
            downloads_dir,
            calls: Rc::new(RefCell::new(Vec::new())),
            failing: Rc::new(RefCell::new(HashSet::new())),
            damaging: Rc::new(RefCell::new(HashSet::new())),
            real_names: Rc::new(RefCell::new(HashMap::new())),
            after_remove: Rc::new(RefCell::new(None)),
            downloads_at_add: Rc::new(RefCell::new(Vec::new())),
            protected_at_remove: Rc::new(RefCell::new(Vec::new())),
        }
    }

    // registers a package without recording a call
    pub fn register(&self, full_name: &str) {
        let package = Packages::from_full_name(full_name, &self.packages_dir);
        GameFolder::create(&package.install_location);
        self.inventory.set(package);
    }

    // the call with this index (counted over adds and removes) fails
    pub fn fail_call(&self, index: usize) {
        self.failing.borrow_mut().insert(index);
    }

    // like `fail_call`, and the registered package loses its exe, as a real `add` that stops half way may do
    pub fn fail_call_and_damage(&self, index: usize) {
        self.failing.borrow_mut().insert(index);
        self.damaging.borrow_mut().insert(index);
    }

    // adding the package file `stem` registers `real` under that other full name
    pub fn real_name(&self, stem: &str, real: &str) {
        self.real_names.borrow_mut().insert(stem.to_string(), real.to_string());
    }

    // the Store installs this package right after the next remove
    pub fn register_after_remove(&self, full_name: &str) {
        *self.after_remove.borrow_mut() = Some(full_name.to_string());
    }

    pub fn calls(&self) -> Vec<FakeCall> {
        self.calls.borrow().clone()
    }

    // the file names and sizes in the downloads folder at each `add`
    pub fn downloads_at_add(&self) -> Vec<BTreeMap<String, u64>> {
        self.downloads_at_add.borrow().clone()
    }

    // the protected names the service passed with each `remove`
    pub fn protected_at_remove(&self) -> Vec<Vec<String>> {
        self.protected_at_remove.borrow().clone()
    }

    fn record(&self, call: FakeCall) -> Result<usize, GdkError> {
        let index = self.calls.borrow().len();
        self.calls.borrow_mut().push(call);
        if self.failing.borrow().contains(&index) {
            if self.damaging.borrow().contains(&index)
                && let Some(package) = self.inventory.current()
            {
                let _ = std::fs::remove_file(package.install_location.join("Minecraft.Windows.exe"));
            }
            return Err(GdkError::Deploy { action: "fake", hresult: -1, text: format!("call {index} fails") });
        }
        Ok(index)
    }

    fn snapshot(&self) -> BTreeMap<String, u64> {
        let mut sizes = BTreeMap::new();
        if let Ok(read) = std::fs::read_dir(&self.downloads_dir) {
            for entry in read {
                let entry = entry.unwrap();
                sizes.insert(entry.file_name().to_string_lossy().to_string(), entry.metadata().unwrap().len());
            }
        }
        sizes
    }
}

impl PackageDeployer for FakeDeployer {
    fn add(&self, package: &Path) -> Result<(), GdkError> {
        let stem = package.file_stem().unwrap().to_string_lossy().to_string();
        self.downloads_at_add.borrow_mut().push(self.snapshot());
        self.record(FakeCall::Add(stem.clone()))?;
        let real = self.real_names.borrow().get(&stem).cloned().unwrap_or(stem);
        self.register(&real);
        Ok(())
    }

    fn remove(&self, full_name: &str, protected: &[String]) -> Result<(), GdkError> {
        self.protected_at_remove.borrow_mut().push(protected.to_vec());
        if protected.iter().any(|name| name.eq_ignore_ascii_case(full_name)) {
            return Err(GdkError::Protected(full_name.to_string()));
        }
        self.record(FakeCall::Remove(full_name.to_string()))?;
        self.inventory.clear(Packages::RELEASE);
        if let Some(next) = self.after_remove.borrow_mut().take() {
            self.register(&next);
        }
        Ok(())
    }
}
