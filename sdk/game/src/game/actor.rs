use std::ffi::c_void;
use std::ptr::NonNull;

use crate::game::{ActorPose, BlockSource, Dimension, RawObject};
use crate::{EntryId, GameEntries};

/// An `Actor*` of the game.
pub struct Actor<'a> {
    ptr: NonNull<c_void>,
    entries: &'a GameEntries,
}

impl<'a> Actor<'a> {
    /// `SharedTypes::Legacy::ActorLocation::Head`, the location `head_position` asks `getAttachPos` for.
    pub const HEAD_LOCATION: i32 = 3;

    /// The feature of `head_position` and `health`.
    pub const EXTRA_FEATURE: &'static str = "game.actor_extra";

    /// The entries of `EXTRA_FEATURE`.
    pub const EXTRA_ENTRIES: [EntryId; 3] =
        [EntryId::ACTOR_GET_ATTACH_POS, EntryId::ACTOR_ENTITY_CONTEXT, EntryId::ACTOR_ATTRIBUTE_GET_HEALTH];

    /// Wraps `ptr`; `None` for a null pointer.
    ///
    /// # Safety
    /// `ptr` must be null or a live `Actor` of the game build `entries` was resolved for, and must stay live while
    /// the wrapper is used.
    #[allow(unsafe_code)]
    pub unsafe fn from_raw(ptr: *mut c_void, entries: &'a GameEntries) -> Option<Self> {
        NonNull::new(ptr).map(|ptr| Self { ptr, entries })
    }

    /// The wrapped pointer.
    pub fn as_ptr(&self) -> *mut c_void {
        self.ptr.as_ptr()
    }

    // The code address that a function entry holds; `None` when the entry is off or does not fit a pointer.
    fn function(&self, id: EntryId) -> Option<*const c_void> {
        Some(usize::try_from(self.entries.get(id)?).ok()? as *const c_void)
    }

    /// The position and look angles. `None` when an entry it needs is off or a pointer on the path is null. Game
    /// thread; the read is not thread-checked.
    ///
    /// `Actor.stateVectorComponent` and `Actor.actorRotationComponent` are offsets from the actor of pointers to the
    /// `StateVectorComponent` and the `ActorRotationComponent`. `StateVectorComponent::mPos` is a `Vec3`;
    /// `ActorRotationComponent::mRot` is a `Vec2` with pitch in x and yaw in y.
    #[allow(unsafe_code)]
    pub fn pose(&self) -> Option<ActorPose> {
        let entries = self.entries;
        let state_vector_at = entries.get(EntryId::ACTOR_STATE_VECTOR_COMPONENT)?;
        let rotation_at = entries.get(EntryId::ACTOR_ACTOR_ROTATION_COMPONENT)?;
        let pos_at = entries.get(EntryId::STATE_VECTOR_COMPONENT_POS)?;
        let rot_at = entries.get(EntryId::ACTOR_ROTATION_COMPONENT_ROT)?;
        // SAFETY: `from_raw` guarantees a live Actor; the entry is the offset of its component pointer.
        let state_vector = unsafe { RawObject::pointer(self.ptr.as_ptr(), state_vector_at) };
        // SAFETY: as above.
        let rotation = unsafe { RawObject::pointer(self.ptr.as_ptr(), rotation_at) };
        let (state_vector, rotation) = (state_vector?, rotation?);
        // SAFETY: a non-null component pointer of a live actor points to its StateVectorComponent, which holds a
        // Vec3 at `pos_at`.
        let pos = unsafe { RawObject::read::<[f32; 3]>(state_vector, pos_at) }?;
        // SAFETY: as above, for the ActorRotationComponent and its Vec2 at `rot_at`.
        let [pitch, yaw] = unsafe { RawObject::read::<[f32; 2]>(rotation, rot_at) }?;
        Some(ActorPose { pos, pitch, yaw })
    }

    /// The dimension the actor is in. `None` when an entry is off or the game returns null. Game thread; the read
    /// is not thread-checked.
    #[allow(unsafe_code)]
    pub fn dimension(&self) -> Option<Dimension<'a>> {
        let function = self.function(EntryId::ACTOR_GET_DIMENSION)?;
        // SAFETY: the entry is the address of `Dimension& Actor::getDimension() const` and `from_raw` guarantees a
        // live Actor.
        let dimension = unsafe { RawObject::call_pointer(function, self.ptr.as_ptr()) }?;
        // SAFETY: the game returns a live Dimension reference.
        unsafe { Dimension::from_raw(dimension, self.entries) }
    }

    /// The block source of the actor's dimension. `None` when an entry is off or the game returns null. Game
    /// thread; the read is not thread-checked.
    #[allow(unsafe_code)]
    pub fn block_source(&self) -> Option<BlockSource<'a>> {
        let function = self.function(EntryId::ACTOR_GET_DIMENSION_BLOCK_SOURCE)?;
        // SAFETY: the entry is the address of `BlockSource& Actor::getDimensionBlockSource() const` and `from_raw`
        // guarantees a live Actor.
        let source = unsafe { RawObject::call_pointer(function, self.ptr.as_ptr()) }?;
        // SAFETY: the game returns a live BlockSource reference.
        unsafe { BlockSource::from_raw(source, self.entries) }
    }

    /// Whether the actor is alive, through the `isAlive` virtual. `None` when the entry is off or the vtable slot
    /// is null. Game thread; the read is not thread-checked.
    #[allow(unsafe_code)]
    pub fn is_alive(&self) -> Option<bool> {
        let slot = self.entries.get(EntryId::ACTOR_IS_ALIVE)?;
        // SAFETY: `from_raw` guarantees a live Actor, whose first qword is its vtable, and the entry is the slot of
        // `isAlive` in that vtable.
        let function = unsafe { RawObject::slot(self.ptr.as_ptr(), slot) }?;
        // SAFETY: the slot holds `bool Actor::isAlive() const`.
        Some(unsafe { RawObject::call_bool(function, self.ptr.as_ptr()) })
    }

    /// The `ILevel*` of the actor. `None` when the entry is off or the pointer is null. Game thread; the read is
    /// not thread-checked.
    #[allow(unsafe_code)]
    pub fn level(&self) -> Option<NonNull<c_void>> {
        let offset = self.entries.get(EntryId::ACTOR_LEVEL)?;
        // SAFETY: `from_raw` guarantees a live Actor, and the entry is the offset of its `mLevel` pointer.
        let level = unsafe { RawObject::pointer(self.ptr.as_ptr(), offset) }?;
        NonNull::new(level)
    }

    /// The head position, as LeviLamina's `Actor::getHeadPos`: `getAttachPos(ActorLocation::Head, 0)`. `None` when
    /// the entry is off. Game thread; the call is not thread-checked.
    ///
    /// `getAttachPos` is a non-static member function that returns a `Vec3`, so the result comes back through a
    /// hidden pointer after `this`; the location is the third argument and the float alpha the fourth (xmm3).
    #[allow(unsafe_code)]
    pub fn head_position(&self) -> Option<[f32; 3]> {
        let function = self.function(EntryId::ACTOR_GET_ATTACH_POS)?;
        if function.is_null() {
            return None;
        }
        // SAFETY: the entry is the address of `Vec3 Actor::getAttachPos(ActorLocation, float) const`.
        let call: unsafe extern "C" fn(*mut c_void, *mut [f32; 3], i32, f32) -> *mut [f32; 3] =
            unsafe { std::mem::transmute(function) };
        let mut out = [0.0f32; 3];
        // SAFETY: `from_raw` guarantees a live Actor, and `out` is a Vec3 buffer the callee fills.
        unsafe { call(self.ptr.as_ptr(), &raw mut out, Self::HEAD_LOCATION, 0.0) };
        Some(out)
    }

    /// The health, as LeviLamina's `Actor::getHealth`: `ActorAttribute::getHealth(mEntityContext)`. `None` when an
    /// entry is off. Game thread; the call is not thread-checked.
    #[allow(unsafe_code)]
    pub fn health(&self) -> Option<i32> {
        let function = self.function(EntryId::ACTOR_ATTRIBUTE_GET_HEALTH)?;
        let context = usize::try_from(self.entries.get(EntryId::ACTOR_ENTITY_CONTEXT)?).ok()?;
        if function.is_null() {
            return None;
        }
        // SAFETY: the entry is the address of `int ActorAttribute::getHealth(EntityContext const&)`.
        let call: unsafe extern "C" fn(*const c_void) -> i32 = unsafe { std::mem::transmute(function) };
        let context = self.ptr.as_ptr().cast::<u8>().wrapping_add(context).cast::<c_void>();
        // SAFETY: the actor is live and holds its EntityContext by value at the entry's offset.
        Some(unsafe { call(context) })
    }
}
