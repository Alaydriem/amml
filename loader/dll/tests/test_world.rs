use std::time::Duration;

use amml::{ClientAdapter, SessionAdapter, StorageDetours, TestWorld};

use crate::support::{ClientGame, GAME_CLIENT_A, Harness, HeapProbe, LogFile, WorldGame};

// Installs the session and client adapters over every world entry but `without`, then the test world for
// `level_id` with no settle time; one frame and one update of client A make it the primary client. The top screen is
// `screen`.
fn start(level_id: Option<&str>, without: &[&str], screen: &str) -> Harness {
    start_settling(level_id, without, screen, Duration::ZERO)
}

// `start` with a restart that waits until the game has been settled for `settle`.
fn start_settling(level_id: Option<&str>, without: &[&str], screen: &str, settle: Duration) -> Harness {
    let harness = Harness::start_with_entries(&WorldGame::entries(without));
    SessionAdapter::install(harness.state);
    ClientAdapter::install(harness.state);
    TestWorld::new(level_id.map(str::to_string), settle).install();
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

    // Ends at the second call: after a start, each frame reads the screen name to watch the load.
    assert_eq!(frames(61), [1, 61]);

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

const LOADING: &str = "world_loading_progress_screen - local_world_load";

// Runs one load that ends on the start screen with no join: the loading screen for 10 frames, then the start
// screen. Gives the 1-based frames, counted from the return to the start screen, in which `startLocalWorld` ran.
fn load_ends_without_a_join(count: usize) -> Vec<usize> {
    ClientGame::set_screen(LOADING);
    frames(10);
    ClientGame::set_screen("start_screen");
    frames(count)
}

#[test]
fn a_load_that_ends_on_the_start_screen_stops_with_a_log_line() {
    let harness = start(Some("abc="), &[], "start_screen");

    assert_eq!(frames(10), [1]);
    assert_eq!(load_ends_without_a_join(600), Vec::<usize>::new());

    assert_eq!(WorldGame::calls(), 1);
    let log = loader_log(&harness, "test world load ended without a join");
    assert!(log.contains(r#""level_id":"abc=""#), "{log}");
}

#[test]
fn a_ui_title_page_after_the_load_stops_it() {
    let harness = start(Some("abc="), &[], "start_screen");

    assert_eq!(frames(10), [1]);
    ClientGame::set_screen(LOADING);
    frames(10);
    ClientGame::set_screen("ui://./hbui/index.html");

    assert_eq!(frames(600), Vec::<usize>::new());
    let log = loader_log(&harness, "test world load ended without a join");
    assert!(log.contains("test world load ended without a join"), "{log}");
}

#[test]
fn the_start_screen_before_the_load_does_not_start_the_world_again() {
    let _harness = start(Some("abc="), &[], "start_screen");

    assert_eq!(frames(600), [1]);
}

#[test]
fn a_world_joined_after_the_start_stops_it() {
    let _harness = start(Some("abc="), &[], "start_screen");

    assert_eq!(frames(10), [1]);
    ClientGame::set_screen(LOADING);
    frames(10);
    ClientGame::set_player(0x5000);
    ClientGame::update(&GAME_CLIENT_A);
    ClientGame::set_screen("start_screen");

    assert_eq!(frames(600), Vec::<usize>::new());
    assert_eq!(WorldGame::calls(), 1);
}

// A world storage the game still holds, as the `DBStorage` constructor detour publishes it.
const LIVE_STORAGE: usize = 0x7000;

#[test]
fn the_first_start_waits_until_no_storage_is_live() {
    let harness = start(Some("abc="), &[], "hud_screen");
    StorageDetours::set_live(LIVE_STORAGE);
    ClientGame::set_screen("start_screen");

    assert_eq!(frames(200), Vec::<usize>::new());
    // `checkShutdownDone` on the storage withdraws it.
    assert!(StorageDetours::withdraw(LIVE_STORAGE));

    assert_eq!(frames(10), [1]);
    let log = loader_log(&harness, "test world settled");
    assert!(log.contains("test world settled"), "{log}");
}

#[test]
fn the_first_start_waits_for_the_settle_time_on_the_start_screen() {
    let harness = start_settling(Some("abc="), &[], "start_screen", Duration::from_millis(300));

    assert_eq!(frames(50), Vec::<usize>::new());
    std::thread::sleep(Duration::from_millis(350));

    assert_eq!(frames(10), [1]);
    let log = loader_log(&harness, "test world settled");
    assert!(log.contains(r#""settle_ms":300"#), "{log}");
}

#[test]
fn a_toast_starts_the_settle_time_again() {
    let _harness = start_settling(Some("abc="), &[], "start_screen", Duration::from_millis(500));
    frames(5);
    std::thread::sleep(Duration::from_millis(300));
    ClientGame::set_screen("toast_screen");
    frames(1);
    ClientGame::set_screen("start_screen");
    frames(1);
    std::thread::sleep(Duration::from_millis(250));

    // More than 500 ms on the start screen in all, but only about 250 ms since the toast.
    assert_eq!(frames(5), Vec::<usize>::new());
    std::thread::sleep(Duration::from_millis(350));
    assert_eq!(frames(5), [1]);
}
