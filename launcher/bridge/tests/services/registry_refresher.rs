use amml_launcher_bridge::{RegistryRefresher, RegistryStatusView, RegistryView};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, mpsc};
use std::time::Duration;

// how long a test waits for a thread to reach a point
const LIMIT: Duration = Duration::from_secs(5);

fn view(base: &str) -> RegistryView {
    RegistryView {
        id: base.to_string(),
        base: base.to_string(),
        name: None,
        official: false,
        status: RegistryStatusView::Ok,
        mod_count: None,
        fetched: None,
    }
}

// Starts a refresh on its own thread whose run says when it starts and then waits for the release.
fn blocked_refresh(
    refresher: &Arc<RegistryRefresher>,
    runs: &Arc<AtomicU32>,
    base: &'static str,
) -> (mpsc::Receiver<()>, mpsc::Sender<()>, std::thread::JoinHandle<Vec<RegistryView>>) {
    let (started_tx, started) = mpsc::channel::<()>();
    let (release, gate) = mpsc::channel::<()>();
    let refresher = Arc::clone(refresher);
    let runs = Arc::clone(runs);
    let handle = std::thread::spawn(move || {
        refresher.refresh(|| {
            runs.fetch_add(1, Ordering::SeqCst);
            started_tx.send(()).unwrap();
            gate.recv_timeout(Duration::from_secs(20)).unwrap();
            vec![view(base)]
        })
    });
    (started, release, handle)
}

#[test]
fn a_second_request_joins_a_running_refresh() {
    let refresher = Arc::new(RegistryRefresher::new());
    let runs = Arc::new(AtomicU32::new(0));
    let (started, release, first) = blocked_refresh(&refresher, &runs, "https://first/");
    started.recv_timeout(LIMIT).unwrap();

    let second = {
        let refresher = Arc::clone(&refresher);
        let runs = Arc::clone(&runs);
        std::thread::spawn(move || {
            refresher.refresh(|| {
                runs.fetch_add(1, Ordering::SeqCst);
                vec![view("https://second/")]
            })
        })
    };
    // the second request must be waiting before the first run ends
    std::thread::sleep(Duration::from_millis(100));
    assert!(!second.is_finished());
    release.send(()).unwrap();

    let first = first.join().unwrap();
    let second = second.join().unwrap();
    assert_eq!(runs.load(Ordering::SeqCst), 1);
    assert_eq!(first, vec![view("https://first/")]);
    assert_eq!(second, first);
}

#[test]
fn a_request_after_a_settings_change_runs_a_new_refresh() {
    let refresher = Arc::new(RegistryRefresher::new());
    let runs = Arc::new(AtomicU32::new(0));
    let (started, release, first) = blocked_refresh(&refresher, &runs, "https://old/");
    started.recv_timeout(LIMIT).unwrap();

    refresher.settings_changed();
    let second = {
        let refresher = Arc::clone(&refresher);
        let runs = Arc::clone(&runs);
        std::thread::spawn(move || {
            refresher.refresh(|| {
                runs.fetch_add(1, Ordering::SeqCst);
                vec![view("https://new/")]
            })
        })
    };
    // the second run waits for the stale one to end
    std::thread::sleep(Duration::from_millis(100));
    assert!(!second.is_finished());
    assert_eq!(runs.load(Ordering::SeqCst), 1);
    release.send(()).unwrap();

    assert_eq!(first.join().unwrap(), vec![view("https://old/")]);
    assert_eq!(second.join().unwrap(), vec![view("https://new/")]);
    assert_eq!(runs.load(Ordering::SeqCst), 2);
}

#[test]
fn a_run_that_panics_does_not_leave_a_waiting_request_stuck() {
    let refresher = Arc::new(RegistryRefresher::new());
    let (started_tx, started) = mpsc::channel::<()>();
    let (release, gate) = mpsc::channel::<()>();
    let first = {
        let refresher = Arc::clone(&refresher);
        std::thread::spawn(move || {
            refresher.refresh(|| {
                started_tx.send(()).unwrap();
                gate.recv_timeout(Duration::from_secs(20)).unwrap();
                panic!("the run failed");
            })
        })
    };
    started.recv_timeout(LIMIT).unwrap();
    let second = {
        let refresher = Arc::clone(&refresher);
        std::thread::spawn(move || refresher.refresh(|| vec![view("https://after/")]))
    };
    std::thread::sleep(Duration::from_millis(100));
    release.send(()).unwrap();

    assert!(first.join().is_err());
    assert_eq!(second.join().unwrap(), vec![view("https://after/")]);
}
