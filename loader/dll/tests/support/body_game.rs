use std::ffi::c_void;
use std::hint::black_box;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

use amml_data::{EntryKind, Resolved};

use super::{CLIENT_A, FakeVtable, Harness, SessionGame};

static QUEUED: AtomicUsize = AtomicUsize::new(0);
static MOVES_HANDLED: AtomicUsize = AtomicUsize::new(0);
// What the dispatcher stand-in drew: the actor and the three stack words, per call.
static DRAWN: Mutex<Vec<(usize, [usize; 3])>> = Mutex::new(Vec::new());

// The `LevelRendererPlayer.queueRenderEntities` slot of the stand-in renderer table.
static RENDERERS: FakeVtable<1> = FakeVtable([queue_stand_in as *const ()]);
// The `ClientNetworkHandler.handle.MovePlayerPacket` slot of the stand-in handler table.
static HANDLERS: FakeVtable<1> = FakeVtable([move_stand_in as *const ()]);

type RenderFn = extern "C" fn(*mut c_void, *mut c_void, *mut c_void, *mut c_void, usize, usize, usize);

/// The game's `queueRenderEntities` as the stand-in plays it: it counts the call; the queue is the one the test built.
#[inline(never)]
extern "C" fn queue_stand_in(renderer: *mut c_void, parameters: *mut c_void) {
    black_box((renderer, parameters));
    QUEUED.fetch_add(black_box(1), Ordering::SeqCst);
}

/// The game's `ActorRenderDispatcher::render` as the stand-in plays it: it notes the actor and the stack words.
#[inline(never)]
extern "C" fn render_stand_in(
    dispatcher: *mut c_void,
    context: *mut c_void,
    actor: *mut c_void,
    target: *mut c_void,
    position: usize,
    rotation: usize,
    flag: usize,
) {
    black_box((dispatcher, context, target));
    DRAWN.lock().unwrap().push((actor.expose_provenance(), black_box([position, rotation, flag])));
}

/// `ActorRuntimeID Actor::getRuntimeID() const` as the stand-in plays it: the id is `BodyGame::runtime_id(actor)`.
#[inline(never)]
extern "C" fn runtime_id_stand_in(actor: *mut c_void, out: *mut u64) -> *mut u64 {
    // SAFETY: the loader passes a writable 8-byte buffer.
    unsafe { out.write(BodyGame::runtime_id(actor.expose_provenance())) };
    out
}

#[inline(never)]
extern "C" fn move_stand_in(this: *mut c_void, source: *mut c_void, packet: *mut c_void) {
    black_box((this, source, packet));
    MOVES_HANDLED.fetch_add(black_box(1), Ordering::SeqCst);
}

/// Stand-in game memory for the body hide and the player move event: a renderer whose actor render queue holds the
/// vectors the test builds, the hooked `queueRenderEntities` slot and dispatcher render, the runtime id read of an
/// actor, and the hooked `MovePlayerPacket` handler slot. Everything is leaked.
pub struct BodyGame {
    pub game: usize,
    pub client: usize,
}

impl BodyGame {
    pub const QUEUE_AT: usize = 0x1d8;
    /// The local player of the joined world.
    pub const LOCAL: usize = 0x5000;
    /// An actor that is not the local player.
    pub const OTHER: usize = 0x6000;
    const RENDERER_SIZE: usize = 0x200;
    const PACKET_SIZE: usize = 0x80;

    /// The session entries and the body entries, pointing to the stand-ins.
    pub fn entries() -> Vec<(&'static str, Resolved)> {
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
        let slot = Resolved { kind: EntryKind::VSlot, value: 0, recipes_agreeing: 2 };
        let mut entries = SessionGame::entries(&[]);
        entries.extend([
            ("LevelRendererPlayer.vtable", global(std::ptr::from_ref(&RENDERERS) as usize)),
            ("LevelRendererPlayer.queueRenderEntities", slot),
            (
                "LevelRendererCamera.mActorRenderQueue",
                Resolved { kind: EntryKind::Field, value: Self::QUEUE_AT as u64, recipes_agreeing: 1 },
            ),
            ("ActorRenderDispatcher.render", function(render_stand_in as *const () as usize)),
            ("Actor.getRuntimeID", function(runtime_id_stand_in as *const () as usize)),
            ("LegacyClientNetworkHandler.vtable", global(std::ptr::from_ref(&HANDLERS) as usize)),
            ("ClientNetworkHandler.handle.MovePlayerPacket", slot),
        ]);
        entries
    }

    /// The entries of `entries` but `without`.
    pub fn entries_without(without: &[&str]) -> Vec<(&'static str, Resolved)> {
        Self::entries().into_iter().filter(|(name, _)| !without.contains(name)).collect()
    }

    /// A game and a client, the client the primary client of the joined world.
    pub fn build() -> Self {
        let game = Self::block(0x40);
        let client = Self::block(0x100);
        Self::write_usize(client, std::ptr::from_ref(CLIENT_A).expose_provenance());
        Self { game, client }
    }

    /// A frame, then the join of a world whose local player is `LOCAL`.
    pub fn join(&self) {
        SessionGame::frame_on(self.game);
        SessionGame::set_player(Self::LOCAL);
        SessionGame::start_join(&CLIENT_A, [0; 7]);
        SessionGame::update_raw(std::ptr::with_exposed_provenance_mut(self.client), 1, 0);
    }

    /// A frame and a client update on the title (no player, so the client is the primary one and no world joins), then
    /// the start of a join whose local player is `LOCAL`: the state of the game after `StartGame`, before the loader's
    /// world join.
    pub fn before_join(&self) {
        SessionGame::frame_on(self.game);
        SessionGame::set_player(0);
        SessionGame::update_raw(std::ptr::with_exposed_provenance_mut(self.client), 1, 0);
        SessionGame::set_player(Self::LOCAL);
        SessionGame::start_join(&CLIENT_A, [0; 7]);
    }

    /// The address of the stand-in primary client.
    pub fn client(&self) -> usize {
        self.client
    }

    /// Another `onStartJoinGame`, as when a second join starts.
    pub fn start_join_again() {
        SessionGame::start_join(&CLIENT_A, [0; 7]);
    }

    /// The client update that joins the world started by `before_join`.
    pub fn finish_join(&self) {
        SessionGame::update_raw(std::ptr::with_exposed_provenance_mut(self.client), 1, 0);
    }

    /// One frame of the stand-in game.
    pub fn frame(&self) {
        SessionGame::frame_on(self.game);
    }

    /// The runtime id the stand-in gives `actor`.
    pub fn runtime_id(actor: usize) -> u64 {
        (actor as u64) >> 4
    }

    /// A renderer whose actor render queue holds `keys` with `values` (the capacities leave room after the end).
    pub fn renderer(keys: &[usize], values: &[u8]) -> usize {
        let renderer = Self::block(Self::RENDERER_SIZE + 0x1c0);
        let key_buffer = Self::block((keys.len() + 4) * 8);
        let value_buffer = Self::block(values.len() + 8);
        for (index, key) in keys.iter().enumerate() {
            Self::write_usize(key_buffer + index * 8, *key);
        }
        for (index, value) in values.iter().enumerate() {
            Self::write_u8(value_buffer + index, *value);
        }
        let queue = renderer + Self::QUEUE_AT;
        Self::write_usize(queue, key_buffer);
        Self::write_usize(queue + 0x08, key_buffer + keys.len() * 8);
        Self::write_usize(queue + 0x10, key_buffer + (keys.len() + 4) * 8);
        Self::write_usize(queue + 0x18, value_buffer);
        Self::write_usize(queue + 0x20, value_buffer + values.len());
        Self::write_usize(queue + 0x28, value_buffer + values.len() + 8);
        renderer
    }

    /// The pairs of the queue of `renderer`, from its begin pointers to its end pointers.
    pub fn queue(renderer: usize) -> Vec<(usize, u8)> {
        let queue = renderer + Self::QUEUE_AT;
        let (keys, keys_end) = (Self::read_usize(queue), Self::read_usize(queue + 0x08));
        let (values, values_end) = (Self::read_usize(queue + 0x18), Self::read_usize(queue + 0x20));
        let keys: Vec<usize> = (keys..keys_end).step_by(8).map(Self::read_usize).collect();
        let values: Vec<u8> = (values..values_end).map(Self::read_u8).collect();
        keys.into_iter().zip(values).collect()
    }

    /// The six pointers of the queue of `renderer`.
    pub fn queue_pointers(renderer: usize) -> [usize; 6] {
        let queue = renderer + Self::QUEUE_AT;
        std::array::from_fn(|index| Self::read_usize(queue + index * 8))
    }

    /// The game queues the entities of `renderer` through the hooked `LevelRendererPlayer::queueRenderEntities` slot.
    pub fn queue_entities(renderer: usize) {
        // SAFETY: the vtable holds the stand-in, which has the signature of the game's function.
        let function =
            unsafe { std::mem::transmute::<*const (), extern "C" fn(*mut c_void, *mut c_void)>(RENDERERS.0[0]) };
        black_box(function)(std::ptr::with_exposed_provenance_mut(renderer), std::ptr::null_mut());
    }

    /// How often the original `queueRenderEntities` ran.
    pub fn queued() -> usize {
        QUEUED.load(Ordering::SeqCst)
    }

    /// The game draws `actor` through the hooked `ActorRenderDispatcher::render`, with the stack words 1, 2 and 3.
    pub fn render(actor: usize) {
        let function = black_box(render_stand_in as RenderFn);
        function(
            std::ptr::with_exposed_provenance_mut(0x10),
            std::ptr::with_exposed_provenance_mut(0x20),
            std::ptr::with_exposed_provenance_mut(actor),
            std::ptr::with_exposed_provenance_mut(0x40),
            black_box(1),
            black_box(2),
            black_box(3),
        );
    }

    /// The actors the original dispatcher drew with the stack words they came with, in order.
    pub fn drawn() -> Vec<(usize, [usize; 3])> {
        DRAWN.lock().unwrap().clone()
    }

    /// A `MovePlayerPacket` of the actor with runtime id `player_id`.
    pub fn move_packet(player_id: u64, pos: [f32; 3], rot: [f32; 2], head_yaw: f32, mode: u8) -> usize {
        let packet = Self::block(Self::PACKET_SIZE);
        Self::write_usize(packet + 0x30, player_id as usize);
        for (index, value) in pos.iter().enumerate() {
            Self::write_f32(packet + 0x38 + index * 4, *value);
        }
        for (index, value) in rot.iter().enumerate() {
            Self::write_f32(packet + 0x44 + index * 4, *value);
        }
        Self::write_f32(packet + 0x4c, head_yaw);
        Self::write_u8(packet + 0x50, mode);
        packet
    }

    /// The game runs its handler of `packet` through the hooked vtable slot.
    pub fn handle_move(packet: usize) {
        // SAFETY: the vtable holds the stand-in handler, which has the signature of the game's handler.
        let function = unsafe {
            std::mem::transmute::<*const (), extern "C" fn(*mut c_void, *mut c_void, *mut c_void)>(HANDLERS.0[0])
        };
        black_box(function)(std::ptr::null_mut(), std::ptr::null_mut(), std::ptr::with_exposed_provenance_mut(packet));
    }

    /// How often the original move handler ran.
    pub fn moves_handled() -> usize {
        MOVES_HANDLED.load(Ordering::SeqCst)
    }

    // A leaked, zeroed, 8-aligned block of `size` bytes; gives its address.
    fn block(size: usize) -> usize {
        Box::leak(vec![0u64; size.div_ceil(8)].into_boxed_slice()).as_mut_ptr().expose_provenance()
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

    fn write_f32(at: usize, value: f32) {
        // SAFETY: `at` is inside a leaked stand-in block, 4-aligned.
        unsafe { std::ptr::with_exposed_provenance_mut::<f32>(at).write_volatile(value) }
    }
}
