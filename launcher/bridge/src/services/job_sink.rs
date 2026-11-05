use crate::{JobSnapshot, RegistryView};

// Where a running job and a registry refresh report. The app sends each snapshot and each registry list to the
// window as an event.
pub trait JobSink: Send + Sync + 'static {
    fn publish(&self, snapshot: &JobSnapshot);

    // the registry views of a refresh that ended
    fn registries(&self, views: &[RegistryView]);
}
