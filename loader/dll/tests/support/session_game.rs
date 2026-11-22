use std::ffi::c_void;
use std::hint::black_box;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

use amml_data::{EntryKind, Resolved};

use super::{ClientVtable, FakeVtable, Harness, MeshGame};

/// A stand-in `ClientInstance`: only its vtable pointer.
pub type FakeClient = &'static ClientVtable;

static CLIENT_VTABLE: ClientVtable = ClientVtable {
    local_player: SessionGame::local_player,
    start_join: SessionGame::join_stand_in,
    item_in_hand_renderer: MeshGame::item_in_hand_renderer,
    start_external: SessionGame::external_stand_in,
};
// A stand-in `MinecraftGame` vtable: `update` is slot 2, `joinMultiplayerWithAddress` slot 3.
static GAME_VTABLE: FakeVtable<4> = FakeVtable([
    SessionGame::unused as *const (),
    SessionGame::unused as *const (),
    SessionGame::game_update as *const (),
    SessionGame::multiplayer_stand_in as *const (),
]);

/// The first client.
pub static CLIENT_A: FakeClient = &CLIENT_VTABLE;
/// A second client.
pub static CLIENT_B: FakeClient = &CLIENT_VTABLE;
// A client whose vtable pointer is null.
static NULL_VTABLE_CLIENT: usize = 0;

static PLAYER: AtomicUsize = AtomicUsize::new(0);
static FRAMES: AtomicUsize = AtomicUsize::new(0);
static UPDATES: AtomicUsize = AtomicUsize::new(0);
static TICKS: AtomicUsize = AtomicUsize::new(0);
static JOINS: AtomicUsize = AtomicUsize::new(0);
static EXITS: AtomicUsize = AtomicUsize::new(0);
static EXTERNALS: AtomicUsize = AtomicUsize::new(0);
static EXTERNAL_ARGS: Mutex<Vec<usize>> = Mutex::new(Vec::new());
static MULTIPLAYERS: AtomicUsize = AtomicUsize::new(0);
static MULTIPLAYER_ARGS: Mutex<Vec<u64>> = Mutex::new(Vec::new());
static JOIN_ARGS: Mutex<Vec<u64>> = Mutex::new(Vec::new());
static UPDATE_ARGS: Mutex<(u64, u64)> = Mutex::new((0, 0));

/// Local functions with the game signatures that stand in for the game functions the session adapter hooks.
pub struct SessionGame;

impl SessionGame {
    /// Every session entry, pointing to the stand-ins; `without` names entries to leave out.
    pub fn entries(without: &[&str]) -> Vec<(&'static str, Resolved)> {
        let function = |address: usize| Resolved {
            kind: EntryKind::Function,
            value: address as u64 - Harness::BASE,
            recipes_agreeing: 2,
        };
        let global = |address: usize| Resolved {
            kind: EntryKind::Global,
            value: address as u64 - Harness::BASE,
            recipes_agreeing: 2,
        };
        let slot = |value| Resolved { kind: EntryKind::VSlot, value, recipes_agreeing: 2 };
        let field = |value| Resolved { kind: EntryKind::Field, value, recipes_agreeing: 2 };
        let entries = vec![
            ("MinecraftGame.update", slot(2)),
            ("MinecraftGame.vtable", global(std::ptr::from_ref(&GAME_VTABLE) as usize)),
            ("ClientInstance.update", function(Self::client_update as *const () as usize)),
            ("ClientInstance.onTick", function(Self::on_tick as *const () as usize)),
            ("ClientInstance.gameCallbacksOffset", field(0x88)),
            ("ClientInstance.onStartJoinGame", slot(1)),
            ("ClientInstance.vtable", global(std::ptr::from_ref(&CLIENT_VTABLE) as usize)),
            ("ClientInstance.getLocalPlayer", slot(0)),
            ("ClientExitLevel.hook", function(Self::exit_level as *const () as usize)),
        ];
        entries.into_iter().filter(|(id, _)| !without.contains(id)).collect()
    }

    pub fn client(client: &'static FakeClient) -> *mut c_void {
        std::ptr::from_ref(client).cast_mut().cast()
    }

    /// What the local player read gives from now on; 0 is null.
    pub fn set_player(value: usize) {
        PLAYER.store(value, Ordering::SeqCst);
    }

    /// Calls (through its address) `MinecraftGame::update`.
    pub fn frame() {
        Self::frame_on(0x6A_0000);
    }

    /// Calls (through its address) `MinecraftGame::update` on the stand-in game at `game`.
    pub fn frame_on(game: usize) {
        let update = black_box(Self::game_update as extern "C" fn(*mut c_void));
        update(black_box(game as *mut c_void));
    }

    /// `MinecraftGame::joinMultiplayerWithAddress` (through its address) with the seven arguments after `this`: the
    /// connection info, the local flag, two strings, the network type, the shared flag and the join context.
    pub fn join_multiplayer(args: [u64; 7]) {
        let join =
            black_box(Self::multiplayer_stand_in as extern "C" fn(*mut c_void, u64, u64, u64, u64, u64, u64, u64));
        join(black_box(0x6A_0000 as *mut c_void), args[0], args[1], args[2], args[3], args[4], args[5], args[6]);
    }

    /// How many times the `joinMultiplayerWithAddress` stand-in ran.
    pub fn multiplayer_calls() -> usize {
        MULTIPLAYERS.load(Ordering::SeqCst)
    }

    /// The arguments the last `joinMultiplayerWithAddress` stand-in call received, after `this`.
    pub fn multiplayer_args() -> Vec<u64> {
        MULTIPLAYER_ARGS.lock().unwrap().clone()
    }

    pub fn update(client: &'static FakeClient) -> u8 {
        Self::update_raw(Self::client(client), 1, 0)
    }

    /// `ClientInstance::update` on a client whose vtable pointer is null.
    pub fn update_null_vtable() -> u8 {
        Self::update_raw(std::ptr::from_ref(&NULL_VTABLE_CLIENT).cast_mut().cast(), 1, 0)
    }

    /// `ClientInstance::update` with whole rdx and r8 values.
    pub fn update_raw(this: *mut c_void, rdx: u64, r8: u64) -> u8 {
        let update = black_box(Self::client_update as extern "C" fn(*mut c_void, u64, u64) -> u8);
        update(this, black_box(rdx), black_box(r8))
    }

    /// The rdx and r8 values the last `ClientInstance::update` stand-in call received.
    pub fn update_args() -> (u64, u64) {
        *UPDATE_ARGS.lock().unwrap()
    }

    /// `ClientInstance::onTick` on the GameCallbacks subobject of client A.
    pub fn tick(tick: i32) {
        Self::tick_on(&CLIENT_A, tick);
    }

    /// `ClientInstance::onTick` on the GameCallbacks subobject of `client`.
    pub fn tick_on(client: &'static FakeClient, tick: i32) {
        let on_tick = black_box(Self::on_tick as extern "C" fn(*mut c_void, i32, i32));
        on_tick(Self::client(client).wrapping_byte_add(0x88), black_box(tick), black_box(20));
    }

    pub fn start_join(client: &'static FakeClient, args: [u64; 7]) {
        let start = black_box(CLIENT_VTABLE.start_join);
        start(Self::client(client), args[0], args[1], args[2], args[3], args[4], args[5], args[6]);
    }

    /// `ClientInstance::startExternalNetworkWorld` on `client`, through its vtable slot 3.
    pub fn start_external(client: &'static FakeClient, info: usize, name: usize, context: usize) {
        let start = black_box(CLIENT_VTABLE.start_external);
        start(Self::client(client), info, name, context);
    }

    /// How many times the `startExternalNetworkWorld` stand-in ran.
    pub fn external_calls() -> usize {
        EXTERNALS.load(Ordering::SeqCst)
    }

    /// The pointer arguments (info, name, context) the last `startExternalNetworkWorld` stand-in call received.
    pub fn external_args() -> Vec<usize> {
        EXTERNAL_ARGS.lock().unwrap().clone()
    }

    pub fn exit() {
        let exit = black_box(Self::exit_level as extern "C" fn(*mut c_void));
        exit(Self::client(&CLIENT_A).wrapping_byte_add(0x88));
    }

    /// How many times each stand-in ran: update, client update, tick, join, exit.
    pub fn calls() -> [usize; 5] {
        [&FRAMES, &UPDATES, &TICKS, &JOINS, &EXITS].map(|count| count.load(Ordering::SeqCst))
    }

    /// The arguments the last `onStartJoinGame` stand-in call received, after `this`.
    pub fn join_args() -> Vec<u64> {
        JOIN_ARGS.lock().unwrap().clone()
    }

    #[inline(never)]
    extern "C" fn unused(_this: *mut c_void) {
        black_box(0);
    }

    #[inline(never)]
    extern "C" fn game_update(this: *mut c_void) {
        black_box(this);
        FRAMES.fetch_add(black_box(1), Ordering::SeqCst);
    }

    #[inline(never)]
    extern "C" fn client_update(this: *mut c_void, init_finished: u64, extra: u64) -> u8 {
        black_box(this);
        *UPDATE_ARGS.lock().unwrap() = (init_finished, extra);
        UPDATES.fetch_add(black_box(1), Ordering::SeqCst);
        black_box(init_finished as u8)
    }

    #[inline(never)]
    extern "C" fn on_tick(this: *mut c_void, tick: i32, max_tick: i32) {
        black_box((this, tick, max_tick));
        TICKS.fetch_add(black_box(1), Ordering::SeqCst);
    }

    #[inline(never)]
    #[allow(clippy::too_many_arguments)]
    extern "C" fn join_stand_in(this: *mut c_void, a: u64, b: u64, c: u64, d: u64, e: u64, f: u64, g: u64) {
        black_box(this);
        *JOIN_ARGS.lock().unwrap() = vec![a, b, c, d, e, f, g];
        JOINS.fetch_add(black_box(1), Ordering::SeqCst);
    }

    #[inline(never)]
    #[allow(clippy::too_many_arguments)]
    extern "C" fn multiplayer_stand_in(this: *mut c_void, a: u64, b: u64, c: u64, d: u64, e: u64, f: u64, g: u64) {
        black_box(this);
        *MULTIPLAYER_ARGS.lock().unwrap() = vec![a, b, c, d, e, f, g];
        MULTIPLAYERS.fetch_add(black_box(1), Ordering::SeqCst);
    }

    #[inline(never)]
    extern "C" fn external_stand_in(this: *mut c_void, info: usize, name: usize, context: usize) {
        black_box(this);
        *EXTERNAL_ARGS.lock().unwrap() = vec![info, name, context];
        EXTERNALS.fetch_add(black_box(1), Ordering::SeqCst);
    }

    #[inline(never)]
    extern "C" fn exit_level(this: *mut c_void) {
        black_box(this);
        EXITS.fetch_add(black_box(1), Ordering::SeqCst);
    }

    #[inline(never)]
    extern "C" fn local_player(this: *mut c_void) -> *mut c_void {
        black_box(this);
        PLAYER.load(Ordering::SeqCst) as *mut c_void
    }
}
