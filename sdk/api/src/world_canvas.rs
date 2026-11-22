use std::marker::PhantomData;

use amml_abi::{AmmlColorQuad, AmmlMeshId, AmmlRenderTable, AmmlStr, AmmlTransform, AmmlVec3};

use crate::{ApiError, ColorQuad, MeshId, MeshStyle, Transform, Vec3};

/// The canvas of one `WorldRendered` dispatch: the calls that belong to the frame the game just drew. It is built only
/// from the event of a running listener call (`WorldRenderedExt::canvas`) and borrows it, so it cannot outlive the
/// call. The loader checks the frame id on every call and refuses a call outside the dispatch.
pub struct WorldCanvas<'f> {
    table: AmmlRenderTable,
    frame: u64,
    module: &'static str,
    _event: PhantomData<&'f ()>,
}

impl<'f> WorldCanvas<'f> {
    /// The most quads one mesh holds: two windings of 4 vertices each stay under the 16-bit index limit.
    pub const MAX_QUADS: usize = 8191;

    pub(crate) fn new(table: AmmlRenderTable, frame: u64, module: &'static str) -> Self {
        Self { table, frame, module, _event: PhantomData }
    }

    /// The camera position of the world view of this frame, in world coordinates. `None` when no view of the frame
    /// belongs to the game's level renderer, or the loader refuses the call.
    #[allow(unsafe_code)]
    pub fn camera_position(&self) -> Option<Vec3> {
        let mut out = AmmlVec3::default();
        // SAFETY: the table function is the loader's; `out` is writable for the call.
        let status = unsafe { (self.table.canvas_camera_position)(self.frame, &raw mut out) };
        status.is_ok().then(|| Vec3::from(out))
    }

    /// The clip matrix of the camera of this frame: the top of its projection stack times the top of its view stack,
    /// relative to the camera, by columns (`matrix[column][row]`, as glm stores it). A world point `p` (camera relative,
    /// with `w` = 1) is at `sum(matrix[column][row] * p[column])` in clip space. `None` when the frame has no camera or a
    /// matrix stack is empty, from a loader without `Area::RENDER_WORLD_CANVAS.at(3)`, or when the loader refuses the
    /// call.
    #[allow(unsafe_code)]
    pub fn clip_matrix(&self) -> Option<[[f32; 4]; 4]> {
        let clip = self.table.canvas_clip_matrix?;
        let mut out = [[0.0f32; 4]; 4];
        // SAFETY: the table function is the loader's; `out` is writable for the call.
        let status = unsafe { clip(self.frame, &raw mut out) };
        status.is_ok().then_some(out)
    }

    /// Frees the mesh `mesh` of this mod. The id is `Stale` at once; the game object is destroyed on the render thread
    /// at the next level pass. `Err(Stale)` for an id of an older generation or one that was freed, `Err(Refused)` for
    /// a mesh of another mod.
    #[allow(unsafe_code)]
    pub fn free(&self, mesh: MeshId) -> Result<(), ApiError> {
        // SAFETY: the table function is the loader's; the module id lives for the call.
        let status = unsafe { (self.table.mesh_free)(AmmlStr::from_str(self.module), mesh.into()) };
        ApiError::check(status)
    }

    /// Builds a mesh of `quads` with `style` in this frame and gives its id; the loader keeps the mesh until it is freed
    /// (`free`, a world leave, a render device reset, a graphics mode switch, the disable of the mod). `Err(Refused)`
    /// above `MAX_QUADS`; `Err(NotAvailable)` for `MeshLight::World`, `MeshTexture::TerrainAtlas` (`MeshStyle::GHOST`: only
    /// `BlockMesher` makes such a mesh) or any blend and depth other than `MeshStyle::OVERLAY`, and from a loader without `Area::RENDER_WORLD_CANVAS.at(2)`;
    /// `Err(Loader(INVALID_ARGUMENT))` for no quads or a corner that is not finite; `Err(Loader(FAILED))` when the
    /// game made no mesh.
    #[allow(unsafe_code)]
    pub fn build(&self, quads: &[ColorQuad], style: MeshStyle) -> Result<MeshId, ApiError> {
        let build = self.table.mesh_build.ok_or(ApiError::NotAvailable)?;
        if quads.len() > Self::MAX_QUADS {
            return Err(ApiError::Refused);
        }
        let quads: Vec<AmmlColorQuad> = quads.iter().copied().map(Into::into).collect();
        let count = u32::try_from(quads.len()).map_err(|_| ApiError::Refused)?;
        let mut out = AmmlMeshId::default();
        // SAFETY: the table function is the loader's; the module id and the quads live for the call; `out` is writable.
        let status = unsafe {
            build(AmmlStr::from_str(self.module), self.frame, quads.as_ptr(), count, style.into(), &raw mut out)
        };
        ApiError::check(status).map(|()| MeshId::from(out))
    }

    /// Draws the mesh `mesh` of this mod in this frame with `transform`. `Err(Stale)` for a freed mesh or one of an
    /// older generation (build it again), `Err(Refused)` for a mesh of another mod, `Err(NotReady)` while the loader's
    /// white texture is not loaded yet (the first frames of a world), `Err(Loader(FAILED))` when the frame has no camera
    /// view or material, `Err(NotAvailable)` from a loader without `Area::RENDER_WORLD_CANVAS.at(2)`.
    #[allow(unsafe_code)]
    pub fn draw(&self, mesh: MeshId, transform: Transform) -> Result<(), ApiError> {
        let draw = self.table.mesh_draw.ok_or(ApiError::NotAvailable)?;
        let transform = AmmlTransform::from(transform);
        // SAFETY: the table function is the loader's; the module id and the transform live for the call.
        let status = unsafe { draw(AmmlStr::from_str(self.module), self.frame, mesh.into(), &raw const transform) };
        ApiError::check(status)
    }
}
