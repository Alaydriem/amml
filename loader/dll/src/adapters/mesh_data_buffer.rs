use amml_msvc::{MsvcHeap, MsvcVec};

use crate::{GameVector, LoaderState, MeshDataLayout};

/// `void mce::MeshData::clear()`: rcx this, which it leaves with the mode, the bounds and the field flags reset (survey
/// section 7, row M9).
type ClearFn = unsafe extern "C" fn(*mut MeshDataLayout);
/// `mce::MeshData::~MeshData()`: rcx this; frees the vectors from +110h down (row M9).
type DtorFn = unsafe extern "C" fn(*mut MeshDataLayout);
/// `mce::MeshData& mce::MeshData::operator=(mce::MeshData&&)`: rcx this, rdx the source (row M9).
type MoveAssignFn = unsafe extern "C" fn(*mut MeshDataLayout, *mut MeshDataLayout) -> *mut MeshDataLayout;

/// The game `MeshData` buffers of the mesh groups: each is a box of the loader holding a `mce::MeshData` the loader built
/// as the game's constructor does (zeroed bytes, then the game's `clear`; 26.60 and later have no constructor function),
/// whose vectors grow on the game heap, so the game's destructor and move assignment work on it. The address of the box
/// is the handle the `GroupBook` keeps. A buffer is used on the game thread only.
pub struct MeshDataBuffer {
    clear: ClearFn,
    dtor: DtorFn,
    move_assign: MoveAssignFn,
}

impl MeshDataBuffer {
    // `mce::PrimitiveMode::TriangleList`.
    const TRIANGLE_LIST: u8 = 2;
    const QUAD: usize = 4;
    const INDICES_PER_QUAD: usize = 6;

    /// The buffer calls of this build from the resolved entries; `Err` names the first entry that is missing (a function
    /// entry of value 0 counts as missing).
    pub fn new(state: &LoaderState) -> Result<Self, &'static str> {
        let address = |name: &'static str| {
            state
                .resolve(name)
                .ok()
                .and_then(|value| usize::try_from(value).ok())
                .filter(|address| *address != 0)
                .ok_or(name)
        };
        // SAFETY: each address is the derived body of the named function, whose signature is the type it becomes
        // (argument registers read on both gate builds, survey section 7, row M9).
        unsafe {
            Ok(Self {
                clear: std::mem::transmute::<usize, ClearFn>(address("MeshData.clear")?),
                dtor: std::mem::transmute::<usize, DtorFn>(address("MeshData.dtor")?),
                move_assign: std::mem::transmute::<usize, MoveAssignFn>(address("MeshData.moveAssign")?),
            })
        }
    }

    /// A new empty buffer; gives its handle.
    pub fn make(&self) -> usize {
        // SAFETY: zeroed bytes are a valid `MeshDataLayout`: null vectors, zero floats and false flags.
        let data = unsafe { Box::<MeshDataLayout>::new_zeroed().assume_init() };
        let pointer = Box::into_raw(data);
        // SAFETY: the entry is `MeshData::clear`; the box is MeshData-sized and 8-aligned, cleared once.
        unsafe { (self.clear)(pointer) };
        pointer.expose_provenance()
    }

    /// The layout of the buffer at `handle`.
    ///
    /// # Safety
    /// `handle` is a handle `make` gave, not destroyed or finished since; no other reference to the buffer lives; game
    /// thread.
    pub unsafe fn layout<'a>(handle: usize) -> &'a mut MeshDataLayout {
        // SAFETY: the caller's contract.
        unsafe { &mut *std::ptr::with_exposed_provenance_mut::<MeshDataLayout>(handle) }
    }

    /// Destroys the buffer at `handle` with the game's destructor and frees its box.
    ///
    /// # Safety
    /// `handle` is a handle `make` gave, destroyed once; game thread.
    pub unsafe fn destroy(&self, handle: usize) {
        let pointer = std::ptr::with_exposed_provenance_mut::<MeshDataLayout>(handle);
        if pointer.is_null() {
            return;
        }
        // SAFETY: the entry is the destructor; the box holds a constructed buffer, destroyed here once, then freed.
        unsafe {
            (self.dtor)(pointer);
            drop(Box::from_raw(pointer));
        }
    }

    /// Moves every vector of the buffer at `handle` into `target` with the game's move assignment; the buffer is left
    /// empty (and is still to be destroyed).
    ///
    /// # Safety
    /// `handle` is a live buffer; `target` is a live `mce::MeshData` of the game; game thread.
    pub unsafe fn move_into(&self, handle: usize, target: *mut MeshDataLayout) {
        // SAFETY: the caller's contract: the entry is the move assignment, with a live target and source.
        unsafe { (self.move_assign)(target, std::ptr::with_exposed_provenance_mut(handle)) };
    }

    /// Adds every vertex field of `from` to the buffer at `handle`, with the bounds. False, with the buffer as it was,
    /// when `from` has indices of its own or a field that does not cover every vertex of the buffer and `from` together;
    /// false also when the game heap gives no block, and the caller then drops the group.
    ///
    /// # Safety
    /// `handle` is a live buffer whose vectors came from `heap`; `heap` is the game's allocator; game thread.
    pub unsafe fn add(heap: &MsvcHeap, handle: usize, from: &MeshDataLayout) -> bool {
        // SAFETY: the caller's contract.
        let to = unsafe { Self::layout(handle) };
        let total = to.positions.len() + from.positions.len();
        if !from.indices.is_empty() || !Self::fields_cover(to, from, total) {
            return false;
        }
        if !from.positions.is_empty() {
            let unfilled = to.positions.is_empty();
            for axis in 0..3 {
                to.aabb[0][axis] = if unfilled { from.aabb[0][axis] } else { to.aabb[0][axis].min(from.aabb[0][axis]) };
                to.aabb[1][axis] = if unfilled { from.aabb[1][axis] } else { to.aabb[1][axis].max(from.aabb[1][axis]) };
            }
            for axis in 0..2 {
                to.uv_aabb[0][axis] =
                    if unfilled { from.uv_aabb[0][axis] } else { to.uv_aabb[0][axis].min(from.uv_aabb[0][axis]) };
                to.uv_aabb[1][axis] =
                    if unfilled { from.uv_aabb[1][axis] } else { to.uv_aabb[1][axis].max(from.uv_aabb[1][axis]) };
            }
        }
        // SAFETY: the buffer's vectors were made by this loader's heap and grow only here, on the game heap the game's
        // destructor frees them to.
        let grown = unsafe {
            Self::extend(heap, &mut to.positions, &from.positions)
                && Self::extend(heap, &mut to.normals, &from.normals)
                && Self::extend(heap, &mut to.tangents, &from.tangents)
                && Self::extend(heap, &mut to.colors, &from.colors)
                && Self::extend(heap, &mut to.bone_id0s, &from.bone_id0s)
                && to.texture_uvs.iter_mut().zip(&from.texture_uvs).all(|(to, from)| Self::extend(heap, to, from))
                && Self::extend(heap, &mut to.pbr_texture_indices, &from.pbr_texture_indices)
                && Self::extend(heap, &mut to.mers, &from.mers)
                && Self::extend(heap, &mut to.geo_type, &from.geo_type)
        };
        if !grown {
            return false;
        }
        for (enabled, more) in to.field_enabled.iter_mut().zip(from.field_enabled) {
            *enabled = *enabled || more;
        }
        true
    }

    /// Turns the quads of the buffer at `handle` into a triangle list with its own 32-bit index buffer, six indices a
    /// quad, each quad as two triangles that keep its vertex order so they face the way the quad-list meshes face. Gives
    /// the number of indices; false when the vertices are not whole quads or the game heap gives no block.
    ///
    /// # Safety
    /// As `add`.
    pub unsafe fn triangulate(heap: &MsvcHeap, handle: usize) -> Option<usize> {
        // SAFETY: the caller's contract.
        let data = unsafe { Self::layout(handle) };
        let vertices = data.positions.len();
        if !vertices.is_multiple_of(Self::QUAD) || !data.indices.is_empty() {
            return None;
        }
        let quads = vertices / Self::QUAD;
        // SAFETY: the buffer's vectors are on the game heap; the index vector is empty.
        unsafe {
            GameVector::extend_with(heap, &mut data.indices, quads * Self::INDICES_PER_QUAD, |index| {
                let first = (index / Self::INDICES_PER_QUAD * Self::QUAD) as u32;
                first + [0, 1, 2, 0, 2, 3][index % Self::INDICES_PER_QUAD]
            })
        }
        .ok()?;
        data.mode = Self::TRIANGLE_LIST;
        Some(data.indices.len())
    }

    // Appends `from` to `to`; true for no elements.
    //
    // Safety: as `add`.
    unsafe fn extend<T: Copy>(heap: &MsvcHeap, to: &mut MsvcVec<T>, from: &MsvcVec<T>) -> bool {
        let items = from.as_slice();
        // SAFETY: passed on from the caller.
        unsafe { GameVector::extend_with(heap, to, items.len(), |index| items[index]) }.is_ok()
    }

    // A field some parts fill and others do not would give later vertices another vertex's value.
    fn fields_cover(to: &MeshDataLayout, from: &MeshDataLayout, total: usize) -> bool {
        Self::covers(&to.normals, &from.normals, total)
            && Self::covers(&to.tangents, &from.tangents, total)
            && Self::covers(&to.colors, &from.colors, total)
            && Self::covers(&to.bone_id0s, &from.bone_id0s, total)
            && Self::covers(&to.pbr_texture_indices, &from.pbr_texture_indices, total)
            && Self::covers(&to.mers, &from.mers, total)
            && Self::covers(&to.geo_type, &from.geo_type, total)
            && to.texture_uvs.iter().zip(&from.texture_uvs).all(|(to, from)| Self::covers(to, from, total))
    }

    // The buffer and the new vertices together hold the field for none or for all of `total` vertices.
    fn covers<T>(had: &MsvcVec<T>, more: &MsvcVec<T>, total: usize) -> bool {
        let held = had.len() + more.len();
        held == 0 || held == total
    }
}
