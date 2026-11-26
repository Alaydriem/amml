use std::ffi::c_void;
use std::hint::black_box;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

use amml_data::{EntryKind, Resolved};

use super::{Harness, MeshGame, SessionGame};

// The `renderLevel` calls so far, as `render this=..` lines.
static CALLS: Mutex<Vec<String>> = Mutex::new(Vec::new());
// The address of the stand-in `FrameRenderObject` the next `render_level` call passes.
static FRAME_OBJECT: AtomicUsize = AtomicUsize::new(0);
// The address of the stand-in `LevelRenderer` the next `render_level` call passes.
static RENDERER: AtomicUsize = AtomicUsize::new(0);

/// A stand-in with the game signature for `LevelRenderer::renderLevel`, and the leaked objects it is given: a
/// `LevelRenderer` whose `LevelRendererPlayer` pointer is at `PLAYER_AT`, the player's camera position at `CAMERA_AT`,
/// and a `FrameRenderObject` with `mViews` at 0x10 (begin pointer, then the end pointer) and views of `VIEW_SIZE`
/// bytes whose `mViewData.mCameraPos` is at offset 0.
pub struct RenderGame;

impl RenderGame {
    pub const RENDER_LEVEL: &'static str = "LevelRenderer.renderLevel";
    pub const MVIEWS_AT: u64 = 0x10;
    const VIEW_SIZE: u64 = 0x20;
    const PLAYER_AT: u64 = 0x20;
    const CAMERA_AT: u64 = 0x40;
    const WIREFRAME_AT: usize = 0x2F0;
    const ATLAS_AT: usize = 0x1D0;

    /// The session entries, the options entries of the `render.world_rendered` row and the entries of the
    /// `render.world_canvas` row (the mirror anchors with the values of the gate builds, the functions pointing to the
    /// stand-ins of `MeshGame`); `without` names entries to leave out.
    pub fn entries(without: &[&str]) -> Vec<(&'static str, Resolved)> {
        let slot = |value| Resolved { kind: EntryKind::VSlot, value, recipes_agreeing: 2 };
        let field = |value| Resolved { kind: EntryKind::Field, value, recipes_agreeing: 2 };
        let function = |address: usize| Resolved {
            kind: EntryKind::Function,
            value: address as u64 - Harness::BASE,
            recipes_agreeing: 2,
        };
        let mut entries = SessionGame::entries(&[]);
        entries.extend([
            (Self::RENDER_LEVEL, function(Self::render_level as *const () as usize)),
            ("ClientInstance.getOptions", slot(5)),
            ("IOptionRegistry.isGraphicsModeDeferred", slot(9)),
            ("FrameRenderObject.mViews", field(Self::MVIEWS_AT)),
            ("ViewRenderObject.mViewData", field(0)),
            ("ViewRenderData.mCameraPos", field(0)),
            ("ViewRenderObject.size", field(Self::VIEW_SIZE)),
            ("LevelRenderer.mLevelRendererPlayer", field(Self::PLAYER_AT)),
            ("LevelRendererCamera.mCameraPos", field(Self::CAMERA_AT)),
            ("ScreenContext.tessellator", field(0xB0)),
            ("ScreenContext.meshContext", field(0x10)),
            ("ScreenContext.camera", field(0x18)),
            ("ScreenContext.constantBuffers", field(0x20)),
            ("ScreenContext.currentShaderColor", field(0x30)),
            ("Tessellator.mTessellating", field(0x2A0)),
            ("Mesh.size", field(0x260)),
            ("Camera.mForward", field(0x118)),
            ("GlobalConstantBuffers.entityConstantBuffer", field(0xC0)),
            ("LevelRendererCamera.mWireframeMaterial", field(0x2F0)),
            ("RenderMaterialInfo.mMaterial", field(0x48)),
            ("RenderMaterial.mEnableBlend", field(0x185)),
            ("RenderMaterial.mBlendSource", field(0x180)),
            ("RenderMaterial.mBlendDestination", field(0x181)),
            ("RenderMaterial.mDepthWriteMask", field(0x193)),
            ("BaseActorRenderer.size", field(0xB8)),
            ("ClientInstance.getItemInHandRenderer", slot(2)),
            ("ItemInHandRenderer.mMatBlendBlock", field(MeshGame::BLEND_AT as u64)),
            ("LevelRenderer.mAtlasTexture", field(Self::ATLAS_AT as u64)),
        ]);
        entries.extend(MeshGame::entries());
        entries.into_iter().filter(|(id, _)| !without.contains(id)).collect()
    }

    /// Makes the next `render` pass a renderer whose camera is at `position` and a frame object whose one view has the
    /// same camera position; with `None` the views vector is empty.
    pub fn set_camera(position: Option<[f32; 3]>) {
        match position {
            Some(position) => Self::set_views(position, &[position]),
            None => Self::set_views([0.0; 3], &[]),
        }
    }

    /// Makes the next `render` pass a renderer whose camera is at `renderer_camera` and a frame object with one view
    /// per element of `views`, each with that camera position.
    pub fn set_views(renderer_camera: [f32; 3], views: &[[f32; 3]]) {
        let floats_per_view = Self::VIEW_SIZE as usize / size_of::<f32>();
        let storage: &'static mut [f32] =
            Box::leak(vec![0.0f32; floats_per_view * views.len().max(1)].into_boxed_slice());
        for (index, view) in views.iter().enumerate() {
            storage[index * floats_per_view..index * floats_per_view + 3].copy_from_slice(view);
        }
        let begin = storage.as_ptr() as usize;
        // The object: the allocator words, then the begin and end pointers of `mViews`.
        let object: &'static mut [usize; 4] = Box::leak(Box::new([0; 4]));
        object[2] = begin;
        object[3] = begin + views.len() * Self::VIEW_SIZE as usize;
        FRAME_OBJECT.store(std::ptr::from_mut(object) as usize, Ordering::SeqCst);
        // The player: its camera position at `CAMERA_AT`, its wireframe `MaterialPtr` at `WIREFRAME_AT`.
        let player: &'static mut [u32; 0x100] = Box::leak(Box::new([0; 0x100]));
        let camera_at = Self::CAMERA_AT as usize / size_of::<f32>();
        for (slot, value) in player[camera_at..camera_at + 3].iter_mut().zip(renderer_camera) {
            *slot = value.to_bits();
        }
        let info = MeshGame::wireframe_info();
        let wireframe_at = Self::WIREFRAME_AT / size_of::<u32>();
        player[wireframe_at] = info as u32;
        player[wireframe_at + 1] = (info >> 32) as u32;
        // The renderer: the player pointer at `PLAYER_AT`.
        let renderer: &'static mut [usize; 0x80] = Box::leak(Box::new([0; 0x80]));
        renderer[Self::PLAYER_AT as usize / size_of::<usize>()] = std::ptr::from_mut(player) as usize;
        RENDERER.store(std::ptr::from_mut(renderer) as usize, Ordering::SeqCst);
    }

    /// Gives the renderer of the next `render` its atlas `TexturePtr` (`LevelRenderer.mAtlasTexture`): the stand-in
    /// texture and location words with the control blocks `controls`, or an empty pointer for `None`. Call it after
    /// `set_camera` or `set_views`, which make the renderer.
    pub fn set_atlas(controls: Option<[usize; 2]>) {
        let words = match controls {
            Some([texture, location]) => [MeshGame::ATLAS_WORDS[0], texture, MeshGame::ATLAS_WORDS[1], location],
            None => [0; 4],
        };
        let renderer = RENDERER.load(Ordering::SeqCst);
        for (index, word) in words.into_iter().enumerate() {
            // SAFETY: the renderer is a leaked block of 400h bytes; the atlas sits inside it.
            unsafe {
                std::ptr::with_exposed_provenance_mut::<usize>(renderer + Self::ATLAS_AT + index * 8).write(word)
            };
        }
    }

    /// Calls (through its address) `renderLevel(renderer, screen, frame_object)`.
    pub fn render() {
        let function = black_box(Self::render_level as extern "C" fn(*mut c_void, usize, usize));
        function(
            black_box(std::ptr::with_exposed_provenance_mut(RENDERER.load(Ordering::SeqCst))),
            black_box(MeshGame::screen()),
            black_box(FRAME_OBJECT.load(Ordering::SeqCst)),
        );
    }

    /// The `renderLevel` calls the stand-in saw so far.
    pub fn render_calls() -> usize {
        CALLS.lock().unwrap().iter().filter(|line| line.starts_with("render")).count()
    }

    #[inline(never)]
    extern "C" fn render_level(this: *mut c_void, screen: usize, frame: usize) {
        black_box((this, screen, frame));
        CALLS.lock().unwrap().push(format!("render this={:#x}", black_box(this) as usize));
    }
}
