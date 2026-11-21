use amml::GameAnswers;
use amml_abi::{AmmlJoinOutcome, AmmlStartWorldOutcome, AmmlStatus};

#[test]
fn join_codes_of_the_failure_paths_are_failures_and_any_other_code_is_other() {
    let cases = [
        (0, AmmlJoinOutcome::OTHER),
        (1, AmmlJoinOutcome::NOT_FOUND),
        (5, AmmlJoinOutcome::REFUSED),
        (7, AmmlJoinOutcome::REFUSED),
        (8, AmmlJoinOutcome::RESOLVE_FAILED),
        (9, AmmlJoinOutcome::CONNECTIVITY_FAILED),
        (0x15, AmmlJoinOutcome::REFUSED),
        (42, AmmlJoinOutcome::OTHER),
        (-1, AmmlJoinOutcome::OTHER),
    ];
    for (code, outcome) in cases {
        assert_eq!(GameAnswers::join_outcome(code), outcome, "{code}");
    }
}

#[test]
fn a_start_the_game_is_not_ready_for_is_not_ready_and_the_refusals_have_names() {
    assert_eq!(GameAnswers::start_outcome(0), Ok(AmmlStartWorldOutcome::STARTED));
    assert_eq!(GameAnswers::start_outcome(3), Err(AmmlStatus::NOT_READY));
    assert_eq!(GameAnswers::start_outcome(13), Ok(AmmlStartWorldOutcome::NOT_FOUND_IN_FILE_SYSTEM));
    assert_eq!(GameAnswers::start_outcome(5), Ok(AmmlStartWorldOutcome::OTHER_REFUSAL));
}
