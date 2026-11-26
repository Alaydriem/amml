use std::ffi::c_void;
use std::sync::atomic::{AtomicBool, Ordering};

use amml_abi::{AmmlAabb, AmmlActorBox, AmmlActorEntry, AmmlStatus, AmmlVec3};

use crate::{AabbShapeLayout, EnttLookup, EnttStorageLayout, LoaderState};

/// `span<not_null<Actor*>> BlockSource::fetchEntities(Actor const* except, AABB const&, bool, bool)`: rcx the block
/// source, the two-word span through rdx (the loader passes a larger zeroed buffer), the actor left out in r8 (null:
/// none), the box in r9, the two flags on the stack.
type FetchFn = unsafe extern "C" fn(*mut c_void, *mut usize, *const c_void, *const [f32; 6], u8, u8) -> *mut usize;

/// The game side of the `world.actors_in` area: `BlockSource::fetchEntities` through its vtable slot, and the box of
/// each actor through its `AABBShapeComponent` (`Actor.aabbShapeComponent`, `AabbShapeLayout`).
///
/// Read from the exe (`fetchEntities` 0x32d3af0 / 0x47a3830, its body 0x32d3b80 / 0x47a38c0): the game clears a member
/// vector of the block source at the start of each fetch, fills it and gives a span of it, so the span is valid only
/// until the next fetch; the actors are copied out before `actors_in` returns. The span is (count, pointer) on 26.63
/// (`*out = size >> 3; out[1] = begin`) and (pointer, count) on 70.26 (`*out = begin; out[1] = size >> 3`): the derived
/// `BlockSource.fetchEntitiesSpanOrder` (0 count first, 1 pointer first; its recipe proves the 8-byte element too) picks
/// the order, and any other value leaves the adapter out. The words are then checked against that order
/// (`span_parts`): a count is below 10000h and a non-empty span has an 8-aligned pointer at or above it (no user memory
/// is below 10000h on Windows). The flags are the ones the 26.63 freecam mod passed (1, 0): the first goes to the
/// per-chunk fetch, the second adds the actors of a second list of the block source; both builds use them the same way.
/// The calls are for the game thread.
///
/// The type of each actor (`actors_in_entries`) is its `ActorTypeComponent`, one `u32` in the entt registry of the
/// actor's `EntityContext` (`Actor.mEntityContext`: the entity registry, the entt registry at 8h, the entity id at
/// 10h). It is read with the bounds-checked `EnttLookup` and no game call, so an actor without the component, or with
/// a null registry, gives type 0. The storage is looked up once for each registry met in a call.
pub struct ActorsInAdapter {
    fetch_slot: usize,
    shape_at: usize,
    // Whether the count is the first word of the span (`BlockSource.fetchEntitiesSpanOrder` 0).
    count_first: bool,
    // The offset of the entity context in Actor and the entt type id of `ActorTypeComponent`; `None` when either entry
    // did not resolve, and then `actors_in_entries` is not available.
    actor_type: Option<(usize, u32)>,
    // Set once the first span that does not fit the derived order is logged.
    span_logged: AtomicBool,
}

impl ActorsInAdapter {
    // The lowest address user memory starts at on Windows; also one more than the most actors a span may name.
    const LOWEST_POINTER: usize = 0x10000;
    // The two flags of the fetch.
    const FIRST_FLAG: u8 = 1;
    const SECOND_FLAG: u8 = 0;
    // The span buffer in words: the two words of the span, and room for a build whose result grew.
    const SPAN_WORDS: usize = 8;
    // `EntityContext::mEnTTRegistry` and `EntityContext::mEntity`, as `CameraEntityLayout`.
    const CONTEXT_ENTT_AT: usize = 0x08;
    const CONTEXT_ENTITY_AT: usize = 0x10;
    // An `ActorTypeComponent` is one `u32`.
    const TYPE_COMPONENT_SIZE: usize = 4;

    /// The adapter for the `BlockSource.fetchEntities` slot, the `Actor.aabbShapeComponent` offset and the
    /// `BlockSource.fetchEntitiesSpanOrder` of the build; `None` when one did not resolve or the order is neither 0 nor 1
    /// (logged as `actor span mismatch`). The `Actor.mEntityContext` and `ActorTypeComponent.typeId` entries are not
    /// needed: without them the adapter has no actor type (`has_actor_type`).
    pub fn new(state: &LoaderState) -> Option<Self> {
        let value = |name: &str| state.resolve(name).ok().and_then(|value| usize::try_from(value).ok());
        let count_first = match value("BlockSource.fetchEntitiesSpanOrder")? {
            0 => true,
            1 => false,
            order => {
                curia::warn!("actor span mismatch", { entry: "BlockSource.fetchEntitiesSpanOrder", value: order });
                return None;
            }
        };
        let type_id = state.resolve("ActorTypeComponent.typeId").ok().and_then(|value| u32::try_from(value).ok());
        let actor_type = value("Actor.mEntityContext").zip(type_id);
        Some(Self {
            fetch_slot: value("BlockSource.fetchEntities")?,
            shape_at: value("Actor.aabbShapeComponent")?,
            count_first,
            actor_type,
            span_logged: AtomicBool::new(false),
        })
    }

    /// Whether the entries of the actor type resolved, so `actors_in_entries` can give the type of each actor.
    pub fn has_actor_type(&self) -> bool {
        self.actor_type.is_some()
    }

    /// The count and the pointer of the span `words` in the derived order (`count_first`); `None` when the words do not
    /// fit it: a count of 10000h or more, or a non-empty span whose pointer is below 10000h or not 8-aligned. An empty
    /// span gives a count of 0 whatever the pointer word holds.
    pub fn span_parts(words: [usize; 2], count_first: bool) -> Option<(usize, usize)> {
        let [first, second] = words;
        let (count, data) = if count_first { (first, second) } else { (second, first) };
        if count >= Self::LOWEST_POINTER {
            return None;
        }
        if count == 0 {
            return Some((0, data));
        }
        (data >= Self::LOWEST_POINTER && data % size_of::<usize>() == 0).then_some((count, data))
    }

    /// Writes the boxes of the actors the game keeps in `source` whose boxes meet `aabb` to `out`, in the game's order,
    /// and gives the number written. An actor with no shape component is left out. `local_player` is marked.
    /// `FAILED` when the block source has no function in the slot or the span does not fit the derived order (logged
    /// once).
    ///
    /// # Safety
    /// `source` is the live block source of the local player, called on the game thread.
    pub unsafe fn actors_in(
        &self,
        source: usize,
        aabb: &AmmlAabb,
        local_player: Option<usize>,
        out: &mut [AmmlActorBox],
    ) -> Result<u32, AmmlStatus> {
        let mut written = 0usize;
        let room = out.len();
        let visit = |actor: usize, aabb: AmmlAabb| {
            out[written] =
                AmmlActorBox { aabb, is_local_player: u8::from(local_player == Some(actor)), _reserved: [0; 3] };
            written += 1;
        };
        // SAFETY: forwarded to the caller.
        unsafe { self.walk(source, aabb, room, visit) }?;
        u32::try_from(written).map_err(|_| AmmlStatus::FAILED)
    }

    /// As `actors_in`, with the `ActorType` of each actor (0 for an actor without an `ActorTypeComponent`).
    /// `NOT_AVAILABLE` when the adapter has no actor type (`has_actor_type`).
    ///
    /// # Safety
    /// As `actors_in`.
    pub unsafe fn actors_in_entries(
        &self,
        source: usize,
        aabb: &AmmlAabb,
        local_player: Option<usize>,
        out: &mut [AmmlActorEntry],
    ) -> Result<u32, AmmlStatus> {
        let (context_at, type_id) = self.actor_type.ok_or(AmmlStatus::NOT_AVAILABLE)?;
        // The registry last met and its `ActorTypeComponent` storage.
        let mut storage: Option<(usize, Option<*const EnttStorageLayout>)> = None;
        let mut written = 0usize;
        let room = out.len();
        let visit = |actor: usize, aabb: AmmlAabb| {
            // SAFETY: the walk gives only live, non-null actors of the fetch, on the game thread.
            let actor_type = unsafe { Self::type_of(actor, context_at, type_id, &mut storage) }.unwrap_or(0);
            out[written] = AmmlActorEntry {
                aabb,
                actor_type,
                is_local_player: u8::from(local_player == Some(actor)),
                _reserved: [0; 3],
            };
            written += 1;
        };
        // SAFETY: forwarded to the caller.
        unsafe { self.walk(source, aabb, room, visit) }?;
        u32::try_from(written).map_err(|_| AmmlStatus::FAILED)
    }

    // Fetches the actors of `source` whose boxes meet `aabb` and hands each one with a shape component, with its box, to
    // `visit`, in the game's order, at most `room` of them.
    //
    // Safety: as `actors_in`.
    unsafe fn walk(
        &self,
        source: usize,
        aabb: &AmmlAabb,
        room: usize,
        mut visit: impl FnMut(usize, AmmlAabb),
    ) -> Result<(), AmmlStatus> {
        // SAFETY: `source` is a live block source (the caller's contract).
        let function = unsafe { Self::virtual_fn(source, self.fetch_slot) }.ok_or(AmmlStatus::FAILED)?;
        // SAFETY: the slot holds `BlockSource::fetchEntities` with the arguments above.
        let fetch = unsafe { std::mem::transmute::<usize, FetchFn>(function) };
        let bounds = [aabb.min.x, aabb.min.y, aabb.min.z, aabb.max.x, aabb.max.y, aabb.max.z];
        let mut buffer = [0usize; Self::SPAN_WORDS];
        // SAFETY: the block source is live; the span buffer (larger than the two words) and the box live across the
        // call; no actor is left out.
        unsafe {
            fetch(
                std::ptr::with_exposed_provenance_mut(source),
                buffer.as_mut_ptr(),
                std::ptr::null(),
                &raw const bounds,
                Self::FIRST_FLAG,
                Self::SECOND_FLAG,
            )
        };
        let words = [buffer[0], buffer[1]];
        let Some((count, data)) = Self::span_parts(words, self.count_first) else {
            if !self.span_logged.swap(true, Ordering::Relaxed) {
                curia::warn!("actor span mismatch", { first: words[0], second: words[1] });
            }
            return Err(AmmlStatus::FAILED);
        };
        let mut given = 0usize;
        for index in 0..count {
            if given == room {
                break;
            }
            // SAFETY: `index` is below the count of the span the game just gave, which points to that many actor
            // pointers in a member vector of the block source; nothing runs between the fetch and this read.
            let actor = unsafe { std::ptr::with_exposed_provenance::<usize>(data + index * size_of::<usize>()).read() };
            // SAFETY: the game lists only live actors (`not_null`); `shape_at` is the offset of the shape pointer.
            let Some(shape) = (unsafe { self.box_of(actor) }) else { continue };
            visit(
                actor,
                AmmlAabb {
                    min: AmmlVec3 { x: shape[0], y: shape[1], z: shape[2] },
                    max: AmmlVec3 { x: shape[3], y: shape[4], z: shape[5] },
                },
            );
            given += 1;
        }
        Ok(())
    }

    // The `ActorType` of the live, non-null `actor` from the `ActorTypeComponent` (entt type id `type_id`) of its entity
    // context at `context_at`; `None` for a null registry or an actor the storage does not hold. `storage` keeps the
    // storage of the registry last met.
    //
    // Safety: `actor` is a live actor of the game, read on the game thread.
    unsafe fn type_of(
        actor: usize,
        context_at: usize,
        type_id: u32,
        storage: &mut Option<(usize, Option<*const EnttStorageLayout>)>,
    ) -> Option<u32> {
        let context = actor.checked_add(context_at)?;
        let (registry_at, entity_at) =
            (context.checked_add(Self::CONTEXT_ENTT_AT)?, context.checked_add(Self::CONTEXT_ENTITY_AT)?);
        // SAFETY: the actor is live and holds its `EntityContext` by value at `context_at`.
        let registry = unsafe { std::ptr::with_exposed_provenance::<usize>(registry_at).read_unaligned() };
        if registry == 0 {
            return None;
        }
        // SAFETY: as above; the entity id follows the entt registry reference.
        let entity = unsafe { std::ptr::with_exposed_provenance::<u32>(entity_at).read_unaligned() };
        let pool = match *storage {
            Some((met, pool)) if met == registry => pool,
            _ => {
                // SAFETY: a non-null entt registry of a live actor, on the game thread.
                let pool = unsafe { EnttLookup::storage(std::ptr::with_exposed_provenance(registry), type_id) };
                *storage = Some((registry, pool));
                pool
            }
        }?;
        // SAFETY: the storage of `ActorTypeComponent`, whose components are one `u32` each.
        let at = unsafe { EnttLookup::component(pool, entity, Self::TYPE_COMPONENT_SIZE) }?;
        // SAFETY: `at` is inside a payload page of the storage, the start of a 4-byte component.
        Some(unsafe { std::ptr::with_exposed_provenance::<u32>(at).read_unaligned() })
    }

    // The box of the live `actor`; `None` for a null actor or a null shape component.
    unsafe fn box_of(&self, actor: usize) -> Option<[f32; 6]> {
        if actor == 0 {
            return None;
        }
        let at = actor.checked_add(self.shape_at)?;
        // SAFETY: the actor is live and holds the pointer to its `AABBShapeComponent` at `shape_at`.
        let shape = unsafe { std::ptr::with_exposed_provenance::<usize>(at).read_unaligned() };
        if shape == 0 {
            return None;
        }
        // SAFETY: a live `AABBShapeComponent`, laid out as the mirror.
        Some(unsafe { std::ptr::with_exposed_provenance::<AabbShapeLayout>(shape).read_unaligned() }.aabb)
    }

    // The function in `slot` of the vtable at the start of `object`.
    unsafe fn virtual_fn(object: usize, slot: usize) -> Option<usize> {
        // SAFETY: `object` is a live game object whose first qword is its vtable.
        let vtable = unsafe { std::ptr::with_exposed_provenance::<usize>(object).read_unaligned() };
        if vtable == 0 {
            return None;
        }
        let entry = vtable.checked_add(slot.checked_mul(size_of::<usize>())?)?;
        // SAFETY: the vtable of the object; the entry's recipe keeps `slot` inside it.
        let function = unsafe { std::ptr::with_exposed_provenance::<usize>(entry).read_unaligned() };
        (function != 0).then_some(function)
    }
}
