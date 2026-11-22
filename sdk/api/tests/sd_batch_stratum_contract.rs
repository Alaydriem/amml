use amml_abi::{
    AmmlOrigin, AmmlThread, AmmlUseRow, AppFocusLost, BlockChanged, BlockEntityPass, BuildBlock, ClientTick,
    DimensionChanging, InputLeaseLost, InputLeaseResumed, InputLeaseSuspended, InventoryRequestDone, ModsLoaded,
    MouseInput, ProviderGone, ProviderReady, UsePressed, WindowFocus, WorldJoined, WorldLeft, WorldRendered,
};
use amml_api::{
    Aabb, Area, Block, BlockChangedExt, BlockEntities, BlockEntityGhost, BlockEntityPassExt, BlockGrid, BlockHit,
    BlockMatch, BlockMesher, BlockSource, Blocks, BuildBlockExt, Client, Connection, Ctx, Dimension, Dirs, Face,
    GameMode, GhostEntityOptions, HitResult, HookOrder, Hud, Input, InputBlocks, Inventory, InventoryRequestDoneExt,
    LocalPlayer, MeshBuilt, MeshGroup, MeshId, MeshStyle, MesherOptions, MouseInputExt, Options, OtherLeases,
    SkullPose, StateValue, Transform, Uses, WindowFocusExt, WorldCanvas, WorldInfo, WorldRenderedExt,
};

const STRATUM: Uses = amml_api::amml_uses! {
    feature "core" {
        required ClientTick; required WorldJoined; required WorldLeft; required DimensionChanging;
        required ProviderReady; required ProviderGone; required ModsLoaded;
        required InputLeaseSuspended; required InputLeaseResumed; required InputLeaseLost;
        required area Area::INPUT_LEASE; required area Area::CLIENT_READ; required area Area::CLIENT_WORLD_INFO;
        required area Area::ACTOR_READ.at(4); required area Area::OPTIONS_VIEW_DISTANCE;
    }
    feature "ghost" {
        required WorldRendered;
        required area Area::RENDER_WORLD_CANVAS.at(3); required area Area::RENDER_BLOCK_MESHER;
        required area Area::WORLD_BLOCK_STATES; required area Area::RENDER_WATER_TINT;
    }
    feature "block_changes" { required BlockChanged; }
    feature "block_entities" {
        required BlockEntityPass;
        required area Area::RENDER_BLOCK_ENTITIES; required area Area::RENDER_CHEST_PAIR; required area Area::RENDER_SKULL_POSE;
    }
    feature "easy_place" {
        required UsePressed; required BuildBlock; required InventoryRequestDone; required MouseInput; required WindowFocus;
        required area Area::WORLD_BLOCKS; required area Area::PLAYER_BUILD; required area Area::PLAYER_PICK_RANGE;
        required area Area::PLAYER_HIT_RESULT;
        required area Area::HUD_SELECT_HOTBAR; required area Area::INVENTORY_READ.at(3); required area Area::INVENTORY_REQUEST.at(2);
    }
    feature "picker" { required AppFocusLost; }
    feature "build" {
        required area Area::CLIENT_SEND_COMMAND; required area Area::ACTOR_GAME_TYPE; required area Area::ACTOR_PERMISSION;
        required area Area::WORLD_KINDS_IN_BOX;
    }
};

fn text(view: amml_abi::AmmlStr) -> String {
    // SAFETY: the rows of `amml_uses!` point to static strings.
    unsafe { view.as_str() }.unwrap().to_string()
}

fn rows(kind: u8) -> Vec<(String, u32)> {
    STRATUM.rows().iter().filter(|row| row.kind == kind).map(|row| (text(row.name), row.version)).collect()
}

#[test]
fn stratum_contract_uses_block_compiles_and_lists_every_row() {
    let events: Vec<String> = rows(AmmlUseRow::KIND_EVENT).into_iter().map(|row| row.0).collect();
    assert_eq!(
        events,
        [
            "client.tick",
            "world.joined",
            "world.left",
            "world.dimension_changing",
            "registry.provider_ready",
            "registry.provider_gone",
            "mods.loaded",
            "input.lease_suspended",
            "input.lease_resumed",
            "input.lease_lost",
            "render.world_rendered",
            "world.block_changed",
            "render.block_entity_pass",
            "player.use_pressed",
            "player.build_block",
            "inventory.request_done",
            "input.mouse",
            "window.focus",
            "client.app_focus_lost",
        ]
    );
    assert_eq!(
        rows(AmmlUseRow::KIND_AREA),
        [
            ("input.lease", 1),
            ("client.read", 1),
            ("client.world_info", 1),
            ("actor.read", 4),
            ("options.view_distance", 1),
            ("render.world_canvas", 3),
            ("render.block_mesher", 1),
            ("world.block_states", 1),
            ("render.water_tint", 1),
            ("render.block_entities", 1),
            ("render.chest_pair", 1),
            ("render.skull_pose", 1),
            ("world.blocks", 1),
            ("player.build", 1),
            ("player.pick_range", 1),
            ("player.hit_result", 1),
            ("hud.select_hotbar", 1),
            ("inventory.read", 3),
            ("inventory.request", 2),
            ("client.send_command", 1),
            ("actor.game_type", 1),
            ("actor.permission", 1),
            ("world.kinds_in_box", 1),
        ]
        .map(|(id, version)| (id.to_string(), version))
    );
    assert!(STRATUM.rows().iter().all(|row| row.required != 0), "every row is required");
}

// Names the calls and types of the contract: a rename or a removed method fails the build of this file.
#[test]
fn stratum_contract_calls_exist() {
    let _ = (HookOrder::READ_AFTER_WRITERS, HookOrder::WRITE);
    let _ = (Client::world_active, Client::local_player, Client::showing_menu, Client::world_info);
    let _ = (Client::send_command, Client::hud);
    let _ = (
        LocalPlayer::position,
        LocalPlayer::rotation,
        LocalPlayer::head_position,
        LocalPlayer::view_vector,
        LocalPlayer::aabb,
        LocalPlayer::dimension,
        LocalPlayer::block_source,
        LocalPlayer::inventory,
        LocalPlayer::game_type,
        LocalPlayer::permission_level,
        LocalPlayer::game_mode,
        LocalPlayer::hit_result,
    );
    let _ = (Input::held_by_others, OtherLeases::any);
    let held = OtherLeases::default();
    let _: (&u32, &InputBlocks) = (&held.leases, &held.blocks);
    let _ = |hit: HitResult| match hit {
        HitResult::Block(BlockHit { pos, point, face }) => Some((pos, point, face)),
        HitResult::Entity { point: _ } | HitResult::Nothing => None,
        _ => None,
    };
    let _ = Dimension::id;
    let info = WorldInfo::default();
    let _: &Option<std::path::PathBuf> = &info.storage_folder;
    let _: &Option<String> = &info.level_name;
    let _: &Option<u64> = &info.seed;
    let _: &Option<bool> = &info.realm;
    let _: &Option<bool> = &info.commands_enabled;
    let _: &Option<Connection> = &info.connection;
    let _ = Options::view_distance_chunks;
    let _ = (Dirs::config_dir, Dirs::data_dir);
    let _ = (BlockEntities::create, BlockEntities::free);
    let _ = (BlockSource::block, BlockSource::kinds_in_box);
    let _ = (Block::type_id, Block::name, Block::is_air, Block::can_be_built_over);
    let _ = (Block::is_interactive, Block::item, Block::handle);
    let _ = (Blocks::from_state, Blocks::rotated, Blocks::name, Blocks::item, Blocks::type_name);
    let _ = (WorldCanvas::camera_position, WorldCanvas::clip_matrix, WorldCanvas::draw, WorldCanvas::free);
    let _ = MeshStyle::GHOST;
    let _ = (BlockMesher::build, BlockMesher::begin_group, BlockMesher::append, BlockMesher::finish);
    let _ = (Ctx::options, Ctx::blocks, Ctx::block_entities, Ctx::dirs);
    let _ = (GameMode::build_at, GameMode::build_delay, GameMode::max_pick_range);
    let _ = (Inventory::selected_slot, Inventory::slot, Inventory::slots, Inventory::request_swap);
    let _ = Inventory::request_pick;
    let _ = Hud::select_hotbar_slot;

    let _ = <WorldRendered as WorldRenderedExt>::frame_id;
    let _ = <WorldRendered as WorldRenderedExt>::canvas;
    let _ = <WorldRendered as WorldRenderedExt>::mesher;
    let _ = <BlockChanged as BlockChangedExt>::dimension;
    let _ = <BlockChanged as BlockChangedExt>::pos;
    let _ = <BlockChanged as BlockChangedExt>::layer;
    let _ = <BlockChanged as BlockChangedExt>::new;
    let _ = <BlockChanged as BlockChangedExt>::old;
    let _ = <BlockChanged as BlockChangedExt>::new_state;
    let _ = <BuildBlock as BuildBlockExt>::pos;
    let _ = <BuildBlock as BuildBlockExt>::face;
    let _ = <BuildBlock as BuildBlockExt>::hand;
    let _ = <BuildBlock as BuildBlockExt>::sim_tick;
    let _ = <BuildBlock as BuildBlockExt>::is_local_player;
    let _ = <InventoryRequestDone as InventoryRequestDoneExt>::id;
    let _ = <InventoryRequestDone as InventoryRequestDoneExt>::outcome;
    let _ = <InventoryRequestDone as InventoryRequestDoneExt>::accepted;
    let _ = <InventoryRequestDone as InventoryRequestDoneExt>::total;
    let _ = <MouseInput as MouseInputExt>::action;
    let _ = <WindowFocus as WindowFocusExt>::gained;
    let _ = <WindowFocus as WindowFocusExt>::replayed;
    let _ = (Face::Down, AmmlOrigin::THIS_MOD, AmmlThread::SERVER);
    let _ = <BlockEntityPass as BlockEntityPassExt>::camera;
    let _ = <BlockEntityPass as BlockEntityPassExt>::draw_block_entity;

    // Names only: the types of the contract exist.
    let _ = std::marker::PhantomData::<(
        Aabb,
        BlockEntityGhost,
        BlockGrid,
        BlockMatch,
        GhostEntityOptions,
        MeshBuilt,
        MeshGroup,
        MeshId,
        MesherOptions,
        SkullPose,
        StateValue,
        Transform,
        DimensionChanging,
        WorldJoined,
        UsePressed,
        BuildBlock,
        InventoryRequestDone,
        MouseInput,
        WindowFocus,
        AppFocusLost,
        ClientTick,
        WorldLeft,
        ProviderReady,
        ProviderGone,
        ModsLoaded,
        InputLeaseSuspended,
        InputLeaseResumed,
        InputLeaseLost,
    )>;
}
