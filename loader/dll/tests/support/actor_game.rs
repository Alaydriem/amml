use std::ffi::c_void;
use std::sync::atomic::{AtomicBool, AtomicI32, AtomicU8, AtomicU32, AtomicUsize, Ordering};

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
// Four floats: `mRot` (pitch, yaw), then the previous rotation (pitch, yaw), as bits.
static ROT: [AtomicU32; 4] = [
    AtomicU32::new(30.0f32.to_bits()),
    AtomicU32::new(90.0f32.to_bits()),
    AtomicU32::new(29.0f32.to_bits()),
    AtomicU32::new(89.0f32.to_bits()),
];
// What `getViewVector` gives for an alpha of 1.0.
static VIEW: [f32; 3] = [0.0, -0.5, 0.866];
// The head yaw and previous head yaw `setYHeadRotations` last stored, as bits; none until a call.
static HEAD_ROT: [AtomicU32; 2] = [AtomicU32::new(0), AtomicU32::new(0)];
static HEAD_CALLS: AtomicI32 = AtomicI32::new(0);
static GAME_TYPE: AtomicI32 = AtomicI32::new(1);
static PERMISSION: AtomicU8 = AtomicU8::new(2);
// The abilities component: byte 1 is the player permission level.
static ABILITIES: [AtomicU8; 4] = [AtomicU8::new(0), AtomicU8::new(2), AtomicU8::new(0), AtomicU8::new(0)];
// What `getAttachPos` gives for the head location.
static HEAD: [f32; 3] = [1.5, 67.1, -3.0];
static PLAYER: PlayerObject = PlayerObject {
    vtable: &PLAYER_VTABLE,
    state_vector: &POS,
    aabb: &AABB,
    rotation: &ROT,
    context: AtomicI32::new(17),
    level: AtomicUsize::new(0),
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
// (a stand-in holding the health) at 20h, then the level pointer at 28h (null until `set_level`).
#[repr(C)]
struct PlayerObject {
    vtable: &'static FakeVtable<2>,
    state_vector: &'static [f32; 3],
    aabb: &'static [f32; 6],
    rotation: &'static [AtomicU32; 4],
    context: AtomicI32,
    level: AtomicUsize,
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
            ("Actor.getViewVector", function(Self::get_view_vector as *const () as usize)),
            ("Actor.setYHeadRotations", function(Self::set_y_head_rotations as *const () as usize)),
            ("Player.getPlayerGameType", function(Self::get_game_type as *const () as usize)),
            ("Player.getPlayerPermissionLevel", function(Self::get_permission as *const () as usize)),
            ("Player.getAbilitiesComponent", function(Self::get_abilities as *const () as usize)),
            ("AbilitiesComponent.mPlayerPermissions", (EntryKind::Field, 1)),
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

    /// The offset of the level pointer in the stand-in player (`Actor.mLevel` of a test that sets it).
    pub const LEVEL_AT: u64 = 0x28;

    /// Makes the level of the stand-in player `level`.
    pub fn set_level(level: usize) {
        PLAYER.level.store(level, Ordering::SeqCst);
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
        [f32::from_bits(ROT[0].load(Ordering::SeqCst)), f32::from_bits(ROT[1].load(Ordering::SeqCst))]
    }

    /// The previous rotation (pitch, yaw) the stand-in holds next to `mRot`.
    pub fn previous_rotation() -> [f32; 2] {
        [f32::from_bits(ROT[2].load(Ordering::SeqCst)), f32::from_bits(ROT[3].load(Ordering::SeqCst))]
    }

    /// The four floats of the rotation component: the rotation, then the previous rotation.
    pub fn rotation_component() -> [f32; 4] {
        let [pitch, yaw] = Self::rotation();
        let [previous_pitch, previous_yaw] = Self::previous_rotation();
        [pitch, yaw, previous_pitch, previous_yaw]
    }

    /// The head yaw and previous head yaw of the last `setYHeadRotations` call, and how many calls came.
    pub fn head_rotations() -> ([f32; 2], i32) {
        let bits = |at: usize| f32::from_bits(HEAD_ROT[at].load(Ordering::SeqCst));
        ([bits(0), bits(1)], HEAD_CALLS.load(Ordering::SeqCst))
    }

    /// What `getViewVector` gives for an alpha of 1.0.
    pub fn view() -> [f32; 3] {
        VIEW
    }

    /// The box of the stand-in player: min x, y, z, then max x, y, z.
    pub fn aabb() -> [f32; 6] {
        AABB
    }

    pub fn set_game_type(value: i32) {
        GAME_TYPE.store(value, Ordering::SeqCst);
    }

    /// Sets the permission both shapes read: the getter result and byte 1 of the abilities component.
    pub fn set_permission(value: u8) {
        PERMISSION.store(value, Ordering::SeqCst);
        ABILITIES[1].store(value, Ordering::SeqCst);
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

    // `Vec3 Actor::getViewVector(float) const`: only the alpha 1.0 gives the view.
    extern "C" fn get_view_vector(_this: *mut c_void, out: *mut [f32; 3], alpha: f32) -> *mut [f32; 3] {
        let value = if alpha == 1.0 { VIEW } else { [0.0; 3] };
        // SAFETY: the adapter passes a writable Vec3 buffer.
        unsafe { out.write(value) };
        out
    }

    // `void Actor::setYHeadRotations(float, float)`: records both angles.
    extern "C" fn set_y_head_rotations(_this: *mut c_void, head: f32, previous: f32) {
        HEAD_ROT[0].store(head.to_bits(), Ordering::SeqCst);
        HEAD_ROT[1].store(previous.to_bits(), Ordering::SeqCst);
        HEAD_CALLS.fetch_add(1, Ordering::SeqCst);
    }

    // `GameType Player::getPlayerGameType() const`.
    extern "C" fn get_game_type(_this: *mut c_void) -> i32 {
        GAME_TYPE.load(Ordering::SeqCst)
    }

    // `PlayerPermissionLevel Player::getPlayerPermissionLevel() const`.
    extern "C" fn get_permission(_this: *mut c_void) -> u8 {
        PERMISSION.load(Ordering::SeqCst)
    }

    // `not_null<AbilitiesComponent*>* Player::getAbilitiesComponent(not_null<AbilitiesComponent*>*) const`.
    extern "C" fn get_abilities(_this: *mut c_void, out: *mut usize) -> *mut usize {
        // SAFETY: the adapter passes a writable 8-byte slot.
        unsafe { out.write(std::ptr::from_ref(&ABILITIES).addr()) };
        out
    }

    // `int ActorAttribute::getHealth(EntityContext const&)`: the context is the stand-in at 20h of the player.
    extern "C" fn get_health(context: *const c_void) -> i32 {
        // SAFETY: the adapter passes the address of `PlayerObject::context`, a live atomic of the static player.
        unsafe { (*context.cast::<AtomicI32>()).load(Ordering::SeqCst) }
    }
}
