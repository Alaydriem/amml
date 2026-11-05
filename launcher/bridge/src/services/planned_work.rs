use crate::JobError;
use amml_launcher_core::{CoreProgress, Manager};

// The work of one job. Manager work waits for the manager slot; plain work does not use the manager and never waits.
pub(crate) enum PlannedWork {
    Manager(JobWork),
    Plain(PlainWork),
}

// work that gets the manager and the progress callback
pub(crate) type JobWork = Box<dyn FnOnce(&mut Manager, &mut dyn FnMut(CoreProgress)) -> Result<(), JobError> + Send>;

// work that gets only the progress callback
pub(crate) type PlainWork = Box<dyn FnOnce(&mut dyn FnMut(CoreProgress)) -> Result<(), JobError> + Send>;
