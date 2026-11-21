use std::ffi::c_void;
use std::hint::black_box;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

use amml_data::{EntryKind, Resolved};

use super::{Harness, SessionGame};

// The calls the stand-ins saw, one line each.
static CALLS: Mutex<Vec<String>> = Mutex::new(Vec::new());
// The stand-in `mce::Camera`, `LevelRendererCamera` and `ViewRenderObject` of the next calls.
static CAMERA: AtomicUsize = AtomicUsize::new(0);
static LEVEL_CAMERA: AtomicUsize = AtomicUsize::new(0);
static VIEW_OBJECT: AtomicUsize = AtomicUsize::new(0);
// The view and projection matrices the camera stacks hold.
static VIEW_TOP: AtomicUsize = AtomicUsize::new(0);
static PROJECTION_TOP: AtomicUsize = AtomicUsize::new(0);

/// Stand-ins with the game signatures for `LevelRendererPlayer::setupCamera`, `createViewRenderObject`,
/// `LevelRendererCamera::preRenderUpdate` and `mce::Camera::updateViewMatrixDependencies`, and the leaked objects they
/// are given: a camera laid out as the gate builds have it (view stack at 0, projection stack at 80h, inverse view at
/// C0h, right 100h, up 10Ch, forward 118h, each stack with one matrix), a level camera with its position at
/// `LEVEL_POS` and target at `LEVEL_TARGET`, and a view object whose view data starts with the position and target.
pub struct CameraGame;

impl CameraGame {
    pub const SETUP_CAMERA: &'static str = "LevelRendererPlayer.setupCamera";
    /// The position the game writes into the view object and the level camera.
    pub const GAME_POSITION: [f32; 3] = [100.0, 65.0, -40.0];
    const LEVEL_POS: usize = 0x64C;
    const LEVEL_TARGET: usize = 0x658;
    const CAMERA_WORDS: usize = 0x130 / 8;

    /// The session entries and every entry of the `camera.setup`, `camera.pose` and `camera.projection` rows, with the
    /// 26.63 offsets; `without` names entries to leave out, `with` replaces values.
    pub fn entries(without: &[&str], with: &[(&'static str, u64)]) -> Vec<(&'static str, Resolved)> {
        let field = |value| Resolved { kind: EntryKind::Field, value, recipes_agreeing: 2 };
        let function = |address: usize| Resolved {
            kind: EntryKind::Function,
            value: address as u64 - Harness::BASE,
            recipes_agreeing: 2,
        };
        let mut entries = SessionGame::entries(&[]);
        entries.extend([
            (Self::SETUP_CAMERA, function(Self::setup_camera as *const () as usize)),
            ("LevelRendererPlayer.createViewRenderObject", function(Self::create_view as *const () as usize)),
            ("LevelRendererCamera.preRenderUpdate", function(Self::pre_render_update as *const () as usize)),
            ("Camera.updateViewMatrixDependencies", function(Self::update_view as *const () as usize)),
            ("Camera.mForward", field(0x118)),
            ("LevelRendererCamera.mCameraPos", field(Self::LEVEL_POS as u64)),
            ("LevelRendererCamera.mCameraTargetPos", field(Self::LEVEL_TARGET as u64)),
            ("ViewRenderObject.mViewData", field(0)),
            ("ViewRenderData.mCameraPos", field(0)),
        ]);
        for (name, value) in with {
            if let Some(entry) = entries.iter_mut().find(|(id, _)| id == name) {
                entry.1.value = *value;
            }
        }
        entries.into_iter().filter(|(id, _)| !without.contains(id)).collect()
    }

    /// New objects for the next calls: an identity view matrix and `projection` on the camera stacks, a zero level
    /// camera and a zero view object.
    pub fn reset(projection: [[f32; 4]; 4]) {
        let view_top: &'static mut [[f32; 4]; 4] = Box::leak(Box::new([[0.0; 4]; 4]));
        let projection_top: &'static mut [[f32; 4]; 4] = Box::leak(Box::new(projection));
        let view_map: &'static mut [usize; 1] = Box::leak(Box::new([std::ptr::from_mut(view_top) as usize]));
        let projection_map: &'static mut [usize; 1] =
            Box::leak(Box::new([std::ptr::from_mut(projection_top) as usize]));
        let camera: &'static mut [usize; Self::CAMERA_WORDS] = Box::leak(Box::new([0; Self::CAMERA_WORDS]));
        // Each deque: proxy, map, map size, offset, size.
        for (stack, map) in [(0usize, view_map.as_ptr() as usize), (0x80 / 8, projection_map.as_ptr() as usize)] {
            camera[stack + 1] = map;
            camera[stack + 2] = 1;
            camera[stack + 3] = 0;
            camera[stack + 4] = 1;
        }
        VIEW_TOP.store(std::ptr::from_mut(view_top) as usize, Ordering::SeqCst);
        PROJECTION_TOP.store(std::ptr::from_mut(projection_top) as usize, Ordering::SeqCst);
        CAMERA.store(camera.as_mut_ptr() as usize, Ordering::SeqCst);
        let level: &'static mut [u8; 0x700] = Box::leak(Box::new([0; 0x700]));
        LEVEL_CAMERA.store(level.as_mut_ptr() as usize, Ordering::SeqCst);
        let view: &'static mut [u8; 0x40] = Box::leak(Box::new([0; 0x40]));
        VIEW_OBJECT.store(view.as_mut_ptr() as usize, Ordering::SeqCst);
        CALLS.lock().unwrap().clear();
    }

    /// Calls (through its address) `setupCamera(player, camera, alpha)`.
    pub fn setup(alpha: f32) {
        let function = black_box(Self::setup_camera as extern "C" fn(*mut c_void, *mut c_void, f32));
        function(
            black_box(std::ptr::with_exposed_provenance_mut(0x77_0000)),
            black_box(std::ptr::with_exposed_provenance_mut(CAMERA.load(Ordering::SeqCst))),
            black_box(alpha),
        );
    }

    /// Calls (through its address) `createViewRenderObject(player, out, screen, 1)` and gives what it returned.
    pub fn create() -> usize {
        let function = black_box(Self::create_view as extern "C" fn(*mut c_void, usize, usize, usize) -> usize);
        function(black_box(std::ptr::with_exposed_provenance_mut(0x77_0000)), Self::view_object(), 0x88_0000, 1)
    }

    /// Calls (through its address) `preRenderUpdate(level_camera, screen, params)`.
    pub fn pre_render() {
        let function = black_box(Self::pre_render_update as extern "C" fn(*mut c_void, usize, usize));
        function(
            black_box(std::ptr::with_exposed_provenance_mut(LEVEL_CAMERA.load(Ordering::SeqCst))),
            0x88_0000,
            0x99,
        );
    }

    pub fn view_object() -> usize {
        VIEW_OBJECT.load(Ordering::SeqCst)
    }

    /// The position and target of the view object.
    pub fn view() -> [[f32; 3]; 2] {
        Self::pair(Self::view_object(), 0, 0x0C)
    }

    /// The position and target of the level camera.
    pub fn level() -> [[f32; 3]; 2] {
        Self::pair(LEVEL_CAMERA.load(Ordering::SeqCst), Self::LEVEL_POS, Self::LEVEL_TARGET)
    }

    /// The camera's forward vector.
    pub fn forward() -> [f32; 3] {
        Self::vector(CAMERA.load(Ordering::SeqCst) + 0x118)
    }

    /// The camera's view matrix (the top of its view stack).
    pub fn view_top() -> [[f32; 4]; 4] {
        // SAFETY: the matrix leaked by `reset`.
        unsafe { std::ptr::with_exposed_provenance::<[[f32; 4]; 4]>(VIEW_TOP.load(Ordering::SeqCst)).read() }
    }

    /// The view stack dirty flag.
    pub fn view_dirty() -> bool {
        // SAFETY: the camera leaked by `reset`, 130h bytes.
        unsafe { std::ptr::with_exposed_provenance::<u8>(CAMERA.load(Ordering::SeqCst) + 0x38).read() != 0 }
    }

    /// The stand-in calls so far.
    pub fn log() -> Vec<String> {
        CALLS.lock().unwrap().clone()
    }

    fn vector(address: usize) -> [f32; 3] {
        // SAFETY: an address inside an object leaked by `reset`.
        unsafe { std::ptr::with_exposed_provenance::<[f32; 3]>(address).read_unaligned() }
    }

    fn pair(base: usize, first: usize, second: usize) -> [[f32; 3]; 2] {
        [Self::vector(base + first), Self::vector(base + second)]
    }

    fn write_pair(base: usize, first: usize, second: usize, value: [f32; 3]) {
        for at in [first, second] {
            // SAFETY: an address inside an object leaked by `reset`.
            unsafe { std::ptr::with_exposed_provenance_mut::<[f32; 3]>(base + at).write_unaligned(value) };
        }
    }

    #[inline(never)]
    extern "C" fn setup_camera(this: *mut c_void, camera: *mut c_void, alpha: f32) {
        black_box((this, camera));
        CALLS.lock().unwrap().push(format!("setup alpha={}", black_box(alpha)));
    }

    #[inline(never)]
    extern "C" fn create_view(this: *mut c_void, out: usize, screen: usize, sub_client: usize) -> usize {
        black_box((this, screen));
        Self::write_pair(out, 0, 0x0C, Self::GAME_POSITION);
        CALLS.lock().unwrap().push(format!("create sub={}", black_box(sub_client) as u8));
        out
    }

    #[inline(never)]
    extern "C" fn pre_render_update(this: *mut c_void, screen: usize, params: usize) {
        black_box((screen, params));
        Self::write_pair(this as usize, Self::LEVEL_POS, Self::LEVEL_TARGET, Self::GAME_POSITION);
        CALLS.lock().unwrap().push("pre_render".to_string());
    }

    #[inline(never)]
    extern "C" fn update_view(camera: *mut c_void) {
        black_box(camera);
        CALLS.lock().unwrap().push("update".to_string());
    }
}
