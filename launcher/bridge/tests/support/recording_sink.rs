use amml_launcher_bridge::{JobSink, JobSnapshot, RegistryView};
use std::sync::{Arc, Mutex};
use std::time::Instant;

// A sink that keeps every snapshot it is given, with the time it arrived, and every registry list it is sent.
#[derive(Clone, Default)]
pub struct RecordingSink {
    seen: Arc<Mutex<Vec<(Instant, JobSnapshot)>>>,
    registries: Arc<Mutex<Vec<Vec<RegistryView>>>>,
}

impl RecordingSink {
    pub fn new() -> RecordingSink {
        RecordingSink::default()
    }

    pub fn snapshots(&self) -> Vec<JobSnapshot> {
        self.seen.lock().unwrap().iter().map(|(_, snapshot)| snapshot.clone()).collect()
    }

    pub fn count(&self) -> usize {
        self.seen.lock().unwrap().len()
    }

    pub fn registry_events(&self) -> Vec<Vec<RegistryView>> {
        self.registries.lock().unwrap().clone()
    }
}

impl JobSink for RecordingSink {
    fn publish(&self, snapshot: &JobSnapshot) {
        self.seen.lock().unwrap().push((Instant::now(), snapshot.clone()));
    }

    fn registries(&self, views: &[RegistryView]) {
        self.registries.lock().unwrap().push(views.to_vec());
    }
}
