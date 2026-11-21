use crate::{InventoryMove, RequestId, RequestOptions};

/// One request the fake took: the moves, the options and, once `answer` ran, how many moves the server accepted.
pub(crate) struct FakeRequest {
    pub(crate) id: RequestId,
    pub(crate) moves: Vec<InventoryMove>,
    pub(crate) options: RequestOptions,
    pub(crate) answer: Option<u16>,
    pub(crate) ended: bool,
}
