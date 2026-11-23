use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use amml_runtime::services::EventBus;
use amml_runtime::structs::{CallOutcome, ListenError};

// A listener is a plain tag; the test closure decides what a tag does.
type Bus = EventBus<u32>;

fn bus() -> Bus {
    let bus = Bus::new();
    bus.offer("tick", 1);
    bus
}

#[test]
fn listeners_run_in_band_order_then_registration_order() {
    let bus = bus();
    // bands 2, 0, 2, 4 with tags 20, 0, 21, 40
    bus.add("tick", 1, 2, 1, 20).unwrap();
    bus.add("tick", 1, 0, 1, 0).unwrap();
    bus.add("tick", 1, 2, 1, 21).unwrap();
    bus.add("tick", 1, 4, 1, 40).unwrap();
    let mut calls = Vec::new();
    let mut value = 0u32;
    bus.dispatch("tick", &mut value, |tag, _| {
        calls.push(*tag);
        CallOutcome::Done
    });
    assert_eq!(calls, vec![0, 20, 21, 40]);
}

#[test]
fn adding_to_an_event_that_was_never_offered_fails() {
    let bus = bus();
    assert_eq!(bus.add("other", 1, 2, 1, 0), Err(ListenError::UnknownEvent));
}

#[test]
fn a_newer_version_than_offered_fails() {
    let bus = bus();
    assert_eq!(bus.add("tick", 2, 2, 1, 0), Err(ListenError::VersionTooNew));
    assert!(bus.is_available("tick", 1));
    assert!(!bus.is_available("tick", 2));
}

#[test]
fn a_cancel_stays_cancelled_for_later_listeners() {
    let bus = bus();
    bus.add("tick", 1, 2, 1, 1).unwrap();
    bus.add("tick", 1, 2, 2, 2).unwrap();
    let mut ran = Vec::new();
    let mut value = 0u32;
    let report = bus.dispatch("tick", &mut value, |tag, _| {
        ran.push(*tag);
        if *tag == 1 { CallOutcome::Cancelled } else { CallOutcome::Done }
    });
    assert!(report.cancelled);
    assert_eq!(ran, vec![1, 2]);
}

#[test]
fn the_report_names_the_owner_of_the_first_cancel() {
    let bus = bus();
    bus.add("tick", 1, 2, 10, 0).unwrap();
    bus.add("tick", 1, 2, 20, 1).unwrap();
    bus.add("tick", 1, 2, 30, 2).unwrap();
    let mut value = 0u32;
    let report =
        bus.dispatch("tick", &mut value, |tag, _| if *tag >= 1 { CallOutcome::Cancelled } else { CallOutcome::Done });
    assert_eq!(report.cancelled_by, Some(20));
}

#[test]
fn a_panicking_listener_has_its_changes_undone() {
    let bus = bus();
    bus.add("tick", 1, 2, 10, 1).unwrap();
    bus.add("tick", 1, 2, 20, 2).unwrap();
    bus.add("tick", 1, 2, 30, 3).unwrap();
    let mut seen_by_third = 0;
    let mut value = 1u32;
    let report = bus.dispatch("tick", &mut value, |tag, v| match *tag {
        1 => {
            *v = 2;
            CallOutcome::Done
        }
        2 => {
            *v = 9;
            CallOutcome::Panicked
        }
        _ => {
            seen_by_third = *v;
            CallOutcome::Done
        }
    });
    assert_eq!(seen_by_third, 2);
    assert_eq!(report.panicked_owners, vec![20]);
}

#[test]
fn remove_waits_for_a_call_in_progress() {
    let bus = Arc::new(bus());
    let id = bus.add("tick", 1, 2, 1, 1).unwrap();
    let started = Arc::new(AtomicBool::new(false));
    let finished = Arc::new(AtomicBool::new(false));

    let dispatcher = {
        let (bus, started, finished) = (bus.clone(), started.clone(), finished.clone());
        thread::spawn(move || {
            let mut value = 0u32;
            bus.dispatch("tick", &mut value, |_, _| {
                started.store(true, Ordering::SeqCst);
                thread::sleep(Duration::from_millis(300));
                finished.store(true, Ordering::SeqCst);
                CallOutcome::Done
            });
        })
    };
    let deadline = Instant::now() + Duration::from_secs(10);
    while !started.load(Ordering::SeqCst) {
        assert!(Instant::now() < deadline, "listener never started");
        thread::sleep(Duration::from_millis(1));
    }
    assert!(bus.remove(id));
    assert!(finished.load(Ordering::SeqCst));
    dispatcher.join().unwrap();
    assert!(!bus.remove(id));
}

#[test]
fn a_removed_listener_is_not_called_by_a_dispatch_that_already_took_its_list() {
    let bus = Arc::new(bus());
    bus.add("tick", 1, 0, 1, 1).unwrap();
    let second = bus.add("tick", 1, 2, 2, 2).unwrap();
    let mut ran = Vec::new();
    let mut value = 0u32;
    bus.dispatch("tick", &mut value, |tag, _| {
        ran.push(*tag);
        if *tag == 1 {
            assert!(bus.remove(second));
        }
        CallOutcome::Done
    });
    assert_eq!(ran, vec![1]);
}

#[test]
fn a_listener_may_remove_itself_without_hanging() {
    let bus = Arc::new(bus());
    let own = bus.add("tick", 1, 2, 1, 1).unwrap();
    let mut value = 0u32;
    let mut removed = false;
    bus.dispatch("tick", &mut value, |_, _| {
        removed = bus.remove(own);
        CallOutcome::Done
    });
    assert!(removed);
    assert!(!bus.remove(own));
}

#[test]
fn remove_owner_removes_only_that_owners_listeners() {
    let bus = bus();
    bus.add("tick", 1, 2, 1, 10).unwrap();
    bus.add("tick", 1, 2, 2, 20).unwrap();
    bus.add("tick", 1, 2, 1, 11).unwrap();
    assert_eq!(bus.remove_owner(1), 2);
    let mut calls = Vec::new();
    let mut value = 0u32;
    bus.dispatch("tick", &mut value, |tag, _| {
        calls.push(*tag);
        CallOutcome::Done
    });
    assert_eq!(calls, vec![20]);
}

#[test]
fn remove_owner_waits_for_a_listener_that_removed_itself() {
    let bus = Arc::new(bus());
    let id = bus.add("tick", 1, 2, 7, 1).unwrap();
    let removed = Arc::new(AtomicBool::new(false));
    let release = Arc::new(AtomicBool::new(false));
    let ended = Arc::new(AtomicBool::new(false));
    let disabled = Arc::new(AtomicBool::new(false));

    let dispatcher = {
        let (bus, removed, release, ended) = (bus.clone(), removed.clone(), release.clone(), ended.clone());
        thread::spawn(move || {
            let mut value = 0u32;
            bus.dispatch("tick", &mut value, |_, _| {
                assert!(bus.remove(id));
                removed.store(true, Ordering::SeqCst);
                while !release.load(Ordering::SeqCst) {
                    thread::sleep(Duration::from_millis(1));
                }
                ended.store(true, Ordering::SeqCst);
                CallOutcome::Done
            });
        })
    };
    let deadline = Instant::now() + Duration::from_secs(10);
    while !removed.load(Ordering::SeqCst) {
        assert!(Instant::now() < deadline, "listener never removed itself");
        thread::sleep(Duration::from_millis(1));
    }
    let disabler = {
        let (bus, disabled, ended) = (bus.clone(), disabled.clone(), ended.clone());
        thread::spawn(move || {
            bus.remove_owner(7);
            assert!(ended.load(Ordering::SeqCst));
            disabled.store(true, Ordering::SeqCst);
        })
    };
    thread::sleep(Duration::from_millis(150));
    assert!(!disabled.load(Ordering::SeqCst));
    release.store(true, Ordering::SeqCst);
    disabler.join().unwrap();
    dispatcher.join().unwrap();
    assert!(disabled.load(Ordering::SeqCst));
}

#[test]
fn adding_for_a_closed_owner_fails_until_it_is_reopened() {
    let bus = bus();
    bus.add("tick", 1, 2, 3, 1).unwrap();
    assert_eq!(bus.remove_owner(3), 1);
    assert_eq!(bus.add("tick", 1, 2, 3, 2), Err(ListenError::OwnerClosed));
    bus.reopen_owner(3);
    assert!(bus.add("tick", 1, 2, 3, 3).is_ok());
}

#[test]
fn the_split_runs_between_normal_and_later_bands() {
    let bus = bus();
    bus.add("tick", 1, 0, 1, 0).unwrap();
    bus.add("tick", 1, 2, 1, 20).unwrap();
    bus.add("tick", 1, 3, 1, 30).unwrap();
    let order = std::cell::RefCell::new(Vec::new());
    let mut value = 0u32;
    bus.dispatch_split(
        "tick",
        &mut value,
        2,
        |tag, _| {
            order.borrow_mut().push(*tag);
            CallOutcome::Done
        },
        |_| order.borrow_mut().push(999),
    );
    assert_eq!(*order.borrow(), vec![0, 20, 999, 30]);
}

#[test]
fn the_split_runs_between_when_the_normal_band_is_empty() {
    let bus = bus();
    bus.add("tick", 1, 0, 1, 0).unwrap();
    bus.add("tick", 1, 3, 1, 30).unwrap();
    let order = std::cell::RefCell::new(Vec::new());
    let mut value = 0u32;
    bus.dispatch_split(
        "tick",
        &mut value,
        2,
        |tag, _| {
            order.borrow_mut().push(*tag);
            CallOutcome::Done
        },
        |_| order.borrow_mut().push(999),
    );
    assert_eq!(*order.borrow(), vec![0, 999, 30]);
}

#[test]
fn the_split_runs_between_once_with_no_listeners_or_none_later() {
    let bus = bus();
    let mut value = 0u32;
    let mut between = 0;
    bus.dispatch_split("tick", &mut value, 2, |_, _| CallOutcome::Done, |_| between += 1);
    assert_eq!(between, 1);
    bus.add("tick", 1, 2, 1, 20).unwrap();
    bus.dispatch_split("tick", &mut value, 2, |_, _| CallOutcome::Done, |_| between += 1);
    assert_eq!(between, 2);
}

#[test]
fn the_split_between_sees_the_value_the_normal_band_left_and_a_cancel_carries_on() {
    let bus = bus();
    bus.add("tick", 1, 2, 1, 20).unwrap();
    bus.add("tick", 1, 3, 1, 30).unwrap();
    let mut value = 0u32;
    let mut seen = 0;
    let report = bus.dispatch_split(
        "tick",
        &mut value,
        2,
        |tag, v| {
            *v += *tag;
            if *tag == 20 { CallOutcome::Cancelled } else { CallOutcome::Done }
        },
        |v| seen = *v,
    );
    assert_eq!(seen, 20);
    assert_eq!(value, 50);
    assert!(report.cancelled);
}

#[test]
fn a_panic_in_the_normal_band_restores_the_value_before_between() {
    let bus = bus();
    bus.add("tick", 1, 2, 7, 20).unwrap();
    let mut value = 5u32;
    let mut seen = 0;
    let report = bus.dispatch_split(
        "tick",
        &mut value,
        2,
        |_, v| {
            *v = 99;
            CallOutcome::Panicked
        },
        |v| seen = *v,
    );
    assert_eq!(seen, 5);
    assert_eq!(report.panicked_owners, vec![7]);
}

#[test]
fn has_listeners_follows_adds_and_removes() {
    let bus = bus();
    assert!(!bus.has_listeners("tick"));
    let id = bus.add("tick", 1, 2, 1, 0).unwrap();
    assert!(bus.has_listeners("tick"));
    assert!(!bus.has_listeners("other"));

    assert!(bus.remove(id));

    assert!(!bus.has_listeners("tick"));
}

#[test]
fn lowest_version_is_the_oldest_listener_asked_for() {
    let bus = bus();
    bus.offer("tick", 2);
    assert_eq!(bus.lowest_version("tick"), None);
    let new = bus.add("tick", 2, 2, 1, 0).unwrap();
    assert_eq!(bus.lowest_version("tick"), Some(2));
    let old = bus.add("tick", 1, 4, 1, 0).unwrap();
    assert_eq!(bus.lowest_version("tick"), Some(1));

    assert!(bus.remove(old));
    assert_eq!(bus.lowest_version("tick"), Some(2));
    assert!(bus.remove(new));
    assert_eq!(bus.lowest_version("tick"), None);
}
