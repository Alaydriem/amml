use super::{JobSink, Locked};
use crate::{JobKind, JobSnapshot};
use amml_gdk::CancelFlag;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64};
use std::thread::JoinHandle;

// The state a `JobRunner` shares with its job thread.
pub(crate) struct JobShared<S: JobSink> {
    pub(crate) sink: S,
    pub(crate) cancel: CancelFlag,
    pub(crate) busy: AtomicBool,
    // launches in progress; a job does not start while one runs
    pub(crate) launching: Arc<AtomicU32>,
    // held from the check of the job id to the end of the sink call, so snapshots reach the sink in the order they
    // were made
    publish_lock: Mutex<()>,
    // the job that runs now; empty between jobs
    pub(crate) current: Mutex<Option<JobKind>>,
    pub(crate) next_id: AtomicU64,
    // the running job, or the last one that ended
    pub(crate) latest: Mutex<Option<JobSnapshot>>,
    pub(crate) thread: Mutex<Option<JoinHandle<()>>>,
}

impl<S: JobSink> JobShared<S> {
    pub(crate) fn new(sink: S, cancel: CancelFlag) -> JobShared<S> {
        JobShared {
            sink,
            cancel,
            busy: AtomicBool::new(false),
            launching: Arc::new(AtomicU32::new(0)),
            publish_lock: Mutex::new(()),
            current: Mutex::new(None),
            next_id: AtomicU64::new(0),
            latest: Mutex::new(None),
            thread: Mutex::new(None),
        }
    }

    // Keeps the snapshot as the latest, then sends it on. A snapshot of an older job than the latest is dropped.
    pub(crate) fn publish(&self, snapshot: JobSnapshot) {
        let _order = Locked::of(&self.publish_lock);
        self.send(snapshot);
    }

    // Frees the runner and sends the last snapshot of a job. The runner is freed while the publish lock is held, so
    // the first snapshot of the next job cannot get ahead of this one.
    pub(crate) fn publish_last(&self, free: impl FnOnce(), snapshot: JobSnapshot) {
        let _order = Locked::of(&self.publish_lock);
        free();
        self.send(snapshot);
    }

    fn send(&self, snapshot: JobSnapshot) {
        {
            let mut latest = Locked::of(&self.latest);
            if latest.as_ref().is_some_and(|held| held.id > snapshot.id) {
                return;
            }
            *latest = Some(snapshot.clone());
        }
        self.sink.publish(&snapshot);
    }
}
