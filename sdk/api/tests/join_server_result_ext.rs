use std::sync::{Arc, Mutex};

use amml_abi::{AmmlJoinOutcome, JoinServerResult};
use amml_api::testing::FakeGame;
use amml_api::{HookOrder, JoinId, JoinOutcome, JoinServerResultExt};

#[test]
fn a_join_result_listener_reads_the_join_and_its_outcome() {
    let fake = FakeGame::new();
    fake.offer::<JoinServerResult>();
    let seen = Arc::new(Mutex::new(Vec::new()));
    let record = seen.clone();
    let _listener = fake
        .ctx()
        .events()
        .listen::<JoinServerResult>(HookOrder::NORMAL, move |event, _| {
            record.lock().unwrap().push((event.join_id(), event.outcome()));
        })
        .unwrap();

    fake.raise_join_result(JoinId(3), AmmlJoinOutcome::RESOLVE_FAILED);
    fake.raise_join_result(JoinId(4), AmmlJoinOutcome(77));

    assert_eq!(*seen.lock().unwrap(), [(JoinId(3), JoinOutcome::ResolveFailed), (JoinId(4), JoinOutcome::Other)]);
}

#[test]
fn only_the_named_outcomes_are_failures() {
    assert!(!JoinOutcome::Other.is_failure());
    assert!(JoinOutcome::Refused.is_failure());
    assert_eq!(JoinOutcome::ResolveFailed.name(), "resolve_failed");
}
