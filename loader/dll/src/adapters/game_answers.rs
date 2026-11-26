use amml_abi::{AmmlJoinOutcome, AmmlStartWorldOutcome, AmmlStatus};

/// Maps the answer codes of the game's session calls to AMML values, so a mod never sees a game code.
pub struct GameAnswers;

impl GameAnswers {
    /// The outcome of the code the game passes to a join's result callback. Only the codes of the game's failure
    /// paths are failures: 1 the server was not found, 8 the host did not resolve, 9 the connectivity test failed,
    /// 5 to 7 and 15h refusals. Any other code (0 included) is `OTHER`.
    pub fn join_outcome(code: i32) -> AmmlJoinOutcome {
        match code {
            1 => AmmlJoinOutcome::NOT_FOUND,
            8 => AmmlJoinOutcome::RESOLVE_FAILED,
            9 => AmmlJoinOutcome::CONNECTIVITY_FAILED,
            5..=7 | 0x15 => AmmlJoinOutcome::REFUSED,
            _ => AmmlJoinOutcome::OTHER,
        }
    }

    /// The outcome of the `World::StartLocalWorldResult` the game's local world start returns: 0 taken; 3 the game
    /// is not ready (`NOT_READY`, ask again later); the refusal codes 1, 2, 9, 12, 13 and 14 by name; any other code
    /// `OTHER_REFUSAL`.
    pub fn start_outcome(code: u32) -> Result<AmmlStartWorldOutcome, AmmlStatus> {
        match code {
            0 => Ok(AmmlStartWorldOutcome::STARTED),
            1 => Ok(AmmlStartWorldOutcome::NOT_IN_LEVEL_LIST),
            2 => Ok(AmmlStartWorldOutcome::STARTUP_IN_PROGRESS),
            3 => Err(AmmlStatus::NOT_READY),
            9 => Ok(AmmlStartWorldOutcome::SAVED_IN_NEWER_VERSION),
            12 => Ok(AmmlStartWorldOutcome::PLATFORM_LOCKED_CONTENT),
            13 => Ok(AmmlStartWorldOutcome::NOT_FOUND_IN_FILE_SYSTEM),
            14 => Ok(AmmlStartWorldOutcome::SLIM_CACHE_DATA),
            _ => Ok(AmmlStartWorldOutcome::OTHER_REFUSAL),
        }
    }

    /// The name of a join outcome in log lines.
    pub fn join_outcome_name(outcome: AmmlJoinOutcome) -> &'static str {
        match outcome {
            AmmlJoinOutcome::NOT_FOUND => "not_found",
            AmmlJoinOutcome::RESOLVE_FAILED => "resolve_failed",
            AmmlJoinOutcome::CONNECTIVITY_FAILED => "connectivity_failed",
            AmmlJoinOutcome::REFUSED => "refused",
            _ => "other",
        }
    }
}
