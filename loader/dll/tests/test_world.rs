use amml::{ClientAdapter, SessionAdapter, TestWorld};

use crate::support::{ClientGame, GAME_CLIENT_A, Harness, HeapProbe, LogFile, WorldGame};

// Installs the session and client adapters over every world entry but `without`, then the test world for
// `level_id`; one frame and one update of client A make it the primary client. The top screen is `screen`.
fn start(level_id: Option<&str>, without: &[&str], screen: &str) -> Harness {
    let harness = Harness::start_with_entries(&WorldGame::entries(without));
    SessionAdapter::install(harness.state);
    ClientAdapter::install(harness.state);
    TestWorld::new(level_id.map(str::to_string)).install();
    ClientGame::set_screen(screen);
    ClientGame::frame();
    ClientGame::update(&GAME_CLIENT_A);
    harness
}

// Runs `count` frames and gives the 1-based frames in which `startLocalWorld` ran.
fn frames(count: usize) -> Vec<usize> {
    let mut seen = Vec::new();
    for frame in 1..=count {
        let before = WorldGame::calls();
        ClientGame::frame();
        if WorldGame::calls() > before {
            seen.push(frame);
        }
    }
    seen
}

fn loader_log(harness: &Harness, needle: &str) -> String {
    LogFile::wait_for(&harness.logs().join("loader.log"), needle)
}

#[test]
fn nothing_happens_without_a_level_id() {
    let _harness = start(None, &[], "start_screen");

    frames(200);

    assert_eq!(WorldGame::calls(), 0);
}

#[test]
fn the_world_starts_on_the_start_screen_only() {
    let harness = start(Some("abc="), &[], "hud_screen");

    assert_eq!(frames(10), Vec::<usize>::new());
    ClientGame::set_screen("start_screen");
    assert_eq!(frames(100), [1]);

    assert_eq!(WorldGame::ids(), ["abc="]);
    assert_eq!(WorldGame::this_seen(), [WorldGame::starter()]);
    let log = loader_log(&harness, "test world started");
    assert!(log.contains("test world started"), "{log}");
}

#[test]
fn not_ready_results_retry_after_60_frames() {
    let harness = start(Some("abc="), &[], "start_screen");
    WorldGame::queue_results(&[3, 3, 0]);

    assert_eq!(frames(300), [1, 61, 121]);

    let log = loader_log(&harness, "test world started");
    assert!(log.contains("test world started"), "{log}");
}

#[test]
fn an_error_result_stops_with_a_log_line() {
    let harness = start(Some("abc="), &[], "start_screen");
    WorldGame::queue_results(&[13]);

    assert_eq!(frames(200), [1]);

    let log = loader_log(&harness, "test world not started");
    assert!(log.contains(r#""result":13"#), "{log}");
}

#[test]
fn the_id_string_is_freed_after_each_call() {
    let id = "a-level-id-longer-than-fifteen=";
    let _harness = start(Some(id), &[], "start_screen");
    WorldGame::queue_results(&[2, 0]);
    let (allocs_before, frees_before) = HeapProbe::counts();

    assert_eq!(frames(100), [1, 61]);

    let (allocs, frees) = HeapProbe::counts();
    // Each attempt reads the screen name and builds the id: two blocks, both freed.
    assert_eq!(allocs - allocs_before, 2 * WorldGame::calls());
    assert_eq!(frees - frees_before, allocs - allocs_before);
    assert_eq!(WorldGame::ids(), [id, id]);
}

#[test]
fn not_ready_results_end_after_30_retries() {
    let harness = start(Some("abc="), &[], "start_screen");
    WorldGame::queue_results(&[3; 40]);

    assert_eq!(frames(60 * 40).len(), 31);

    let log = loader_log(&harness, "test world not started");
    assert!(log.contains(r#""result":3"#), "{log}");
}

#[test]
fn a_missing_entry_stops_with_a_log_line() {
    let harness = start(Some("abc="), &["LocalWorldStarter.startLocalWorld"], "start_screen");

    frames(10);

    assert_eq!(WorldGame::calls(), 0);
    let log = loader_log(&harness, "test world not started");
    assert!(log.contains(r#""missing":"LocalWorldStarter.startLocalWorld""#), "{log}");
}

#[test]
fn a_null_starter_stops_with_a_log_line() {
    let harness = start(Some("abc="), &[], "start_screen");
    WorldGame::set_starter(0);

    frames(100);

    assert_eq!(WorldGame::calls(), 0);
    let log = loader_log(&harness, "test world not started");
    assert!(log.contains("WorldSystem.mLocalWorldStarter"), "{log}");
}

#[test]
fn a_joined_world_stops_it() {
    let _harness = start(Some("abc="), &[], "hud_screen");
    ClientGame::set_player(0x5000);
    ClientGame::update(&GAME_CLIENT_A);
    ClientGame::set_screen("start_screen");

    frames(100);

    assert_eq!(WorldGame::calls(), 0);
}
