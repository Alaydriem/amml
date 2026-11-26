use std::ffi::c_void;
use std::sync::Mutex;
use std::sync::atomic::{AtomicI32, AtomicUsize, Ordering};

use amml_data::{EntryKind, Resolved};

use super::FakeVtable;

/// A `BlockSource::clip` call the stand-in game saw: the block source, the two points, then the eight arguments after
/// them (liquid, shape type, steps, border blocks, full blocks only, the actor, whether the `std::function` had a
/// callable, stop on the first liquid).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClipCall {
    pub source: usize,
    pub from: [f32; 3],
    pub to: [f32; 3],
    pub liquid: u8,
    pub shape: u8,
    pub steps: i32,
    pub border: u8,
    pub full_only: u8,
    pub actor: usize,
    pub has_callable: bool,
    pub stop_on_liquid: u8,
}

/// A `BlockSource::fetchEntities` call: the block source, the actor left out, the box and the two flags.
pub type FetchCall = (usize, usize, [f32; 6], u8, u8);

/// A block the stand-in clip offers to the `std::function`, in order: the block, its position, the face and the point
/// it gives for a hit. The first block the function accepts is the hit.
#[derive(Clone, Copy, Debug)]
pub struct Candidate {
    pub block: usize,
    pub pos: [i32; 3],
    pub face: u8,
    pub point: [f32; 3],
}

/// The order of the two words of the span `fetchEntities` gives: 26.63 the count first, 70.26 the pointer first.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpanOrder {
    CountFirst,
    PointerFirst,
}

// A stand-in actor for `fetchEntities`: a vtable word, a pad word, then the pointer to its box at 10h, as the stand-in
// player of `ActorGame` (`Actor.aabbShapeComponent` 10h); then its `EntityContext` at 20h (`Actor.mEntityContext`):
// the entity registry, the entt registry at 28h and the entity id at 30h.
#[repr(C)]
struct StandInActor {
    _vtable: usize,
    _pad: usize,
    aabb: *const [f32; 6],
    _rotation: usize,
    _entity_registry: usize,
    entt: *const c_void,
    entity: u32,
}

// A stand-in `_Ref_count_base` for the weak entity reference of a `HitResult`: vtable, uses, weaks (at 0Ch).
#[repr(C)]
struct WeakControl {
    vtable: &'static FakeVtable<2>,
    uses: AtomicI32,
    weaks: AtomicI32,
}

struct Scene {
    candidates: Vec<Candidate>,
    weak: bool,
    order: SpanOrder,
    actors: Vec<usize>,
    kept: Vec<(Box<StandInActor>, Box<[f32; 6]>)>,
    // The span words given as they are, when set.
    raw_span: Option<[usize; 2]>,
}

// SAFETY: the scene holds addresses and boxes only the test thread and the stand-in calls (on the same thread) touch.
unsafe impl Send for Scene {}

static SCENE: Mutex<Option<Scene>> = Mutex::new(None);
static CLIP_CALLS: Mutex<Vec<ClipCall>> = Mutex::new(Vec::new());
static FETCH_CALLS: Mutex<Vec<FetchCall>> = Mutex::new(Vec::new());
static ASKED_BLOCKS: Mutex<Vec<(usize, usize)>> = Mutex::new(Vec::new());
static DELETED: AtomicUsize = AtomicUsize::new(0);
// Slot 1 is `_Delete_this`.
static WEAK_VTABLE: FakeVtable<2> = FakeVtable([RayGame::delete_this as *const (), RayGame::delete_this as *const ()]);
static WEAK: WeakControl = WeakControl { vtable: &WEAK_VTABLE, uses: AtomicI32::new(0), weaks: AtomicI32::new(2) };

/// Stand-ins for `BlockSource::clip` (slot 7 of the stand-in block source vtable) and `BlockSource::fetchEntities`
/// (slot 8). The clip writes a `HitResult` at the offsets of the game (kind 18h, facing 1Ch, block 20h, point 2Ch, the
/// weak entity control block 40h); the fetch writes a span of actor pointers in the word order of a build.
pub struct RayGame;

impl RayGame {
    pub const CLIP_SLOT: u64 = 7;
    pub const FETCH_SLOT: u64 = 8;
    /// `ActorTypeComponent.typeId` of 26.63 and 70.26.
    pub const ACTOR_TYPE_ID: u32 = 0x4F6B_A419;

    /// The block source entries with `BlockSource.fetchEntitiesSpanOrder` at `order`, and `ActorTypeComponent.typeId`;
    /// `without` names entries to leave out. Resets the scene.
    pub fn entries_in_order(without: &[&str], order: u64) -> Vec<(&'static str, Resolved)> {
        Self::reset();
        [
            ("BlockSource.clip", EntryKind::VSlot, Self::CLIP_SLOT),
            ("BlockSource.fetchEntities", EntryKind::VSlot, Self::FETCH_SLOT),
            ("BlockSource.fetchEntitiesSpanOrder", EntryKind::Field, order),
            ("ActorTypeComponent.typeId", EntryKind::Field, u64::from(Self::ACTOR_TYPE_ID)),
        ]
        .into_iter()
        .filter(|(name, _, _)| !without.contains(name))
        .map(|(name, kind, value)| (name, Resolved { kind, value, recipes_agreeing: 2 }))
        .collect()
    }

    /// An empty scene: no candidate block, no weak reference, no actor, the count-first span.
    pub fn reset() {
        *SCENE.lock().unwrap() = Some(Scene {
            candidates: Vec::new(),
            weak: false,
            order: SpanOrder::CountFirst,
            actors: Vec::new(),
            kept: Vec::new(),
            raw_span: None,
        });
        CLIP_CALLS.lock().unwrap().clear();
        FETCH_CALLS.lock().unwrap().clear();
        ASKED_BLOCKS.lock().unwrap().clear();
        DELETED.store(0, Ordering::SeqCst);
        WEAK.weaks.store(2, Ordering::SeqCst);
    }

    /// The blocks the clip offers to the `std::function`, in order.
    pub fn set_candidates(candidates: &[Candidate]) {
        SCENE.lock().unwrap().as_mut().unwrap().candidates = candidates.to_vec();
    }

    /// Whether the clip leaves a weak entity reference (two weak counts) in the result.
    pub fn set_weak(weak: bool) {
        SCENE.lock().unwrap().as_mut().unwrap().weak = weak;
    }

    /// The weak count of the stand-in reference, and how many times its block was deleted.
    pub fn weak_state() -> (i32, usize) {
        (WEAK.weaks.load(Ordering::SeqCst), DELETED.load(Ordering::SeqCst))
    }

    /// The word order of the span the fetch gives.
    pub fn set_order(order: SpanOrder) {
        SCENE.lock().unwrap().as_mut().unwrap().order = order;
    }

    /// The fetch gives these two words as they are.
    pub fn set_raw_span(words: [usize; 2]) {
        SCENE.lock().unwrap().as_mut().unwrap().raw_span = Some(words);
    }

    /// Adds one stand-in actor with the box `aabb` to the actors the fetch gives; gives its address.
    pub fn add_actor(aabb: [f32; 6]) -> usize {
        Self::add_entity_actor(aabb, std::ptr::null(), 0)
    }

    /// `add_actor` for an actor whose entity context names the entt registry `entt` (null: none) and the entity id
    /// `entity`.
    pub fn add_entity_actor(aabb: [f32; 6], entt: *const c_void, entity: u32) -> usize {
        let aabb = Box::new(aabb);
        let actor = Box::new(StandInActor {
            _vtable: 0,
            _pad: 0,
            aabb: &raw const *aabb,
            _rotation: 0,
            _entity_registry: 0,
            entt,
            entity,
        });
        let address = std::ptr::from_ref(&*actor).addr();
        let mut scene = SCENE.lock().unwrap();
        let scene = scene.as_mut().unwrap();
        scene.actors.push(address);
        scene.kept.push((actor, aabb));
        address
    }

    /// Adds an actor that is already somewhere (the stand-in player) to the actors the fetch gives.
    pub fn add_existing_actor(address: usize) {
        SCENE.lock().unwrap().as_mut().unwrap().actors.push(address);
    }

    pub fn clip_calls() -> Vec<ClipCall> {
        CLIP_CALLS.lock().unwrap().clone()
    }

    pub fn fetch_calls() -> Vec<FetchCall> {
        FETCH_CALLS.lock().unwrap().clone()
    }

    /// The (block source, block) pairs the `std::function` was asked about, and what it said.
    pub fn asked_blocks() -> Vec<(usize, usize)> {
        ASKED_BLOCKS.lock().unwrap().clone()
    }

    /// `HitResult BlockSource::clip(Vec3 const&, Vec3 const&, bool, ShapeType, int, bool, bool, Actor*,
    /// std::function<bool(BlockSource const&, Block const&, bool)> const&, bool) const`.
    #[allow(clippy::too_many_arguments)]
    pub extern "C" fn clip(
        source: *mut c_void,
        out: *mut u8,
        from: *const [f32; 3],
        to: *const [f32; 3],
        liquid: u8,
        shape: u8,
        steps: i32,
        border: u8,
        full_only: u8,
        actor: *mut c_void,
        function: *const c_void,
        stop_on_liquid: u8,
    ) -> *mut u8 {
        // SAFETY: the adapter passes live points, a writable buffer of at least 88h bytes and a live `std::function`.
        unsafe {
            let callable = function.cast::<u8>().add(0x38).cast::<usize>().read();
            CLIP_CALLS.lock().unwrap().push(ClipCall {
                source: source.addr(),
                from: from.read(),
                to: to.read(),
                liquid,
                shape,
                steps,
                border,
                full_only,
                actor: actor.addr(),
                has_callable: callable != 0,
                stop_on_liquid,
            });
            let (candidates, weak) = {
                let scene = SCENE.lock().unwrap();
                let scene = scene.as_ref().unwrap();
                (scene.candidates.clone(), scene.weak)
            };
            std::ptr::write_bytes(out, 0, 0x88);
            let hit = candidates.into_iter().find(|candidate| {
                // `_Do_call` is slot 2 of the vtable of the callable object; every argument by reference.
                let vtable = (callable as *const *const usize).read();
                let do_call: extern "C" fn(usize, *const c_void, *const c_void, *const bool) -> bool =
                    std::mem::transmute(vtable.add(2).read());
                let flag = false;
                let accepted = do_call(callable, source, std::ptr::with_exposed_provenance(candidate.block), &flag);
                ASKED_BLOCKS.lock().unwrap().push((source.addr(), candidate.block));
                accepted
            });
            match hit {
                Some(hit) => {
                    out.add(0x18).cast::<i32>().write_unaligned(0);
                    out.add(0x1C).write(hit.face);
                    out.add(0x20).cast::<[i32; 3]>().write_unaligned(hit.pos);
                    out.add(0x2C).cast::<[f32; 3]>().write_unaligned(hit.point);
                }
                None => {
                    out.add(0x18).cast::<i32>().write_unaligned(3);
                    out.add(0x2C).cast::<[f32; 3]>().write_unaligned(to.read());
                }
            }
            if weak {
                out.add(0x40).cast::<usize>().write_unaligned(std::ptr::from_ref(&WEAK).addr());
            }
        }
        out
    }

    /// `span<not_null<Actor*>> BlockSource::fetchEntities(Actor const*, AABB const&, bool, bool)`.
    pub extern "C" fn fetch(
        source: *mut c_void,
        out: *mut [usize; 2],
        except: *const c_void,
        aabb: *const [f32; 6],
        first: u8,
        second: u8,
    ) -> *mut [usize; 2] {
        let scene = SCENE.lock().unwrap();
        let scene = scene.as_ref().unwrap();
        // SAFETY: the adapter passes a live box and a writable span of two words.
        unsafe {
            FETCH_CALLS.lock().unwrap().push((source.addr(), except.addr(), aabb.read(), first, second));
            let data = if scene.actors.is_empty() { 0 } else { scene.actors.as_ptr().addr() };
            let words = scene.raw_span.unwrap_or(match scene.order {
                SpanOrder::CountFirst => [scene.actors.len(), data],
                SpanOrder::PointerFirst => [data, scene.actors.len()],
            });
            out.write(words);
        }
        out
    }

    /// Slot 1 of the stand-in weak control block: `_Delete_this`.
    extern "C" fn delete_this(_this: *mut c_void) {
        DELETED.fetch_add(1, Ordering::SeqCst);
    }
}
