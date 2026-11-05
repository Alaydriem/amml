use super::{JobRunner, JobSink, LaunchGate, Locked, ManagerSlot, RegistryRefresher};
use crate::{BridgeError, FaultView, JobKind, RegistryView};
use amml_gdk::{GdkError, WindowsProcessInspector};
use amml_launcher_core::{CoreProgress, CoreServices, LauncherError, Manager, ProcessGameStarter, WindowsWindowProbe};
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::sync::Mutex;

// What a `LauncherSession` shares with its job threads.
pub(crate) struct SessionState<S: JobSink> {
    pub(crate) services: CoreServices,
    // built on first use; a job holds it for as long as it runs
    pub(crate) manager: ManagerSlot<Manager>,
    pub(crate) runner: JobRunner<S>,
    // Taken by job start and by every call that refuses while a job runs and then acts, so the check and the act
    // cannot straddle a job start. Never held across a whole job.
    pub(crate) start_guard: Mutex<()>,
    pub(crate) gate: LaunchGate<ProcessGameStarter, WindowsWindowProbe, WindowsProcessInspector>,
    // the views of the last refresh; replaced whole, so bounded by the registries in the settings
    pub(crate) registries: Mutex<Vec<RegistryView>>,
    // updates waiting per instance; rebuilt on each refresh, so bounded by the instances
    pub(crate) updates: Mutex<BTreeMap<String, u32>>,
    // one registry refresh at a time, on its own catalogue; it never takes the manager
    pub(crate) refresher: RegistryRefresher,
    pub(crate) startup: Mutex<Option<FaultView>>,
    // the last job that started, for a retry
    pub(crate) last_job: Mutex<Option<JobKind>>,
}

impl<S: JobSink> SessionState<S> {
    // Runs a job's work on the manager. The job waits for a reader that holds it, shows a waiting step meanwhile and
    // ends as cancelled when the player cancels.
    pub(crate) fn run_job<R, E: From<LauncherError> + From<GdkError>>(
        &self,
        progress: &mut dyn FnMut(CoreProgress),
        f: impl FnOnce(&mut Manager, &mut dyn FnMut(CoreProgress)) -> Result<R, E>,
    ) -> Result<R, E> {
        // the wait and the work both report through the one callback, one after the other
        let progress = RefCell::new(progress);
        self.manager.run_waiting(
            self.runner.cancel_flag(),
            |waiting| (progress.borrow_mut())(CoreProgress::WaitingForManager { waiting }),
            || self.services.manager(),
            |manager| f(manager, &mut **progress.borrow_mut()),
        )
    }

    // Runs `f` on the manager. `Busy` as soon as a job runs; another reader is waited for.
    pub(crate) fn try_manager<R>(
        &self,
        f: impl FnOnce(&mut Manager) -> Result<R, BridgeError>,
    ) -> Result<R, BridgeError> {
        if self.runner.is_busy() {
            return Err(BridgeError::Busy);
        }
        self.manager.run_unless_busy(|| self.runner.is_busy(), || self.services.manager(), f)
    }

    // Runs `f` while no job can start, if no job runs and no launch is in progress. `Busy` otherwise. Job start takes the same guard, so a check
    // that no job runs holds until `f` ends. Keep `f` short.
    pub(crate) fn unless_busy<R>(&self, f: impl FnOnce() -> Result<R, BridgeError>) -> Result<R, BridgeError> {
        let _guard = Locked::of(&self.start_guard);
        if self.runner.is_busy() || self.runner.is_launching() {
            return Err(BridgeError::Busy);
        }
        f()
    }

    // Drops the manager so the next use builds it from the settings on disk.
    pub(crate) fn reset_manager(&self) {
        self.manager.reset();
    }
}
