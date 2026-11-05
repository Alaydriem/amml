use super::job_shared::JobShared;
use super::{BusyReset, FaultMapper, InstallSteps, JobSink, LaunchTicket, Locked, UnixMillis};
use crate::{BridgeError, FaultKindView, FaultView, JobError, JobKind, JobSnapshot, JobStateView};
use amml_gdk::CancelFlag;
use amml_launcher_core::CoreProgress;
use std::collections::BTreeMap;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

// Runs one job at a time on one thread and tells the sink where it stands. A snapshot of the job is always
// available, so a screen that opens mid-job has the current step and bytes at once.
pub struct JobRunner<S: JobSink> {
    shared: Arc<JobShared<S>>,
}

impl<S: JobSink> JobRunner<S> {
    // byte progress is sent at most this often; a step change is sent at once
    const BYTE_GAP: Duration = Duration::from_millis(100);

    pub fn new(sink: S, cancel: CancelFlag) -> JobRunner<S> {
        JobRunner { shared: Arc::new(JobShared::new(sink, cancel)) }
    }

    // Refuses with `Busy` while a job runs; the work does not run then.
    pub fn start<W>(&self, kind: JobKind, mut steps: InstallSteps, work: W) -> Result<u64, BridgeError>
    where
        W: FnOnce(&mut dyn FnMut(CoreProgress)) -> Result<(), JobError> + Send + 'static,
    {
        if self.shared.busy.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst).is_err() {
            return Err(BridgeError::Busy);
        }
        // A launch that began before this job holds the game's files. `begin_launch` raises its count before it reads
        // the busy flag and this reads the count after it sets the flag, so one of the two always sees the other.
        if self.shared.launching.load(Ordering::SeqCst) > 0 {
            self.shared.busy.store(false, Ordering::SeqCst);
            return Err(BridgeError::Busy);
        }
        *Locked::of(&self.shared.current) = Some(kind.clone());
        self.shared.cancel.reset();
        let id = self.shared.next_id.fetch_add(1, Ordering::SeqCst) + 1;
        let started = UnixMillis::now();
        steps.start(Instant::now());
        self.shared.publish(Self::build(id, &kind, &steps, JobStateView::Running, None, started));

        let shared = Arc::clone(&self.shared);
        let spawned = std::thread::Builder::new()
            .name("launcher-job".to_string())
            .spawn(move || Self::run(&shared, id, &kind, steps, started, work));
        match spawned {
            Ok(handle) => {
                match self.shared.thread.lock() {
                    Ok(mut held) => *held = Some(handle),
                    Err(poisoned) => *poisoned.into_inner() = Some(handle),
                }
                Ok(id)
            }
            Err(error) => {
                *Locked::of(&self.shared.current) = None;
                self.shared.busy.store(false, Ordering::SeqCst);
                Err(error.into())
            }
        }
    }

    // the running job, or the last one that ended until the next one starts
    pub fn snapshot(&self) -> Option<JobSnapshot> {
        match self.shared.latest.lock() {
            Ok(latest) => latest.clone(),
            Err(poisoned) => poisoned.into_inner().clone(),
        }
    }

    // stops the job at its next download
    pub fn cancel(&self) {
        self.shared.cancel.cancel();
    }

    pub fn is_busy(&self) -> bool {
        self.shared.busy.load(Ordering::SeqCst)
    }

    // the sink the jobs report to; a registry refresh reports to it too
    pub(crate) fn sink(&self) -> &S {
        &self.shared.sink
    }

    // the flag a cancel sets; a job that waits for the manager reads it too
    pub fn cancel_flag(&self) -> &CancelFlag {
        &self.shared.cancel
    }

    // whether a launch is in progress
    pub fn is_launching(&self) -> bool {
        self.shared.launching.load(Ordering::SeqCst) > 0
    }

    // Starts a launch of this instance: `Busy` when a job stops it. While the ticket lives no job starts.
    pub fn begin_launch(&self, instance: &str) -> Result<LaunchTicket, BridgeError> {
        let ticket = LaunchTicket::new(&self.shared.launching);
        if self.blocks_launch(instance) {
            return Err(BridgeError::Busy);
        }
        Ok(ticket)
    }

    // Whether the running job stops a launch of this instance: a job for that instance, or a job that is about the
    // whole app.
    pub fn blocks_launch(&self, instance: &str) -> bool {
        if !self.is_busy() {
            return false;
        }
        match Locked::of(&self.shared.current).as_ref() {
            Some(kind) => kind.instance().is_none_or(|held| held.eq_ignore_ascii_case(instance)),
            None => true,
        }
    }

    // waits for the job thread to end
    pub fn wait(&self) {
        let handle = match self.shared.thread.lock() {
            Ok(mut held) => held.take(),
            Err(poisoned) => poisoned.into_inner().take(),
        };
        if let Some(handle) = handle {
            let _ = handle.join();
        }
    }

    fn run<W>(shared: &JobShared<S>, id: u64, kind: &JobKind, mut steps: InstallSteps, started: u64, work: W)
    where
        W: FnOnce(&mut dyn FnMut(CoreProgress)) -> Result<(), JobError>,
    {
        // frees the runner on a panic that skips the explicit release below
        let free = BusyReset::new(&shared.busy, &shared.current);
        let mut last_publish = Instant::now();
        let outcome = catch_unwind(AssertUnwindSafe(|| {
            let mut report = |event: CoreProgress| {
                let now = Instant::now();
                let changed = steps.apply(&event, now);
                if changed || now.saturating_duration_since(last_publish) >= Self::BYTE_GAP {
                    last_publish = now;
                    shared.publish(Self::build(id, kind, &steps, JobStateView::Running, None, started));
                }
            };
            work(&mut report)
        }));

        let (state, fault) = match outcome {
            Ok(Ok(())) => {
                steps.finish(Instant::now());
                (JobStateView::Done, None)
            }
            Ok(Err(error)) if error.is_cancelled() => {
                curia::info!("a job was cancelled", { job: kind.slug() });
                (JobStateView::Cancelled, None)
            }
            Ok(Err(error)) => {
                curia::error!("a job failed", { job: kind.slug(), error: error.to_string() });
                steps.fail();
                (JobStateView::Failed, Some(Self::fault_of(&error, &steps, kind)))
            }
            Err(_) => {
                curia::error!("a job stopped on a panic", { job: kind.slug() });
                steps.fail();
                let fault = FaultView::new(
                    FaultKindView::OperationFailed,
                    Some("the task stopped unexpectedly".to_string()),
                    BTreeMap::new(),
                );
                (JobStateView::Failed, Some(fault))
            }
        };
        let last = Self::build(id, kind, &steps, state, fault, started);
        // the runner is free before the last snapshot goes out, so a screen that reacts to it can start the next call
        shared.publish_last(|| free.release(), last);
    }

    fn fault_of(error: &JobError, steps: &InstallSteps, kind: &JobKind) -> FaultView {
        let mut fault = match error {
            JobError::Launcher(inner) => FaultMapper::from_launcher(inner),
            JobError::Gdk(inner) => FaultMapper::from_gdk(inner),
        };
        // a restore that failed is the one case where the Store install is known not to be back
        fault.restored = steps.restore_seen() && fault.kind != FaultKindView::RestoreFailed;
        // the install dialogs and banners name the game version the job was making
        if let JobKind::CreateInstance { game, .. } = kind {
            fault.values.entry("version".to_string()).or_insert_with(|| game.clone());
        }
        if fault.kind == FaultKindView::DownloadFailed
            && let Some(progress) = steps.progress()
        {
            fault.values.insert("done".to_string(), progress.done.to_string());
            fault.values.insert("total".to_string(), progress.total.to_string());
        }
        fault
    }

    fn build(
        id: u64,
        kind: &JobKind,
        steps: &InstallSteps,
        state: JobStateView,
        fault: Option<FaultView>,
        started: u64,
    ) -> JobSnapshot {
        JobSnapshot {
            id,
            kind: kind.slug().to_string(),
            title_name: kind.title_name(),
            version: kind.version(),
            state,
            steps: steps.steps(),
            step_number: steps.active_number(),
            step_total: steps.total(),
            progress: if state == JobStateView::Running { steps.progress() } else { None },
            fault,
            started,
        }
    }
}
