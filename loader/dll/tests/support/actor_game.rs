use std::ffi::c_void;
use std::sync::atomic::{AtomicBool, AtomicI32, AtomicU8, Ordering};

use amml_data::{EntryKind, Resolved};

use super::{BlockGame, ChunkGame, ClientGame, FakeVtable, Harness, RayGame};

// The Actor vtable of the stand-in player: 0 unused, 1 isAlive.
static PLAYER_VTABLE: FakeVtable<2> = FakeVtable([ActorGame::unused as *const (), ActorGame::is_alive as *const ()]);
// The Dimension vtable of the stand-in dimension: slot 13 is `getBlockSourceFromMainChunkSource`.
static DIMENSION_VTABLE: FakeVtable<14> = FakeVtable([
    ActorGame::unused as *const (),
    ActorGame::unused as *const (),
    ActorGame::unused as *const (),
    ActorGame::unused as *const (),
    ActorGame::unused as *const (),
    ActorGame::unused as *const (),
    ActorGame::unused as *const (),
    ActorGame::unused as *const (),
    ActorGame::unused as *const (),
    ActorGame::unused as *const (),
    ActorGame::unused as *const (),
    ActorGame::unused as *const (),
    ActorGame::unused as *const (),
    ActorGame::get_block_source as *const (),
]);
// `StateVectorComponent.mPos` (the eyes), the `AABBShapeComponent` box (min, then max) and `mRot` (pitch, yaw).
static POS: [f32; 3] = [1.5, 65.62, -3.0];
static AABB: [f32; 6] = [1.2, 64.0, -3.3, 1.8, 65.8, -2.7];
static ROT: [f32; 2] = [30.0, 90.0];
// What `getAttachPos` gives for the head location.
static HEAD: [f32; 3] = [1.5, 67.1, -3.0];
static PLAYER: PlayerObject = PlayerObject {
    vtable: &PLAYER_VTABLE,
    state_vector: &POS,
    aabb: &AABB,
    rotation: &ROT,
    context: AtomicI32::new(17),
};
static DIMENSION: DimensionObject = DimensionObject { vtable: &DIMENSION_VTABLE, sky_darken: AtomicU8::new(4) };
// Two stand-in block sources: the one of the player's dimension, and one of another dimension.
static BLOCK_SOURCE_VTABLE: FakeVtable<9> = FakeVtable([
    ActorGame::unused as *const (),
    ActorGame::unused as *const (),
    ActorGame::unused as *const (),
    ChunkGame::get_chunk as *const (),
    BlockGame::add_listener as *const (),
    BlockGame::remove_listener as *const (),
    BlockGame::get_dimension_id as *const (),
    RayGame::clip as *const (),
    RayGame::fetch as *const (),
]);
static BLOCK_SOURCE: BlockSourceObject = BlockSourceObject { vtable: &BLOCK_SOURCE_VTABLE };
static OTHER_BLOCK_SOURCE: [u8; 8] = [2; 8];

// The stand-in player: its vtable, then the three component pointers at 8h, 10h and 18h, then the entity context
// (a stand-in holding the health) at 20h.
#[repr(C)]
struct PlayerObject {
    vtable: &'static FakeVtable<2>,
    state_vector: &'static [f32; 3],
    aabb: &'static [f32; 6],
    rotation: &'static [f32; 2],
    context: AtomicI32,
}

// The stand-in block source: its vtable, with `getChunk` in slot 3, the listener calls in slots 4 to 6, `clip` in slot 7
// and `fetchEntities` in slot 8.
#[repr(C)]
struct BlockSourceObject {
    vtable: &'static FakeVtable<9>,
}

// The stand-in dimension: its vtable, then `mSkyDarken` at 8h.
#[repr(C)]
struct DimensionObject {
    vtable: &'static FakeVtable<14>,
    sky_darken: AtomicU8,
}
static ALIVE: AtomicBool = AtomicBool::new(true);

/// A stand-in local player for the actor adapter: an object whose first qword points to an Actor vtable with
/// `isAlive` in slot 1, followed by its state vector, box and rotation component pointers and its entity context. Its
/// dimension is a stand-in object whose vtable gives a block source in slot 13.
pub struct ActorGame;

impl ActorGame {
    /// The offset of the sky darkness byte in the stand-in dimension.
    pub const SKY_DARKEN_AT: u64 = 8;
    /// The health the stand-in entity context holds.
    pub const HEALTH: i32 = 17;

    /// The client entries with `Actor.isAlive`; `without` names entries to leave out.
    pub fn entries(without: &[&str]) -> Vec<(&'static str, Resolved)> {
        let mut entries = ClientGame::entries(without);
        if !without.contains(&"Actor.isAlive") {
            entries.push(("Actor.isAlive", Resolved { kind: EntryKind::VSlot, value: 1, recipes_agreeing: 2 }));
        }
        let function = |address: usize| (EntryKind::Function, address as u64 - Harness::BASE);
        // The v2 reads, at the offsets of `PlayerObject`, and the v3 reads.
        let more = [
            ("Actor.stateVectorComponent", (EntryKind::Field, 8)),
            ("StateVectorComponent.mPos", (EntryKind::Field, 0)),
            ("Actor.aabbShapeComponent", (EntryKind::Field, 0x10)),
            ("Actor.actorRotationComponent", (EntryKind::Field, 0x18)),
            ("ActorRotationComponent.mRot", (EntryKind::Field, 0)),
            ("Actor.getDimension", function(Self::get_dimension as *const () as usize)),
            ("Dimension.getDimensionId", (EntryKind::VSlot, 3)),
            ("Actor.getAttachPos", function(Self::get_attach_pos as *const () as usize)),
            ("Dimension.getBlockSourceFromMainChunkSource", (EntryKind::VSlot, 13)),
            ("ActorAttribute.getHealth", function(Self::get_health as *const () as usize)),
            ("Actor.mEntityContext", (EntryKind::Field, 0x20)),
            ("Dimension.mSkyDarken", (EntryKind::Field, Self::SKY_DARKEN_AT)),
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

    /// The address of the stand-in dimension of the player.
    pub fn dimension() -> usize {
        std::ptr::from_ref(&DIMENSION).addr()
    }

    /// The block source of the stand-in dimension of the player, and a block source of another dimension.
    pub fn block_source() -> usize {
        std::ptr::from_ref(&BLOCK_SOURCE).addr()
    }

    pub fn other_block_source() -> usize {
        std::ptr::from_ref(&OTHER_BLOCK_SOURCE).addr()
    }

    /// The eyes, the feet y and the rotation of the stand-in player.
    pub fn eyes() -> [f32; 3] {
        POS
    }

    pub fn feet_y() -> f32 {
        AABB[1]
    }

    pub fn rotation() -> [f32; 2] {
        ROT
    }

    /// The head position `getAttachPos` gives for the head location.
    pub fn head() -> [f32; 3] {
        HEAD
    }

    pub fn set_alive(alive: bool) {
        ALIVE.store(alive, Ordering::SeqCst);
    }

    pub fn set_sky_darken(value: u8) {
        DIMENSION.sky_darken.store(value, Ordering::SeqCst);
    }

    extern "C" fn unused(_this: *mut c_void) -> u8 {
        0
    }

    extern "C" fn is_alive(_this: *mut c_void) -> u8 {
        u8::from(ALIVE.load(Ordering::SeqCst))
    }

    extern "C" fn get_dimension(_this: *mut c_void) -> *mut c_void {
        std::ptr::from_ref(&DIMENSION).cast_mut().cast()
    }

    extern "C" fn get_block_source(_this: *mut c_void) -> *mut c_void {
        std::ptr::from_ref(&BLOCK_SOURCE).cast_mut().cast()
    }

    // `Vec3 Actor::getAttachPos(ActorLocation, float) const`: only the head location (3) gives the head.
    extern "C" fn get_attach_pos(_this: *mut c_void, out: *mut [f32; 3], location: i32, alpha: f32) -> *mut [f32; 3] {
        let value = if location == 3 && alpha == 0.0 { HEAD } else { [0.0; 3] };
        // SAFETY: the adapter passes a writable Vec3 buffer.
        unsafe { out.write(value) };
        out
    }

    // `int ActorAttribute::getHealth(EntityContext const&)`: the context is the stand-in at 20h of the player.
    extern "C" fn get_health(context: *const c_void) -> i32 {
        // SAFETY: the adapter passes the address of `PlayerObject::context`, a live atomic of the static player.
        unsafe { (*context.cast::<AtomicI32>()).load(Ordering::SeqCst) }
    }
}
