use amml_apitest::{Progress, Step};

fn ticks(progress: &Progress) {
    for _ in 0..Progress::TICKS {
        progress.tick();
    }
}

#[test]
fn pass_needs_every_step_and_the_ticks() {
    let progress = Progress::new();
    for step in Step::ALL.into_iter().filter(|step| *step != Step::ProviderReady) {
        progress.set(step);
    }
    ticks(&progress);
    assert_eq!(progress.missing(), vec!["provider_ready"]);
    assert!(!progress.try_pass());

    progress.set(Step::ProviderReady);

    assert!(progress.try_pass());
    assert!(!progress.try_pass(), "the pass is reported once");
}

#[test]
fn apitest_ok_needs_every_in_world_step() {
    let progress = Progress::new();
    for step in Step::ALL.into_iter().filter(|step| *step != Step::PauseOpened) {
        progress.set(step);
    }
    ticks(&progress);

    assert!(!progress.try_pass());
    assert_eq!(progress.missing(), vec!["pause_opened"]);
}

#[test]
fn apitest_ok_needs_the_overlay_steps() {
    let progress = Progress::new();
    for step in Step::ALL.into_iter().filter(|step| !Step::OVERLAY.contains(step)) {
        progress.set(step);
    }
    ticks(&progress);

    assert!(!progress.try_pass());
    assert_eq!(progress.missing(), Step::OVERLAY.map(Step::name).to_vec());

    for step in Step::OVERLAY.into_iter().filter(|step| *step != Step::WindowChar) {
        progress.set(step);
    }
    assert!(!progress.try_pass());
    assert_eq!(progress.missing(), vec!["window_char"]);

    progress.set(Step::WindowChar);

    assert!(progress.try_pass());
}

#[test]
fn steps_seen_twice_count_once() {
    let progress = Progress::new();
    assert!(progress.set(Step::ModsLoaded));
    assert!(!progress.set(Step::ModsLoaded));
    assert!(progress.set(Step::Turn));
    assert!(!progress.set(Step::Turn));

    let missing = progress.missing();
    assert_eq!(missing.len(), Step::ALL.len() - 2);
    assert!(!missing.contains(&"mods_loaded") && !missing.contains(&"turn"), "{missing:?}");
}

#[test]
fn a_reset_forgets_every_step() {
    let progress = Progress::new();
    progress.set(Step::WorldJoined);
    progress.tick();

    progress.reset();

    assert_eq!(progress.missing().len(), Step::ALL.len());
    assert_eq!(progress.tick(), 1);
}
