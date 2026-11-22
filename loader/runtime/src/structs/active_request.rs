use crate::RequestMove;
use crate::structs::{RequestPhase, RequestShown};

/// The request the book runs now.
#[derive(Clone, Debug)]
pub struct ActiveRequest {
    pub id: u64,
    pub owner: usize,
    pub moves: Vec<RequestMove>,
    pub hidden_open: bool,
    pub predict: bool,
    pub phase: RequestPhase,
    /// The id of each request on the wire, in move order, once the requests are made.
    pub wire_ids: Vec<i32>,
    /// The server's answer to each request: accepted or not; `None` before it comes.
    pub answers: Vec<Option<bool>>,
    /// For each move shown on the client: what is shown.
    pub predicted: Vec<Option<RequestShown>>,
    /// For each move: the net id the server named for the slot a pick made; `None` before it comes.
    pub slot_net_ids: Vec<Option<i32>>,
    /// When the slots first did not hold what the moves need.
    pub settle_from: Option<f64>,
    /// A screen opened before the server's open arrived.
    pub cancel_pending: bool,
    /// The end was reported already (a screen opened after the send); the answers are still owed.
    pub detached: bool,
}
