use std::ffi::c_void;
use std::sync::Mutex;
use std::time::Duration;

use amml::{ModListener, SessionAdapter, SteadyClock, TimeAdapter, TimeDetours};
use amml_abi::{
    AmmlApi, AmmlBand, AmmlEventContext, AmmlEventHeader, AmmlHoldRequest, AmmlServices, AmmlStatus, AmmlTimeHold,
    AmmlTimeTable, FrameEnd, LatencyBarrier, TimeStepApplied,
};

use crate::support::{Harness, LogFile, TimeGame};

const OWNER: usize = 3;
const OTHER: usize = 4;
const AREA: &str = "time.control";
const EVENT: &str = "time.step_applied";

// What the `TimeStepApplied` listener saw, one line per call.
static SEEN: Mutex<Vec<String>> = Mutex::new(Vec::new());

fn install(without: &[&str]) -> Harness {
    let harness = Harness::start_with_entries(&TimeGame::entries_without(without));
    SessionAdapter::install(harness.state);
    TimeAdapter::install(harness.state);
    harness
}

// The adapters over every stand-in entry, with a joined world.
fn start() -> (Harness, TimeGame) {
    let harness = install(&[]);
    let game = TimeGame::build();
    game.join();
    (harness, game)
}

fn adapter() -> &'static TimeAdapter {
    TimeAdapter::get().unwrap()
}

fn request(watchdog_ms: u32) -> AmmlHoldRequest {
    AmmlHoldRequest { virtual_offset_ms: 0, watchdog_ms, _reserved: 0 }
}

fn table(harness: &Harness) -> AmmlTimeTable {
    // SAFETY: the loader's tables live for the process.
    let services = unsafe { AmmlApi::services(harness.state.api()) }.unwrap();
    // SAFETY: as above.
    let table = unsafe { AmmlServices::time(services) }.unwrap();
    // SAFETY: as above.
    unsafe { AmmlTimeTable::read(table) }.unwrap()
}

unsafe extern "C" fn record(event: *mut AmmlEventHeader, _: *const AmmlEventContext, _: *mut c_void) -> AmmlStatus {
    // SAFETY: the loader passes a live `TimeStepApplied` for the call.
    let event = unsafe { &*event.cast::<TimeStepApplied>() };
    SEEN.lock().unwrap().push(format!(
        "stepped={} ticks={} alpha={} phase={}",
        event.stepped_us, event.sim_ticks, event.sim_alpha, event.header.phase.0
    ));
    AmmlStatus::OK
}

fn seen() -> Vec<String> {
    std::mem::take(&mut *SEEN.lock().unwrap())
}

#[test]
fn the_rows_are_offered_with_their_entries() {
    let harness = install(&[]);

    assert!(harness.state.areas().is_available(AREA, 1));
    assert!(harness.state.events().is_available(EVENT, 1));
}

#[test]
fn the_rows_are_not_offered_without_their_entries() {
    let harness = install(&["ClientInstance.mRealTimer"]);

    assert!(!harness.state.areas().is_available(AREA, 1));
    assert!(!harness.state.events().is_available(EVENT, 1));
    assert_eq!(adapter().hold(OWNER, request(0)), Err(AmmlStatus::NOT_AVAILABLE));
}

#[test]
fn held_timers_return_the_virtual_ticks() {
    let (_harness, game) = start();
    let server = TimeGame::timer();
    let hold = adapter().hold(OWNER, request(0)).unwrap();

    adapter().step_us(hold, 75_000).unwrap();
    for timer in game.timers() {
        TimeGame::advance(timer);
        // 75 ms at 20 ticks per second: one tick and a half. The overflow the game added is taken back.
        assert_eq!(TimeGame::reading(timer), (1, 0.5, 0.5, 0.0));
    }
    // A timer that is none of the four runs on the game's own clock.
    TimeGame::advance(server);
    assert_eq!(TimeGame::reading(server), (1, 0.25, 0.25, 2.0));

    // A frame with no step hands out no tick.
    game.frame();
    let [sim, ..] = game.timers();
    TimeGame::advance(sim);
    assert_eq!(TimeGame::reading(sim), (0, 0.5, 0.5, 0.0));
    let state = adapter().state();
    assert_eq!((state.held, state.stepped_us, state.sim_ticks, state.sim_alpha), (1, 75_000, 0, 0.5));

    // Released, the timers run on the game's clock in the next frame.
    adapter().release(hold).unwrap();
    TimeGame::advance(sim);
    assert_eq!(TimeGame::reading(sim), (1, 0.25, 0.25, 2.0));
    assert_eq!(adapter().state().held, 0);
}

#[test]
fn no_hold_reads_one_flags_word() {
    let (_harness, game) = start();
    assert_eq!(TimeDetours::flags(), 0);

    // A hold that came and went leaves the flags word clear again.
    let hold = adapter().hold(OWNER, request(0)).unwrap();
    assert_eq!(TimeDetours::flags(), TimeDetours::HELD);
    adapter().release(hold).unwrap();
    assert_eq!(TimeDetours::flags(), 0);

    let [sim, ..] = game.timers();
    let before = TimeGame::advances();
    TimeGame::advance(sim);
    assert_eq!(TimeGame::advances(), before + 1);
    // The game's own values stand: the loader wrote nothing.
    assert_eq!(TimeGame::reading(sim), (1, 0.25, 0.25, 2.0));
}

#[test]
fn step_raises_time_step_applied_once() {
    let (harness, game) = start();
    let listener = ModListener { owner: OTHER, callback: record, user: 0 };
    harness.state.events().listen(EVENT, 1, AmmlBand::NORMAL, listener).unwrap();
    let hold = adapter().hold(OWNER, request(0)).unwrap();
    let [sim, real, ..] = game.timers();

    // A step that has not reached the timers raises nothing.
    adapter().step_us(hold, 50_000).unwrap();
    game.frame();
    assert!(seen().is_empty());

    TimeGame::advance(sim);
    TimeGame::advance(real);
    game.frame();
    game.frame();

    assert_eq!(seen(), ["stepped=50000 ticks=1 alpha=0 phase=1"]);
}

#[test]
fn second_holder_is_refused() {
    let (_harness, _game) = start();
    adapter().hold(OWNER, request(0)).unwrap();

    assert_eq!(adapter().hold(OTHER, request(0)), Err(AmmlStatus::REFUSED));
    assert_eq!(adapter().hold(OWNER, request(0)), Err(AmmlStatus::REFUSED));
}

#[test]
fn a_negative_step_and_a_stale_hold_are_refused() {
    let (_harness, _game) = start();
    let hold = adapter().hold(OWNER, request(0)).unwrap();

    assert_eq!(adapter().step_us(hold, -1), Err(AmmlStatus::INVALID_ARGUMENT));
    adapter().release(hold).unwrap();
    assert_eq!(adapter().step_us(hold, 1), Err(AmmlStatus::STALE));
    assert_eq!(adapter().release(hold), Err(AmmlStatus::STALE));
    assert_eq!(adapter().step_us(AmmlTimeHold { id: 0, generation: 0 }, 1), Err(AmmlStatus::STALE));
}

#[test]
fn hold_needs_a_joined_world() {
    let _harness = install(&[]);
    let game = TimeGame::build();
    game.frame();

    assert_eq!(adapter().hold(OWNER, request(0)), Err(AmmlStatus::NOT_READY));
}

#[test]
fn watchdog_releases_at_the_next_frame() {
    let (harness, game) = start();
    let hold = adapter().hold(OWNER, request(1)).unwrap();
    std::thread::sleep(Duration::from_millis(20));

    game.frame();

    assert_eq!(TimeDetours::flags(), 0);
    assert_eq!(adapter().step_us(hold, 1), Err(AmmlStatus::STALE));
    let log = LogFile::wait_for(&harness.logs().join("loader.log"), "time hold released");
    assert!(log.contains(r#""reason":"watchdog""#), "{log}");
}

#[test]
fn world_left_releases_the_hold() {
    let (harness, _game) = start();
    let hold = adapter().hold(OWNER, request(0)).unwrap();

    crate::support::SessionGame::exit();

    assert_eq!(TimeDetours::flags(), 0);
    assert_eq!(adapter().step_us(hold, 1), Err(AmmlStatus::STALE));
    let log = LogFile::wait_for(&harness.logs().join("loader.log"), "time hold released");
    assert!(log.contains(r#""reason":"world_left""#), "{log}");
}

#[test]
fn disable_releases_the_hold() {
    let (harness, _game) = start();
    let hold = adapter().hold(OWNER, request(0)).unwrap();

    adapter().disable_owner(OTHER);
    assert_eq!(TimeDetours::flags(), TimeDetours::HELD);
    adapter().disable_owner(OWNER);

    assert_eq!(TimeDetours::flags(), 0);
    assert_eq!(adapter().step_us(hold, 1), Err(AmmlStatus::STALE));
    let log = LogFile::wait_for(&harness.logs().join("loader.log"), "time hold released");
    assert!(log.contains(r#""reason":"disabled""#), "{log}");
}

#[test]
fn off_the_game_thread_hold_and_step_are_refused_and_release_waits_for_the_frame() {
    let (_harness, game) = start();
    let hold = adapter().hold(OWNER, request(0)).unwrap();

    let results = std::thread::scope(|scope| {
        scope
            .spawn(|| {
                [adapter().hold(OTHER, request(0)).map(|_| ()), adapter().step_us(hold, 1), adapter().release(hold)]
            })
            .join()
            .unwrap()
    });

    assert_eq!(results, [Err(AmmlStatus::REFUSED), Err(AmmlStatus::REFUSED), Ok(())]);
    // The release runs at the next frame, on the game thread.
    assert_eq!(TimeDetours::flags(), TimeDetours::HELD);
    game.frame();
    assert_eq!(TimeDetours::flags(), 0);
    assert_eq!(adapter().step_us(hold, 1), Err(AmmlStatus::STALE));
}

#[test]
fn the_table_is_in_the_services() {
    let (harness, _game) = start();
    let table = table(&harness);
    let mut out = AmmlTimeHold { id: 9, generation: 9 };

    // SAFETY: the table is live; an unknown module is refused before `out` is written.
    let status = unsafe { (table.hold)(amml_abi::AmmlStr::from_str("@none/none"), &request(0), &mut out) };

    assert_ne!(status, AmmlStatus::OK);
    assert_eq!((out.id, out.generation), (9, 9));
}

const END_EVENT: &str = "time.frame_end";
const BARRIER_EVENT: &str = "time.latency_barrier";
const WALL: i64 = 5_000_000_000;

// The `count` of each `FrameEnd`, and what each `LatencyBarrier` saw: its `create_time` and how often the game's own
// handler had run by then.
static ENDS: Mutex<Vec<u64>> = Mutex::new(Vec::new());
static BARRIERS: Mutex<Vec<String>> = Mutex::new(Vec::new());

unsafe extern "C" fn record_end(event: *mut AmmlEventHeader, _: *const AmmlEventContext, _: *mut c_void) -> AmmlStatus {
    // SAFETY: the loader passes a live `FrameEnd` for the call.
    let event = unsafe { &*event.cast::<FrameEnd>() };
    ENDS.lock().unwrap().push(event.count);
    AmmlStatus::OK
}

unsafe extern "C" fn record_barrier(
    event: *mut AmmlEventHeader,
    _: *const AmmlEventContext,
    _: *mut c_void,
) -> AmmlStatus {
    // SAFETY: the loader passes a live `LatencyBarrier` for the call.
    let event = unsafe { &*event.cast::<LatencyBarrier>() };
    BARRIERS.lock().unwrap().push(format!(
        "create_time={} handled={} phase={}",
        event.create_time,
        TimeGame::latency_handled(),
        event.header.phase.0
    ));
    AmmlStatus::OK
}

fn listen(harness: &Harness, event: &str, callback: amml_abi::EventCallbackFn) {
    let listener = ModListener { owner: OTHER, callback, user: 0 };
    harness.state.events().listen(event, 1, AmmlBand::NORMAL, listener).unwrap();
}

#[test]
fn the_time_rows_follow_their_hooks() {
    let harness = install(&[]);
    assert!(harness.state.events().is_available(END_EVENT, 1));
    assert!(harness.state.events().is_available(BARRIER_EVENT, 1));
}

#[test]
fn the_particle_rows_gate_the_area() {
    let harness = install(&["ParticleSystemEngine.tick"]);

    assert!(!harness.state.areas().is_available(AREA, 1));
    assert!(!harness.state.events().is_available(EVENT, 1));
    assert!(harness.state.events().is_available(END_EVENT, 1));
    assert!(harness.state.events().is_available(BARRIER_EVENT, 1));
}

#[test]
fn a_missing_end_frame_or_latency_slot_withdraws_only_its_event() {
    let harness = install(&["MinecraftGame.endFrame"]);
    assert!(harness.state.areas().is_available(AREA, 1));
    assert!(!harness.state.events().is_available(END_EVENT, 1));
    assert!(harness.state.events().is_available(BARRIER_EVENT, 1));
}

#[test]
fn particle_time_moves_only_when_valid() {
    let (_harness, game) = start();
    let [sim, ..] = game.timers();
    let engine = TimeGame::engine(12_345);
    let hold = adapter().hold(OWNER, request(0)).unwrap();

    // The engine is not valid yet: its last tick time stays as the game has it.
    adapter().step_us(hold, 50_000).unwrap();
    TimeGame::advance(sim);
    TimeGame::tick_particles(engine);
    assert_eq!(TimeGame::particles(), [(0, 12_345)]);
    assert_eq!(adapter().state().particle_count, TimeGame::STAND_IN_PARTICLES as u32);

    // Valid: the step the engine measures is the game time stepped since the last tick (25 ms), whatever the wall clock.
    adapter().step_us(hold, 25_000).unwrap();
    TimeGame::advance(sim);
    let (before, last) = (SteadyClock::now_ns(), TimeGame::last_tick(engine));
    TimeGame::tick_particles(engine);
    let after = SteadyClock::now_ns();
    let [_, (valid, moved)] = TimeGame::particles()[..] else { panic!("two ticks") };
    assert_eq!(valid, 1);
    assert!(before - 25_000_000 <= moved && moved <= after - 25_000_000, "{moved} {before} {after}");
    assert!(last > 0);

    // A frame with no step: the step is none.
    let before = SteadyClock::now_ns();
    TimeGame::tick_particles(engine);
    let after = SteadyClock::now_ns();
    let (_, frozen) = TimeGame::particles()[2];
    assert!(before <= frozen && frozen <= after, "{frozen} {before} {after}");

    // Released: the game's own last tick time stands.
    adapter().release(hold).unwrap();
    let wall = TimeGame::last_tick(engine);
    TimeGame::tick_particles(engine);
    assert_eq!(TimeGame::particles()[3], (1, wall));
    assert_eq!(adapter().state().particle_count, 0);
}

#[test]
fn uniform_now_is_restored_after_the_call() {
    let (_harness, game) = start();
    let [sim, ..] = game.timers();
    let uniforms = TimeGame::uniforms(WALL);

    // No hold: the game's own value, all ten arguments as they came.
    TimeGame::predict(uniforms);
    assert_eq!(TimeGame::moves(), [(WALL, [1, 2, 3, 4, 5, 6, 7, 8, 9, uniforms])]);

    let hold = adapter().hold(OWNER, request(0)).unwrap();
    adapter().step_us(hold, 75_000).unwrap();
    TimeGame::advance(sim);
    let virtual_now = adapter().virtual_now_ns().unwrap();
    TimeGame::predict(uniforms);
    let (seen, arguments) = TimeGame::moves()[1];
    assert_eq!((seen, arguments), (virtual_now, [1, 2, 3, 4, 5, 6, 7, 8, 9, uniforms]));
    assert_ne!(seen, WALL);
    assert_eq!(TimeGame::now_of(uniforms), WALL);

    adapter().release(hold).unwrap();
    TimeGame::predict(uniforms);
    assert_eq!(TimeGame::moves()[2].0, WALL);
    assert_eq!(TimeGame::now_of(uniforms), WALL);
}

#[test]
fn frame_end_counts_every_call() {
    let (harness, _game) = start();
    listen(&harness, END_EVENT, record_end);

    TimeGame::end_frame();
    TimeGame::end_frame();
    // Held or not.
    let hold = adapter().hold(OWNER, request(0)).unwrap();
    TimeGame::end_frame();
    adapter().release(hold).unwrap();
    TimeGame::end_frame();

    assert_eq!(std::mem::take(&mut *ENDS.lock().unwrap()), [1, 2, 3, 4]);
    assert_eq!(TimeGame::end_frames(), 4);
}

#[test]
fn frame_end_without_listener_or_hold_reads_one_flags_word() {
    let (_harness, _game) = start();
    assert_eq!(TimeDetours::flags(), 0);

    TimeGame::end_frame();

    assert_eq!(TimeGame::end_frames(), 1);
    assert!(ENDS.lock().unwrap().is_empty());
}

#[test]
fn latency_barrier_reads_create_time_after_the_handler() {
    let (harness, _game) = start();
    // No listener: the game's handler runs and nothing is raised.
    TimeGame::handle_latency(TimeGame::latency_packet(7));
    assert!(BARRIERS.lock().unwrap().is_empty());

    listen(&harness, BARRIER_EVENT, record_barrier);
    TimeGame::handle_latency(TimeGame::latency_packet(0x1122_3344_5566_7788));

    assert_eq!(TimeGame::latency_handled(), 2);
    assert_eq!(std::mem::take(&mut *BARRIERS.lock().unwrap()), ["create_time=1234605616436508552 handled=2 phase=1"]);
}
