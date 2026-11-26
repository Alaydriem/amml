use crate::{RequestDone, RequestStep};

/// What one client tick asks of the loader: the acts to do, in order, and the requests that ended.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RequestTick {
    pub steps: Vec<RequestStep>,
    pub done: Vec<RequestDone>,
}
