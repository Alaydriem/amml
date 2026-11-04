use amml_gdk::CancelFlag;
use amml_launcher_bridge::{BridgeError, JobError, ManagerSlot};
use amml_launcher_core::LauncherError;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex, mpsc};
use std::time::{Duration, Instant};

fn build() -> Result<u32, LauncherError> {
    Ok(0)
}

#[test]
fn the_manager_is_built_on_first_use_and_kept() {
    let slot: ManagerSlot<u32> = ManagerSlot::new();
    let mut builds = 0;

    for _ in 0..3 {
        slot.run(
            || {
                builds += 1;
                build()
            },
            |manager| {
                *manager += 1;
                Ok::<_, LauncherError>(*manager)
            },
        )
        .unwrap();
    }

    assert_eq!(builds, 1);
    let kept = slot.run(build, |manager| Ok::<_, LauncherError>(*manager)).unwrap();
    assert_eq!(kept, 3);
    slot.reset();
    assert_eq!(slot.run(build, |manager| Ok::<_, LauncherError>(*manager)).unwrap(), 0);
}

#[test]
fn a_second_reader_waits_for_the_first_and_is_not_told_busy() {
    let slot = Arc::new(ManagerSlot::<u32>::new());
    let (inside, entered) = mpsc::channel::<()>();
    let (release, gate) = mpsc::channel::<()>();
    let first = {
        let slot = Arc::clone(&slot);
        std::thread::spawn(move || {
            slot.run(build, |_| {
                inside.send(()).unwrap();
                let _ = gate.recv_timeout(Duration::from_secs(10));
                Ok::<_, BridgeError>(())
            })
        })
    };
    entered.recv_timeout(Duration::from_secs(10)).unwrap();
    let second = {
        let slot = Arc::clone(&slot);
        std::thread::spawn(move || slot.run(build, |_| Ok::<_, BridgeError>("second ran")))
    };
    std::thread::sleep(Duration::from_millis(100));
    release.send(()).unwrap();

    assert!(first.join().unwrap().is_ok());
    assert_eq!(second.join().unwrap().unwrap(), "second ran");
}

#[test]
fn a_manager_that_cannot_be_built_is_an_error_and_the_next_use_tries_again() {
    let slot: ManagerSlot<u32> = ManagerSlot::new();

    let failed = slot.run(|| Err(LauncherError::UnknownMod("x".to_string())), |_| Ok::<_, LauncherError>(()));
    let later = slot.run(build, |_| Ok::<_, LauncherError>(()));

    assert!(failed.is_err());
    assert!(later.is_ok());
}

#[test]
fn a_reader_waiting_on_a_held_lock_answers_busy_when_the_runner_turns_busy() {
    let slot = Arc::new(ManagerSlot::<u32>::new());
    let busy = Arc::new(AtomicBool::new(false));
    let (release, holder) = hold(&slot);
    let reader = {
        let slot = Arc::clone(&slot);
        let busy = Arc::clone(&busy);
        std::thread::spawn(move || {
            slot.run_unless_busy(|| busy.load(Ordering::SeqCst), build, |_| Ok::<_, BridgeError>("reader ran"))
        })
    };
    std::thread::sleep(Duration::from_millis(100));
    assert!(!reader.is_finished());

    let turned = Instant::now();
    busy.store(true, Ordering::SeqCst);
    let answer = reader.join().unwrap();

    assert!(matches!(answer, Err(BridgeError::Busy)));
    assert!(turned.elapsed() < Duration::from_secs(2));
    release.send(()).unwrap();
    holder.join().unwrap();
}

#[test]
fn a_reader_runs_when_the_lock_frees_and_the_runner_is_not_busy() {
    let slot = Arc::new(ManagerSlot::<u32>::new());
    let (release, holder) = hold(&slot);
    let reader = {
        let slot = Arc::clone(&slot);
        std::thread::spawn(move || slot.run_unless_busy(|| false, build, |_| Ok::<_, BridgeError>("reader ran")))
    };
    std::thread::sleep(Duration::from_millis(60));
    release.send(()).unwrap();

    assert_eq!(reader.join().unwrap().unwrap(), "reader ran");
    holder.join().unwrap();
}

// holds the lock on a thread until the returned sender is used
fn hold(slot: &Arc<ManagerSlot<u32>>) -> (mpsc::Sender<()>, std::thread::JoinHandle<()>) {
    let (inside, entered) = mpsc::channel::<()>();
    let (release, gate) = mpsc::channel::<()>();
    let slot = Arc::clone(slot);
    let handle = std::thread::spawn(move || {
        let _ = slot.run(build, |_| {
            inside.send(()).unwrap();
            let _ = gate.recv_timeout(Duration::from_secs(10));
            Ok::<_, BridgeError>(())
        });
    });
    entered.recv_timeout(Duration::from_secs(10)).unwrap();
    (release, handle)
}

#[test]
fn a_job_behind_a_reader_says_it_waits_then_says_it_has_the_lock_and_runs() {
    let slot = Arc::new(ManagerSlot::<u32>::new());
    let (release, holder) = hold(&slot);
    let told = Arc::new(Mutex::new(Vec::<bool>::new()));
    let job = {
        let slot = Arc::clone(&slot);
        let told = Arc::clone(&told);
        std::thread::spawn(move || {
            slot.run_waiting(
                &CancelFlag::new(),
                |waiting| told.lock().unwrap().push(waiting),
                build,
                |_| Ok::<_, JobError>("job ran"),
            )
        })
    };
    std::thread::sleep(Duration::from_millis(100));
    assert_eq!(*told.lock().unwrap(), vec![true]);

    release.send(()).unwrap();

    assert_eq!(job.join().unwrap().unwrap(), "job ran");
    assert_eq!(*told.lock().unwrap(), vec![true, false]);
    holder.join().unwrap();
}

#[test]
fn a_job_that_does_not_wait_never_says_it_waits() {
    let slot: ManagerSlot<u32> = ManagerSlot::new();
    let told = Mutex::new(Vec::<bool>::new());

    let ran = slot.run_waiting(
        &CancelFlag::new(),
        |waiting| told.lock().unwrap().push(waiting),
        build,
        |_| Ok::<_, JobError>(()),
    );

    assert!(ran.is_ok());
    assert!(told.lock().unwrap().is_empty());
}

#[test]
fn cancel_ends_a_job_that_waits_behind_a_reader_as_cancelled() {
    let slot = Arc::new(ManagerSlot::<u32>::new());
    let (release, holder) = hold(&slot);
    let cancel = CancelFlag::new();
    let job = {
        let slot = Arc::clone(&slot);
        let cancel = cancel.clone();
        std::thread::spawn(move || slot.run_waiting(&cancel, |_| {}, build, |_| Ok::<_, JobError>("job ran")))
    };
    std::thread::sleep(Duration::from_millis(60));
    assert!(!job.is_finished());

    cancel.cancel();
    let ended = job.join().unwrap();

    assert!(ended.unwrap_err().is_cancelled());
    release.send(()).unwrap();
    holder.join().unwrap();
}

#[test]
fn a_reset_does_not_wait_for_a_reader_and_the_next_use_builds_a_new_manager() {
    let slot = Arc::new(ManagerSlot::<u32>::new());
    let builds = Arc::new(AtomicU32::new(0));
    let count = Arc::clone(&builds);
    let build = move || Ok::<_, LauncherError>(count.fetch_add(1, Ordering::SeqCst) + 1);
    slot.run(build.clone(), |_| Ok::<_, LauncherError>(())).unwrap();
    let (inside, entered) = mpsc::channel::<u32>();
    let (release, gate) = mpsc::channel::<()>();
    let reader = {
        let slot = Arc::clone(&slot);
        let build = build.clone();
        std::thread::spawn(move || {
            slot.run(build, |manager| {
                inside.send(*manager).unwrap();
                let _ = gate.recv_timeout(Duration::from_secs(10));
                Ok::<_, LauncherError>(*manager)
            })
        })
    };
    assert_eq!(entered.recv_timeout(Duration::from_secs(10)).unwrap(), 1);

    let began = Instant::now();
    slot.reset();
    let waited = began.elapsed();
    release.send(()).unwrap();
    reader.join().unwrap().unwrap();

    assert!(waited < Duration::from_millis(500), "{waited:?}");
    let next = slot.run(build, |manager| Ok::<_, LauncherError>(*manager)).unwrap();
    assert_eq!(next, 2);
    assert_eq!(builds.load(Ordering::SeqCst), 2);
}

#[test]
fn a_reset_on_an_empty_slot_does_not_cost_the_manager_built_after_it() {
    let slot: ManagerSlot<u32> = ManagerSlot::new();
    slot.reset();

    slot.run(|| Ok::<_, LauncherError>(7), |_| Ok::<_, LauncherError>(())).unwrap();
    let kept = slot.run(|| Ok::<_, LauncherError>(8), |manager| Ok::<_, LauncherError>(*manager)).unwrap();

    assert_eq!(kept, 7);
}
