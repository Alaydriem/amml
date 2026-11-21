use amml_abi::{AmmlMouseButton, AmmlMoveIntent, AmmlReleaseReason};
use amml_runtime::{InjectedOp, InjectionQueue, LeaseNotice, NoticeKind, ReleaseOutcome, ResourceId};

const A: ResourceId = ResourceId(1);
const B: ResourceId = ResourceId(2);
const W: u16 = 0x57;
const S: u16 = 0x53;

fn forward() -> AmmlMoveIntent {
    AmmlMoveIntent { forward: 1.0, left: 0.0, jump: 0, sneak: 0, sprint: 0, _reserved: 0 }
}

#[test]
fn turns_of_one_frame_are_summed() {
    let queue = InjectionQueue::new();
    queue.turn(A, 1.0, 2.0);
    queue.turn(B, 3.0, 4.0);
    assert_eq!(queue.take_frame().turn, Some((4.0, 6.0)));
    assert_eq!(queue.take_frame().turn, None);
}

#[test]
fn queued_ops_reach_the_frame_in_order() {
    let queue = InjectionQueue::new();
    queue.key(A, W, true);
    queue.button(B, AmmlMouseButton::LEFT, true);
    queue.key(A, W, false);
    let frame = queue.take_frame();
    assert_eq!(
        frame.ops,
        vec![
            (A, InjectedOp::Key { key: W, down: true }),
            (B, InjectedOp::Button { button: AmmlMouseButton::LEFT, down: true }),
            (A, InjectedOp::Key { key: W, down: false }),
        ]
    );
    assert!(queue.take_frame().ops.is_empty());
}

#[test]
fn a_second_down_and_an_up_of_a_key_not_held_are_dropped() {
    let queue = InjectionQueue::new();
    queue.key(A, W, true);
    queue.key(A, W, true);
    queue.key(A, S, false);
    assert_eq!(queue.take_frame().ops, vec![(A, InjectedOp::Key { key: W, down: true })]);
    assert_eq!(queue.held(A), vec![InjectedOp::Key { key: W, down: true }]);
}

#[test]
fn release_gives_one_up_per_held_key_and_drops_queued_downs() {
    let queue = InjectionQueue::new();
    queue.key(A, W, true);
    queue.button(A, AmmlMouseButton::RIGHT, true);
    queue.take_frame();
    queue.key(A, S, true);
    assert_eq!(
        queue.release(A).ups,
        vec![
            InjectedOp::Key { key: W, down: false },
            InjectedOp::Button { button: AmmlMouseButton::RIGHT, down: false },
        ]
    );
    assert_eq!(
        queue.take_frame().ops,
        vec![
            (A, InjectedOp::Key { key: W, down: false }),
            (A, InjectedOp::Button { button: AmmlMouseButton::RIGHT, down: false }),
        ]
    );
    assert!(queue.held(A).is_empty());
    assert!(queue.take_frame().ops.is_empty());
}

#[test]
fn a_down_dropped_before_its_frame_sends_nothing() {
    let queue = InjectionQueue::new();
    queue.key(A, W, true);
    assert_eq!(queue.held(A), vec![InjectedOp::Key { key: W, down: true }]);
    assert!(queue.release(A).ups.is_empty());
    assert!(queue.take_frame().ops.is_empty());
}

#[test]
fn a_held_key_gets_exactly_one_up_however_often_it_is_released() {
    let queue = InjectionQueue::new();
    queue.key(A, W, true);
    queue.take_frame();
    queue.key(A, W, false);
    assert!(queue.release(A).ups.is_empty());
    assert!(queue.release(A).ups.is_empty());
    queue.key(A, W, false);
    assert_eq!(queue.take_frame().ops, vec![(A, InjectedOp::Key { key: W, down: false })]);

    queue.key(B, S, true);
    queue.take_frame();
    assert_eq!(queue.release(B).ups, vec![InjectedOp::Key { key: S, down: false }]);
    assert!(queue.release(B).ups.is_empty());
    assert_eq!(queue.take_frame().ops, vec![(B, InjectedOp::Key { key: S, down: false })]);
}

#[test]
fn release_everything_releases_every_lease_and_the_move_intent() {
    let queue = InjectionQueue::new();
    queue.key(A, W, true);
    queue.key(B, S, true);
    queue.set_move(B, 2, Some(forward()));
    queue.take_frame();
    assert!(queue.has_move(B));
    queue.set_move(B, 2, None);
    queue.set_move(ResourceId(3), 3, Some(forward()));
    let released = queue.release_everything();
    assert_eq!(
        released,
        vec![
            (A, ReleaseOutcome { ups: vec![InjectedOp::Key { key: W, down: false }], had_input: true }),
            (B, ReleaseOutcome { ups: vec![InjectedOp::Key { key: S, down: false }], had_input: true }),
            (ResourceId(3), ReleaseOutcome { ups: Vec::new(), had_input: true }),
        ]
    );
    assert!(!queue.has_move(ResourceId(3)));
    let frame = queue.take_frame();
    assert_eq!(
        frame.ops,
        vec![(A, InjectedOp::Key { key: W, down: false }), (B, InjectedOp::Key { key: S, down: false })]
    );
    assert_eq!(frame.move_intent, None);
}

#[test]
fn release_says_whether_the_lease_held_input_before_it_cleared_it() {
    let queue = InjectionQueue::new();
    queue.key(A, W, true);
    assert!(queue.release(A).had_input);
    queue.set_move(B, 2, Some(forward()));
    assert!(queue.release(B).had_input);
    assert!(!queue.release(B).had_input);
    assert!(!queue.release(ResourceId(9)).had_input);
}

#[test]
fn release_drops_the_leases_queued_wheel_and_turn() {
    let queue = InjectionQueue::new();
    queue.wheel(A, 2.0);
    queue.turn(A, 10.0, 5.0);
    queue.turn(B, 1.0, 1.0);
    queue.wheel(B, 1.0);
    queue.release(A);
    let frame = queue.take_frame();
    assert_eq!(frame.ops, vec![(B, InjectedOp::Wheel { notches: 1.0 })]);
    assert_eq!(frame.turn, Some((1.0, 1.0)));
}

#[test]
fn a_second_move_intent_replaces_the_first() {
    let queue = InjectionQueue::new();
    assert!(queue.set_move(A, 1, Some(forward())).is_empty());
    let mut back = forward();
    back.forward = -1.0;
    let notices = queue.set_move(B, 2, Some(back));
    assert_eq!(
        notices,
        vec![LeaseNotice { owner: 1, lease: A, kind: NoticeKind::Released(AmmlReleaseReason::REPLACED) }]
    );
    assert_eq!(queue.take_frame().move_intent, Some((B, back)));
    assert!(!queue.has_move(A));
    // A clear from the replaced lease leaves the new intent alone.
    assert!(queue.set_move(A, 1, None).is_empty());
    assert_eq!(queue.take_frame().move_intent, Some((B, back)));
}

#[test]
fn a_move_intent_stays_until_changed() {
    let queue = InjectionQueue::new();
    queue.set_move(A, 1, Some(forward()));
    assert_eq!(queue.take_frame().move_intent, Some((A, forward())));
    assert_eq!(queue.take_frame().move_intent, Some((A, forward())));
    queue.set_move(A, 1, None);
    assert_eq!(queue.take_frame().move_intent, None);
}

#[test]
fn a_wheel_fraction_is_kept_until_a_whole_notch_builds_up() {
    let queue = InjectionQueue::new();
    queue.wheel(A, 0.5);
    queue.wheel(B, 0.5);
    assert!(queue.take_frame().ops.is_empty());
    queue.wheel(A, 0.75);
    assert_eq!(queue.take_frame().ops, vec![(A, InjectedOp::Wheel { notches: 1.0 })]);
    queue.wheel(A, 0.75);
    assert_eq!(queue.take_frame().ops, vec![(A, InjectedOp::Wheel { notches: 1.0 })]);
    queue.wheel(B, -2.5);
    assert_eq!(queue.take_frame().ops, vec![(B, InjectedOp::Wheel { notches: -2.0 })]);
}

#[test]
fn release_forgets_the_wheel_fraction() {
    let queue = InjectionQueue::new();
    queue.wheel(A, 0.75);
    queue.release(A);
    queue.wheel(A, 0.5);
    assert!(queue.take_frame().ops.is_empty());
}

#[test]
fn reading_the_move_intent_leaves_it_and_the_frame_alone() {
    let queue = InjectionQueue::new();
    assert_eq!(queue.move_intent(), None);
    queue.set_move(A, 1, Some(forward()));
    queue.turn(A, 1.0, 0.0);

    assert_eq!(queue.move_intent(), Some(forward()));
    assert_eq!(queue.move_intent(), Some(forward()));
    let frame = queue.take_frame();
    assert_eq!(frame.turn, Some((1.0, 0.0)));
    assert_eq!(frame.move_intent, Some((A, forward())));
}

#[test]
fn press_again_queues_a_down_only_for_a_held_code() {
    let queue = InjectionQueue::new();
    assert!(queue.key(A, W, true));
    assert!(!queue.key(A, W, true));
    queue.take_frame();

    queue.press_again(A, InjectedOp::Key { key: W, down: true });
    queue.press_again(A, InjectedOp::Key { key: S, down: true });
    queue.press_again(B, InjectedOp::Key { key: W, down: true });
    queue.press_again(A, InjectedOp::Key { key: W, down: false });

    assert_eq!(queue.take_frame().ops, vec![(A, InjectedOp::Key { key: W, down: true })]);
    assert!(queue.key(A, W, false));
    assert_eq!(queue.take_frame().ops, vec![(A, InjectedOp::Key { key: W, down: false })]);
}

#[test]
fn a_forgotten_key_queues_no_up_and_a_new_down_presses_again() {
    let queue = InjectionQueue::new();
    queue.key(A, W, true);
    queue.take_frame();
    queue.forget(A, InjectedOp::Key { key: W, down: true });
    assert!(!queue.release(A).had_input);
    assert!(queue.take_frame().ops.is_empty());
    assert!(queue.key(A, W, true));
}
