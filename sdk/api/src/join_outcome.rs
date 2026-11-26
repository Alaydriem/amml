use amml_abi::AmmlJoinOutcome;

/// How the game answered a server join (`JoinServerResultExt::outcome`). `Other` is not a failure: the game gave
/// an answer the loader does not name (or no answer before the world was joined), and the join goes on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JoinOutcome {
    Other,
    /// The game did not find the server.
    NotFound,
    /// The host name did not resolve.
    ResolveFailed,
    /// The game's connectivity test failed.
    ConnectivityFailed,
    /// The game refused the join.
    Refused,
}

impl JoinOutcome {
    /// Whether the join failed.
    pub fn is_failure(self) -> bool {
        self != Self::Other
    }

    /// The name of the outcome in log lines: `other`, `not_found`, `resolve_failed`, `connectivity_failed`,
    /// `refused`.
    pub fn name(self) -> &'static str {
        match self {
            Self::Other => "other",
            Self::NotFound => "not_found",
            Self::ResolveFailed => "resolve_failed",
            Self::ConnectivityFailed => "connectivity_failed",
            Self::Refused => "refused",
        }
    }
}

impl From<AmmlJoinOutcome> for JoinOutcome {
    fn from(outcome: AmmlJoinOutcome) -> Self {
        match outcome {
            AmmlJoinOutcome::NOT_FOUND => Self::NotFound,
            AmmlJoinOutcome::RESOLVE_FAILED => Self::ResolveFailed,
            AmmlJoinOutcome::CONNECTIVITY_FAILED => Self::ConnectivityFailed,
            AmmlJoinOutcome::REFUSED => Self::Refused,
            _ => Self::Other,
        }
    }
}
