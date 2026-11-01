use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Barrier};
use std::time::Duration;

use amml::StartupGate;

#[test]
fn two_threads_at_the_gate_run_start_up_once_and_both_continue_after_it_finished() {
    let gate = Arc::new(StartupGate::new());
    let runs = Arc::new(AtomicUsize::new(0));
    let barrier = Arc::new(Barrier::new(2));

    let threads: Vec<_> = (0..2)
        .map(|_| {
            let (gate, runs, barrier) = (gate.clone(), runs.clone(), barrier.clone());
            std::thread::spawn(move || {
                barrier.wait();
                gate.pass(|| {
                    std::thread::sleep(Duration::from_millis(200));
                    runs.fetch_add(1, Ordering::SeqCst);
                });
                runs.load(Ordering::SeqCst)
            })
        })
        .collect();
    let seen: Vec<usize> = threads.into_iter().map(|thread| thread.join().unwrap()).collect();

    assert_eq!(runs.load(Ordering::SeqCst), 1);
    assert_eq!(seen, [1, 1]);
    assert!(gate.is_done());
}

#[test]
fn a_panic_in_start_up_is_stopped_and_start_up_does_not_run_again() {
    let gate = StartupGate::new();
    let runs = AtomicUsize::new(0);

    gate.pass(|| {
        runs.fetch_add(1, Ordering::SeqCst);
        panic!("start-up failed");
    });
    gate.pass(|| {
        runs.fetch_add(1, Ordering::SeqCst);
    });

    assert_eq!(runs.load(Ordering::SeqCst), 1);
    assert!(gate.is_done());
}
