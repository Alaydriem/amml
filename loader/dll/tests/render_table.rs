use std::ffi::c_void;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

use amml::{
    ModListener, ModLoader, RenderLevelDetours, SessionAdapter, TextureAdapter, TextureCalls, WorldRenderAdapter,
};
use amml_abi::{
    AmmlApi, AmmlBand, AmmlBlockPos, AmmlColorQuad, AmmlEventContext, AmmlEventHeader, AmmlMeshId, AmmlMeshStyle,
    AmmlRenderTable, AmmlRgba, AmmlServices, AmmlStatus, AmmlStr, AmmlTransform, AmmlVec3, WorldRendered,
};
use amml_runtime::{ModRegistry, TextureJob};

use crate::support::{CLIENT_A, EventsMod, Harness, HeapProbe, LogFile, MeshGame, RenderGame, SessionGame};

// What the `render.world_rendered` listener saw, one line per event.
static SEEN: Mutex<Vec<String>> = Mutex::new(Vec::new());
// The address of the loader's render table, for the listener.
static TABLE: AtomicUsize = AtomicUsize::new(0);

const POSITION: [f32; 3] = [12.5, 70.0, -3.25];

fn table() -> &'static AmmlRenderTable {
    // SAFETY: `TABLE` holds the address of the loader's render table, which lives for the process.
    unsafe { &*std::ptr::with_exposed_provenance::<AmmlRenderTable>(TABLE.load(Ordering::SeqCst)) }
}

fn camera(frame: u64) -> (AmmlStatus, AmmlVec3) {
    let mut out = AmmlVec3::default();
    // SAFETY: the table is live; `out` is writable for the call.
    let status = unsafe { (table().canvas_camera_position)(frame, &raw mut out) };
    (status, out)
}

unsafe extern "C" fn on_rendered(
    event: *mut AmmlEventHeader,
    context: *const AmmlEventContext,
    _: *mut c_void,
) -> AmmlStatus {
    // SAFETY: the loader raises `render.world_rendered` with a `WorldRendered` and a live context.
    let (rendered, context) = unsafe { (&*event.cast::<WorldRendered>(), &*context) };
    let frame = rendered.frame_id;
    let (own, position) = camera(frame);
    let (next, _) = camera(frame + 1);
    let (zero, _) = camera(0);
    // The frame id is the dispatch of this thread only.
    let other_thread = std::thread::spawn(move || camera(frame).0).join().unwrap();
    SEEN.lock().unwrap().push(format!(
        "frame={frame} vibrant={} thread={} own={} position={},{},{} next={} zero={} other_thread={}",
        rendered.vibrant, context.thread.0, own.0, position.x, position.y, position.z, next.0, zero.0, other_thread.0,
    ));
    AmmlStatus::OK
}

fn seen() -> Vec<String> {
    SEEN.lock().unwrap().clone()
}

// The session and world render adapters over the stand-ins, the table of the loader found, and a `WorldRendered`
// listener when `listen` is true.
fn start(listen: bool) -> Harness {
    let harness = Harness::start_with_entries(&RenderGame::entries(&[]));
    SessionAdapter::install(harness.state);
    WorldRenderAdapter::install(harness.state);
    // SAFETY: the loader's tables live for the process.
    let services = unsafe { AmmlApi::services(harness.state.api()) }.unwrap();
    // SAFETY: as above.
    let render = unsafe { AmmlServices::render(services) }.unwrap();
    TABLE.store(render.expose_provenance(), Ordering::SeqCst);
    if listen {
        let listener = ModListener { owner: 0, callback: on_rendered, user: 0 };
        assert!(harness.state.events().listen("render.world_rendered", 1, AmmlBand::NORMAL, listener).is_ok());
    }
    harness
}

fn join() {
    SessionGame::frame();
    SessionGame::set_player(0x5000);
    SessionGame::start_join(&CLIENT_A, [0; 7]);
    SessionGame::update(&CLIENT_A);
}

#[test]
fn canvas_call_with_another_frame_id_is_refused() {
    let harness = start(true);
    join();
    RenderGame::set_camera(Some(POSITION));

    RenderGame::render();

    // The own frame answers; the next frame, the frame 0 and the frame from another thread are refused.
    assert_eq!(
        seen(),
        [format!(
            "frame=1 vibrant=0 thread=0 own=0 position=12.5,70,-3.25 next={} zero={} other_thread={}",
            AmmlStatus::REFUSED.0,
            AmmlStatus::REFUSED.0,
            AmmlStatus::REFUSED.0
        )]
    );
    // After the dispatch the frame is refused.
    assert_eq!(camera(1).0, AmmlStatus::REFUSED);
    assert_eq!(RenderGame::render_calls(), 1);
    drop(harness);
}

#[test]
fn a_frame_without_a_view_gives_failed() {
    let _harness = start(true);
    join();
    RenderGame::set_camera(None);

    RenderGame::render();

    let line = &seen()[0];
    assert!(line.contains(&format!("own={}", AmmlStatus::FAILED.0)), "{line}");
}

#[test]
fn each_dispatch_has_a_new_frame_id() {
    let _harness = start(true);
    join();
    RenderGame::set_camera(Some(POSITION));

    RenderGame::render();
    RenderGame::render();

    let frames: Vec<String> = seen().iter().map(|line| line.split(' ').next().unwrap().to_string()).collect();
    assert_eq!(frames, ["frame=1", "frame=2"]);
}

#[test]
fn nothing_is_raised_before_a_world_is_joined() {
    let _harness = start(true);
    SessionGame::frame();
    RenderGame::set_camera(Some(POSITION));

    RenderGame::render();

    assert!(seen().is_empty());
    assert_eq!(RenderGame::render_calls(), 1);
}

#[test]
fn without_a_listener_the_original_runs_and_nothing_is_raised() {
    let _harness = start(false);
    join();
    RenderGame::set_camera(Some(POSITION));

    RenderGame::render();

    assert!(seen().is_empty());
    assert_eq!(RenderGame::render_calls(), 1);
}

#[test]
fn a_mesh_free_for_an_unknown_mod_is_an_invalid_argument() {
    let _harness = start(false);
    let mesh = AmmlMeshId { id: 1, generation: 0, _reserved: 0 };

    // SAFETY: the table is live; the view lives for the call.
    let status = unsafe { (table().mesh_free)(AmmlStr::from_str("@nobody/none"), mesh) };

    assert_eq!(status, AmmlStatus::INVALID_ARGUMENT);
}

#[test]
fn lighting_work_still_runs_in_the_shared_detour() {
    let harness = start(false);
    // No lighting entry resolves here, so the lighting adapter hooks nothing; its flag is set by hand, as its install
    // does after the terrain hooks are in.
    amml::LightingAdapter::install(harness.state);
    RenderLevelDetours::set(RenderLevelDetours::LIGHTING, true);
    SessionGame::frame();
    RenderGame::set_camera(Some(POSITION));

    RenderGame::render();

    // The lighting step ran on the game thread before the game's own call, and the game's call ran once.
    let line = "lighting render thread {\"game\":true}";
    let log = LogFile::wait_for(&harness.logs().join("loader.log"), line);
    assert!(log.contains(line), "{log}");
    assert_eq!(RenderGame::render_calls(), 1);
}

#[test]
fn camera_position_picks_the_view_of_the_running_renderer() {
    let _harness = start(true);
    join();
    let other = [1.0, 2.0, 3.0];
    RenderGame::set_views(POSITION, &[other, POSITION]);

    RenderGame::render();

    let line = &seen()[0];
    assert!(line.contains("own=0 position=12.5,70,-3.25"), "{line}");
}

#[test]
fn no_view_of_the_running_renderer_gives_failed() {
    let _harness = start(true);
    join();
    RenderGame::set_views(POSITION, &[[1.0, 2.0, 3.0]]);

    RenderGame::render();

    let line = &seen()[0];
    assert!(line.contains(&format!("own={}", AmmlStatus::FAILED.0)), "{line}");
}

#[test]
fn the_graphics_mode_is_read_only_with_a_listener_or_a_mesh() {
    let _harness = start(false);
    let adapter = WorldRenderAdapter::get().unwrap();
    assert!(!adapter.reads_graphics_mode());

    let _with_listener = start_listener();

    assert!(adapter.reads_graphics_mode());
}

// The module of the mod that `start_with_mod` loads.
const MODULE: &str = "@events/events";

// What the build listener saw, one status per call.
static BUILDS: Mutex<Vec<i32>> = Mutex::new(Vec::new());

fn quads(count: usize) -> Vec<AmmlColorQuad> {
    vec![AmmlColorQuad { corners: [AmmlVec3::default(); 4], rgba: AmmlRgba { r: 255, g: 255, b: 255, a: 255 } }; count]
}

fn overlay() -> AmmlMeshStyle {
    AmmlMeshStyle { blend: AmmlMeshStyle::BLEND_ALPHA, depth_test: 1, ..AmmlMeshStyle::default() }
}

fn build(frame: u64, quads: &[AmmlColorQuad], style: AmmlMeshStyle) -> AmmlStatus {
    let mut out = AmmlMeshId::default();
    // SAFETY: the table is live; the module, the quads and `out` live for the call.
    unsafe {
        (table().mesh_build.unwrap())(
            AmmlStr::from_str(MODULE),
            frame,
            quads.as_ptr(),
            quads.len() as u32,
            style,
            &raw mut out,
        )
    }
}

unsafe extern "C" fn on_rendered_build(
    event: *mut AmmlEventHeader,
    _: *const AmmlEventContext,
    _: *mut c_void,
) -> AmmlStatus {
    // SAFETY: the loader raises `render.world_rendered` with a `WorldRendered`.
    let frame = unsafe { &*event.cast::<WorldRendered>() }.frame_id;
    let statuses = [
        build(frame, &quads(8192), overlay()),
        build(frame, &quads(1), AmmlMeshStyle { light: AmmlMeshStyle::LIGHT_WORLD, ..overlay() }),
        build(frame, &quads(1), AmmlMeshStyle { texture: AmmlMeshStyle::TEXTURE_TERRAIN_ATLAS, ..overlay() }),
        build(frame, &quads(0), overlay()),
        build(frame + 1, &quads(1), overlay()),
    ];
    BUILDS.lock().unwrap().extend(statuses.map(|status| status.0));
    AmmlStatus::OK
}

// The render adapters over the stand-ins with one loaded mod, the render table found, and a listener that builds.
fn start_with_mod() -> Harness {
    let harness = Harness::start_with_entries(&RenderGame::entries(&[]));
    SessionAdapter::install(harness.state);
    WorldRenderAdapter::install(harness.state);
    let plain = EventsMod::build("plain", &[]);
    harness.install_mod("events", "events", &plain, "events.dll", 3);
    let scan = ModRegistry::new(ModRegistry::MAX_MODS).scan(&harness.mods()).unwrap();
    assert!(ModLoader::new(harness.state).load_all(&scan.mods).iter().all(|report| report.is_ok()));
    // SAFETY: the loader's tables live for the process.
    let services = unsafe { AmmlApi::services(harness.state.api()) }.unwrap();
    // SAFETY: as above.
    let render = unsafe { AmmlServices::render(services) }.unwrap();
    TABLE.store(render.expose_provenance(), Ordering::SeqCst);
    let listener = ModListener { owner: 0, callback: on_rendered_build, user: 0 };
    assert!(harness.state.events().listen("render.world_rendered", 1, AmmlBand::NORMAL, listener).is_ok());
    harness
}

// A `WorldRendered` listener of owner 0 on the running harness.
fn start_listener() -> amml_runtime::ListenerId {
    let state = amml::LoaderState::get().unwrap();
    let listener = ModListener { owner: 0, callback: on_rendered, user: 0 };
    state.events().listen("render.world_rendered", 1, AmmlBand::NORMAL, listener).unwrap()
}

#[test]
fn quad_count_above_max_is_refused() {
    let _harness = start_with_mod();
    join();
    RenderGame::set_camera(Some(POSITION));

    RenderGame::render();

    let builds = BUILDS.lock().unwrap().clone();
    assert_eq!(builds[0], AmmlStatus::REFUSED.0);
    // No quad was read and no game function was called.
    assert!(MeshGame::log().is_empty(), "{:?}", MeshGame::log());
    // An empty build is an invalid argument, a build for another frame is refused.
    assert_eq!(builds[3..], [AmmlStatus::INVALID_ARGUMENT.0, AmmlStatus::REFUSED.0]);
}

#[test]
fn world_light_is_not_available() {
    let _harness = start_with_mod();
    join();
    RenderGame::set_camera(Some(POSITION));

    RenderGame::render();

    // The area is offered (the count check runs after the area check), and `MeshLight::World` and
    // `MeshTexture::TerrainAtlas` are not served in this version.
    let builds = BUILDS.lock().unwrap().clone();
    assert_eq!(builds[0], AmmlStatus::REFUSED.0);
    assert_eq!(builds[1..3], [AmmlStatus::NOT_AVAILABLE.0, AmmlStatus::NOT_AVAILABLE.0]);
    assert!(MeshGame::log().is_empty(), "{:?}", MeshGame::log());
}

// What the draw listener did, one line per call: `build=<status>`, `draw=<status>`.
static DRAWN: Mutex<Vec<String>> = Mutex::new(Vec::new());
// The mesh the draw listener built, as `id`, `generation`; 0 before the first build.
static BUILT: Mutex<(u64, u32)> = Mutex::new((0, 0));
// The draws the listener makes in one frame.
static DRAWS_PER_FRAME: AtomicUsize = AtomicUsize::new(1);
// Whether the listener asks for a two-sided mesh.
static TWO_SIDED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

fn drawn() -> Vec<String> {
    DRAWN.lock().unwrap().clone()
}

// Builds one two-quad mesh at the first frame that allows it, then draws it `DRAWS_PER_FRAME` times each frame.
unsafe extern "C" fn on_rendered_draw(
    event: *mut AmmlEventHeader,
    _: *const AmmlEventContext,
    _: *mut c_void,
) -> AmmlStatus {
    // SAFETY: the loader raises `render.world_rendered` with a `WorldRendered`.
    let frame = unsafe { &*event.cast::<WorldRendered>() }.frame_id;
    let mut built = BUILT.lock().unwrap();
    if built.0 == 0 {
        let mut out = AmmlMeshId::default();
        let quads = quads(2);
        let style = AmmlMeshStyle { two_sided: u8::from(TWO_SIDED.load(Ordering::SeqCst)), ..overlay() };
        // SAFETY: the table is live; the module, the quads and `out` live for the call.
        let status = unsafe {
            (table().mesh_build.unwrap())(AmmlStr::from_str(MODULE), frame, quads.as_ptr(), 2, style, &raw mut out)
        };
        DRAWN.lock().unwrap().push(format!("build={}", status.0));
        if status == AmmlStatus::OK {
            *built = (out.id, out.generation);
        }
        return AmmlStatus::OK;
    }
    let mesh = AmmlMeshId { id: built.0, generation: built.1, _reserved: 0 };
    let transform = AmmlTransform {
        origin: AmmlBlockPos { x: 10, y: 64, z: -3 },
        offset: AmmlVec3 { x: 0.5, y: 0.0, z: 0.0 },
        scale: 1.0,
        yaw_degrees: 0.0,
    };
    for _ in 0..DRAWS_PER_FRAME.load(Ordering::SeqCst) {
        // SAFETY: the table is live; the module and the transform live for the call.
        let status = unsafe { (table().mesh_draw.unwrap())(AmmlStr::from_str(MODULE), frame, mesh, &transform) };
        DRAWN.lock().unwrap().push(format!("draw={}", status.0));
    }
    AmmlStatus::OK
}

fn texture_calls() -> TextureCalls {
    TextureCalls {
        resource_location_ctor: 1,
        image_buffer_ctor: 1,
        image_buffer_words: 6,
        buffer_deleter: 0,
        buffer_data: 8,
        buffer_size: 16,
        upload: 1,
        group_get: 1,
        unload: 1,
        group_base: 0x18,
        get_texture_group_slot: 1,
        context_get_texture_slot: 1,
    }
}

// The render adapters and the texture adapter over the stand-ins with one loaded mod, and the draw listener.
fn start_drawing() -> Harness {
    let harness = Harness::start_with_entries(&RenderGame::entries(&[]));
    SessionAdapter::install(harness.state);
    assert!(TextureAdapter::install_with(harness.state, texture_calls()));
    WorldRenderAdapter::install(harness.state);
    let plain = EventsMod::build("plain", &[]);
    harness.install_mod("events", "events", &plain, "events.dll", 3);
    let scan = ModRegistry::new(ModRegistry::MAX_MODS).scan(&harness.mods()).unwrap();
    assert!(ModLoader::new(harness.state).load_all(&scan.mods).iter().all(|report| report.is_ok()));
    // SAFETY: the loader's tables live for the process.
    let services = unsafe { AmmlApi::services(harness.state.api()) }.unwrap();
    // SAFETY: as above.
    let render = unsafe { AmmlServices::render(services) }.unwrap();
    TABLE.store(render.expose_provenance(), Ordering::SeqCst);
    let listener = ModListener { owner: 0, callback: on_rendered_draw, user: 0 };
    assert!(harness.state.events().listen("render.world_rendered", 1, AmmlBand::NORMAL, listener).is_ok());
    MeshGame::set_material(MeshGame::GAME_BYTES);
    harness
}

// A stand-in `shared_ptr` control block with `uses` uses: vtable, uses, weaks.
fn control_block(uses: i32) -> usize {
    let block: &'static mut [i32; 4] = Box::leak(Box::new([0, 0, uses, 1]));
    std::ptr::from_mut(block) as usize
}

fn uses(control: usize) -> i32 {
    // SAFETY: a leaked block of `control_block`.
    unsafe { ((control + 8) as *const i32).read() }
}

// Answers the loader's white texture upload as the texture pump and the game would: the book holds a `TexturePtr`
// whose two control blocks are `controls`, and the texture is ready.
fn give_white_texture(controls: [usize; 2]) {
    let book = TextureAdapter::get().unwrap().book();
    let work = book.take_work(0);
    let key = work
        .jobs
        .iter()
        .find_map(|job| match job {
            TextureJob::Upload { key, .. } => Some(*key),
            TextureJob::Bind { .. } => None,
        })
        .unwrap();
    let held: &'static mut [usize; 4] = Box::leak(Box::new([0x1000, controls[0], 0x2000, controls[1]]));
    book.uploaded(key);
    assert!(book.bound(key, std::ptr::from_mut(held) as usize));
    book.mark_ready(key);
}

// Joins, renders the build frame and the first draw frame (which asks for the white texture), then gives the texture.
fn build_and_get_texture(controls: [usize; 2]) {
    join();
    RenderGame::set_camera(Some(POSITION));
    RenderGame::render();
    RenderGame::render();
    give_white_texture(controls);
}

#[test]
fn draw_gives_the_game_a_list_it_may_destroy() {
    let _harness = start_drawing();
    let controls = [control_block(3), control_block(3)];
    build_and_get_texture(controls);
    DRAWS_PER_FRAME.store(4, Ordering::SeqCst);

    RenderGame::render();
    RenderGame::render();

    let ok = format!("draw={}", AmmlStatus::OK.0);
    assert_eq!(drawn().iter().filter(|line| **line == ok).count(), 8, "{:?}", drawn());
    assert_eq!(MeshGame::log().iter().filter(|line| line.starts_with("render ")).count(), 8);
    // The game destroyed element 0 of each list; the loader's copy took the uses it gave back.
    assert_eq!(controls.map(uses), [3, 3]);
}

#[test]
fn draw_pops_the_world_matrix_and_restores_the_shader_color() {
    let _harness = start_drawing();
    build_and_get_texture([control_block(1), control_block(1)]);

    RenderGame::render();

    let log = MeshGame::log();
    let draw: Vec<&String> = log.iter().skip_while(|line| *line != "name tag renderer").collect();
    assert_eq!(draw.len(), 5, "{log:?}");
    assert_eq!(draw[0], "name tag renderer");
    assert_eq!(draw[1], "constants");
    assert_eq!(draw[2], "push flag=1");
    assert!(draw[3].starts_with("render vertices=8 context=0x10 material=name_tag count=1 index=1 capture=0 optional=0 color=[1.0, 1.0, 1.0, 1.0]"), "{}", draw[3]);
    assert_eq!(draw[4], "pop");
    // Mesh point (0, 0, 0) at origin + offset - camera: (10.5 - 12.5, 64 - 70, -3 + 3.25).
    assert_eq!(MeshGame::popped_translation(), [-2.0, -6.0, 0.25]);
    assert_eq!(MeshGame::camera_dirty(), 1);
    assert_eq!(MeshGame::shader_color(), (MeshGame::FRAME_COLOR, 1));
}

#[test]
fn vibrant_draw_patches_then_restore_writes_the_kept_bytes_once() {
    let _harness = start_drawing();
    let adapter = WorldRenderAdapter::get().unwrap();
    adapter.set_vibrant(true);
    build_and_get_texture([control_block(1), control_block(1)]);

    RenderGame::render();
    let patched = MeshGame::material();
    adapter.world_left();
    RenderGame::render();
    let restored = MeshGame::material();
    MeshGame::set_material([9; 4]);
    RenderGame::render();

    assert_eq!(patched, [1, 6, 8, 0]);
    assert!(MeshGame::log().iter().any(|line| line.contains("material=wireframe")), "{:?}", MeshGame::log());
    assert_eq!(restored, MeshGame::GAME_BYTES);
    // The kept bytes are written once: a later frame leaves the material as it is.
    assert_eq!(MeshGame::material(), [9; 4]);
}

#[test]
fn a_failed_vertex_write_leaves_the_tessellator_closed() {
    let _harness = start_drawing();
    join();
    RenderGame::set_camera(Some(POSITION));
    HeapProbe::set_failing(true);

    RenderGame::render();
    HeapProbe::set_failing(false);

    assert_eq!(drawn(), [format!("build={}", AmmlStatus::FAILED.0)]);
    assert!(!MeshGame::tessellating());
    assert_eq!(
        MeshGame::log(),
        [
            "begin callback=empty mode=1 reserve=8",
            "clear",
            "end upload=2 vertices=0 count=0 name=15 mode=0",
            "dtor vertices=0"
        ]
    );
    // The tessellator works again for the next build.
    RenderGame::render();
    assert_eq!(drawn()[1], format!("build={}", AmmlStatus::OK.0));
}

#[test]
fn a_build_gives_begin_an_empty_debug_callback_by_reference() {
    let _harness = start_drawing();
    join();
    RenderGame::set_camera(Some(POSITION));

    RenderGame::render();

    assert_eq!(drawn(), [format!("build={}", AmmlStatus::OK.0)]);
    let begin = MeshGame::log().into_iter().find(|line| line.starts_with("begin ")).unwrap_or_default();
    assert_eq!(begin, "begin callback=empty mode=1 reserve=8");
}

// The vertex count of the first mesh the stand-in tessellator ended.
fn built_vertices() -> String {
    MeshGame::log().into_iter().find(|line| line.starts_with("end ")).unwrap_or_default()
}

#[test]
fn two_sided_writes_the_second_winding_for_the_name_tag_material() {
    let _harness = start_drawing();
    TWO_SIDED.store(true, Ordering::SeqCst);
    join();
    RenderGame::set_camera(Some(POSITION));

    RenderGame::render();

    // Two quads, each written in both windings.
    assert!(built_vertices().starts_with("end upload=0 vertices=16 "), "{}", built_vertices());
}

#[test]
fn two_sided_writes_one_winding_for_the_vibrant_material() {
    let _harness = start_drawing();
    TWO_SIDED.store(true, Ordering::SeqCst);
    WorldRenderAdapter::get().unwrap().set_vibrant(true);
    join();
    RenderGame::set_camera(Some(POSITION));

    RenderGame::render();

    // The wireframe material shows both sides of each quad already.
    assert!(built_vertices().starts_with("end upload=0 vertices=8 "), "{}", built_vertices());
}
