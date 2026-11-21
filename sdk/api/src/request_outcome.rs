use amml_abi::AmmlRequestOutcome;

/// How a request of `Inventory::request_moves` ended.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RequestOutcome {
    /// The server accepted every move.
    Done,
    /// The server refused a move.
    Refused,
    /// The server did not answer every move in time.
    NoAnswer,
    /// The server did not open the inventory in time (`hidden_open` only).
    NoOpen,
    /// A screen opened before the moves were sent, or after them with the hidden open (the server still does the
    /// moves), or the world was left.
    Cancelled,
    /// The slots did not hold what a move needs when it was to be sent; nothing was sent.
    Stale,
    /// The loader could not send a packet of the request; nothing was predicted.
    NotSent,
    /// An outcome this crate does not name.
    Other(u8),
}

impl From<u8> for RequestOutcome {
    fn from(outcome: u8) -> Self {
        match AmmlRequestOutcome(outcome) {
            AmmlRequestOutcome::DONE => Self::Done,
            AmmlRequestOutcome::REFUSED => Self::Refused,
            AmmlRequestOutcome::NO_ANSWER => Self::NoAnswer,
            AmmlRequestOutcome::NO_OPEN => Self::NoOpen,
            AmmlRequestOutcome::CANCELLED => Self::Cancelled,
            AmmlRequestOutcome::STALE => Self::Stale,
            AmmlRequestOutcome::NOT_SENT => Self::NotSent,
            AmmlRequestOutcome(other) => Self::Other(other),
        }
    }
}

impl From<RequestOutcome> for AmmlRequestOutcome {
    fn from(outcome: RequestOutcome) -> Self {
        match outcome {
            RequestOutcome::Done => Self::DONE,
            RequestOutcome::Refused => Self::REFUSED,
            RequestOutcome::NoAnswer => Self::NO_ANSWER,
            RequestOutcome::NoOpen => Self::NO_OPEN,
            RequestOutcome::Cancelled => Self::CANCELLED,
            RequestOutcome::Stale => Self::STALE,
            RequestOutcome::NotSent => Self::NOT_SENT,
            RequestOutcome::Other(other) => Self(other),
        }
    }
}
