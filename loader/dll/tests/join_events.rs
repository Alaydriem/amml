use std::ffi::c_void;
use std::sync::Mutex;

use amml::{ClientAdapter, JoinEvents, ModListener, SessionAdapter};
use amml_abi::{
    AmmlApi, AmmlBand, AmmlClientTable, AmmlEventContext, AmmlEventHeader, AmmlServices, AmmlStatus, AmmlStr,
    JoinStarted, ServerJoinStarting,
};

use crate::support::{CLIENT_A, CLIENT_B, Harness, InfoGame, JoinGame, SessionGame};

// The events a test listens to; a listener's `user` value is its index here.
const EVENTS: [&str; 2] = ["world.join_started", "world.server_join_starting"];

static SEEN: Mutex<Vec<String>> = Mutex::new(Vec::new());

fn text(view: AmmlStr) -> String {
    if view.len == 0 {
        return String::new();
    }
    // SAFETY: the loader lends `len` bytes at `ptr` for the call.
    String::from_utf8_lossy(unsafe { std::slice::from_raw_parts(view.ptr, view.len) }).into_owned()
}

unsafe extern "C" fn record(
    event: *mut AmmlEventHeader,
    context: *const AmmlEventContext,
    user: *mut c_void,
) -> AmmlStatus {
    // SAFETY: the loader passes a live header and context for the call.
    let (header, context) = unsafe { (&*event, &*context) };
    let line = match EVENTS[user as usize] {
        "world.join_started" => {
            // SAFETY: a `world.join_started` event of version 2 is a `JoinStarted`.
            let join = unsafe { &*event.cast::<JoinStarted>() };
            format!(
                "join_started phase={} version={} size={} local={} server={} world={}",
                header.phase.0,
                header.version,
                header.size,
                join.local,
                text(join.server_name),
                text(join.world_name)
            )
        }
        _ => {
            // SAFETY: a `world.server_join_starting` event is a `ServerJoinStarting`.
            let join = unsafe { &*event.cast::<ServerJoinStarting>() };
            format!(
                "server_join_starting phase={} version={} thread={} calls={} host={} url={} name={} port={} \
                 transfer={}",
                header.phase.0,
                header.version,
                context.thread.0,
                SessionGame::external_calls(),
                text(join.host),
                text(join.url),
                text(join.name),
                join.port,
                join.transfer
            )
        }
    };
    SEEN.lock().unwrap().push(line);
    AmmlStatus::OK
}

// Installs the session and join adapters over the join entries and listens to both events.
fn start(size: usize) -> Harness {
    let harness = Harness::start_with_entries(&JoinGame::entries(size));
    SessionAdapter::install(harness.state);
    JoinEvents::install(harness.state);
    for (index, event) in EVENTS.iter().enumerate() {
        let listener = ModListener { owner: 9, callback: record, user: index };
        harness.state.events().listen(event, 1, AmmlBand::NORMAL, listener).unwrap();
    }
    harness
}

fn seen() -> Vec<String> {
    SEEN.lock().unwrap().clone()
}

#[test]
fn join_started_v2_carries_local_and_names() {
    let harness = start(InfoGame::SIZE_2663);
    assert!(harness.state.events().is_available("world.join_started", 2));
    let (id, server, world) =
        (JoinGame::string("0f3a-correlation"), JoinGame::string("play.example.net"), JoinGame::string("Survival"));

    // The upper bits of rdx are not the flag; the stack arguments are passed on untouched.
    SessionGame::start_join(&CLIENT_A, [0xAB00_0001, id as u64, server as u64, world as u64, 5, 6, 7]);
    SessionGame::start_join(&CLIENT_A, [0xAB00_0100, id as u64, 0, world as u64, 0, 0, 0]);

    assert_eq!(
        seen(),
        [
            "join_started phase=1 version=2 size=56 local=1 server=play.example.net world=Survival",
            "join_started phase=1 version=2 size=56 local=0 server= world=Survival",
        ]
    );
    assert_eq!(SessionGame::join_args(), [0xAB00_0100, id as u64, 0, world as u64, 0, 0, 0]);
    assert_eq!(SessionGame::calls()[3], 2);
}

#[test]
fn server_join_starting_before_the_game_call() {
    let _harness = start(InfoGame::SIZE_2663);
    let info = JoinGame::info(InfoGame::SIZE_2663, "79.127.172.199", "play.example.net", "Example", 19132);
    let (name, context) = (JoinGame::string("Example Server"), JoinGame::context(true));

    SessionGame::start_external(&CLIENT_A, info, name, context);

    let main = amml_abi::AmmlThread::GAME.0;
    assert_eq!(
        seen(),
        [format!(
            "server_join_starting phase=0 version=2 thread={main} calls=0 host=79.127.172.199 \
             url=play.example.net name=Example port=19132 transfer=1"
        )]
    );
    assert_eq!(SessionGame::external_calls(), 1);
    assert_eq!(SessionGame::external_args(), [info, name, context]);
}

#[test]
fn server_join_starting_reads_the_70_26_layout_and_falls_back_to_the_name_argument() {
    let _harness = start(InfoGame::SIZE_2670);
    let info = JoinGame::info(InfoGame::SIZE_2670, "10.0.0.7", "lan.example", "", 19133);
    let (name, context) = (JoinGame::string("Lan"), JoinGame::context(false));

    SessionGame::start_external(&CLIENT_A, info, name, context);

    let main = amml_abi::AmmlThread::GAME.0;
    assert_eq!(
        seen(),
        [format!(
            "server_join_starting phase=0 version=2 thread={main} calls=0 host=10.0.0.7 url=lan.example name=Lan \
             port=19133 transfer=0"
        )]
    );
}

#[test]
fn server_join_starting_v2_is_raised_for_a_game_join_through_join_multiplayer_with_address() {
    let harness = start(InfoGame::SIZE_2663);
    assert!(harness.state.events().is_available("world.server_join_starting", 2));
    SessionGame::frame();
    let info = JoinGame::info(InfoGame::SIZE_2663, "79.127.172.199", "play.example.net", "Example", 19132);
    let (first, second, context) = (JoinGame::string("a"), JoinGame::string("b"), JoinGame::context(false));
    // The upper bits of r8 are not the local flag; every argument is passed on untouched.
    let args = [info as u64, 0xAB00, first as u64, second as u64, 4, 0x5150, context as u64];

    SessionGame::join_multiplayer(args);

    let main = amml_abi::AmmlThread::GAME.0;
    assert_eq!(
        seen(),
        [format!(
            "server_join_starting phase=0 version=2 thread={main} calls=0 host=79.127.172.199 \
             url=play.example.net name=Example port=19132 transfer=0"
        )]
    );
    assert_eq!((SessionGame::multiplayer_calls(), SessionGame::multiplayer_args()), (1, args.to_vec()));
}

#[test]
fn a_local_join_through_join_multiplayer_with_address_raises_nothing() {
    let _harness = start(InfoGame::SIZE_2663);
    SessionGame::frame();
    let info = JoinGame::info(InfoGame::SIZE_2663, "127.0.0.1", "", "", 19132);
    let args = [info as u64, 1, JoinGame::string("a") as u64, 0, 0, 0, JoinGame::context(false) as u64];

    SessionGame::join_multiplayer(args);

    assert!(seen().is_empty(), "{:?}", seen());
    assert_eq!(SessionGame::multiplayer_args(), args.to_vec());
}

#[test]
fn a_join_through_join_multiplayer_with_address_off_the_game_thread_raises_nothing() {
    let _harness = start(InfoGame::SIZE_2663);
    SessionGame::frame();
    let info = JoinGame::info(InfoGame::SIZE_2663, "79.127.172.199", "", "", 19132);
    let args = [info as u64, 0, JoinGame::string("a") as u64, 0, 0, 0, JoinGame::context(false) as u64];

    std::thread::spawn(move || SessionGame::join_multiplayer(args)).join().unwrap();

    assert!(seen().is_empty(), "{:?}", seen());
    assert_eq!(SessionGame::multiplayer_calls(), 1);
}

#[test]
fn a_direct_join_is_raised_once_when_the_game_passes_it_on_to_join_multiplayer_with_address() {
    let _harness = start(InfoGame::SIZE_2663);
    SessionGame::frame();
    let direct = JoinGame::typed(JoinGame::info(InfoGame::SIZE_2663, "127.0.0.1", "", "", 28231), 8);
    let (name, context) = (JoinGame::string("DejaVu replay"), JoinGame::context(false));
    // The info the game builds again before it joins: the same address and port, type 1 (an IPv4 address).
    let passed_on = JoinGame::typed(JoinGame::info(InfoGame::SIZE_2663, "127.0.0.1", "", "", 28231), 1);
    let other = JoinGame::info(InfoGame::SIZE_2663, "79.127.172.199", "play.example.net", "Example", 19132);
    let game_args = |info: usize| [info as u64, 0, name as u64, 0, 0, 0, context as u64];

    SessionGame::start_external(&CLIENT_A, direct, name, context);
    SessionGame::join_multiplayer(game_args(passed_on));
    SessionGame::join_multiplayer(game_args(other));

    let main = amml_abi::AmmlThread::GAME.0;
    assert_eq!(
        seen(),
        [
            format!(
                "server_join_starting phase=0 version=2 thread={main} calls=0 host=127.0.0.1 url= \
                 name=DejaVu replay port=28231 transfer=0"
            ),
            format!(
                "server_join_starting phase=0 version=2 thread={main} calls=1 host=79.127.172.199 \
                 url=play.example.net name=Example port=19132 transfer=0"
            ),
        ]
    );
    assert_eq!(SessionGame::multiplayer_calls(), 2);
}

// Raises `startExternalNetworkWorld` with `direct`, runs `between`, then `joinMultiplayerWithAddress` with `passed_on`,
// and gives how many `ServerJoinStarting` were raised.
fn direct_then_game(direct: usize, between: impl FnOnce(), passed_on: usize) -> usize {
    let (name, context) = (JoinGame::string("x"), JoinGame::context(false));
    SessionGame::start_external(&CLIENT_A, direct, name, context);
    between();
    SessionGame::join_multiplayer([passed_on as u64, 0, name as u64, 0, 0, 0, context as u64]);
    seen().len()
}

#[test]
fn an_ipv6_direct_join_passed_on_in_another_text_form_is_raised_once() {
    let _harness = start(InfoGame::SIZE_2663);
    SessionGame::frame();
    let direct = JoinGame::typed(JoinGame::info(InfoGame::SIZE_2663, "::1", "", "", 28231), 8);
    let passed_on = JoinGame::typed(JoinGame::info(InfoGame::SIZE_2663, "0:0:0:0:0:0:0:1", "", "", 28231), 2);

    assert_eq!(direct_then_game(direct, || {}, passed_on), 1, "{:?}", seen());
}

#[test]
fn a_start_external_join_that_is_not_unknown_ip_keeps_no_address() {
    let _harness = start(InfoGame::SIZE_2663);
    SessionGame::frame();
    // Only an `UnknownIP` info is built again and passed on by the game; any other type is a join of its own.
    let direct = JoinGame::typed(JoinGame::info(InfoGame::SIZE_2663, "10.0.0.7", "", "", 19132), 1);
    let game = JoinGame::typed(JoinGame::info(InfoGame::SIZE_2663, "10.0.0.7", "", "", 19132), 1);

    assert_eq!(direct_then_game(direct, || {}, game), 2, "{:?}", seen());
}

#[test]
fn a_kept_direct_join_address_expires_after_its_frames() {
    let _harness = start(InfoGame::SIZE_2663);
    SessionGame::frame();
    let direct = JoinGame::typed(JoinGame::info(InfoGame::SIZE_2663, "127.0.0.1", "", "", 28231), 8);
    let game = JoinGame::typed(JoinGame::info(InfoGame::SIZE_2663, "127.0.0.1", "", "", 28231), 1);
    let frames = || {
        for _ in 0..=JoinEvents::DIRECT_FRAMES {
            SessionGame::frame();
        }
    };

    assert_eq!(direct_then_game(direct, frames, game), 2, "{:?}", seen());
}

#[test]
fn a_kept_direct_join_address_is_dropped_at_the_next_join_started() {
    let _harness = start(InfoGame::SIZE_2663);
    SessionGame::frame();
    let direct = JoinGame::typed(JoinGame::info(InfoGame::SIZE_2663, "127.0.0.1", "", "", 28231), 8);
    let game = JoinGame::typed(JoinGame::info(InfoGame::SIZE_2663, "127.0.0.1", "", "", 28231), 1);
    let world = JoinGame::string("w");
    let join_started = || SessionGame::start_join(&CLIENT_A, [0, 0, 0, world as u64, 0, 0, 0]);

    // The join started line is in `seen` too: one direct raise, one join started, one game raise.
    direct_then_game(direct, join_started, game);
    let raised = seen().iter().filter(|line| line.starts_with("server_join_starting")).count();
    assert_eq!(raised, 2, "{:?}", seen());
}

// The client table, through the services table as a mod reaches it.
fn table(harness: &Harness) -> &'static AmmlClientTable {
    // SAFETY: the loader's tables live for the process.
    let services = unsafe { AmmlApi::services(harness.state.api()) }.unwrap();
    // SAFETY: as above.
    let table = unsafe { AmmlServices::client(services) }.unwrap();
    // SAFETY: as above.
    unsafe { &*table }
}

fn in_gameplay(table: &AmmlClientTable, client: *const c_void) -> (AmmlStatus, u8) {
    let mut out = 0xEEu8;
    // SAFETY: the table is live and `out` is writable.
    let status = unsafe { (table.in_gameplay.unwrap())(client, &mut out) };
    (status, out)
}

#[test]
fn in_gameplay_reads_is_in_game_of_the_frame_game() {
    let harness = Harness::start_with_entries(&JoinGame::entries(InfoGame::SIZE_2663));
    SessionAdapter::install(harness.state);
    ClientAdapter::install(harness.state);
    assert!(harness.state.areas().is_available("client.in_gameplay", 1));
    assert_eq!(table(&harness).version, 8);
    let client = SessionGame::client(&CLIENT_A).cast_const();
    let table = table(&harness);

    // No frame yet: the gate is closed.
    assert_eq!(in_gameplay(table, client).0, AmmlStatus::NOT_READY);

    SessionGame::frame_on(JoinGame::game(true));
    SessionGame::update(&CLIENT_A);
    assert_eq!(in_gameplay(table, client), (AmmlStatus::OK, 1));
    JoinGame::set_in_game(false);
    assert_eq!(in_gameplay(table, client), (AmmlStatus::OK, 0));
    // A client that is not the primary one is refused.
    assert_eq!(in_gameplay(table, SessionGame::client(&CLIENT_B).cast_const()).0, AmmlStatus::REFUSED);
}
