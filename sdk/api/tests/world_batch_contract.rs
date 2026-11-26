use amml_abi::{
    AmmlUseRow, BlockChanged, CameraInstructionReceived, CameraSetup, ClientTick, InjectedInputReleased,
    InputLeaseLost, InputLeaseResumed, InputLeaseSuspended, KeyInput, ModsLoaded, MouseInput, ProviderGone,
    ProviderReady, ScreenshotTaken, ScreenshotWritten, WindowFocus, WorldLeft, WorldRendered,
};
use amml_api::{
    Aabb, ActorBox, ApplyStats, Area, Biomes, BlockChangedExt, BlockHit, BlockSource, BlockTypeId, Blocks, Camera,
    CameraControl, CameraInstructionReceivedExt, CameraSetupExt, ChunkLoad, Client, ColorQuad, ColumnPos,
    ColumnRequest, ColumnSample, Connection, ConnectionKind, Ctx, CursorClip, Dimension, HookOrder, LevelChunk,
    LocalPlayer, MeshId, MeshStyle, Pose, Projection, RayOptions, ShapeOp, SurfaceSample, Transform, Uses, WorldCanvas,
    WorldInfo, WorldRenderedExt, WorldShapes,
};

const LEVELS: Uses = amml_api::amml_uses! {
    feature "overlay" {
        required ClientTick; required WorldRendered; required ProviderReady; required ProviderGone; required ModsLoaded;
        required InputLeaseSuspended; required InputLeaseResumed; required InputLeaseLost; required InjectedInputReleased;
        required ScreenshotTaken; required ScreenshotWritten;
        required area Area::INPUT_LEASE; required area Area::CLIENT_READ; required area Area::CLIENT_HUD;
        required area Area::ACTOR_READ.at(3); required area Area::WORLD_SKY_DARKEN;
        required area Area::WORLD_SCAN_SURFACES; required area Area::RENDER_WORLD_CANVAS.at(2);
    }
};

const FREECAM: Uses = amml_api::amml_uses! {
    feature "flight" {
        required ClientTick; required CameraSetup; required CameraInstructionReceived; required KeyInput; required MouseInput;
        required WorldLeft; required ProviderReady; required ProviderGone; required ModsLoaded;
        required InputLeaseSuspended; required InputLeaseResumed; required InputLeaseLost; required InjectedInputReleased;
        required WindowFocus;
        required area Area::CLIENT_READ; required area Area::INPUT_LEASE;
        required area Area::ACTOR_READ.at(3); required area Area::ACTOR_HEALTH;
        required area Area::CAMERA_POSE; required area Area::CAMERA_PROJECTION; required area Area::CAMERA_FREE;
    }
    feature "markers" { required area Area::WORLD_SHAPES; }
    feature "camera_watch" { required area Area::CAMERA_ACTIVE; }
    feature "look_at" { required area Area::WORLD_RAYCAST; required area Area::WORLD_ACTORS_IN; }
    feature "screenshots" { required ScreenshotTaken; required ScreenshotWritten; required area Area::OVERLAY_SCREENSHOT; }
};

const MINIMAP: Uses = amml_api::amml_uses! {
    feature "minimap" {
        required ClientTick; required KeyInput; required WorldLeft; required ProviderReady; required ProviderGone; required ModsLoaded;
        required InputLeaseSuspended; required InputLeaseResumed; required InputLeaseLost; required InjectedInputReleased;
        required ScreenshotTaken; required ScreenshotWritten;
        required area Area::INPUT_LEASE; required area Area::CLIENT_READ; required area Area::CLIENT_MOUSE;
        required area Area::WINDOW_READ; required area Area::ACTOR_READ.at(3); required area Area::WORLD_CHUNK_READ;
    }
    feature "world_identity" { required area Area::CLIENT_WORLD_INFO; }
    feature "biomes" { required area Area::WORLD_BIOMES; }
    feature "block_changes" { required BlockChanged; }
    feature "vanilla_colors" { required area Area::WORLD_MAP_COLOR; }
    feature "cursor_clip" { required area Area::WINDOW_CURSOR_CLIP; }
};

type Row = (String, u8, String, u32, bool);

fn text(view: amml_abi::AmmlStr) -> String {
    // SAFETY: the rows of `amml_uses!` point to static strings.
    unsafe { view.as_str() }.unwrap().to_string()
}

// (feature, kind, id, version, required) per row.
fn rows(uses: &Uses) -> Vec<Row> {
    uses.rows()
        .iter()
        .map(|row: &AmmlUseRow| (text(row.feature), row.kind, text(row.name), row.version, row.required != 0))
        .collect()
}

fn ids(uses: &Uses, kind: u8) -> Vec<String> {
    rows(uses).into_iter().filter(|row| row.1 == kind).map(|row| row.2).collect()
}

fn versions(uses: &Uses, kind: u8) -> Vec<(String, u32)> {
    rows(uses).into_iter().filter(|row| row.1 == kind).map(|row| (row.2, row.3)).collect()
}

const COMMON_EVENTS: [&str; 9] = [
    "registry.provider_ready",
    "registry.provider_gone",
    "mods.loaded",
    "input.lease_suspended",
    "input.lease_resumed",
    "input.lease_lost",
    "input.injected_released",
    "overlay.screenshot_taken",
    "overlay.screenshot_written",
];

#[test]
fn world_contract_uses_blocks_compile_and_list_every_row() {
    assert_eq!(
        ids(&LEVELS, AmmlUseRow::KIND_EVENT),
        ["client.tick", "render.world_rendered"].into_iter().chain(COMMON_EVENTS).collect::<Vec<_>>()
    );
    assert_eq!(
        versions(&LEVELS, AmmlUseRow::KIND_AREA),
        [
            ("input.lease", 1),
            ("client.read", 1),
            ("client.hud", 1),
            ("actor.read", 3),
            ("world.sky_darken", 1),
            ("world.scan_surfaces", 1),
            ("render.world_canvas", 2)
        ]
        .map(|(id, version)| (id.to_string(), version))
    );
    assert_eq!(
        ids(&FREECAM, AmmlUseRow::KIND_EVENT),
        [
            "client.tick",
            "camera.setup",
            "camera.instruction_received",
            "input.key",
            "input.mouse",
            "world.left",
            "registry.provider_ready",
            "registry.provider_gone",
            "mods.loaded",
            "input.lease_suspended",
            "input.lease_resumed",
            "input.lease_lost",
            "input.injected_released",
            "window.focus",
            "overlay.screenshot_taken",
            "overlay.screenshot_written"
        ]
    );
    assert_eq!(
        ids(&FREECAM, AmmlUseRow::KIND_AREA),
        [
            "client.read",
            "input.lease",
            "actor.read",
            "actor.health",
            "camera.pose",
            "camera.projection",
            "camera.free",
            "world.shapes",
            "camera.active",
            "world.raycast",
            "world.actors_in",
            "overlay.screenshot"
        ]
    );
    assert_eq!(
        ids(&MINIMAP, AmmlUseRow::KIND_EVENT),
        ["client.tick", "input.key", "world.left"]
            .into_iter()
            .chain(COMMON_EVENTS)
            .chain(["world.block_changed"])
            .collect::<Vec<_>>()
    );
    assert_eq!(
        ids(&MINIMAP, AmmlUseRow::KIND_AREA),
        [
            "input.lease",
            "client.read",
            "client.mouse",
            "window.read",
            "actor.read",
            "world.chunk_read",
            "client.world_info",
            "world.biomes",
            "world.map_color",
            "window.cursor_clip"
        ]
    );
    for uses in [&LEVELS, &FREECAM, &MINIMAP] {
        assert!(rows(uses).iter().all(|row| row.4), "every row is required");
    }
}

// Names the calls and types of the contract: a rename or a removed method fails the build of this file.
#[test]
fn world_contract_calls_exist() {
    let _ = (HookOrder::READ_AFTER_WRITERS, HookOrder::WRITE, HookOrder::NORMAL);
    let _ = (Client::world_active, Client::showing_menu, Client::hide_hud, Client::local_player);
    let _ = (Client::mouse_captured, Client::world_info);
    let _ = (LocalPlayer::position, LocalPlayer::rotation, LocalPlayer::dimension, LocalPlayer::block_source);
    let _ = (LocalPlayer::head_position, LocalPlayer::health, LocalPlayer::is_alive);
    let _ = (Dimension::id, Dimension::sky_darken);
    let _ = (BlockSource::scan_surfaces, BlockSource::chunk, BlockSource::raycast, BlockSource::actors_in);
    let _ = (LevelChunk::load_state, LevelChunk::read_columns);
    let _ = (WorldCanvas::camera_position, WorldCanvas::build, WorldCanvas::draw, WorldCanvas::free);
    let _ = WorldCanvas::MAX_QUADS;
    let _ = MeshStyle::OVERLAY;
    let _ = (Camera::set_pose, Camera::rendered_position, Camera::projection);
    let _ = (CameraControl::set_free, CameraControl::release_free, CameraControl::free_available);
    let _ = CameraControl::active_camera;
    let _ = (WorldShapes::apply, WorldShapes::max_batch, WorldShapes::target_epoch);
    let _ = (Ctx::camera, Ctx::world_shapes, Ctx::blocks, Ctx::biomes, Ctx::window);
    let _ = (Blocks::type_name, Biomes::name);
    let _ = amml_api::Window::is_foreground;
    let _ = amml_api::Window::clip_cursor;
    let _ = CursorClip::Centre;
    let _ = ChunkLoad::TopPlaceholder;
    let _ = ConnectionKind::Remote;

    let _ = <WorldRendered as WorldRenderedExt>::frame_id;
    let _ = <WorldRendered as WorldRenderedExt>::vibrant;
    let _ = <WorldRendered as WorldRenderedExt>::canvas;
    let _ = <CameraSetup as CameraSetupExt>::frame_id;
    let _ = <CameraSetup as CameraSetupExt>::first_in_frame;
    let _ = <CameraSetup as CameraSetupExt>::camera;
    let _ = <CameraInstructionReceived as CameraInstructionReceivedExt>::kind;
    let _ = <BlockChanged as BlockChangedExt>::dimension;
    let _ = <BlockChanged as BlockChangedExt>::pos;
    let _ = <BlockChanged as BlockChangedExt>::layer;
    let _ = <BlockChanged as BlockChangedExt>::new;
    let _ = <BlockChanged as BlockChangedExt>::old;

    // Names only: the types of the contract exist.
    let _ = std::marker::PhantomData::<(
        Aabb,
        ActorBox,
        ApplyStats,
        BlockHit,
        BlockTypeId,
        ColorQuad,
        ColumnPos,
        ColumnRequest,
        ColumnSample,
        Connection,
        MeshId,
        Pose,
        Projection,
        RayOptions,
        ShapeOp,
        SurfaceSample,
        Transform,
        WorldInfo,
    )>;
}
