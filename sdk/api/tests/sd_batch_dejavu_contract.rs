use amml_abi::{
    AmmlUseRow, AppFocusGained, AppFocusLost, CameraFov, CameraSetup, ClientTick, ConnectionStarted, FrameEnd,
    FrameUpdate, IdleScreenOpen, InputLeaseLost, InputLeaseResumed, InputLeaseSuspended, JoinServerResult, JoinStarted,
    LatencyBarrier, ModsLoaded, PacketWire, PlayerMoved, ProviderGone, ProviderReady, ServerJoinStarting,
    TimeStepApplied, WorldJoined, WorldLeft, WorldStorageClosed, WorldStorageOpened,
};
use amml_api::{
    Area, BlockSource, Camera, CameraControl, CameraSetupExt, Client, ConnectionStartedExt, Ctx, FrameEndExt,
    HoldRequest, IdleScreenOpenExt, JoinServerResultExt, JoinStartedExt, LatencyBarrierExt, LocalPlayer, Network,
    OptionKey, Options, PacketWireExt, Persistence, PlayerMovedExt, Pose, ServerJoinStartingExt, Time, TimeHold,
    TimeStepAppliedExt, Uses, Window, WorldShapes, WorldStorageOpenedExt,
};

const DEJAVU: Uses = amml_api::amml_uses! {
    feature "dejavu.core" {
        required FrameUpdate; required ClientTick; required WorldJoined; required WorldLeft; required ModsLoaded;
        required ProviderReady; required ProviderGone;
        required InputLeaseSuspended; required InputLeaseResumed; required InputLeaseLost;
        required area Area::CLIENT_READ; required area Area::CLIENT_TOP_SCREEN.at(2); required area Area::INPUT_LEASE;
    }
    feature "dejavu.capture" {
        required PacketWire; required ConnectionStarted; required JoinStarted;
        required area Area::NETWORK_CHUNK_CACHE; required area Area::CLIENT_WORLD_INFO;
    }
    feature "dejavu.join" {
        required ServerJoinStarting; required JoinServerResult;
        required area Area::CLIENT_JOIN_SERVER; required area Area::CLIENT_LEAVE_WORLD;
        required area Area::CLIENT_IN_GAMEPLAY; required area Area::CLIENT_CAN_JOIN;
    }
    feature "dejavu.camera" { required CameraSetup; required area Area::CAMERA_POSE; required area Area::CAMERA_FREE; }
    feature "dejavu.fov" { required CameraFov; }
    feature "dejavu.body" { required area Area::CAMERA_HIDE_ACTOR; }
    feature "dejavu.viewer" { required PlayerMoved; required area Area::ACTOR_READ.at(3); required area Area::ACTOR_LOOK_WRITE; }
    feature "dejavu.clock" {
        required TimeStepApplied; required FrameEnd; required LatencyBarrier; required area Area::TIME_CONTROL;
    }
    feature "dejavu.guards" {
        required AppFocusLost; required AppFocusGained; required IdleScreenOpen;
        required area Area::CLIENT_FOCUS_REPLAY; required area Area::CLIENT_ACTIVITY;
    }
    feature "dejavu.graphics" { required area Area::OPTIONS_INT; }
    feature "dejavu.markers" {
        required area Area::WORLD_SHAPES; required area Area::WORLD_RAYCAST; required area Area::WORLD_ACTORS_IN;
        required area Area::WINDOW_READ;
    }
    feature "dejavu.world_folder" { required WorldStorageOpened; required WorldStorageClosed; }
    feature "dejavu.protocol" { required area Area::NETWORK_PROTOCOL; }
};

fn text(view: amml_abi::AmmlStr) -> String {
    // SAFETY: the rows of `amml_uses!` point to static strings.
    unsafe { view.as_str() }.unwrap().to_string()
}

fn rows(kind: u8) -> Vec<(String, u32)> {
    DEJAVU.rows().iter().filter(|row| row.kind == kind).map(|row| (text(row.name), row.version)).collect()
}

#[test]
fn dejavu_contract_uses_block_compiles_and_lists_every_row() {
    assert_eq!(
        rows(AmmlUseRow::KIND_EVENT),
        [
            ("frame.update", 1),
            ("client.tick", 1),
            ("world.joined", 1),
            ("world.left", 1),
            ("mods.loaded", 1),
            ("registry.provider_ready", 1),
            ("registry.provider_gone", 1),
            ("input.lease_suspended", 1),
            ("input.lease_resumed", 1),
            ("input.lease_lost", 1),
            ("network.packet_wire", 1),
            ("network.connection_started", 1),
            ("world.join_started", 2),
            ("world.server_join_starting", 2),
            ("client.join_server_result", 1),
            ("camera.setup", 1),
            ("camera.fov", 1),
            ("player.moved", 1),
            ("time.step_applied", 1),
            ("time.frame_end", 1),
            ("time.latency_barrier", 1),
            ("client.app_focus_lost", 1),
            ("client.app_focus_gained", 1),
            ("client.idle_screen_open", 1),
            ("world.storage_opened", 1),
            ("world.storage_closed", 1),
        ]
        .map(|(id, version)| (id.to_string(), version))
    );
    assert_eq!(
        rows(AmmlUseRow::KIND_AREA),
        [
            ("client.read", 1),
            ("client.top_screen", 2),
            ("input.lease", 1),
            ("network.chunk_cache", 1),
            ("client.world_info", 1),
            ("client.join_server", 1),
            ("client.leave_world", 1),
            ("client.in_gameplay", 1),
            ("client.can_join", 1),
            ("camera.pose", 1),
            ("camera.free", 1),
            ("camera.hide_actor", 1),
            ("actor.read", 3),
            ("actor.look_write", 1),
            ("time.control", 1),
            ("client.focus_replay", 1),
            ("client.activity", 1),
            ("options.int", 1),
            ("world.shapes", 1),
            ("world.raycast", 1),
            ("world.actors_in", 1),
            ("window.read", 1),
            ("network.protocol", 1),
        ]
        .map(|(id, version)| (id.to_string(), version))
    );
    assert!(DEJAVU.rows().iter().all(|row| row.required != 0), "every row is required");
}

// Names the calls and types of the contract: a rename or a removed method fails the build of this file.
#[test]
fn dejavu_contract_calls_exist() {
    let _ = (
        Client::top_screen,
        Client::world_info,
        Client::join_server,
        Client::join_server_direct,
        Client::leave_world,
        Client::in_gameplay,
        Client::can_join,
        Client::note_activity,
        Client::replay_app_focus_lost,
        Client::local_player,
    );
    let _ = (
        LocalPlayer::head_position,
        LocalPlayer::rotation,
        LocalPlayer::dimension,
        LocalPlayer::block_source,
        LocalPlayer::set_rotation,
    );
    let _ = (Camera::set_pose, Camera::rendered_position);
    let _ = (Ctx::camera, Ctx::world_shapes, Ctx::options, Ctx::time, Ctx::network, Ctx::window);
    let _ = (CameraControl::set_free, CameraControl::release_free, CameraControl::hide_actor);
    let _ = (WorldShapes::apply, WorldShapes::max_batch, WorldShapes::target_epoch);
    let _ = (BlockSource::raycast, BlockSource::actors_in);
    let _ = (Options::get_int, Options::set_int, Options::release_transient);
    let _ = (OptionKey::VSYNC, OptionKey::MAX_FRAMERATE, Persistence::Transient);
    let _ = (Time::hold, TimeHold::step_us);
    let _ = HoldRequest::default();
    let _ = (Network::set_client_chunk_cache, Network::protocol_version);
    let _ = Window::client_size;
    let _: Option<Pose> = None;

    let _ = <CameraSetup as CameraSetupExt>::frame_id;
    let _ = <CameraSetup as CameraSetupExt>::first_in_frame;
    let _ = <CameraSetup as CameraSetupExt>::camera;
    let _ = <PacketWire as PacketWireExt>::dir;
    let _ = <PacketWire as PacketWireExt>::connection;
    let _ = <PacketWire as PacketWireExt>::bytes;
    let _ = <ConnectionStarted as ConnectionStartedExt>::connection;
    let _ = <ConnectionStarted as ConnectionStartedExt>::protocol;
    let _ = <ConnectionStarted as ConnectionStartedExt>::port;
    let _ = <ConnectionStarted as ConnectionStartedExt>::host;
    let _ = <JoinStarted as JoinStartedExt>::local;
    let _ = <JoinStarted as JoinStartedExt>::server_name;
    let _ = <JoinStarted as JoinStartedExt>::world_name;
    let _ = <ServerJoinStarting as ServerJoinStartingExt>::host;
    let _ = <ServerJoinStarting as ServerJoinStartingExt>::url;
    let _ = <ServerJoinStarting as ServerJoinStartingExt>::name;
    let _ = <ServerJoinStarting as ServerJoinStartingExt>::port;
    let _ = <ServerJoinStarting as ServerJoinStartingExt>::transfer;
    let _ = <JoinServerResult as JoinServerResultExt>::join_id;
    let _ = <JoinServerResult as JoinServerResultExt>::outcome;
    let _ = <WorldStorageOpened as WorldStorageOpenedExt>::path;
    let _ = <PlayerMoved as PlayerMovedExt>::pos;
    let _ = <PlayerMoved as PlayerMovedExt>::rotation;
    let _ = <PlayerMoved as PlayerMovedExt>::head_yaw;
    let _ = <PlayerMoved as PlayerMovedExt>::mode;
    let _ = <TimeStepApplied as TimeStepAppliedExt>::stepped_us;
    let _ = <TimeStepApplied as TimeStepAppliedExt>::sim_ticks;
    let _ = <TimeStepApplied as TimeStepAppliedExt>::sim_alpha;
    let _ = <FrameEnd as FrameEndExt>::count;
    let _ = <LatencyBarrier as LatencyBarrierExt>::create_time;
    let _ = <IdleScreenOpen as IdleScreenOpenExt>::via;

    // `result` is the field a camera fov listener writes; `steady` is the user's setting.
    let _ = |fov: &mut CameraFov| {
        let steady: f32 = fov.steady;
        fov.result = steady;
    };

    // Names only: the event types of the contract exist.
    let _ = std::marker::PhantomData::<(
        FrameUpdate,
        ClientTick,
        WorldJoined,
        WorldLeft,
        ModsLoaded,
        ProviderReady,
        ProviderGone,
        InputLeaseSuspended,
        InputLeaseResumed,
        InputLeaseLost,
        WorldStorageClosed,
        AppFocusLost,
        AppFocusGained,
    )>;
}
