use std::panic::{AssertUnwindSafe, catch_unwind};

use amml_abi::AmmlOrigin;
use amml_runtime::BuildReentry;

#[test]
fn no_action_gives_no_actor_and_origin_game() {
    let reentry = BuildReentry::new();

    assert_eq!(reentry.actor(), None);
    assert_eq!(reentry.origin_for(3), AmmlOrigin::GAME);
}

#[test]
fn nested_action_marks_origin_this_mod() {
    let reentry = BuildReentry::new();
    let outer = reentry.enter(3);
    assert_eq!(reentry.actor(), Some(3));
    assert_eq!(reentry.origin_for(3), AmmlOrigin::THIS_MOD);
    assert_eq!(reentry.origin_for(5), AmmlOrigin::OTHER_MOD);

    {
        let _inner = reentry.enter(5);
        assert_eq!(reentry.actor(), Some(5));
        assert_eq!(reentry.origin_for(5), AmmlOrigin::THIS_MOD);
        assert_eq!(reentry.origin_for(3), AmmlOrigin::OTHER_MOD);
    }

    assert_eq!(reentry.actor(), Some(3));
    drop(outer);
    assert_eq!(reentry.actor(), None);
}

#[test]
fn scope_ends_on_unwind() {
    let reentry = BuildReentry::new();
    let _outer = reentry.enter(3);

    let unwound = catch_unwind(AssertUnwindSafe(|| {
        let _inner = reentry.enter(5);
        assert_eq!(reentry.actor(), Some(5));
        panic!("a listener panics inside the action");
    }));

    assert!(unwound.is_err());
    assert_eq!(reentry.actor(), Some(3));
}

#[test]
fn an_action_on_one_thread_is_not_seen_on_another() {
    let reentry = BuildReentry::new();
    let _scope = reentry.enter(3);

    let other = std::thread::scope(|scope| scope.spawn(|| reentry.actor()).join().unwrap());

    assert_eq!(other, None);
    assert_eq!(reentry.actor(), Some(3));
}
