use std::ffi::c_void;
use std::hint::black_box;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicI64, AtomicU64, AtomicUsize, Ordering};

use amml::HitResultLayout;
use amml_abi::AmmlBlockPos;
use amml_data::{EntryKind, Resolved};

use super::{ClientGame, FakeVtable, Harness};

// The GameMode vtable: slot 6 is `buildBlock`, the others never run.
static GAME_MODE_VTABLE: FakeVtable<7> = FakeVtable([
    BuildGame::unused as *const (),
    BuildGame::unused as *const (),
    BuildGame::unused as *const (),
    BuildGame::unused as *const (),
    BuildGame::unused as *const (),
    BuildGame::unused as *const (),
    BuildGame::build_block as *const (),
]);
// The Level vtable: slot 1 is `getHitResult`.
static LEVEL_VTABLE: FakeVtable<2> =
    FakeVtable([BuildGame::unused as *const (), BuildGame::get_hit_result as *const ()]);

// The stand-in GameMode: its vtable, `mPlayer` at 8h and `buildingTickDelay` at A8h.
#[repr(C)]
struct GameModeObject {
    vtable: &'static FakeVtable<7>,
    player: AtomicUsize,
    _between: [usize; 19],
    delay: AtomicI64,
}

// The stand-in Level: its vtable only.
#[repr(C)]
struct LevelObject {
    vtable: &'static FakeVtable<2>,
}

// The stand-in local player: no vtable, `mGameMode` at 8h and `mLevel` at 10h.
#[repr(C)]
struct PlayerObject {
    vtable: usize,
    game_mode: AtomicUsize,
    level: AtomicUsize,
}

static GAME_MODE: GameModeObject = GameModeObject {
    vtable: &GAME_MODE_VTABLE,
    player: AtomicUsize::new(0),
    _between: [0; 19],
    delay: AtomicI64::new(200),
};
static LEVEL: LevelObject = LevelObject { vtable: &LEVEL_VTABLE };
static PLAYER: PlayerObject = PlayerObject { vtable: 0, game_mode: AtomicUsize::new(0), level: AtomicUsize::new(0) };
// The level's `HitResult`, 88h bytes.
static HIT: [AtomicU64; 17] = [const { AtomicU64::new(0) }; 17];
static CREATIVE: AtomicBool = AtomicBool::new(false);
// What the stand-in game functions received, in order.
static CALLS: Mutex<Vec<String>> = Mutex::new(Vec::new());

/// Stand-ins for the build functions: a local player whose game mode has `buildBlock` in slot 6 of its vtable, a level
/// whose vtable gives its hit result in slot 1, `handleBuildAction`, `getMaxPickRange` and `isCreative`.
pub struct BuildGame;

impl BuildGame {
    /// The range the stand-in `getMaxPickRange` gives, unlike both constants of the creative shape.
    pub const GETTER_RANGE: f32 = 9.5;
    /// What the stand-in `handleBuildAction` returns.
    pub const GAME_USE_RESULT: u8 = 0;

    /// The client entries and every build entry, pointing to the stand-ins; `without` names entries to leave out. The
    /// player's game mode and level are set, and the game mode's player is the stand-in player.
    pub fn entries(without: &[&str]) -> Vec<(&'static str, Resolved)> {
        PLAYER.game_mode.store(std::ptr::from_ref(&GAME_MODE).addr(), Ordering::SeqCst);
        PLAYER.level.store(std::ptr::from_ref(&LEVEL).addr(), Ordering::SeqCst);
        GAME_MODE.player.store(Self::player(), Ordering::SeqCst);
        let function = |address: usize| (EntryKind::Function, address as u64 - Harness::BASE);
        let global = |address: usize| (EntryKind::Global, address as u64 - Harness::BASE);
        let mut entries = ClientGame::entries(without);
        let more = [
            ("GameMode.vtable", global(std::ptr::from_ref(&GAME_MODE_VTABLE).addr())),
            ("GameMode.buildBlock", (EntryKind::VSlot, 6)),
            ("GameMode.mPlayer", (EntryKind::Field, 8)),
            ("GameMode.buildingTickDelay", (EntryKind::Field, 0xA8)),
            ("GameMode.getMaxPickRange", function(Self::get_max_pick_range as *const () as usize)),
            ("Player.mGameMode", (EntryKind::Field, 8)),
            ("Player.isCreative", function(Self::is_creative as *const () as usize)),
            ("Actor.mLevel", (EntryKind::Field, 0x10)),
            ("Level.getHitResult", (EntryKind::VSlot, 1)),
            ("ClientInputCallbacks.handleBuildAction", function(Self::handle_build_action as *const () as usize)),
        ];
        for (name, (kind, value)) in more {
            if !without.contains(&name) {
                entries.push((name, Resolved { kind, value, recipes_agreeing: 2 }));
            }
        }
        entries
    }

    /// The address of the stand-in player.
    pub fn player() -> usize {
        std::ptr::from_ref(&PLAYER).addr()
    }

    /// The address of the stand-in game mode.
    pub fn game_mode() -> *mut c_void {
        std::ptr::from_ref(&GAME_MODE).cast_mut().cast()
    }

    /// Makes the game mode's player `player`.
    pub fn set_game_mode_player(player: usize) {
        GAME_MODE.player.store(player, Ordering::SeqCst);
    }

    pub fn set_delay(value: i64) {
        GAME_MODE.delay.store(value, Ordering::SeqCst);
    }

    pub fn set_creative(creative: bool) {
        CREATIVE.store(creative, Ordering::SeqCst);
    }

    /// Fills the hit result as the game left it: kind 3 (no hit), facing 4, block 9,9,9, point 0.5,0.5,0.5 and the
    /// weak entity control block 0x77.
    pub fn reset_hit() {
        let mut hit = HitResultLayout::default();
        hit.kind = 3;
        hit.facing = 4;
        hit.block = [9, 9, 9];
        hit.pos = [0.5; 3];
        hit.weak_control = 0x77;
        Self::write_hit(hit);
    }

    /// Sets the kind, facing, block, point and entity id of the hit result, as the game's pick leaves them.
    pub fn set_hit(kind: i32, facing: u8, block: [i32; 3], pos: [f32; 3]) {
        let mut hit = Self::hit();
        hit.kind = kind;
        hit.facing = facing;
        hit.block = block;
        hit.pos = pos;
        Self::write_hit(hit);
    }

    /// The hit result now.
    pub fn hit() -> HitResultLayout {
        let words: [u64; 17] = std::array::from_fn(|index| HIT[index].load(Ordering::SeqCst));
        // SAFETY: 17 qwords are the 88h bytes of a `HitResultLayout`, a plain-old-data mirror.
        unsafe { std::mem::transmute::<[u64; 17], HitResultLayout>(words) }
    }

    fn write_hit(hit: HitResultLayout) {
        // SAFETY: a `HitResultLayout` is 88h bytes of plain-old-data, 17 qwords.
        let words = unsafe { std::mem::transmute::<HitResultLayout, [u64; 17]>(hit) };
        for (slot, word) in HIT.iter().zip(words) {
            slot.store(word, Ordering::SeqCst);
        }
    }

    /// The calls of the stand-in game functions so far, in order.
    pub fn calls() -> Vec<String> {
        CALLS.lock().unwrap().clone()
    }

    /// The game builds: a call of `buildBlock` through its address (the loader's hook is on it), with the whole
    /// registers given.
    pub fn game_build(this: *mut c_void, pos: AmmlBlockPos, face: u64, hand: u64, sim_tick: u64) -> u8 {
        let function =
            black_box(Self::build_block as extern "C" fn(*mut c_void, *const AmmlBlockPos, u64, u64, u64) -> u8);
        function(black_box(this), black_box(&raw const pos), black_box(face), black_box(hand), black_box(sim_tick))
    }

    /// The game's use press: a call of `handleBuildAction` through its address.
    pub fn game_use() -> u8 {
        let function = black_box(
            Self::handle_build_action as extern "C" fn(*mut c_void, *mut c_void, *const c_void, *const c_void) -> u8,
        );
        function(
            black_box(0x100 as *mut c_void),
            black_box(0x200 as *mut c_void),
            black_box(0x300 as *const c_void),
            black_box(0x400 as *const c_void),
        )
    }

    #[inline(never)]
    extern "C" fn unused(this: *mut c_void) {
        black_box(this);
    }

    #[inline(never)]
    extern "C" fn get_hit_result(this: *mut c_void) -> *mut HitResultLayout {
        black_box(this);
        std::ptr::from_ref(&HIT).cast_mut().cast()
    }

    // Records the arguments (the face, hand and isSimTick registers in full, then the low byte of each, which is all the
    // game reads) and the four hit result members it reads.
    #[inline(never)]
    extern "C" fn build_block(this: *mut c_void, pos: *const AmmlBlockPos, face: u64, hand: u64, sim_tick: u64) -> u8 {
        // SAFETY: every caller passes a live position.
        let pos = unsafe { black_box(pos).read() };
        let hit = Self::hit();
        CALLS.lock().unwrap().push(format!(
            "build this={:#x} pos={},{},{} face={face:#x} hand={hand:#x} sim={sim_tick:#x} bytes={},{},{} hit={} {} {},{},{} {},{},{}",
            black_box(this).addr(),
            pos.x,
            pos.y,
            pos.z,
            face & 0xFF,
            hand & 0xFF,
            sim_tick & 0xFF,
            hit.kind,
            hit.facing,
            hit.block[0],
            hit.block[1],
            hit.block[2],
            hit.pos[0],
            hit.pos[1],
            hit.pos[2],
        ));
        1
    }

    #[inline(never)]
    extern "C" fn handle_build_action(
        client: *mut c_void,
        intention: *mut c_void,
        solid: *const c_void,
        liquid: *const c_void,
    ) -> u8 {
        CALLS.lock().unwrap().push(format!(
            "use client={:#x} intention={:#x} solid={:#x} liquid={:#x}",
            black_box(client).addr(),
            black_box(intention).addr(),
            black_box(solid).addr(),
            black_box(liquid).addr()
        ));
        Self::GAME_USE_RESULT
    }

    #[inline(never)]
    extern "C" fn get_max_pick_range(this: *mut c_void) -> f32 {
        CALLS.lock().unwrap().push(format!("pick range this={:#x}", black_box(this).addr()));
        Self::GETTER_RANGE
    }

    #[inline(never)]
    extern "C" fn is_creative(this: *mut c_void) -> u8 {
        CALLS.lock().unwrap().push(format!("is creative this={:#x}", black_box(this).addr()));
        u8::from(CREATIVE.load(Ordering::SeqCst))
    }
}
