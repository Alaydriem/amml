use std::panic::AssertUnwindSafe;
use std::sync::Arc;
use std::time::Duration;

use amml::ShutdownLatch;

#[test]
fn one_run_per_process_also_after_a_queue() {
    let latch = ShutdownLatch::new();
    assert!(latch.queue());
    assert!(!latch.queue());
    assert!(!latch.is_closing());

    let run = latch.run();

    assert!(run.is_some());
    assert!(latch.is_closing());
    assert!(latch.run().is_none());
    drop(run);
    assert!(latch.wait_done(Duration::ZERO));
    assert!(latch.run().is_none());
}

#[test]
fn a_close_passed_on_counts_as_closing_and_still_lets_the_run_in() {
    let latch = ShutdownLatch::new();
    latch.queue();

    latch.pass_on();

    assert!(latch.is_closing());
    assert!(latch.is_queued());
    assert!(!latch.wait_done(Duration::ZERO));
    assert!(latch.run().is_some());
    assert!(latch.wait_done(Duration::ZERO));
}

#[test]
fn a_run_that_panics_still_finishes_and_wakes_the_waiter() {
    let latch = Arc::new(ShutdownLatch::new());
    latch.queue();
    let waiter = {
        let latch = latch.clone();
        std::thread::spawn(move || latch.wait_done(Duration::from_secs(10)))
    };

    let outcome = std::panic::catch_unwind(AssertUnwindSafe(|| {
        let _run = latch.run();
        panic!("a fault inside the shutdown");
    }));

    assert!(outcome.is_err());
    assert!(waiter.join().unwrap());
    assert!(latch.run().is_none());
}
