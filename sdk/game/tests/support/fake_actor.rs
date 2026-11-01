use std::ffi::c_void;

use amml_game::{EntryId, GameEntries};

use super::FakeEntries;

/// A fake `Actor`: 16 padding bytes, then the state vector pointer (offset 16), a shape pointer, the rotation
/// pointer (offset 32) and a walk animation pointer, then the level, the dimension and the block source the fake
/// getters return.
#[allow(dead_code)]
#[repr(C)]
pub struct FakeActor {
    pad: [u8; 16],
    state_vector: *mut c_void,
    shape: *mut c_void,
    rotation: *mut c_void,
    walk_animation: *mut c_void,
    level: *mut c_void,
    dimension: *mut c_void,
    block_source: *mut c_void,
}

impl FakeActor {
    pub fn new(
        state_vector: *mut c_void,
        rotation: *mut c_void,
        dimension: *mut c_void,
        block_source: *mut c_void,
    ) -> Self {
        Self {
            pad: [0; 16],
            state_vector,
            shape: std::ptr::null_mut(),
            rotation,
            walk_animation: std::ptr::null_mut(),
            level: 0x4000 as *mut c_void,
            dimension,
            block_source,
        }
    }

    pub fn ptr(&mut self) -> *mut c_void {
        (&raw mut *self).cast()
    }

    /// Entries for every actor read, with the offsets of this layout.
    pub fn entries() -> GameEntries {
        FakeEntries::build(&Self::values())
    }

    /// The same entries without the one named by `id`.
    pub fn entries_without(id: EntryId) -> GameEntries {
        let values: Vec<_> = Self::values().into_iter().filter(|(entry, _)| *entry != id).collect();
        FakeEntries::build(&values)
    }

    fn values() -> Vec<(EntryId, u64)> {
        vec![
            (EntryId::ACTOR_STATE_VECTOR_COMPONENT, 16),
            (EntryId::ACTOR_ACTOR_ROTATION_COMPONENT, 32),
            (EntryId::STATE_VECTOR_COMPONENT_POS, 4),
            (EntryId::ACTOR_ROTATION_COMPONENT_ROT, 8),
            (EntryId::ACTOR_LEVEL, 48),
            (
                EntryId::ACTOR_GET_DIMENSION,
                Self::get_dimension as extern "C" fn(*mut c_void) -> *mut c_void as usize as u64,
            ),
            (
                EntryId::ACTOR_GET_DIMENSION_BLOCK_SOURCE,
                Self::get_block_source as extern "C" fn(*mut c_void) -> *mut c_void as usize as u64,
            ),
            (EntryId::DIMENSION_GET_DIMENSION_ID, super::FakeDimension::ID_SLOT),
        ]
    }

    extern "C" fn get_dimension(this: *mut c_void) -> *mut c_void {
        // SAFETY: `this` is a FakeActor.
        unsafe { (*this.cast::<FakeActor>()).dimension }
    }

    extern "C" fn get_block_source(this: *mut c_void) -> *mut c_void {
        // SAFETY: `this` is a FakeActor.
        unsafe { (*this.cast::<FakeActor>()).block_source }
    }
}
