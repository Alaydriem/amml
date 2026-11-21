use std::cell::{Cell, RefCell};

use amml_abi::{AmmlColorQuad, AmmlMeshStyle, AmmlStatus, AmmlTransform};
use amml_runtime::{MeshBook, MeshKey, MeshRefusal};

use crate::testing::FakeRenderRun;
use crate::testing::fake_canvas::FakeCanvas;
use crate::{MeshId, Vec3};

/// The render side of a `FakeGame`: the dispatch that runs now, and the meshes of the fake mod. A canvas call answers
/// only inside the dispatch of its frame id, as the loader does; a mesh id of an older generation gives `Stale`.
#[derive(Default)]
pub(crate) struct FakeRender {
    next_frame: Cell<u64>,
    running: RefCell<Option<FakeRenderRun>>,
    meshes: MeshBook,
    next_handle: Cell<usize>,
    vibrant: Cell<bool>,
}

impl FakeRender {
    // The owner of the one fake mod.
    const OWNER: usize = 1;
    // The most quads one mesh holds.
    const MAX_QUADS: usize = 8191;

    /// A dispatch starts; gives its frame id.
    pub(crate) fn enter(&self, canvas: &FakeCanvas) -> u64 {
        let frame = self.next_frame.get() + 1;
        self.next_frame.set(frame);
        *self.running.borrow_mut() = Some(FakeRenderRun {
            frame,
            camera_position: canvas.camera_position,
            freed: Vec::new(),
            calls: Vec::new(),
        });
        frame
    }

    /// The running dispatch ends; gives the meshes freed in it and the `build` and `draw` lines it accepted.
    pub(crate) fn leave(&self) -> (Vec<MeshId>, Vec<String>) {
        self.running.borrow_mut().take().map(|run| (run.freed, run.calls)).unwrap_or_default()
    }

    // Runs `body` on the dispatch of `frame`; `REFUSED` outside it.
    fn in_frame<R>(
        &self,
        frame: u64,
        body: impl FnOnce(&mut FakeRenderRun) -> Result<R, AmmlStatus>,
    ) -> Result<R, AmmlStatus> {
        let mut running = self.running.borrow_mut();
        let run = running.as_mut().filter(|run| run.frame == frame).ok_or(AmmlStatus::REFUSED)?;
        body(run)
    }

    /// A `build` of the loader in the dispatch of `frame`, with its checks: the overlay style only, 1 to 8191 quads.
    pub(crate) fn build(
        &self,
        frame: u64,
        quads: &[AmmlColorQuad],
        style: AmmlMeshStyle,
    ) -> Result<MeshId, AmmlStatus> {
        self.in_frame(frame, |run| {
            let overlay = style.blend == AmmlMeshStyle::BLEND_ALPHA && style.depth_test == 1 && style.depth_write == 0;
            if !overlay || style.light != AmmlMeshStyle::LIGHT_FULL || style.texture != AmmlMeshStyle::TEXTURE_WHITE {
                return Err(AmmlStatus::NOT_AVAILABLE);
            }
            if quads.is_empty() {
                return Err(AmmlStatus::INVALID_ARGUMENT);
            }
            if quads.len() > Self::MAX_QUADS {
                return Err(AmmlStatus::REFUSED);
            }
            run.calls.push(format!(
                "build quads={} style={}/{}/{}/{}/{}/{}",
                quads.len(),
                if style.blend == AmmlMeshStyle::BLEND_ALPHA { "alpha" } else { "opaque" },
                style.depth_test,
                style.depth_write,
                if style.two_sided != 0 { 2 } else { 1 },
                if style.light == AmmlMeshStyle::LIGHT_FULL { "full" } else { "world" },
                if style.texture == AmmlMeshStyle::TEXTURE_WHITE { "white" } else { "atlas" },
            ));
            Ok(())
        })?;
        Ok(self.add_mesh())
    }

    /// A `draw` of the loader in the dispatch of `frame`.
    pub(crate) fn draw(&self, frame: u64, mesh: MeshId, transform: &AmmlTransform) -> Result<(), AmmlStatus> {
        let key = MeshKey { id: mesh.id, generation: mesh.generation };
        self.meshes.handle(Self::OWNER, key).map_err(Self::refusal)?;
        self.in_frame(frame, |run| {
            let at = |origin: i32, offset: f32| f64::from(origin) + f64::from(offset);
            run.calls.push(format!(
                "draw {} {} {} {} yaw={}",
                mesh.id,
                at(transform.origin.x, transform.offset.x),
                at(transform.origin.y, transform.offset.y),
                at(transform.origin.z, transform.offset.z),
                transform.yaw_degrees,
            ));
            Ok(())
        })
    }

    /// A graphics mode switch: every mesh is freed, and a later use of one gives `Stale`.
    pub(crate) fn switch_graphics(&self) {
        let vibrant = self.vibrant.get();
        self.meshes.note_vibrant(vibrant);
        self.meshes.note_vibrant(!vibrant);
        self.vibrant.set(!vibrant);
    }

    fn refusal(refusal: MeshRefusal) -> AmmlStatus {
        match refusal {
            MeshRefusal::Stale => AmmlStatus::STALE,
            MeshRefusal::NotOwner => AmmlStatus::REFUSED,
        }
    }

    pub(crate) fn camera_position(&self, frame: u64) -> Result<Vec3, AmmlStatus> {
        let running = self.running.borrow();
        let run = running.as_ref().filter(|run| run.frame == frame).ok_or(AmmlStatus::REFUSED)?;
        run.camera_position.ok_or(AmmlStatus::FAILED)
    }

    /// Makes a mesh of the fake mod, as the loader would on a `build`.
    pub(crate) fn add_mesh(&self) -> MeshId {
        let handle = self.next_handle.get() + 1;
        self.next_handle.set(handle);
        let key = self.meshes.create(Self::OWNER, handle);
        MeshId { id: key.id, generation: key.generation }
    }

    pub(crate) fn free(&self, mesh: MeshId) -> Result<(), AmmlStatus> {
        let key = MeshKey { id: mesh.id, generation: mesh.generation };
        self.meshes.free(Self::OWNER, key).map_err(Self::refusal)?;
        if let Some(run) = self.running.borrow_mut().as_mut() {
            run.freed.push(mesh);
        }
        Ok(())
    }

    /// The world is left: every mesh is stale.
    pub(crate) fn world_left(&self) {
        self.meshes.world_left();
    }

    /// The meshes the book holds.
    pub(crate) fn live_meshes(&self) -> usize {
        self.meshes.len()
    }
}
