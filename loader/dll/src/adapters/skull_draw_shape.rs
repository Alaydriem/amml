use std::ffi::c_void;

use amml_abi::AmmlStatus;
use amml_data::AdapterCatalog;
use amml_msvc::MemberCall;
use amml_runtime::{SkullTransform, WorldTransform};

use crate::{InventoryAdapter, LoaderState, MatrixStackRefLayout, MeshCalls, NameReader, ObjectRead};

/// `void SkullBlockRenderer::renderSkull(BaseActorRenderContext&, RenderMetadata const&, Vec3 const& pos, int face,
/// float rotation, HashedString const& name, MaterialPtr const* material, ClientTexture const* texture, int breaking,
/// float animation, bool glint)`, 26.63 (survey section 7.6, row K8): rcx the renderer, rdx the context, r8 the
/// metadata, r9 the position, then the face (int), the rotation in degrees (float), the name, the material and texture
/// pointers (null: the default path), the breaking amount (not read), the animation (float) and the glint byte.
type RenderSkullFn =
    unsafe extern "C" fn(usize, usize, *const u64, *const [f32; 3], i32, f32, usize, usize, usize, i32, f32, u8);
/// `void SkullBlockRenderer::renderSkullUpright(BaseActorRenderContext&, RenderMetadata const&, Vec3 const& pos,
/// HashedString const& name, float animation, bool glint)`, 70.x: rcx the renderer, rdx the context, r8 the metadata, r9
/// the position, then the name, the animation (float) and the glint byte.
type RenderSkullUprightFn = unsafe extern "C" fn(usize, usize, *const u64, *const [f32; 3], usize, f32, u8);
/// `MatrixStack::MatrixStackRef::~MatrixStackRef()`: pops the pushed matrix.
type RefDtorFn = unsafe extern "C" fn(*mut MatrixStackRefLayout);
/// `bool BlockActor::isType(BlockActorType) const`: rcx the actor, dl the type; the bool in al.
type IsTypeFn = unsafe extern "C" fn(usize, u8) -> u8;

/// How a skull ghost is drawn on the running build (area `render.skull_pose`), with the dispatcher's skull renderer
/// (`BlockActorRenderDispatcher.mSkullRenderer`) and the full name of the ghost's block type.
///
/// Exe readings (26.63 / 70.26):
/// - The skull renderer's `render` (0x6a90c50 / 0x8939da0) asks `isType` (`mov rax,[rax+80h]`, slot 10h) with
///   `mov dl,6` and returns when it is false (0x6a90cc3 / 0x8939e31), so 6 is the skull type.
/// - 26.63 `renderSkull` (0x6a5ea50, 8 pushes and F8h, entry rsp = rbp + B8h): `mov esi,[rbp+0E0h]; dec esi; cmp
///   esi,3` (the face, 0x6a5ef72), `movss xmm2,[rbp+0E8h]` (the rotation, 0x6a5eff0, kept for face 1 and turned by
///   `mulss` at 0x6a5f0ff), `mov rbx,[rbp+0F0h]; mov rdi,[rbx]` (the name's hash, 0x6a5ea90), `cmp qword ptr
///   [rbp+0F8h],0` and `mov r15,[rbp+100h]; test r15,r15` (material and texture: either null takes the default path,
///   0x6a5f55a, 0x6a5f554), `movss xmm6,[rbp+110h]` (the animation, 0x6a5ea88), `movzx esi,byte ptr [rbp+118h]` (the
///   glint, 0x6a5ea81); [rbp+108h] (breaking) is not read; r9 is read as a Vec3 (`addss xmm1,[rax]`, 0x6a5f085); r8 goes
///   to the draw (`mov r8,[rbp+18h]`, 0x6a5f79d). The game's own `render` passes the face, the rotation for face 1 and 0
///   for any other face, and the name `BlockType+E0h` (row K8).
/// - 70.26 `renderSkullUpright` (0x893abe0, 8 pushes and A8h, entry rsp = rbp + 68h): `mov r14,r9` read as a Vec3
///   (0x893ac74), `mov r13,[rbp+90h]` (the name, passed to `getSkullModel`, 0x893ad2a), `movss xmm2,[rbp+98h]` (the
///   animation, 0x893ad22), `movzx r12d,byte ptr [rbp+0A0h]` (the glint, 0x893ad1a), `mov [rbp-38h],r8` then `mov
///   r8,[rbp-38h]` into the draw (0x893ac06, 0x893ae13). It takes the matrix stack as `[[rdx+28h]+18h]+40h`
///   (0x893ac10..0x893ac18): the `ScreenContext` of the context (survey row E6), its camera (`ScreenContext.camera`),
///   the camera's world matrix stack. The game's item path calls it with the name `BlockType+E8h`, animation 0
///   (`mov dword ptr [rsp+28h],0`) and the item's glint (0x6bc73c5..0x6bc73eb).
/// - The metadata is a `RenderMetadata` taken by reference (the item path passes the one it got); the loader passes
///   zero bytes: hash 0, flag 0 and variant index 0, the empty alternative the game's own `render` copies with no work
///   (case 0 of its switch on the variant index).
pub enum SkullDrawShape {
    /// 26.63: `renderSkull` with the face and the turn.
    RenderSkull { function: RenderSkullFn, renderer_at: usize, name_at: usize, is_type_slot: usize },
    /// 70.x: `renderSkullUpright` inside a pushed face and turn transform (`SkullTransform`).
    Upright {
        function: RenderSkullUprightFn,
        push: usize,
        pop: RefDtorFn,
        camera_at: usize,
        renderer_at: usize,
        name_at: usize,
        is_type_slot: usize,
    },
}

impl SkullDrawShape {
    /// `BlockActorType::Skull`.
    pub const SKULL_TYPE: u8 = 6;
    /// The offset of the `ScreenContext&` in `BaseActorRenderContext` (survey section 7.5, row E6: `mov rax,[rdx+28h]`
    /// at 70.26 0x893ac10, and every skull path of both builds).
    pub const SCREEN_CONTEXT_AT: usize = 0x28;
    // `MatrixStack::push(bool)`: the flag the game's own pushes and the mesh draw pass.
    const PUSH_FLAG: u8 = 1;

    /// The shape of the running build for the `render.skull_pose` shape `shape`, or the first entry that is missing or
    /// does not hold the value the loader's layouts were checked with.
    pub fn new(state: &LoaderState, shape: &str) -> Result<Self, &'static str> {
        let value = |name: &'static str| state.resolve(name).ok().ok_or(name);
        let offset = |name: &'static str| value(name).and_then(|value| usize::try_from(value).map_err(|_| name));
        let address =
            |name: &'static str| offset(name).and_then(|value| if value == 0 { Err(name) } else { Ok(value) });
        NameReader::new(value("HashedString.layout")?).ok_or("HashedString.layout")?;
        let renderer_at = offset("BlockActorRenderDispatcher.mSkullRenderer")?;
        let name_at = offset("BlockType.mNameInfo.mFullName")?;
        let is_type_slot = offset("BlockActor.isType")?;
        if shape == AdapterCatalog::SKULL_RENDER_SHAPE {
            let function = address("SkullBlockRenderer.renderSkull")?;
            // SAFETY: the derived `renderSkull`, whose signature is `RenderSkullFn` (row K8 and the readings above).
            let function = unsafe { std::mem::transmute::<usize, RenderSkullFn>(function) };
            return Ok(Self::RenderSkull { function, renderer_at, name_at, is_type_slot });
        }
        if value("Camera.mForward")? != MeshCalls::CAMERA_FORWARD {
            return Err("Camera.mForward");
        }
        let function = address("SkullBlockRenderer.renderSkullUpright")?;
        let pop = address("MatrixStackRef.dtor")?;
        // SAFETY: the derived `renderSkullUpright` and `~MatrixStackRef`, whose signatures are these types.
        let (function, pop) = unsafe {
            (std::mem::transmute::<usize, RenderSkullUprightFn>(function), std::mem::transmute::<usize, RefDtorFn>(pop))
        };
        Ok(Self::Upright {
            function,
            push: address("MatrixStack.push")?,
            pop,
            camera_at: offset("ScreenContext.camera")?,
            renderer_at,
            name_at,
            is_type_slot,
        })
    }

    /// Whether the game's own `isType(Skull)` says the actor at `actor` is a skull block actor.
    ///
    /// # Safety
    /// `actor` is a live block actor the loader holds a use of; game thread.
    pub unsafe fn is_skull(&self, actor: usize) -> bool {
        let slot = match self {
            Self::RenderSkull { is_type_slot, .. } | Self::Upright { is_type_slot, .. } => *is_type_slot,
        };
        let Some(function) = InventoryAdapter::virtual_fn(actor, slot) else { return false };
        // SAFETY: the slot of `BlockActor.isType` in the vtable of a live actor holds `isType`.
        let is_type = unsafe { std::mem::transmute::<usize, IsTypeFn>(function) };
        // SAFETY: called on its own live actor with a type byte; it reads the actor only.
        unsafe { is_type(actor, Self::SKULL_TYPE) != 0 }
    }

    /// Draws a skull of the block type at `block_type` with the pose `(face, turn)` (turn in sixteenths) at `render_pos`
    /// (the position relative to the camera target), with the skull renderer of `dispatcher`, in the pass context
    /// `context`. `FAILED` when the dispatcher has no skull renderer, or (70.x) the context has no screen or camera or the
    /// push gave no matrix; nothing stays pushed.
    ///
    /// # Safety
    /// `context` is the `BaseActorRenderContext` of the running block entity pass and `dispatcher` its dispatcher, on its
    /// thread; `block_type` a live block type; `metadata` covers a `RenderMetadata` of zero bytes for the call.
    pub unsafe fn draw(
        &self,
        context: usize,
        dispatcher: usize,
        block_type: usize,
        (face, turn): (u8, u8),
        render_pos: [f32; 3],
        metadata: *const u64,
    ) -> Result<(), AmmlStatus> {
        match self {
            Self::RenderSkull { function, renderer_at, name_at, .. } => {
                // SAFETY: the live dispatcher of the pass; the skull renderer is a pointer member.
                let renderer = unsafe { ObjectRead::pointer(dispatcher, *renderer_at) }.ok_or(AmmlStatus::FAILED)?;
                let name = block_type.checked_add(*name_at).ok_or(AmmlStatus::FAILED)?;
                // SAFETY: the renderer and context of the running pass, a metadata, position and name that live for the
                // call, no material or texture (the default path), no breaking, no animation, no glint.
                unsafe {
                    function(
                        renderer,
                        context,
                        metadata,
                        &render_pos,
                        i32::from(face),
                        SkullTransform::degrees(face, turn),
                        name,
                        0,
                        0,
                        0,
                        0.0,
                        0,
                    )
                };
                Ok(())
            }
            Self::Upright { function, push, pop, camera_at, renderer_at, name_at, .. } => {
                // SAFETY: as above.
                let renderer = unsafe { ObjectRead::pointer(dispatcher, *renderer_at) }.ok_or(AmmlStatus::FAILED)?;
                let name = block_type.checked_add(*name_at).ok_or(AmmlStatus::FAILED)?;
                // SAFETY: the live context of the pass; its screen context is a pointer member.
                let screen =
                    unsafe { ObjectRead::pointer(context, Self::SCREEN_CONTEXT_AT) }.ok_or(AmmlStatus::FAILED)?;
                // SAFETY: the live screen context of the pass; its camera is a pointer member.
                let camera = unsafe { ObjectRead::pointer(screen, *camera_at) }.ok_or(AmmlStatus::FAILED)?;
                let stack = camera.wrapping_add(MeshCalls::CAMERA_WORLD_STACK);
                // SAFETY: the entry is `MatrixStack::push(bool)`, a member that returns its 16-byte ref through the
                // hidden pointer; `stack` is the live camera's world matrix stack (`Camera.mForward` checked).
                let mut pushed: MatrixStackRefLayout = unsafe {
                    MemberCall::call_sret1(
                        *push as *const c_void,
                        std::ptr::with_exposed_provenance_mut(stack),
                        Self::PUSH_FLAG,
                    )
                };
                let result = if pushed.matrix.is_null() {
                    Err(AmmlStatus::FAILED)
                } else {
                    let transform = SkullTransform::matrix(face, turn, render_pos);
                    // SAFETY: `push` gave the new top matrix of the stack, a `glm::mat4` stored by columns; the stack is
                    // live and its dirty flag tells the game the top changed.
                    unsafe {
                        *pushed.matrix = WorldTransform::compose(&*pushed.matrix, &transform);
                        std::ptr::with_exposed_provenance_mut::<u8>(stack + MeshCalls::MATRIX_STACK_DIRTY).write(1);
                    }
                    // SAFETY: the renderer and context of the running pass, a metadata, position and name that live for
                    // the call, no animation and no glint.
                    unsafe { function(renderer, context, metadata, &render_pos, name, 0.0, 0) };
                    Ok(())
                };
                // SAFETY: the entry is `~MatrixStackRef`; the ref is the one `push` returned, popped here once.
                unsafe { pop(&mut pushed) };
                result
            }
        }
    }
}
