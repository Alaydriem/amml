use amml_gdk::{DownloadProgress, InstallStep};
use amml_launcher_bridge::{InstallSteps, JobStepStateView, JobStepView};
use amml_launcher_core::CoreProgress;
use std::time::{Duration, Instant};

fn bytes(file: &str, done: u64, total: u64) -> CoreProgress {
    CoreProgress::Downloading {
        file: file.to_string(),
        progress: DownloadProgress { bytes_done: done, bytes_total: total },
    }
}

fn version(step: InstallStep, download: Option<(u64, u64)>) -> CoreProgress {
    CoreProgress::InstallingVersion {
        step,
        download: download.map(|(done, total)| DownloadProgress { bytes_done: done, bytes_total: total }),
    }
}

fn state_of(steps: &InstallSteps, id: &str) -> JobStepView {
    steps.steps().into_iter().find(|step| step.id == id).unwrap()
}

#[test]
fn rate_is_bytes_over_the_window_and_remaining_time_follows() {
    let mut steps = InstallSteps::for_amml();
    let base = Instant::now();
    steps.start(base);
    steps.apply(&bytes("amml.zip", 0, 10_000_000), base);
    steps.apply(&bytes("amml.zip", 1_000_000, 10_000_000), base + Duration::from_secs(1));
    steps.apply(&bytes("amml.zip", 2_000_000, 10_000_000), base + Duration::from_secs(2));

    let progress = steps.progress().unwrap();

    assert_eq!(progress.rate, Some(1_000_000));
    assert_eq!(progress.remaining_seconds, Some(8));
    assert_eq!(progress.file.as_deref(), Some("amml.zip"));
}

#[test]
fn samples_older_than_five_seconds_do_not_count() {
    let mut steps = InstallSteps::for_amml();
    let base = Instant::now();
    steps.start(base);
    steps.apply(&bytes("a.zip", 0, 100_000_000), base);
    steps.apply(&bytes("a.zip", 50_000_000, 100_000_000), base + Duration::from_secs(10));
    steps.apply(&bytes("a.zip", 51_000_000, 100_000_000), base + Duration::from_secs(11));

    assert_eq!(steps.progress().unwrap().rate, Some(1_000_000));
}

#[test]
fn the_sample_ring_holds_sixteen_samples_and_events_inside_the_gap_add_none() {
    let mut steps = InstallSteps::for_amml();
    let base = Instant::now();
    steps.start(base);
    for second in 0..60u64 {
        steps.apply(&bytes("a.zip", second * 1000, 1_000_000), base + Duration::from_secs(second));
    }
    assert_eq!(steps.samples_held(), InstallSteps::SAMPLE_CAPACITY);

    let mut quick = InstallSteps::for_amml();
    quick.start(base);
    for tick in 0..100u64 {
        quick.apply(&bytes("a.zip", tick * 10, 1_000_000), base + Duration::from_millis(tick));
    }
    assert_eq!(quick.samples_held(), 1);
}

#[test]
fn no_rate_is_given_from_a_single_sample() {
    let mut steps = InstallSteps::for_amml();
    steps.start(Instant::now());
    steps.apply(&bytes("a.zip", 10, 100), Instant::now());

    let progress = steps.progress().unwrap();

    assert_eq!(progress.rate, None);
    assert_eq!(progress.remaining_seconds, None);
}

#[test]
fn a_version_step_that_never_ran_is_done_from_the_cache() {
    let mut steps = InstallSteps::new(true, Vec::new());
    let base = Instant::now();
    steps.start(base);

    steps.apply(&version(InstallStep::DownloadingWanted, Some((10, 100))), base + Duration::from_secs(1));

    assert_eq!(state_of(&steps, "version-check").state, JobStepStateView::Done);
    let current = state_of(&steps, "version-download-current");
    assert_eq!(current.state, JobStepStateView::Done);
    assert!(current.cached);
    let wanted = state_of(&steps, "version-download-wanted");
    assert_eq!(wanted.state, JobStepStateView::Active);
    assert_eq!(wanted.percent, Some(10));
    assert_eq!(state_of(&steps, "version-install").state, JobStepStateView::Waiting);
    assert!(state_of(&steps, "version-check").millis.is_some());
}

#[test]
fn the_amml_zip_and_each_mod_zip_belong_to_their_own_steps() {
    let mut steps = InstallSteps::for_mods(vec!["@a/one".to_string(), "@a/two".to_string()]);
    let base = Instant::now();
    steps.start(base);

    steps.apply(&bytes("one.zip", 5, 10), base);
    assert_eq!(state_of(&steps, "mod:@a/one").state, JobStepStateView::Active);
    steps.apply(&CoreProgress::InstallingMod { id: "@a/one".to_string() }, base);
    steps.apply(&bytes("two.zip", 5, 10), base + Duration::from_secs(1));

    assert_eq!(state_of(&steps, "mod:@a/one").state, JobStepStateView::Done);
    assert_eq!(state_of(&steps, "mod:@a/two").state, JobStepStateView::Active);
}

#[test]
fn a_mod_that_was_not_known_beforehand_is_added_before_finish() {
    let mut steps = InstallSteps::new(false, Vec::new());
    let base = Instant::now();
    steps.start(base);

    let changed = steps.apply(&CoreProgress::InstallingMod { id: "@x/late".to_string() }, base);

    let ids: Vec<String> = steps.steps().into_iter().map(|step| step.id).collect();
    assert!(changed);
    assert_eq!(ids, vec!["instance", "amml-download", "amml-patch", "mod:@x/late", "finish"]);
    assert_eq!(state_of(&steps, "mod:@x/late").state, JobStepStateView::Active);
    assert_eq!(steps.total(), 5);
    assert_eq!(steps.active_number(), 4);
}

#[test]
fn finishing_the_job_marks_every_step_done_and_a_failure_marks_the_active_one() {
    let mut finished = InstallSteps::new(false, Vec::new());
    finished.start(Instant::now());
    finished.finish(Instant::now());
    assert!(finished.steps().iter().all(|step| step.state == JobStepStateView::Done));
    assert!(finished.progress().is_none());

    let mut failed = InstallSteps::new(false, Vec::new());
    failed.start(Instant::now());
    failed.apply(&CoreProgress::InstallingAmml { version: "1".to_string() }, Instant::now());
    failed.fail();
    assert_eq!(state_of(&failed, "amml-patch").state, JobStepStateView::Failed);
    assert_eq!(state_of(&failed, "amml-download").state, JobStepStateView::Done);
    assert_eq!(state_of(&failed, "finish").state, JobStepStateView::Waiting);
}

#[test]
fn a_job_that_waits_for_the_manager_shows_a_first_step_that_goes_when_it_has_the_manager() {
    let mut steps = InstallSteps::for_amml();
    let base = Instant::now();
    steps.start(base);

    let began = steps.apply(&CoreProgress::WaitingForManager { waiting: true }, base);
    let waiting = steps.steps();

    assert!(began);
    assert_eq!(waiting[0].id, "manager-wait");
    assert_eq!(waiting[0].state, JobStepStateView::Active);
    assert_eq!(waiting[1].state, JobStepStateView::Waiting);
    assert_eq!(steps.active_number(), 1);
    assert_eq!(steps.total(), 3);

    let ended = steps.apply(&CoreProgress::WaitingForManager { waiting: false }, base + Duration::from_secs(3));
    let after = steps.steps();

    assert!(ended);
    let ids: Vec<&str> = after.iter().map(|step| step.id.as_str()).collect();
    assert_eq!(ids, ["amml-download", "amml-patch", "finish"]);
    assert_eq!(after[0].state, JobStepStateView::Active);
    assert_eq!(steps.total(), 3);
}

#[test]
fn a_waiting_event_that_changes_nothing_is_not_a_step_change() {
    let mut steps = InstallSteps::for_amml();
    let base = Instant::now();
    steps.start(base);

    assert!(!steps.apply(&CoreProgress::WaitingForManager { waiting: false }, base));
    steps.apply(&CoreProgress::WaitingForManager { waiting: true }, base);
    assert!(!steps.apply(&CoreProgress::WaitingForManager { waiting: true }, base));
}

#[test]
fn the_wait_step_is_not_counted_and_the_step_that_was_active_goes_on_after_it() {
    let mut steps = InstallSteps::for_amml();
    let base = Instant::now();
    steps.start(base);
    let before = (steps.active_number(), steps.total());

    steps.apply(&CoreProgress::WaitingForManager { waiting: true }, base);
    let during = (steps.active_number(), steps.total());
    steps.apply(&CoreProgress::WaitingForManager { waiting: false }, base);

    assert_eq!(before, (1, 3));
    assert_eq!(during, (1, 3));
    assert_eq!((steps.active_number(), steps.total()), (1, 3));
    assert_eq!(state_of(&steps, "amml-download").state, JobStepStateView::Active);
}

#[test]
fn a_step_that_had_not_begun_stays_waiting_after_the_wait() {
    let mut steps = InstallSteps::for_amml();
    let base = Instant::now();

    steps.apply(&CoreProgress::WaitingForManager { waiting: true }, base);
    steps.apply(&CoreProgress::WaitingForManager { waiting: false }, base);

    assert!(steps.steps().iter().all(|step| step.state == JobStepStateView::Waiting));
}

fn setup_ids(steps: &InstallSteps) -> Vec<String> {
    steps.steps().into_iter().map(|step| step.id).collect()
}

fn active_id(steps: &InstallSteps) -> Option<String> {
    steps.steps().into_iter().find(|step| step.state == JobStepStateView::Active).map(|step| step.id)
}

fn copying(done: u64, total: u64) -> CoreProgress {
    CoreProgress::CopyingData {
        detail: "a -> b".to_string(),
        progress: DownloadProgress { bytes_done: done, bytes_total: total },
    }
}

#[test]
fn setup_steps_move_through_check_measure_copy_and_settings() {
    let mut steps = InstallSteps::for_setup(true);
    let base = Instant::now();
    steps.start(base);
    let mut seen = vec![active_id(&steps).unwrap()];

    steps.apply(&CoreProgress::MeasuringData { merge: false }, base);
    seen.push(active_id(&steps).unwrap());
    steps.apply(&copying(50, 100), base);
    seen.push(active_id(&steps).unwrap());
    let during = steps.progress().unwrap();
    steps.apply(&CoreProgress::SavingSettings, base);
    seen.push(active_id(&steps).unwrap());
    steps.apply(&CoreProgress::Done, base);

    assert_eq!(seen, vec!["setup-check", "setup-measure", "setup-copy", "setup-settings"]);
    assert_eq!((during.done, during.total), (50, 100));
    assert_eq!(during.file.as_deref(), Some("a -> b"));
    assert_eq!(steps.total(), 4);
    assert!(!setup_ids(&steps).contains(&"finish".to_string()));
    assert!(steps.steps().iter().all(|step| step.state == JobStepStateView::Done));
}

#[test]
fn a_scratch_setup_has_only_the_settings_step() {
    let steps = InstallSteps::for_setup(false);

    assert_eq!(setup_ids(&steps), vec!["setup-settings"]);
}

#[test]
fn a_copy_job_has_check_measure_and_copy_and_no_finish() {
    let steps = InstallSteps::for_copy();

    assert_eq!(setup_ids(&steps), vec!["setup-check", "setup-measure", "setup-copy"]);
}

#[test]
fn a_merge_failure_is_not_restored_and_a_fresh_failure_is() {
    let mut merge = InstallSteps::for_setup(true);
    merge.start(Instant::now());
    merge.apply(&CoreProgress::MeasuringData { merge: true }, Instant::now());
    let mut fresh = InstallSteps::for_setup(true);
    fresh.start(Instant::now());
    fresh.apply(&CoreProgress::MeasuringData { merge: false }, Instant::now());

    assert!(!merge.restore_seen());
    assert!(fresh.restore_seen());
}

#[test]
fn a_failed_copy_of_worlds_on_its_own_is_never_restored() {
    let mut steps = InstallSteps::for_copy();
    steps.start(Instant::now());
    steps.apply(&CoreProgress::MeasuringData { merge: false }, Instant::now());

    assert!(!steps.restore_seen());
}

#[test]
fn a_setup_that_fails_before_the_copy_began_is_restored() {
    let mut steps = InstallSteps::for_setup(true);
    steps.start(Instant::now());

    assert!(steps.restore_seen());
}

#[test]
fn a_copy_of_zero_bytes_shows_no_nan_percent() {
    let mut steps = InstallSteps::for_setup(true);
    let base = Instant::now();
    steps.start(base);
    steps.apply(&CoreProgress::MeasuringData { merge: false }, base);
    steps.apply(&copying(0, 0), base);

    let active = steps.steps().into_iter().find(|step| step.state == JobStepStateView::Active).unwrap();

    assert!(matches!(active.percent, None | Some(100)), "{:?}", active.percent);
    assert!(steps.progress().is_some());
}
