use amml_gdk::{
    DeveloperModeControl, DownloadProgress, GdkError, InstallOutcome, InstallRequest, InstallStep, Installer,
};
use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::rc::Rc;

type Queue<T> = Rc<RefCell<VecDeque<Result<T, GdkError>>>>;

// Answers `install` with queued results. Clones share the queue and the counters.
#[derive(Clone, Default)]
pub struct FakeInstaller {
    results: Queue<InstallOutcome>,
    recovered: Queue<Option<String>>,
    install_calls: Rc<Cell<usize>>,
}

impl FakeInstaller {
    pub fn new() -> FakeInstaller {
        FakeInstaller::default()
    }

    pub fn queue(&self, result: Result<InstallOutcome, GdkError>) {
        self.results.borrow_mut().push_back(result);
    }

    pub fn queue_recover(&self, result: Result<Option<String>, GdkError>) {
        self.recovered.borrow_mut().push_back(result);
    }

    pub fn install_calls(&self) -> usize {
        self.install_calls.get()
    }
}

impl Installer for FakeInstaller {
    fn install(
        &self,
        _request: &InstallRequest,
        on_step: &mut dyn FnMut(InstallStep, Option<DownloadProgress>),
    ) -> Result<InstallOutcome, GdkError> {
        self.install_calls.set(self.install_calls.get() + 1);
        on_step(InstallStep::Checking, None);
        self.results.borrow_mut().pop_front().unwrap()
    }

    fn recover(&self) -> Result<Option<String>, GdkError> {
        self.recovered.borrow_mut().pop_front().unwrap()
    }
}

// A Developer Mode switch that starts off and counts how often the player was asked to turn it on.
#[derive(Clone, Default)]
pub struct FakeDeveloperMode {
    enable_calls: Rc<Cell<usize>>,
    on: Rc<Cell<bool>>,
}

impl FakeDeveloperMode {
    pub fn new() -> FakeDeveloperMode {
        FakeDeveloperMode::default()
    }

    pub fn enable_calls(&self) -> usize {
        self.enable_calls.get()
    }
}

impl DeveloperModeControl for FakeDeveloperMode {
    fn is_on(&self) -> Result<bool, GdkError> {
        Ok(self.on.get())
    }

    fn enable_with_consent(&self) -> Result<bool, GdkError> {
        self.enable_calls.set(self.enable_calls.get() + 1);
        self.on.set(true);
        Ok(true)
    }
}
