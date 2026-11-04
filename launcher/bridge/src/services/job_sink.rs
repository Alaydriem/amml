use crate::JobSnapshot;

// Where a running job reports. The app sends each snapshot to the window as an event.
pub trait JobSink: Send + Sync + 'static {
    fn publish(&self, snapshot: &JobSnapshot);
}
