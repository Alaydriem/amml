use std::ffi::c_void;

use amml_abi::{AmmlBlockHit, AmmlBlockPos, AmmlFace, AmmlRayOptions, AmmlStatus, AmmlVec3};
use amml_msvc::{MsvcFunction, RefCountBase};

use crate::{HitResultLayout, LoaderState};

/// `HitResult BlockSource::clip(Vec3 const& from, Vec3 const& to, bool checkLiquid, ShapeType, int maxSteps,
/// bool ignoreBorderBlocks, bool fullBlocksOnly, Actor*, std::function<bool(BlockSource const&, Block const&, bool)>
/// const& shouldCheckBlock, bool stopOnFirstLiquid) const`: rcx the block source, the result through rdx, the two points
/// in r8 and r9, the eight other arguments on the stack.
type ClipFn = unsafe extern "C" fn(
    *mut c_void,
    *mut HitResultLayout,
    *const [f32; 3],
    *const [f32; 3],
    u8,
    u8,
    i32,
    u8,
    u8,
    *mut c_void,
    *const c_void,
    u8,
) -> *mut HitResultLayout;

/// The game side of the `world.raycast` area: `BlockSource::clip` through its vtable slot, with a `std::function` that
/// calls a Rust closure (`MsvcFunction`) to pick the candidate blocks.
///
/// Read from the exe on 26.63 and 70.26 (`clip` 0x32da050 / 0x47ab6f0): the arguments are the same on both; the shape
/// type 1 clips against `BlockType::getCollisionShape` (slot 4, the derived entry) and 0 against the outline; the
/// `std::function` is used only through the callable at +38h, whose slot 2 (`_Do_call`) gets the block source, the block
/// and a bool by reference, and it is never copied or kept; the walk visits at most the steps given (a caller of the
/// game passes 200). The calls are for the game thread.
pub struct RaycastAdapter {
    clip_slot: usize,
}

impl RaycastAdapter {
    /// The steps a ray with `max_steps` 0 walks: what the game's own callers of `clip` pass (26.63 0x32db750, 70.26
    /// 0x47acfb0).
    pub const DEFAULT_STEPS: i32 = 200;
    /// `ShapeType::Outline`.
    const SHAPE_OUTLINE: u8 = 0;
    /// `ShapeType::Collision`.
    const SHAPE_COLLISION: u8 = 1;
    // The result buffer in qwords: four times the 88h of the `HitResult`, so a build whose result grew does not write
    // past it.
    const BUFFER_WORDS: usize = 0x44;

    /// The adapter for the `BlockSource.clip` slot of the build; `None` when it did not resolve.
    pub fn new(state: &LoaderState) -> Option<Self> {
        let clip_slot = state.resolve("BlockSource.clip").ok().and_then(|value| usize::try_from(value).ok())?;
        Some(Self { clip_slot })
    }

    /// The steps for `max_steps`: the game's own count for 0, at most the largest `int`.
    pub fn steps(max_steps: u32) -> i32 {
        if max_steps == 0 { Self::DEFAULT_STEPS } else { i32::try_from(max_steps).unwrap_or(i32::MAX) }
    }

    /// The first block the ray from `from` to `to` meets in `source`, among the blocks `candidate` accepts (it gets the
    /// address of each `Block` the walk reaches). A miss is `hit` 0. `FAILED` when the block source has no function in
    /// the slot.
    ///
    /// The `std::function` lives on this frame until `clip` returns; the game only calls it, on this thread. The weak
    /// entity reference of the result is given back as the game's own callers do.
    ///
    /// # Safety
    /// `source` is the live block source of the local player, called on the game thread; `candidate` reads only live
    /// game memory.
    pub unsafe fn clip(
        &self,
        source: usize,
        from: AmmlVec3,
        to: AmmlVec3,
        options: AmmlRayOptions,
        mut candidate: impl FnMut(usize) -> bool,
    ) -> Result<AmmlBlockHit, AmmlStatus> {
        // SAFETY: `source` is a live block source (the caller's contract).
        let function = unsafe { Self::virtual_fn(source, self.clip_slot) }.ok_or(AmmlStatus::FAILED)?;
        // SAFETY: the slot holds `BlockSource::clip` with the ten arguments above.
        let clip = unsafe { std::mem::transmute::<usize, ClipFn>(function) };
        let should_check = MsvcFunction::new(move |_: *const c_void, block: *const c_void, _: bool| {
            candidate(block.expose_provenance())
        });
        let from = [from.x, from.y, from.z];
        let to = [to.x, to.y, to.z];
        let shape = if options.outline != 0 { Self::SHAPE_OUTLINE } else { Self::SHAPE_COLLISION };
        let mut buffer = [0u64; Self::BUFFER_WORDS];
        let result = buffer.as_mut_ptr().cast::<HitResultLayout>();
        // SAFETY: the block source is live; the buffer is larger than a `HitResult` and 8-aligned; the points and the
        // `std::function` live across the call; no liquid, no border or full-block rule, no actor.
        unsafe {
            clip(
                std::ptr::with_exposed_provenance_mut(source),
                result,
                &raw const from,
                &raw const to,
                0,
                shape,
                Self::steps(options.max_steps),
                0,
                0,
                std::ptr::null_mut(),
                should_check.as_ptr(),
                0,
            )
        };
        drop(should_check);
        // SAFETY: `clip` wrote a whole `HitResult` at the start of the buffer.
        let hit = unsafe { result.read() };
        if hit.weak_control != 0 {
            // SAFETY: the result holds one weak count of this control block, which the game's destructor gives back.
            unsafe { RefCountBase::decwref(std::ptr::with_exposed_provenance_mut(hit.weak_control as usize)) };
        }
        if hit.kind != HitResultLayout::TILE {
            return Ok(AmmlBlockHit::default());
        }
        Ok(AmmlBlockHit {
            hit: 1,
            face: AmmlFace(hit.facing),
            _reserved: [0; 2],
            pos: AmmlBlockPos { x: hit.block[0], y: hit.block[1], z: hit.block[2] },
            point: AmmlVec3 { x: hit.pos[0], y: hit.pos[1], z: hit.pos[2] },
        })
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
