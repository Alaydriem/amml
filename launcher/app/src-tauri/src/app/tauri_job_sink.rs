use amml_launcher_bridge::{JobSink, JobSnapshot};
use tauri::{AppHandle, Emitter};

// Sends each job snapshot to the window as an event.
pub struct TauriJobSink {
    handle: AppHandle,
}

impl TauriJobSink {
    pub const EVENT: &'static str = "launcher://job";

    pub fn new(handle: AppHandle) -> TauriJobSink {
        TauriJobSink { handle }
    }
}

impl JobSink for TauriJobSink {
    fn publish(&self, snapshot: &JobSnapshot) {
        if let Err(error) = self.handle.emit(Self::EVENT, snapshot) {
            curia::warn!("a job snapshot was not sent to the window", { error: error.to_string() });
        }
    }
}
