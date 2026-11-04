use crate::support::RecordingSink;
use amml_gdk::{CancelFlag, DownloadProgress, GdkError, InstallStep};
use amml_launcher_bridge::{
    BridgeError, FaultKindView, InstallSteps, JobError, JobKind, JobRunner, JobSnapshot, JobStateView, JobStepStateView,
};
use amml_launcher_core::CoreProgress;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock, mpsc};
use std::time::{Duration, Instant};

fn kind() -> JobKind {
    JobKind::UpdateAmml { name: "play".to_string() }
}

fn download(step: InstallStep, done: u64, total: u64) -> CoreProgress {
    CoreProgress::InstallingVersion { step, download: Some(DownloadProgress { bytes_done: done, bytes_total: total }) }
}

// polls until the runner's snapshot satisfies the check, up to five seconds
fn wait_for(runner: &JobRunner<RecordingSink>, check: impl Fn(&JobSnapshot) -> bool) -> JobSnapshot {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if let Some(snapshot) = runner.snapshot()
            && check(&snapshot)
        {
            return snapshot;
        }
        assert!(Instant::now() < deadline, "the job did not reach the expected state");
        std::thread::sleep(Duration::from_millis(5));
    }
}

#[test]
fn a_second_job_while_one_runs_is_refused_and_changes_nothing() {
    let runner = JobRunner::new(RecordingSink::new(), CancelFlag::new());
    let (release, gate) = mpsc::channel::<()>();
    let first = runner
        .start(kind(), InstallSteps::for_amml(), move |_| {
            let _ = gate.recv();
            Ok(())
        })
        .unwrap();
    let second_ran = Arc::new(AtomicBool::new(false));
    let flag = Arc::clone(&second_ran);

    let refused = runner.start(JobKind::InstallGameInput, InstallSteps::single("game-input"), move |_| {
        flag.store(true, Ordering::SeqCst);
        Ok(())
    });

    assert!(matches!(refused, Err(BridgeError::Busy)));
    assert!(runner.is_busy());
    assert_eq!(runner.snapshot().unwrap().id, first);
    release.send(()).unwrap();
    runner.wait();
    assert!(!second_ran.load(Ordering::SeqCst));
    assert!(!runner.is_busy());
    assert_eq!(runner.snapshot().unwrap().state, JobStateView::Done);
}

#[test]
fn a_snapshot_taken_mid_job_has_the_current_step_and_bytes() {
    let runner = JobRunner::new(RecordingSink::new(), CancelFlag::new());
    let (release, gate) = mpsc::channel::<()>();
    runner
        .start(kind(), InstallSteps::new(true, Vec::new()), move |progress| {
            progress(download(InstallStep::DownloadingWanted, 500, 2000));
            let _ = gate.recv();
            Ok(())
        })
        .unwrap();

    let snapshot = wait_for(&runner, |snapshot| snapshot.progress.is_some());

    assert_eq!(snapshot.state, JobStateView::Running);
    let active: Vec<_> = snapshot.steps.iter().filter(|step| step.state == JobStepStateView::Active).collect();
    assert_eq!(active.len(), 1);
    assert_eq!(active[0].id, "version-download-wanted");
    assert_eq!(active[0].percent, Some(25));
    let progress = snapshot.progress.unwrap();
    assert_eq!((progress.done, progress.total), (500, 2000));
    assert_eq!(snapshot.step_number, 3);
    release.send(()).unwrap();
    runner.wait();
}

#[test]
fn byte_progress_publishes_at_most_ten_snapshots_a_second() {
    let sink = RecordingSink::new();
    let runner = JobRunner::new(sink.clone(), CancelFlag::new());
    let began = Instant::now();
    runner
        .start(kind(), InstallSteps::for_amml(), move |progress| {
            let mut done = 0;
            while began.elapsed() < Duration::from_millis(350) {
                done += 1000;
                progress(CoreProgress::Downloading {
                    file: "amml.zip".to_string(),
                    progress: DownloadProgress { bytes_done: done, bytes_total: 10_000_000 },
                });
                std::thread::sleep(Duration::from_millis(2));
            }
            Ok(())
        })
        .unwrap();
    runner.wait();

    let elapsed_ms = began.elapsed().as_millis() as usize;
    // the first snapshot, the final one, and one for each full tenth of a second
    assert!(sink.count() <= elapsed_ms / 100 + 3, "{} snapshots in {elapsed_ms} ms", sink.count());
    assert!(sink.count() >= 3);
}

#[test]
fn a_step_change_publishes_at_once() {
    let sink = RecordingSink::new();
    let runner = JobRunner::new(sink.clone(), CancelFlag::new());
    runner
        .start(kind(), InstallSteps::new(true, Vec::new()), move |progress| {
            progress(CoreProgress::InstallingVersion { step: InstallStep::Checking, download: None });
            progress(CoreProgress::InstallingVersion { step: InstallStep::DownloadingCurrent, download: None });
            progress(CoreProgress::InstallingVersion { step: InstallStep::DownloadingWanted, download: None });
            Ok(())
        })
        .unwrap();
    runner.wait();

    let active: Vec<String> = sink
        .snapshots()
        .iter()
        .filter_map(|snapshot| {
            snapshot.steps.iter().find(|step| step.state == JobStepStateView::Active).map(|step| step.id.clone())
        })
        .collect();
    assert!(active.contains(&"version-download-current".to_string()), "{active:?}");
    assert!(active.contains(&"version-download-wanted".to_string()), "{active:?}");
}

#[test]
fn cancel_set_during_a_step_stops_the_job_at_its_next_download() {
    let cancel = CancelFlag::new();
    let runner = JobRunner::new(RecordingSink::new(), cancel.clone());
    let seen_by_download = cancel.clone();
    runner
        .start(kind(), InstallSteps::new(true, Vec::new()), move |progress| {
            progress(CoreProgress::InstallingVersion { step: InstallStep::Installing, download: None });
            // the Store install has no download; the next download reads the flag
            while !seen_by_download.is_cancelled() {
                std::thread::sleep(Duration::from_millis(2));
            }
            Err(JobError::Gdk(GdkError::Cancelled))
        })
        .unwrap();
    wait_for(&runner, |snapshot| {
        snapshot.steps.iter().any(|step| step.id == "version-install" && step.state == JobStepStateView::Active)
    });

    runner.cancel();
    runner.wait();

    let snapshot = runner.snapshot().unwrap();
    assert_eq!(snapshot.state, JobStateView::Cancelled);
    assert!(snapshot.fault.is_none());
}

#[test]
fn a_failed_job_marks_the_active_step_failed_and_keeps_the_fault() {
    let runner = JobRunner::new(RecordingSink::new(), CancelFlag::new());
    runner
        .start(kind(), InstallSteps::new(true, Vec::new()), |progress| {
            progress(download(InstallStep::DownloadingWanted, 300, 900));
            Err(JobError::Gdk(GdkError::Download("both mirrors failed".to_string())))
        })
        .unwrap();
    runner.wait();

    let snapshot = runner.snapshot().unwrap();
    assert_eq!(snapshot.state, JobStateView::Failed);
    let failed: Vec<_> = snapshot.steps.iter().filter(|step| step.state == JobStepStateView::Failed).collect();
    assert_eq!(failed.len(), 1);
    assert_eq!(failed[0].id, "version-download-wanted");
    let fault = snapshot.fault.unwrap();
    assert_eq!(fault.kind, FaultKindView::DownloadFailed);
    assert_eq!(fault.values.get("done").map(String::as_str), Some("300"));
    assert_eq!(fault.values.get("total").map(String::as_str), Some("900"));
    assert!(!fault.restored);
}

#[test]
fn a_create_without_a_version_install_has_no_version_steps() {
    let runner = JobRunner::new(RecordingSink::new(), CancelFlag::new());
    runner.start(kind(), InstallSteps::new(false, vec!["@a/b".to_string()]), |_| Ok(())).unwrap();
    runner.wait();

    let ids: Vec<String> = runner.snapshot().unwrap().steps.into_iter().map(|step| step.id).collect();
    assert_eq!(ids, vec!["instance", "amml-download", "amml-patch", "mod:@a/b", "finish"]);
    assert!(runner.snapshot().unwrap().steps.iter().all(|step| step.state == JobStepStateView::Done));
}

#[test]
fn a_restore_step_seen_in_a_failed_job_sets_restored() {
    let runner = JobRunner::new(RecordingSink::new(), CancelFlag::new());
    runner
        .start(kind(), InstallSteps::new(true, Vec::new()), |progress| {
            progress(CoreProgress::InstallingVersion { step: InstallStep::CopyingFiles, download: None });
            progress(CoreProgress::InstallingVersion { step: InstallStep::Restoring, download: None });
            Err(JobError::Gdk(GdkError::IncompleteCopy { expected: 10, copied: 4 }))
        })
        .unwrap();
    runner.wait();

    let fault = runner.snapshot().unwrap().fault.unwrap();
    assert_eq!(fault.kind, FaultKindView::CopyFailed);
    assert!(fault.restored);
}

#[test]
fn the_flag_is_reset_when_the_next_job_starts() {
    let cancel = CancelFlag::new();
    let runner = JobRunner::new(RecordingSink::new(), cancel.clone());
    cancel.cancel();
    let (seen, read) = mpsc::channel::<bool>();
    let flag = cancel.clone();

    runner
        .start(kind(), InstallSteps::for_amml(), move |_| {
            let _ = seen.send(flag.is_cancelled());
            Ok(())
        })
        .unwrap();
    runner.wait();

    assert!(!read.recv().unwrap());
}

#[test]
fn a_panic_in_the_work_ends_the_job_as_failed_and_frees_the_runner() {
    let runner = JobRunner::new(RecordingSink::new(), CancelFlag::new());
    runner.start(kind(), InstallSteps::for_amml(), |_| panic!("boom")).unwrap();
    runner.wait();

    let snapshot = runner.snapshot().unwrap();
    assert_eq!(snapshot.state, JobStateView::Failed);
    assert_eq!(snapshot.fault.unwrap().kind, FaultKindView::OperationFailed);
    assert!(!runner.is_busy());
    assert!(runner.start(kind(), InstallSteps::for_amml(), |_| Ok(())).is_ok());
    runner.wait();
}

#[test]
fn a_failed_create_names_the_game_version_it_was_making() {
    let runner = JobRunner::new(RecordingSink::new(), CancelFlag::new());
    let create = JobKind::CreateInstance {
        name: "play".to_string(),
        game: "1.26.60.5".to_string(),
        preview: false,
        own_data: false,
    };
    runner
        .start(create, InstallSteps::new(true, Vec::new()), |_| {
            Err(JobError::Gdk(GdkError::IncompleteCopy { expected: 10, copied: 4 }))
        })
        .unwrap();
    runner.wait();

    let fault = runner.snapshot().unwrap().fault.unwrap();
    assert_eq!(fault.values.get("version").map(String::as_str), Some("1.26.60.5"));
}

#[test]
fn a_failed_restore_is_never_marked_restored_even_after_the_restore_step_began() {
    let runner = JobRunner::new(RecordingSink::new(), CancelFlag::new());
    runner
        .start(kind(), InstallSteps::new(true, Vec::new()), |progress| {
            progress(CoreProgress::InstallingVersion { step: InstallStep::Restoring, download: None });
            Err(JobError::Gdk(GdkError::RestoreFailed {
                original: "p".to_string(),
                package: std::path::PathBuf::from("original.msixvc"),
                cause: "denied".to_string(),
            }))
        })
        .unwrap();
    runner.wait();

    let fault = runner.snapshot().unwrap().fault.unwrap();
    assert_eq!(fault.kind, FaultKindView::RestoreFailed);
    assert!(!fault.restored);
}

// a sink that panics on the last snapshot of a job
struct PanicAtEnd;

impl amml_launcher_bridge::JobSink for PanicAtEnd {
    fn publish(&self, snapshot: &JobSnapshot) {
        assert!(snapshot.state == JobStateView::Running, "the sink fails on the last snapshot");
    }
}

#[test]
fn a_sink_that_panics_on_the_last_snapshot_does_not_leave_the_runner_busy() {
    let runner = JobRunner::new(PanicAtEnd, CancelFlag::new());
    runner.start(kind(), InstallSteps::for_amml(), |_| Ok(())).unwrap();
    runner.wait();

    assert!(!runner.is_busy());
}

// what the sink saw: the snapshot it was given, whether the runner was busy and what `snapshot()` returned
type Seen = (JobSnapshot, bool, Option<JobSnapshot>);

// A sink that asks the runner, on each snapshot, whether it is busy and what `snapshot()` returns.
#[derive(Clone, Default)]
struct ProbeSink {
    runner: Arc<OnceLock<Arc<JobRunner<ProbeSink>>>>,
    seen: Arc<Mutex<Vec<Seen>>>,
}

impl amml_launcher_bridge::JobSink for ProbeSink {
    fn publish(&self, snapshot: &JobSnapshot) {
        if let Some(runner) = self.runner.get() {
            self.seen.lock().unwrap().push((snapshot.clone(), runner.is_busy(), runner.snapshot()));
        }
    }
}

#[test]
fn from_inside_the_sink_the_last_snapshot_is_already_the_latest_and_the_runner_is_free() {
    let sink = ProbeSink::default();
    let runner = Arc::new(JobRunner::new(sink.clone(), CancelFlag::new()));
    sink.runner.set(Arc::clone(&runner)).ok().unwrap();

    runner.start(kind(), InstallSteps::for_amml(), |_| Ok(())).unwrap();
    runner.wait();

    let seen = sink.seen.lock().unwrap();
    let (last, busy, latest) = seen.last().unwrap();
    assert_eq!(last.state, JobStateView::Done);
    assert!(!busy);
    assert_eq!(latest.as_ref(), Some(last));
    for (snapshot, _, latest) in seen.iter() {
        assert_eq!(latest.as_ref(), Some(snapshot));
    }
    let running: Vec<_> = seen.iter().filter(|(snapshot, _, _)| snapshot.state == JobStateView::Running).collect();
    assert!(running.iter().all(|(_, busy, _)| *busy));
}

#[test]
fn a_launch_is_blocked_by_a_job_for_the_same_instance_and_by_an_app_wide_job_only() {
    let runner = JobRunner::new(RecordingSink::new(), CancelFlag::new());
    assert!(!runner.blocks_launch("play"));

    for (job, same, other) in [
        (JobKind::UpdateAmml { name: "play".to_string() }, true, false),
        (JobKind::InstallMods { name: "play".to_string(), ids: Vec::new() }, true, false),
        (JobKind::UpdateMods { name: "Play".to_string(), ids: Vec::new() }, true, false),
        (JobKind::InstallUrl { name: "play".to_string(), url: "u".to_string(), consent: false }, true, false),
        (JobKind::InstallGameInput, true, true),
        (JobKind::SetupData { source: "scratch".to_string() }, true, true),
        (JobKind::CopyWorlds { source: "s".to_string() }, true, true),
        (
            JobKind::CreateInstance {
                name: "new".to_string(),
                game: "1.26.60.5".to_string(),
                preview: false,
                own_data: false,
            },
            true,
            true,
        ),
    ] {
        let (release, gate) = mpsc::channel::<()>();
        runner
            .start(job.clone(), InstallSteps::for_amml(), move |_| {
                let _ = gate.recv_timeout(Duration::from_secs(10));
                Ok(())
            })
            .unwrap();

        assert_eq!(runner.blocks_launch("play"), same, "{job:?}");
        assert_eq!(runner.blocks_launch("other"), other, "{job:?}");
        release.send(()).unwrap();
        runner.wait();
        assert!(!runner.blocks_launch("play"));
    }
}

#[test]
fn a_job_start_during_a_launch_answers_busy_and_after_the_launch_it_starts() {
    let runner = JobRunner::new(RecordingSink::new(), CancelFlag::new());
    let ticket = runner.begin_launch("play").unwrap();
    let ran = Arc::new(AtomicBool::new(false));
    let flag = Arc::clone(&ran);

    let refused = runner.start(kind(), InstallSteps::for_amml(), move |_| {
        flag.store(true, Ordering::SeqCst);
        Ok(())
    });

    assert!(matches!(refused, Err(BridgeError::Busy)));
    assert!(!runner.is_busy());
    assert!(!ran.load(Ordering::SeqCst));
    drop(ticket);
    runner.start(kind(), InstallSteps::for_amml(), |_| Ok(())).unwrap();
    runner.wait();
    assert_eq!(runner.snapshot().unwrap().state, JobStateView::Done);
}

#[test]
fn a_launch_is_refused_while_a_job_for_that_instance_runs_and_allowed_for_another_instance() {
    let runner = JobRunner::new(RecordingSink::new(), CancelFlag::new());
    let (release, gate) = mpsc::channel::<()>();
    runner
        .start(kind(), InstallSteps::for_amml(), move |_| {
            let _ = gate.recv_timeout(Duration::from_secs(10));
            Ok(())
        })
        .unwrap();

    assert!(matches!(runner.begin_launch("play"), Err(BridgeError::Busy)));
    let other = runner.begin_launch("other");
    assert!(other.is_ok());
    drop(other);
    release.send(()).unwrap();
    runner.wait();
    assert!(runner.begin_launch("play").is_ok());
}

// records the order of the snapshots; the last snapshot of a job is slow
#[derive(Clone, Default)]
struct SlowEndSink {
    order: Arc<Mutex<Vec<(u64, JobStateView)>>>,
}

impl amml_launcher_bridge::JobSink for SlowEndSink {
    fn publish(&self, snapshot: &JobSnapshot) {
        if snapshot.state != JobStateView::Running {
            std::thread::sleep(Duration::from_millis(80));
        }
        self.order.lock().unwrap().push((snapshot.id, snapshot.state));
    }
}

#[test]
fn the_last_snapshot_of_a_job_reaches_the_sink_before_the_first_of_the_next_job() {
    let sink = SlowEndSink::default();
    let runner = Arc::new(JobRunner::new(sink.clone(), CancelFlag::new()));
    runner.start(kind(), InstallSteps::for_amml(), |_| Ok(())).unwrap();
    // the runner is free while the slow sink still holds the first job's last snapshot
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        match runner.start(kind(), InstallSteps::for_amml(), |_| Ok(())) {
            Ok(_) => break,
            Err(_) => assert!(Instant::now() < deadline, "the runner never freed"),
        }
        std::thread::yield_now();
    }
    while sink.order.lock().unwrap().iter().filter(|(_, state)| *state == JobStateView::Done).count() < 2 {
        assert!(Instant::now() < deadline, "both jobs did not end");
        std::thread::sleep(Duration::from_millis(5));
    }

    let order = sink.order.lock().unwrap();
    let first_done = order.iter().position(|entry| *entry == (1, JobStateView::Done)).unwrap();
    let second_first = order.iter().position(|(id, _)| *id == 2).unwrap();
    assert!(first_done < second_first, "{order:?}");
}
