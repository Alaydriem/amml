use std::ffi::c_void;
use std::hint::black_box;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

use amml::SteadyClock;
use amml_data::{EntryKind, Resolved};

use super::{CLIENT_A, FakeVtable, Harness, SessionGame};

static ADVANCES: AtomicUsize = AtomicUsize::new(0);
static END_FRAMES: AtomicUsize = AtomicUsize::new(0);
static LATENCY_HANDLED: AtomicUsize = AtomicUsize::new(0);
// What the particle engine stand-in saw at its entry: the valid byte and the last tick time.
static PARTICLES: Mutex<Vec<(u8, i64)>> = Mutex::new(Vec::new());
// What the predicted movement stand-in saw: the `mNow` of its uniforms and its ten arguments.
static MOVES: Mutex<Vec<(i64, [usize; 10])>> = Mutex::new(Vec::new());

// The `ClientNetworkHandler.handle.NetworkStackLatencyPacket` slot of the stand-in handler table.
static HANDLERS: FakeVtable<1> = FakeVtable([latency_stand_in as *const ()]);

type AdvanceFn = extern "C" fn(*mut c_void, f32, u8) -> f32;

/// The game's wall-clock advance as the stand-in plays it: one tick, an alpha and passed time of a quarter, and two
/// ticks more of overflow time. Gives the step back.
#[inline(never)]
extern "C" fn advance_stand_in(timer: *mut c_void, step: f32, single: u8) -> f32 {
    black_box(single);
    ADVANCES.fetch_add(black_box(1), Ordering::SeqCst);
    let at = timer.expose_provenance();
    TimeGame::write_i32(at + TimeGame::TICKS_AT, TimeGame::STAND_IN_TICKS);
    TimeGame::write_f32(at + TimeGame::ALPHA_AT, TimeGame::STAND_IN_ALPHA);
    TimeGame::write_f32(at + TimeGame::PASSED_AT, TimeGame::STAND_IN_ALPHA);
    let overflow = TimeGame::read_f32(at + TimeGame::OVERFLOW_AT);
    TimeGame::write_f32(at + TimeGame::OVERFLOW_AT, overflow + TimeGame::STAND_IN_OVERFLOW);
    black_box(step)
}

/// The game's particle tick as the stand-in plays it: it notes the valid byte and the last tick time it finds, then
/// takes the wall clock as its last tick time, marks the engine valid and counts 42 particles.
#[inline(never)]
extern "C" fn particle_stand_in(engine: *mut c_void, alpha: f32, paused: u32) {
    black_box((alpha, paused));
    let at = engine.expose_provenance();
    let seen = (TimeGame::read_u8(at + TimeGame::VALID_AT), TimeGame::read_i64(at + TimeGame::LAST_AT));
    PARTICLES.lock().unwrap().push(seen);
    TimeGame::write_u8(at + TimeGame::VALID_AT, 1);
    TimeGame::write_i64(at + TimeGame::LAST_AT, SteadyClock::now_ns());
    TimeGame::write_i64(at + TimeGame::COUNT_AT, TimeGame::STAND_IN_PARTICLES);
}

/// The game's predicted movement as the stand-in plays it: it notes the `mNow` of the uniforms (argument ten) and every
/// argument it got.
#[inline(never)]
#[allow(clippy::too_many_arguments)]
extern "C" fn movement_stand_in(
    a1: usize,
    a2: usize,
    a3: usize,
    a4: usize,
    a5: usize,
    a6: usize,
    a7: usize,
    a8: usize,
    a9: usize,
    a10: usize,
) {
    let now = TimeGame::read_i64(a10 + TimeGame::NOW_AT);
    MOVES.lock().unwrap().push((now, black_box([a1, a2, a3, a4, a5, a6, a7, a8, a9, a10])));
}

#[inline(never)]
extern "C" fn end_frame_stand_in(game: *mut c_void) {
    black_box(game);
    END_FRAMES.fetch_add(black_box(1), Ordering::SeqCst);
}

#[inline(never)]
extern "C" fn latency_stand_in(this: *mut c_void, source: *mut c_void, packet: *mut c_void) {
    black_box((this, source, packet));
    LATENCY_HANDLED.fetch_add(black_box(1), Ordering::SeqCst);
}

/// Stand-in game memory for the time adapter (a timer is given by its address; its first 20h bytes are ticks per second,
/// ticks, alpha, time scale, passed time, two other floats and the overflow time): a `MinecraftGame` that holds pointers to its two timers, a client that
/// holds its two timers inline (its first word is the stand-in client vtable, so the session adapter takes it as the
/// primary client), and the hooked `Timer::advanceTime`. Everything is leaked.
pub struct TimeGame {
    pub game: usize,
    pub client: usize,
}

impl TimeGame {
    pub const GAME_SIM_AT: usize = 0x20;
    pub const GAME_REAL_AT: usize = 0x28;
    pub const CLIENT_SIM_AT: usize = 0x40;
    pub const CLIENT_REAL_AT: usize = 0x80;
    pub const STAND_IN_TICKS: i32 = 1;
    pub const STAND_IN_ALPHA: f32 = 0.25;
    pub const STAND_IN_OVERFLOW: f32 = 2.0;
    const TPS_AT: usize = 0x0;
    const TICKS_AT: usize = 0x4;
    const ALPHA_AT: usize = 0x8;
    const PASSED_AT: usize = 0x10;
    const OVERFLOW_AT: usize = 0x1c;
    const TIMER_SIZE: usize = 0x40;
    const VALID_AT: usize = 0xb8;
    const LAST_AT: usize = 0xc0;
    const COUNT_AT: usize = 0x10f8;
    const ENGINE_SIZE: usize = 0x1100;
    const NOW_AT: usize = 8;
    const UNIFORMS_SIZE: usize = 0x20;
    const CREATE_TIME_AT: usize = 0x30;
    const PACKET_SIZE: usize = 0x48;
    pub const STAND_IN_PARTICLES: i64 = 42;

    /// The session entries and the time entries, pointing to the stand-ins.
    pub fn entries() -> Vec<(&'static str, Resolved)> {
        let function = |address: usize| Resolved {
            kind: EntryKind::Function,
            value: address as u64 - Harness::BASE,
            recipes_agreeing: 2,
        };
        let field = |value: usize| Resolved { kind: EntryKind::Field, value: value as u64, recipes_agreeing: 2 };
        let global = |address: usize| Resolved {
            kind: EntryKind::Global,
            value: address as u64 - Harness::BASE,
            recipes_agreeing: 2,
        };
        let mut entries = SessionGame::entries(&[]);
        entries.extend([
            ("ParticleSystemEngine.tick", function(particle_stand_in as *const () as usize)),
            ("ParticleSystemEngine.mValid", field(Self::VALID_AT)),
            ("ParticleSystemEngine.mLastTickTime", field(Self::LAST_AT)),
            ("ParticleSystemEngine.mParticleCount", field(Self::COUNT_AT)),
            ("PredictedMovementSystem.tickPredictedMovement", function(movement_stand_in as *const () as usize)),
            ("SystemUniforms.mNow", field(Self::NOW_AT)),
            ("MinecraftGame.endFrame", function(end_frame_stand_in as *const () as usize)),
            ("LegacyClientNetworkHandler.vtable", global(std::ptr::from_ref(&HANDLERS) as usize)),
            (
                "ClientNetworkHandler.handle.NetworkStackLatencyPacket",
                Resolved { kind: EntryKind::VSlot, value: 0, recipes_agreeing: 2 },
            ),
            ("Timer.advanceTime", function(advance_stand_in as *const () as usize)),
            ("MinecraftGame.mSimTimer", field(Self::GAME_SIM_AT)),
            ("MinecraftGame.mRealTimer", field(Self::GAME_REAL_AT)),
            ("ClientInstance.mSimTimer", field(Self::CLIENT_SIM_AT)),
            ("ClientInstance.mRealTimer", field(Self::CLIENT_REAL_AT)),
        ]);
        entries
    }

    /// The entries of `entries` but `without`.
    pub fn entries_without(without: &[&str]) -> Vec<(&'static str, Resolved)> {
        Self::entries().into_iter().filter(|(name, _)| !without.contains(name)).collect()
    }

    /// A game and a client whose four timers run at 20 ticks per second.
    pub fn build() -> Self {
        let game = Self::block(0x40);
        Self::write_usize(game + Self::GAME_SIM_AT, Self::timer());
        Self::write_usize(game + Self::GAME_REAL_AT, Self::timer());
        let client = Self::block(0x100);
        Self::write_usize(client, std::ptr::from_ref(CLIENT_A).expose_provenance());
        for at in [Self::CLIENT_SIM_AT, Self::CLIENT_REAL_AT] {
            Self::write_f32(client + at + Self::TPS_AT, 20.0);
        }
        Self { game, client }
    }

    /// A timer that is none of the four (a server timer).
    pub fn timer() -> usize {
        let timer = Self::block(Self::TIMER_SIZE);
        Self::write_f32(timer + Self::TPS_AT, 20.0);
        timer
    }

    /// The game sim, game real, client sim and client real timers.
    pub fn timers(&self) -> [usize; 4] {
        [
            Self::read_usize(self.game + Self::GAME_SIM_AT),
            Self::read_usize(self.game + Self::GAME_REAL_AT),
            self.client + Self::CLIENT_SIM_AT,
            self.client + Self::CLIENT_REAL_AT,
        ]
    }

    /// One frame of the stand-in game: `MinecraftGame::update` through its hooked slot.
    pub fn frame(&self) {
        SessionGame::frame_on(self.game);
    }

    /// A frame, then the join of a world with this client as the primary client.
    pub fn join(&self) {
        self.frame();
        SessionGame::set_player(0x5000);
        SessionGame::start_join(&CLIENT_A, [0; 7]);
        SessionGame::update_raw(std::ptr::with_exposed_provenance_mut(self.client), 1, 0);
    }

    /// The game advances `timer` through the hooked `Timer::advanceTime`.
    pub fn advance(timer: usize) {
        let function = black_box(advance_stand_in as AdvanceFn);
        function(std::ptr::with_exposed_provenance_mut(timer), black_box(1.0), black_box(0));
    }

    /// How often the original `advanceTime` ran.
    pub fn advances() -> usize {
        ADVANCES.load(Ordering::SeqCst)
    }

    /// The ticks, alpha, passed time and overflow time of `timer`.
    pub fn reading(timer: usize) -> (i32, f32, f32, f32) {
        (
            Self::read_i32(timer + Self::TICKS_AT),
            Self::read_f32(timer + Self::ALPHA_AT),
            Self::read_f32(timer + Self::PASSED_AT),
            Self::read_f32(timer + Self::OVERFLOW_AT),
        )
    }

    /// A particle engine that is not valid yet and has the last tick time `last`.
    pub fn engine(last: i64) -> usize {
        let engine = Self::block(Self::ENGINE_SIZE);
        Self::write_i64(engine + Self::LAST_AT, last);
        engine
    }

    /// The last tick time of `engine`.
    pub fn last_tick(engine: usize) -> i64 {
        Self::read_i64(engine + Self::LAST_AT)
    }

    /// The game ticks `engine` through the hooked `ParticleSystemEngine::tick`.
    pub fn tick_particles(engine: usize) {
        let function = black_box(particle_stand_in as extern "C" fn(*mut c_void, f32, u32));
        function(std::ptr::with_exposed_provenance_mut(engine), black_box(0.5), black_box(0));
    }

    /// The (valid byte, last tick time) the original particle tick found at its entry, per call, in order.
    pub fn particles() -> Vec<(u8, i64)> {
        PARTICLES.lock().unwrap().clone()
    }

    /// A `SystemUniforms` whose `mNow` is `now`.
    pub fn uniforms(now: i64) -> usize {
        let uniforms = Self::block(Self::UNIFORMS_SIZE);
        Self::write_i64(uniforms + Self::NOW_AT, now);
        uniforms
    }

    /// The `mNow` of `uniforms`.
    pub fn now_of(uniforms: usize) -> i64 {
        Self::read_i64(uniforms + Self::NOW_AT)
    }

    /// The game runs the hooked `tickPredictedMovement` with the arguments 1 to 9 and `uniforms` as the tenth.
    pub fn predict(uniforms: usize) {
        let function = black_box(
            movement_stand_in as extern "C" fn(usize, usize, usize, usize, usize, usize, usize, usize, usize, usize),
        );
        function(1, 2, 3, 4, 5, 6, 7, 8, 9, black_box(uniforms));
    }

    /// The (`mNow`, arguments) the original predicted movement saw, per call, in order.
    pub fn moves() -> Vec<(i64, [usize; 10])> {
        MOVES.lock().unwrap().clone()
    }

    /// The game ends a frame through the hooked `MinecraftGame::endFrame`.
    pub fn end_frame() {
        let function = black_box(end_frame_stand_in as extern "C" fn(*mut c_void));
        function(std::ptr::null_mut());
    }

    /// How often the original `endFrame` ran.
    pub fn end_frames() -> usize {
        END_FRAMES.load(Ordering::SeqCst)
    }

    /// A `NetworkStackLatencyPacket` whose `createTime` is `create_time`.
    pub fn latency_packet(create_time: u64) -> usize {
        let packet = Self::block(Self::PACKET_SIZE);
        Self::write_i64(packet + Self::CREATE_TIME_AT, create_time as i64);
        packet
    }

    /// The game runs its handler of `packet` through the hooked vtable slot.
    pub fn handle_latency(packet: usize) {
        // SAFETY: the vtable holds the stand-in handler, which has the signature of the game's handler.
        let function = unsafe {
            std::mem::transmute::<*const (), extern "C" fn(*mut c_void, *mut c_void, *mut c_void)>(HANDLERS.0[0])
        };
        let function = black_box(function);
        function(std::ptr::null_mut(), std::ptr::null_mut(), std::ptr::with_exposed_provenance_mut(packet));
    }

    /// How often the original latency handler ran.
    pub fn latency_handled() -> usize {
        LATENCY_HANDLED.load(Ordering::SeqCst)
    }

    // A leaked, zeroed, 8-aligned block of `size` bytes; gives its address.
    fn block(size: usize) -> usize {
        Box::leak(vec![0u64; size / 8].into_boxed_slice()).as_mut_ptr().expose_provenance()
    }

    fn read_usize(at: usize) -> usize {
        // SAFETY: `at` is inside a leaked stand-in block, 8-aligned.
        unsafe { std::ptr::with_exposed_provenance::<usize>(at).read_volatile() }
    }

    fn write_usize(at: usize, value: usize) {
        // SAFETY: as in `read_usize`.
        unsafe { std::ptr::with_exposed_provenance_mut::<usize>(at).write_volatile(value) }
    }

    fn read_u8(at: usize) -> u8 {
        // SAFETY: `at` is inside a leaked stand-in block.
        unsafe { std::ptr::with_exposed_provenance::<u8>(at).read_volatile() }
    }

    fn write_u8(at: usize, value: u8) {
        // SAFETY: as in `read_u8`.
        unsafe { std::ptr::with_exposed_provenance_mut::<u8>(at).write_volatile(value) }
    }

    fn read_i64(at: usize) -> i64 {
        // SAFETY: `at` is inside a leaked stand-in block, 8-aligned.
        unsafe { std::ptr::with_exposed_provenance::<i64>(at).read_volatile() }
    }

    fn write_i64(at: usize, value: i64) {
        // SAFETY: as in `read_i64`.
        unsafe { std::ptr::with_exposed_provenance_mut::<i64>(at).write_volatile(value) }
    }

    fn read_i32(at: usize) -> i32 {
        // SAFETY: `at` is inside a leaked stand-in block, 4-aligned.
        unsafe { std::ptr::with_exposed_provenance::<i32>(at).read_volatile() }
    }

    fn write_i32(at: usize, value: i32) {
        // SAFETY: as in `read_i32`.
        unsafe { std::ptr::with_exposed_provenance_mut::<i32>(at).write_volatile(value) }
    }

    fn read_f32(at: usize) -> f32 {
        // SAFETY: as in `read_i32`.
        unsafe { std::ptr::with_exposed_provenance::<f32>(at).read_volatile() }
    }

    fn write_f32(at: usize, value: f32) {
        // SAFETY: as in `read_i32`.
        unsafe { std::ptr::with_exposed_provenance_mut::<f32>(at).write_volatile(value) }
    }
}
