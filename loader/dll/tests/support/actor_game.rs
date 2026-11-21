use std::ffi::c_void;
use std::sync::atomic::{AtomicBool, Ordering};

use amml_data::{EntryKind, Resolved};

use super::{ClientGame, FakeVtable};

// The Actor vtable of the stand-in player: 0 unused, 1 isAlive.
static PLAYER_VTABLE: FakeVtable<2> = FakeVtable([ActorGame::unused as *const (), ActorGame::is_alive as *const ()]);
// `StateVectorComponent.mPos` (the eyes), the `AABBShapeComponent` box (min, then max) and `mRot` (pitch, yaw).
static POS: [f32; 3] = [1.5, 65.62, -3.0];
static AABB: [f32; 6] = [1.2, 64.0, -3.3, 1.8, 65.8, -2.7];
static ROT: [f32; 2] = [30.0, 90.0];
static PLAYER: PlayerObject = PlayerObject { vtable: &PLAYER_VTABLE, state_vector: &POS, aabb: &AABB, rotation: &ROT };

// The stand-in player: its vtable, then the three component pointers at 8h, 10h and 18h.
#[repr(C)]
struct PlayerObject {
    vtable: &'static FakeVtable<2>,
    state_vector: &'static [f32; 3],
    aabb: &'static [f32; 6],
    rotation: &'static [f32; 2],
}
static ALIVE: AtomicBool = AtomicBool::new(true);

/// A stand-in local player for the actor adapter: an object whose first qword points to an Actor vtable with
/// `isAlive` in slot 1, followed by its state vector, box and rotation component pointers.
pub struct ActorGame;

impl ActorGame {
    /// The client entries with `Actor.isAlive`; `without` names entries to leave out.
    pub fn entries(without: &[&str]) -> Vec<(&'static str, Resolved)> {
        let mut entries = ClientGame::entries(without);
        if !without.contains(&"Actor.isAlive") {
            entries.push(("Actor.isAlive", Resolved { kind: EntryKind::VSlot, value: 1, recipes_agreeing: 2 }));
        }
        // The v2 reads, at the offsets of `PlayerObject`. No test calls the dimension reads.
        let v2 = [
            ("Actor.stateVectorComponent", EntryKind::Field, 8),
            ("StateVectorComponent.mPos", EntryKind::Field, 0),
            ("Actor.aabbShapeComponent", EntryKind::Field, 0x10),
            ("Actor.actorRotationComponent", EntryKind::Field, 0x18),
            ("ActorRotationComponent.mRot", EntryKind::Field, 0),
            ("Actor.getDimension", EntryKind::Function, 0x1000),
            ("Dimension.getDimensionId", EntryKind::VSlot, 3),
        ];
        for (name, kind, value) in v2 {
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

    pub fn set_alive(alive: bool) {
        ALIVE.store(alive, Ordering::SeqCst);
    }

    extern "C" fn unused(_this: *mut c_void) -> u8 {
        0
    }

    extern "C" fn is_alive(_this: *mut c_void) -> u8 {
        u8::from(ALIVE.load(Ordering::SeqCst))
    }
}
