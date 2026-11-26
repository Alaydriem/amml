use std::cell::RefCell;

use amml_abi::AmmlRequestOutcome;

use crate::testing::FakeRequest;
use crate::{BlockHandle, InventoryMove, RequestId, RequestOptions};

/// The requests of `inventory.request` in a `FakeGame`: one runs at a time, as in the loader.
#[derive(Default)]
pub(crate) struct FakeRequests {
    requests: RefCell<Vec<FakeRequest>>,
    next: RefCell<u64>,
}

impl FakeRequests {
    /// Whether a request runs: taken and not yet ended by a frame.
    pub(crate) fn running(&self) -> bool {
        self.requests.borrow().iter().any(|request| !request.ended)
    }

    /// Takes a request and gives its id.
    pub(crate) fn take(&self, moves: Vec<InventoryMove>, options: RequestOptions) -> RequestId {
        let mut next = self.next.borrow_mut();
        *next += 1;
        let id = RequestId(*next);
        self.requests.borrow_mut().push(FakeRequest { id, moves, options, pick: None, answer: None, ended: false });
        id
    }

    /// Takes a creative pick of `block` into hotbar slot `slot` and gives its id.
    pub(crate) fn take_pick(&self, block: BlockHandle, slot: u8) -> RequestId {
        let mut next = self.next.borrow_mut();
        *next += 1;
        let id = RequestId(*next);
        let options = RequestOptions { hidden_open: false, predict: true };
        let request =
            FakeRequest { id, moves: Vec::new(), options, pick: Some((block, slot)), answer: None, ended: false };
        self.requests.borrow_mut().push(request);
        id
    }

    /// Every creative pick taken so far, in order: the id, the block and the slot.
    pub(crate) fn picks(&self) -> Vec<(RequestId, BlockHandle, u8)> {
        self.requests
            .borrow()
            .iter()
            .filter_map(|request| request.pick.map(|(block, slot)| (request.id, block, slot)))
            .collect()
    }

    /// Every request taken so far, in order.
    pub(crate) fn all(&self) -> Vec<(RequestId, Vec<InventoryMove>, RequestOptions)> {
        self.requests.borrow().iter().map(|request| (request.id, request.moves.clone(), request.options)).collect()
    }

    /// The server's answer to request `id`: `accepted` moves accepted. False for an id that is not running.
    pub(crate) fn answer(&self, id: RequestId, accepted: u16) -> bool {
        let mut requests = self.requests.borrow_mut();
        match requests.iter_mut().find(|request| request.id == id && !request.ended) {
            Some(request) => {
                request.answer = Some(accepted);
                true
            }
            None => false,
        }
    }

    /// The requests the server answered since the last call, as (id, outcome, accepted, total); they end here.
    pub(crate) fn take_ended(&self) -> Vec<(RequestId, AmmlRequestOutcome, u16, u16)> {
        let mut ended = Vec::new();
        for request in self.requests.borrow_mut().iter_mut().filter(|request| !request.ended) {
            let Some(accepted) = request.answer else { continue };
            request.ended = true;
            let total = u16::try_from(request.moves.len().max(usize::from(request.pick.is_some()))).unwrap_or(u16::MAX);
            let outcome = if accepted >= total { AmmlRequestOutcome::DONE } else { AmmlRequestOutcome::REFUSED };
            ended.push((request.id, outcome, accepted.min(total), total));
        }
        ended
    }
}
