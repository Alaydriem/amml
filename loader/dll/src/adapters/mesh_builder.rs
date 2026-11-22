use std::mem::MaybeUninit;

use amml_abi::{AmmlColorQuad, AmmlStatus};
use amml_msvc::MsvcHeap;

use crate::{GameVector, MeshCalls, MeshLayout, ObjectRead, TessellatorLayout};

/// `void Tessellator::begin(DebugContextCallback, mce::PrimitiveMode, int reserve, bool)`: this, the callback (an MSVC
/// `std::function`, 40h bytes, by value, so passed as a pointer to the caller's copy), the mode, the vertex count to
/// reserve, then a flag passed as false (survey section 8.2). 70.26 reads the impl pointer at 38h of the callback, moves
/// it into the tessellator and destroys it; 26.63 does not read it.
type BeginFn = unsafe extern "C" fn(*mut TessellatorLayout, *mut [u64; 8], u8, i32, u8);
/// `mce::Mesh Tessellator::end(UploadMode, std::string_view, SupplementaryFieldAutoGenerationMode)`: this, the hidden
/// mesh buffer, the upload mode, a pointer to the debug name view, then the 2-byte mode.
type EndFn =
    unsafe extern "C" fn(*mut TessellatorLayout, *mut MeshLayout, i32, *const [usize; 2], u16) -> *mut MeshLayout;
/// `void Tessellator::clear()`.
type ClearFn = unsafe extern "C" fn(*mut TessellatorLayout);
/// `mce::Mesh::~Mesh()`.
type DtorFn = unsafe extern "C" fn(*mut MeshLayout);
/// `bool mce::Mesh::isValid() const`.
type IsValidFn = unsafe extern "C" fn(*const MeshLayout) -> u8;

/// Builds world meshes in the game's tessellator and destroys them. Render thread only, inside a `renderLevel` call.
///
/// A build opens a quad list, writes the positions, the packed colours and the first UV set (a point inside the
/// loader's white texture) straight into the tessellator's mesh data on the game heap, sets the vertex count and ends
/// the list into a mesh the game uploads on its first draw. One-sided, each quad goes in once with its corners in
/// order; two-sided, it goes in a second time with its corners reversed. A mesh lives in a box of the loader, so it
/// never moves once the game made it; its address is the handle the `MeshBook` keeps.
pub struct MeshBuilder {
    calls: MeshCalls,
}

impl MeshBuilder {
    // `mce::PrimitiveMode::QuadList`.
    pub const QUAD_LIST: u8 = 1;
    // `Tessellator::UploadMode::Buffered`: the mesh is uploaded on its first draw.
    const UPLOAD_BUFFERED: i32 = 0;
    // `Tessellator::UploadMode::Never`: for a mesh that is dropped at once.
    const UPLOAD_NEVER: i32 = 2;
    /// `mce::PrimitiveMode::TriangleList`: a mesh with its own index buffer.
    pub const TRIANGLE_LIST: u8 = 2;
    // `SupplementaryFieldAutoGenerationMode::None`.
    const NO_SUPPLEMENTARY_FIELDS: u16 = 0;
    const DEBUG_NAME: &'static str = "amml world mesh";
    const CORNERS: usize = 4;
    /// The UV of every vertex: inside the one pixel of the white texture.
    pub const WHITE_UV: [f32; 2] = [0.5, 0.5];

    pub fn new(calls: MeshCalls) -> Self {
        Self { calls }
    }

    /// The game's packed `mce::Color` of a vertex: red in the low byte, alpha in the high one.
    pub fn pack(quad: &AmmlColorQuad) -> u32 {
        u32::from_le_bytes([quad.rgba.r, quad.rgba.g, quad.rgba.b, quad.rgba.a])
    }

    /// The quad and the corner of vertex `index`, for `per` vertices a quad; the second four go in reverse order.
    pub fn corner(index: usize, per: usize) -> (usize, usize) {
        let step = index % per;
        let corner = if step < Self::CORNERS { step } else { per - 1 - step };
        (index / per, corner)
    }

    /// Builds a mesh of `quads` in the tessellator of the screen context at `screen` and gives its handle. `FAILED`
    /// when the context has no tessellator, a list is already open, the game opened none, a vector could not grow or
    /// the game made no valid mesh; nothing is left open in the tessellator.
    ///
    /// # Safety
    /// `screen` is the screen context the game passed to the running `renderLevel` call, on its thread; `heap` is the
    /// game's allocator.
    pub unsafe fn build(
        &self,
        screen: usize,
        quads: &[AmmlColorQuad],
        two_sided: bool,
        heap: &MsvcHeap,
    ) -> Result<usize, AmmlStatus> {
        let per = if two_sided { 2 * Self::CORNERS } else { Self::CORNERS };
        let vertices = quads.len().checked_mul(per).ok_or(AmmlStatus::REFUSED)?;
        let reserve = i32::try_from(vertices).map_err(|_| AmmlStatus::REFUSED)?;
        // SAFETY: the caller guarantees a live screen context of the running call, on its thread.
        let tessellator = unsafe { self.open(screen, reserve) }?;
        // SAFETY: the list is open; the vectors are the game's, on `heap`.
        let written = unsafe { Self::write(tessellator, quads, per, heap) };
        if !written {
            // SAFETY: the list is open.
            unsafe { self.abandon(tessellator) };
            return Err(AmmlStatus::FAILED);
        }
        // SAFETY: the list is open and holds the written vertices.
        unsafe { self.close(tessellator) }
    }

    /// Opens a quad list of `reserve` vertices in the tessellator of the screen context at `screen` and gives the
    /// tessellator, for a caller that fills its mesh data itself and then calls `close` or `abandon`. `FAILED` when the
    /// context has no tessellator, a list is already open or the game opened none.
    ///
    /// # Safety
    /// `screen` is the screen context the game passed to the running `renderLevel` call, on its thread.
    pub unsafe fn open(&self, screen: usize, reserve: i32) -> Result<*mut TessellatorLayout, AmmlStatus> {
        // SAFETY: the caller's contract.
        unsafe { self.open_mode(screen, Self::QUAD_LIST, reserve) }
    }

    /// As `open`, for the primitive mode `mode` (`QUAD_LIST` or `TRIANGLE_LIST`).
    ///
    /// # Safety
    /// As for `open`.
    pub unsafe fn open_mode(
        &self,
        screen: usize,
        mode: u8,
        reserve: i32,
    ) -> Result<*mut TessellatorLayout, AmmlStatus> {
        // SAFETY: the caller guarantees a live screen context; its `Tessellator&` is stored as a pointer.
        let tessellator = unsafe { ObjectRead::pointer(screen, self.calls.screen_tessellator) }
            .map(std::ptr::with_exposed_provenance_mut::<TessellatorLayout>)
            .ok_or(AmmlStatus::FAILED)?;
        // SAFETY: the tessellator is live and laid out as the mirror (`Tessellator.mTessellating` checked).
        if unsafe { (*tessellator).tessellating } {
            return Err(AmmlStatus::FAILED);
        }
        // SAFETY: the entry is `Tessellator::begin` with the signature of `BeginFn`.
        let begin = unsafe { std::mem::transmute::<usize, BeginFn>(self.calls.tessellator_begin) };
        // An empty `std::function`: a null impl pointer at 38h, which the callee neither calls nor destroys.
        let mut callback = [0u64; 8];
        // SAFETY: a live tessellator with no open list; the callback lives for the call.
        unsafe { begin(tessellator, &raw mut callback, mode, reserve.max(0), 0) };
        // SAFETY: as above.
        if !unsafe { (*tessellator).tessellating } {
            return Err(AmmlStatus::FAILED);
        }
        Ok(tessellator)
    }

    /// Ends the open list of `tessellator` into a mesh the game uploads on its first draw and gives its handle. `FAILED`,
    /// with the mesh destroyed, when the game made no valid mesh.
    ///
    /// # Safety
    /// A list `open` gave is open on the live `tessellator`, on the thread of the running `renderLevel` call.
    pub unsafe fn close(&self, tessellator: *mut TessellatorLayout) -> Result<usize, AmmlStatus> {
        // SAFETY: the caller guarantees the open list.
        let mesh = unsafe { self.end(tessellator, Self::UPLOAD_BUFFERED) };
        // SAFETY: the entry is `Mesh::isValid` with the signature of `IsValidFn`.
        let is_valid = unsafe { std::mem::transmute::<usize, IsValidFn>(self.calls.mesh_is_valid) };
        // SAFETY: `end` constructed a mesh in the box.
        if unsafe { is_valid(mesh) } == 0 {
            // SAFETY: the box holds a constructed mesh, destroyed here once.
            unsafe { self.destroy(mesh.expose_provenance()) };
            return Err(AmmlStatus::FAILED);
        }
        Ok(mesh.expose_provenance())
    }

    /// Drops what the open list of `tessellator` holds and ends it into a mesh that is never uploaded and destroyed at
    /// once, so nothing is left open.
    ///
    /// # Safety
    /// As for `close`.
    pub unsafe fn abandon(&self, tessellator: *mut TessellatorLayout) {
        // SAFETY: the caller guarantees the open list.
        unsafe { self.discard(tessellator) };
    }

    /// Destroys the mesh at `handle` with the game's destructor and frees its box.
    ///
    /// # Safety
    /// `handle` is a handle `build` gave, not destroyed before; render thread, while the game's render device that
    /// made the mesh is alive.
    pub unsafe fn destroy(&self, handle: usize) {
        let mesh = std::ptr::with_exposed_provenance_mut::<MeshLayout>(handle);
        if mesh.is_null() {
            return;
        }
        // SAFETY: the entry is `Mesh::~Mesh` with the signature of `DtorFn`.
        let dtor = unsafe { std::mem::transmute::<usize, DtorFn>(self.calls.mesh_dtor) };
        // SAFETY: the caller guarantees a constructed mesh that is destroyed once; the box came from `end`.
        unsafe {
            dtor(mesh);
            drop(Box::from_raw(mesh.cast::<MaybeUninit<MeshLayout>>()));
        }
    }

    // Appends the vertices of `quads` (`per` each) and sets `mCount`. False when a vector could not grow or the total
    // does not fit `mCount`.
    //
    // Safety: a quad list is open on the live `tessellator`, whose vectors came from `heap`.
    unsafe fn write(tessellator: *mut TessellatorLayout, quads: &[AmmlColorQuad], per: usize, heap: &MsvcHeap) -> bool {
        let count = quads.len() * per;
        // SAFETY: the caller guarantees a live tessellator; the borrow ends with this function.
        let data = unsafe { &mut (*tessellator).mesh_data };
        let position = |index: usize| {
            let (quad, corner) = Self::corner(index, per);
            quads.get(quad).and_then(|quad| quad.corners.get(corner)).map_or([0.0; 3], |at| [at.x, at.y, at.z])
        };
        let color = |index: usize| quads.get(index / per).map_or(0, Self::pack);
        // SAFETY: the vectors are the game's, on the game heap the caller passes.
        let grown = unsafe {
            GameVector::extend_with(heap, &mut data.positions, count, position)
                .and_then(|()| GameVector::extend_with(heap, &mut data.colors, count, color))
                .and_then(|()| GameVector::extend_with(heap, &mut data.texture_uvs[0], count, |_| Self::WHITE_UV))
        };
        if grown.is_err() {
            return false;
        }
        let Ok(total) = u32::try_from(data.positions.len()) else { return false };
        // SAFETY: as above.
        unsafe { (*tessellator).count = total };
        true
    }

    // Ends the open list into a new boxed mesh.
    //
    // Safety: a quad list is open on the live `tessellator`.
    unsafe fn end(&self, tessellator: *mut TessellatorLayout, upload: i32) -> *mut MeshLayout {
        // SAFETY: the entry is `Tessellator::end` with the signature of `EndFn`.
        let end = unsafe { std::mem::transmute::<usize, EndFn>(self.calls.tessellator_end) };
        let name = [Self::DEBUG_NAME.as_ptr().addr(), Self::DEBUG_NAME.len()];
        let mesh = Box::into_raw(Box::new(MaybeUninit::<MeshLayout>::uninit())).cast::<MeshLayout>();
        // SAFETY: the box holds a mesh-sized, 8-aligned buffer that `end` constructs the mesh in; the name lives for
        // the call.
        unsafe { end(tessellator, mesh, upload, &name, Self::NO_SUPPLEMENTARY_FIELDS) };
        mesh
    }

    // Drops what the open list holds and ends it into a mesh that is never uploaded and destroyed at once.
    //
    // Safety: a quad list is open on the live `tessellator`.
    unsafe fn discard(&self, tessellator: *mut TessellatorLayout) {
        // SAFETY: the entry is `Tessellator::clear` with the signature of `ClearFn`.
        let clear = unsafe { std::mem::transmute::<usize, ClearFn>(self.calls.tessellator_clear) };
        // SAFETY: a live tessellator.
        unsafe { clear(tessellator) };
        // SAFETY: as above. The list is ended only when it is still open after `clear`.
        if unsafe { (*tessellator).tessellating } {
            // SAFETY: the list is open.
            let mesh = unsafe { self.end(tessellator, Self::UPLOAD_NEVER) };
            // SAFETY: `end` constructed the mesh; it is destroyed once.
            unsafe { self.destroy(mesh.expose_provenance()) };
        }
    }
}
