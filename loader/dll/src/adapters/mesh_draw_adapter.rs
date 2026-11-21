use std::ffi::c_void;

use amml_abi::AmmlStatus;
use amml_msvc::MemberCall;
use amml_runtime::WorldTransform;

use crate::{
    MatrixStackRefLayout, MeshCalls, MeshContextLayout, MeshLayout, ShaderColorLayout, TextureListLayout,
    TexturePtrLayout,
};

/// `void mce::Mesh::renderMesh(MeshContext&, MaterialPtr const&, TextureList const&, ...)`, the list overload (entry
/// notes of `Mesh.renderMeshTextureList`): the mesh, the `MeshContext`, the material, the texture list (by value: the
/// callee destroys every element at its exit); two unread stack words; the `OffscreenCaptureDescription` (by
/// reference); an unread word; an optional value (by value, destroyed by the callee when engaged; the engaged byte is
/// at 38h on 26.63 and at 30h on 70.26).
type RenderFn = unsafe extern "C" fn(
    *const MeshLayout,
    *mut u8,
    *const u8,
    *mut TextureListLayout,
    u32,
    u32,
    *const [u64; 8],
    *const u8,
    *const [u64; 8],
);
/// `MatrixStack::MatrixStackRef::~MatrixStackRef()`: pops the pushed matrix.
type RefDtorFn = unsafe extern "C" fn(*mut MatrixStackRefLayout);
/// `static void ActorShaderManager::setEntityConstants(ActorConstants&, RenderContext&, Color const& tileLight,
/// Vec2 const& tileLightUV, glm::vec4 const& blockLight, Color const& overlay, Color const& change,
/// Color const& change2, Color const& glint, Vec2 const& glintUVScale, Vec4 const& uvAnim, float, float, float,
/// float)` (survey section 8.2: 15 arguments on 26.63 and 70.26).
type SetEntityConstantsFn = unsafe extern "C" fn(
    *mut u8,
    *mut u8,
    *const [f32; 4],
    *const [f32; 2],
    *const [f32; 4],
    *const [f32; 4],
    *const [f32; 4],
    *const [f32; 4],
    *const [f32; 4],
    *const [f32; 2],
    *const [f32; 4],
    f32,
    f32,
    f32,
    f32,
);

/// Draws one world mesh in the screen context of the running `renderLevel` call. Render thread only.
///
/// A draw writes the fixed actor constants (white tile and block light, no overlay, no glint), pushes a copy of the
/// camera's world matrix, multiplies the mesh matrix into it and marks the stack dirty, sets the shader colour to
/// opaque white, draws the whole mesh with `Mesh.renderMeshTextureList`, then puts the shader colour back and pops the
/// matrix.
pub struct MeshDrawAdapter {
    calls: MeshCalls,
}

impl MeshDrawAdapter {
    // `MatrixStack::push(bool)`: the flag the working levels mod passes.
    const PUSH_FLAG: u8 = 1;

    pub fn new(calls: MeshCalls) -> Self {
        Self { calls }
    }

    /// Draws the mesh at `mesh` with the `MaterialPtr` at `material`, the texture `texture` and the world matrix
    /// `matrix` (relative to the camera, stored by columns). `texture` is a copy that holds its own uses: it goes to the
    /// game as element 0 of the texture list, which the game destroys, or, when nothing is drawn, it is released here.
    /// `FAILED` when the context has no camera or the push gave no matrix; nothing stays pushed.
    ///
    /// # Safety
    /// `screen` is the screen context of the running `renderLevel` call, on its thread; `mesh` a live mesh of
    /// `MeshBuilder`; `material` a live `MaterialPtr` whose material is set; each pointer of `texture` is empty or holds
    /// one use of its own.
    pub unsafe fn draw(
        &self,
        screen: usize,
        mesh: usize,
        material: usize,
        mut texture: TexturePtrLayout,
        matrix: &[[f32; 4]; 4],
    ) -> Result<(), AmmlStatus> {
        let context = screen.checked_add(self.calls.screen_mesh_context).ok_or(AmmlStatus::FAILED)?;
        // SAFETY: the caller guarantees a live screen context; its `MeshContext` base is laid out as the mirror (the
        // members were checked against their entries).
        let fields = unsafe { &*std::ptr::with_exposed_provenance::<MeshContextLayout>(context) };
        if fields.camera.is_null() {
            // SAFETY: the caller guarantees the copy holds its own uses; nothing else holds them.
            unsafe { texture.release() };
            return Err(AmmlStatus::FAILED);
        }
        // SAFETY: as above.
        unsafe { self.set_entity_constants(fields) };
        let stack = fields.camera.wrapping_add(MeshCalls::CAMERA_WORLD_STACK);
        // SAFETY: the entry is `MatrixStack::push(bool)`, a member that returns its 16-byte ref through the hidden
        // pointer; `stack` is the live camera's world matrix stack (`Camera.mForward` checked).
        let mut pushed: MatrixStackRefLayout = unsafe {
            MemberCall::call_sret1(self.calls.matrix_stack_push as *const c_void, stack.cast(), Self::PUSH_FLAG)
        };
        let result = if pushed.matrix.is_null() {
            // SAFETY: the caller guarantees the copy holds its own uses; nothing else holds them.
            unsafe { texture.release() };
            Err(AmmlStatus::FAILED)
        } else {
            // SAFETY: `push` gave the new top matrix of the stack, a `glm::mat4` stored by columns.
            unsafe { *pushed.matrix = WorldTransform::compose(&*pushed.matrix, matrix) };
            // SAFETY: the stack is live; its dirty flag tells the game the top changed.
            unsafe { stack.wrapping_add(MeshCalls::MATRIX_STACK_DIRTY).write(1) };
            let render = || {
                // SAFETY: passed on from the caller.
                unsafe { self.render(context, mesh, material, texture) }
            };
            // SAFETY: passed on from the caller.
            unsafe { self.with_white_shader_color(fields, render) };
            Ok(())
        };
        // SAFETY: the entry is `~MatrixStackRef`; the ref is the one `push` returned, popped here once.
        unsafe { std::mem::transmute::<usize, RefDtorFn>(self.calls.matrix_stack_ref_dtor)(&mut pushed) };
        result
    }

    // Safety: as `draw`.
    unsafe fn render(&self, context: usize, mesh: usize, material: usize, texture: TexturePtrLayout) {
        let mut list = TextureListLayout::one(texture);
        // An empty `OffscreenCaptureDescription` (index 0, monostate, whatever offset up to 38h the index is read at) and
        // an empty optional (the engaged byte, 38h on 26.63 and 30h on 70.26, is 0): 40h zero bytes each.
        let capture = [0u64; 8];
        let optional = [0u64; 8];
        // SAFETY: the entry is `Mesh.renderMeshTextureList` with the signature of `RenderFn`.
        let render = unsafe { std::mem::transmute::<usize, RenderFn>(self.calls.mesh_render) };
        // SAFETY: a live mesh, the frame's `MeshContext`, a live material, and a list, capture and optional that live
        // for the call.
        unsafe {
            render(
                std::ptr::with_exposed_provenance(mesh),
                std::ptr::with_exposed_provenance_mut(context),
                std::ptr::with_exposed_provenance(material),
                &raw mut list,
                0,
                0,
                &capture,
                std::ptr::null(),
                &optional,
            );
        }
    }

    // Runs `draw` with the current shader colour set to opaque white, then puts the colour back; both writes mark it
    // dirty. A null colour pointer runs `draw` with the colour as it is.
    //
    // Safety: `fields` is the live `MeshContext` of the running render call, on its thread.
    unsafe fn with_white_shader_color(&self, fields: &MeshContextLayout, draw: impl FnOnce()) {
        let color = fields.current_shader_color.cast::<ShaderColorLayout>();
        if color.is_null() {
            draw();
            return;
        }
        // SAFETY: the `ShaderColor&` of the context is the frame's live shared colour, owned by this thread during
        // the call.
        let saved = unsafe { (*color).color };
        // SAFETY: as above.
        unsafe {
            (*color).color = [1.0; 4];
            (*color).dirty = true;
        }
        draw();
        // SAFETY: as above.
        unsafe {
            (*color).color = saved;
            (*color).dirty = true;
        }
    }

    // Writes the fixed actor constants of a world mesh draw: white tile light and block light, no overlay, no change
    // colours, no glint, UV animation offset 0 and scale 1. Without them a draw takes the last actor's light.
    //
    // Safety: `fields` is the live `MeshContext` of the running render call, on its thread.
    unsafe fn set_entity_constants(&self, fields: &MeshContextLayout) {
        if fields.constant_buffers.is_null() || fields.render_context.is_null() {
            return;
        }
        let entity = fields.constant_buffers.wrapping_add(self.calls.entity_constant_buffer);
        let white = [1.0f32; 4];
        let none = [0.0f32; 4];
        let one = [1.0f32; 2];
        let uv_anim = [0.0f32, 0.0, 1.0, 1.0];
        // SAFETY: the entry is the static `ActorShaderManager::setEntityConstants`.
        let set = unsafe { std::mem::transmute::<usize, SetEntityConstantsFn>(self.calls.set_entity_constants) };
        // SAFETY: the frame's live entity constant buffer and render context, and values that live for the call.
        unsafe {
            set(
                entity,
                fields.render_context,
                &white,
                &one,
                &white,
                &none,
                &none,
                &none,
                &none,
                &one,
                &uv_anim,
                0.0,
                0.0,
                0.0,
                0.0,
            );
        }
    }
}
