/// A part of the loader's API that a game build may or may not serve, by id and version. Name it in
/// `amml_uses!` (`required area Area::KEY_INJECT;`) and ask `Ctx::supports` at run time.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Area {
    pub id: &'static str,
    pub version: u32,
}

impl Area {
    /// `Client` reads: `local_player`, `world_active`, `showing_menu`.
    pub const CLIENT_READ: Self = Self::v1("client.read");
    /// `Client::top_screen`. Version 2 gives `Screen::Hud`, `Loading`, `Toast` and `Container`.
    pub const CLIENT_TOP_SCREEN: Self = Self { id: "client.top_screen", version: 2 };
    /// `Client::hide_hud` and `Client::set_hide_hud`.
    pub const CLIENT_HUD: Self = Self::v1("client.hud");
    /// `Client::mouse_captured` and `Client::set_mouse_captured`.
    pub const CLIENT_MOUSE: Self = Self::v1("client.mouse");
    /// `Client::open_pause_screen`.
    pub const CLIENT_PAUSE: Self = Self::v1("client.pause");
    /// `Input::acquire` and the lease rules; the loader serves it on every build.
    pub const INPUT_LEASE: Self = Self::v1("input.lease");
    /// `InputLease::key_down`.
    pub const KEY_INJECT: Self = Self::v1("input.key_inject");
    /// `InputLease::button_down` and `InputLease::wheel`.
    pub const MOUSE_INJECT: Self = Self::v1("input.mouse_inject");
    /// `InputLease::turn`.
    pub const TURN: Self = Self::v1("input.turn");
    /// `InputLease::set_move`.
    pub const MOVE: Self = Self::v1("input.move");
    /// The `Window` reads and `Window::replay_focus_loss`.
    pub const WINDOW_READ: Self = Self::v1("window.read");
    /// `Window::request_size`.
    pub const WINDOW_REQUEST_SIZE: Self = Self::v1("window.request_size");
    /// `Window::clip_cursor`.
    pub const WINDOW_CURSOR_CLIP: Self = Self::v1("window.cursor_clip");
    /// The `FramePresent` surface, `DrawSurface::copy_back_buffer` and `Overlay::present_count`.
    pub const OVERLAY_SURFACE: Self = Self::v1("overlay.surface");
    /// `Overlay::screenshot`.
    pub const OVERLAY_SCREENSHOT: Self = Self::v1("overlay.screenshot");
    /// `Overlay::open_capture_stream`.
    pub const OVERLAY_CAPTURE: Self = Self::v1("overlay.capture");
    /// `Client::leave_world`.
    pub const CLIENT_LEAVE_WORLD: Self = Self::v1("client.leave_world");
    /// `Client::can_join`.
    pub const CLIENT_CAN_JOIN: Self = Self::v1("client.can_join");
    /// `Client::start_local_world`.
    pub const CLIENT_START_LOCAL_WORLD: Self = Self::v1("client.start_local_world");
    /// `Client::join_server` and the `JoinServerResult` event.
    pub const CLIENT_JOIN_SERVER: Self = Self::v1("client.join_server");
    /// `Client::join_server_direct`; its answer is the `JoinServerResult` of `CLIENT_JOIN_SERVER`.
    pub const CLIENT_JOIN_SERVER_DIRECT: Self = Self::v1("client.join_server_direct");
    /// `Client::open_menu`.
    pub const CLIENT_OPEN_MENU: Self = Self::v1("client.open_menu");
    /// `Client::on_title_screen`: whether the client shows the title screen (no world, the router's last route the
    /// main menu).
    pub const CLIENT_TITLE_SCREEN: Self = Self::v1("client.title_screen");
    /// `Client::go_to_title`: back to the title screen with the menu stack emptied.
    pub const CLIENT_GO_TO_TITLE: Self = Self::v1("client.go_to_title");
    /// `LocalPlayer::hit_result`: what the crosshair points at, from the game's own hit result.
    pub const PLAYER_HIT_RESULT: Self = Self::v1("player.hit_result");
    /// `Client::send_command`.
    pub const CLIENT_SEND_COMMAND: Self = Self::v1("client.send_command");
    /// `Hud::select_hotbar_slot`, served through the client table.
    pub const HUD_SELECT_HOTBAR: Self = Self::v1("hud.select_hotbar");
    /// `Client::replay_app_focus_lost`: the game's handler of an app focus loss that a listener cancelled.
    pub const CLIENT_FOCUS_REPLAY: Self = Self::v1("client.focus_replay");
    /// `Client::in_gameplay`: whether the game says a world is being played (`MinecraftGame::isInGame`).
    pub const CLIENT_IN_GAMEPLAY: Self = Self::v1("client.in_gameplay");
    /// `Client::note_activity`: the game's any-input handler, with no input event.
    pub const CLIENT_ACTIVITY: Self = Self::v1("client.activity");
    /// `Client::world_info`: the storage folder, level name, seed, realm flag, commands flag and connection of the
    /// joined world.
    pub const CLIENT_WORLD_INFO: Self = Self::v1("client.world_info");
    /// `LocalPlayer::is_alive` (version 1). Version 2 (`Area::ACTOR_READ.at(2)`) adds `position`, `eye_position`,
    /// `rotation` and `dimension`; version 3 (`Area::ACTOR_READ.at(3)`) adds `head_position` and `block_source`;
    /// version 4 (`Area::ACTOR_READ.at(4)`) adds `view_vector` and `aabb`.
    pub const ACTOR_READ: Self = Self::v1("actor.read");
    /// `LocalPlayer::health`.
    pub const ACTOR_HEALTH: Self = Self::v1("actor.health");
    /// `LocalPlayer::game_type`.
    pub const ACTOR_GAME_TYPE: Self = Self::v1("actor.game_type");
    /// `LocalPlayer::permission_level`.
    pub const ACTOR_PERMISSION: Self = Self::v1("actor.permission");
    /// `LocalPlayer::set_rotation`.
    pub const ACTOR_LOOK_WRITE: Self = Self::v1("actor.look_write");
    /// `GameMode::build_at` and `GameMode::build_delay`.
    pub const PLAYER_BUILD: Self = Self::v1("player.build");
    /// `GameMode::max_pick_range`.
    pub const PLAYER_PICK_RANGE: Self = Self::v1("player.pick_range");
    /// `Dimension::sky_darken`.
    pub const WORLD_SKY_DARKEN: Self = Self::v1("world.sky_darken");
    /// `BlockSource::chunk`, `LevelChunk::{load_state, read_columns}` and `Blocks::type_name`.
    pub const WORLD_CHUNK_READ: Self = Self::v1("world.chunk_read");
    /// The game's own map colours in `LevelChunk::read_columns` (`ColumnRequest::vanilla_colors`). Needs
    /// `WORLD_CHUNK_READ`.
    pub const WORLD_MAP_COLOR: Self = Self::v1("world.map_color");
    /// The biome id of each column of `LevelChunk::read_columns` and `Biomes::name`. Needs `WORLD_CHUNK_READ`. Version 2
    /// (`Area::WORLD_BIOMES.at(2)`) adds `BlockSource::biome_at`, the biome of one block, cave biomes included.
    pub const WORLD_BIOMES: Self = Self::v1("world.biomes");
    /// `BlockSource::scan_surfaces`. Needs `WORLD_CHUNK_READ`.
    pub const WORLD_SCAN_SURFACES: Self = Self::v1("world.scan_surfaces");
    /// `WorldShapes::{apply, max_batch, target_epoch}` (`Ctx::world_shapes`).
    pub const WORLD_SHAPES: Self = Self::v1("world.shapes");
    /// `BlockSource::raycast`. `RayOptions::every_block` false needs `WORLD_CHUNK_READ` too.
    pub const WORLD_RAYCAST: Self = Self::v1("world.raycast");
    /// `BlockSource::actors_in`. Version 2 (`Area::WORLD_ACTORS_IN.at(2)`) adds `BlockSource::actors_in_entries`, the
    /// same actors with the `ActorType` of each.
    pub const WORLD_ACTORS_IN: Self = Self::v1("world.actors_in");
    /// `Blocks::{from_state, rotated, name, item}`: block states as owned `BlockHandle`s.
    pub const WORLD_BLOCK_STATES: Self = Self::v1("world.block_states");
    /// `BlockSource::block` and the `Block` reads: type, name, air, built over, interactive, item and handle. The
    /// name and type id need `WORLD_CHUNK_READ` for `Blocks::type_name`.
    pub const WORLD_BLOCKS: Self = Self::v1("world.blocks");
    /// `BlockSource::kinds_in_box`: the distinct block types of a box. Needs `WORLD_CHUNK_READ` for the type names.
    pub const WORLD_KINDS_IN_BOX: Self = Self::v1("world.kinds_in_box");
    /// `WorldRenderedExt::canvas`, `WorldCanvas::camera_position` and `WorldCanvas::free` (version 1). Version 2
    /// (`Area::RENDER_WORLD_CANVAS.at(2)`) adds `WorldCanvas::build` and `WorldCanvas::draw`. Version 3
    /// (`Area::RENDER_WORLD_CANVAS.at(3)`) adds `WorldCanvas::clip_matrix` and the draw of block meshes
    /// (`MeshStyle::GHOST`).
    pub const RENDER_WORLD_CANVAS: Self = Self::v1("render.world_canvas");
    /// `WorldRenderedExt::mesher` and `BlockMesher::build`: a grid of blocks built into a ghost mesh with the game's block
    /// tessellator. Offered only with `RENDER_WORLD_CANVAS.at(3)`, which draws the mesh.
    pub const RENDER_BLOCK_MESHER: Self = Self::v1("render.block_mesher");
    /// `MesherOptions::water_tint`: water cells of a block mesh take the biome colour of their chunk. Offered only with
    /// `RENDER_BLOCK_MESHER`; without it water stays the texture's grey.
    pub const RENDER_WATER_TINT: Self = Self::v1("render.water_tint");
    /// `Ctx::block_entities` and `BlockEntityPassExt::draw_block_entity`: block entity ghosts, the game's own block
    /// actors drawn in the `BlockEntityPass`.
    pub const RENDER_BLOCK_ENTITIES: Self = Self::v1("render.block_entities");
    /// `GhostEntityOptions::chest_pair`: a chest ghost made the lead of a large chest. Offered only with
    /// `RENDER_BLOCK_ENTITIES`.
    pub const RENDER_CHEST_PAIR: Self = Self::v1("render.chest_pair");
    /// `GhostEntityOptions::skull`: a skull ghost drawn with a face and a turn. Offered only with
    /// `RENDER_BLOCK_ENTITIES`.
    pub const RENDER_SKULL_POSE: Self = Self::v1("render.skull_pose");
    /// `Camera::set_pose` and `Camera::rendered_position` of a `CameraSetup` camera.
    pub const CAMERA_POSE: Self = Self::v1("camera.pose");
    /// `Camera::projection` of a `CameraSetup` camera.
    pub const CAMERA_PROJECTION: Self = Self::v1("camera.projection");
    /// `CameraControl::set_free`, `release_free` and `free_available` (`Ctx::camera`).
    pub const CAMERA_FREE: Self = Self::v1("camera.free");
    /// `CameraControl::active_camera`: which vanilla camera the client shows the world through.
    pub const CAMERA_ACTIVE: Self = Self::v1("camera.active");
    /// `CameraControl::hide_actor`: hide the local player's body.
    pub const CAMERA_HIDE_ACTOR: Self = Self::v1("camera.hide_actor");
    /// `Options::view_distance_chunks`.
    pub const OPTIONS_VIEW_DISTANCE: Self = Self::v1("options.view_distance");
    /// `Options::get_int`, `set_int` and `release_transient`.
    pub const OPTIONS_INT: Self = Self::v1("options.int");
    /// `Time::hold`, `TimeHold::step_us` and the `Time` reads, with the `TimeStepApplied` event.
    pub const TIME_CONTROL: Self = Self::v1("time.control");
    /// `Inventory::request_moves` and `Inventory::request_swap`, with the `InventoryRequestDone` event. Version 2
    /// (`Area::INVENTORY_REQUEST.at(2)`) adds `Inventory::request_pick`.
    pub const INVENTORY_REQUEST: Self = Self::v1("inventory.request");
    /// `LocalPlayer::held_item`, `LocalPlayer::inventory` and the `ItemView` and `Inventory` reads. Version 2 adds
    /// the off hand and the block of an item; version 3 (`Area::INVENTORY_READ.at(3)`) adds `Inventory::slot`,
    /// `Inventory::slots` and the aux, max count, glint and identity of an item; version 4 (`Area::INVENTORY_READ.at(4)`)
    /// adds `ItemView::container_contents` and the `ItemDrawHandle` it gives.
    pub const INVENTORY_READ: Self = Self::v1("inventory.read");
    /// `Lighting::set_light_source`: the loader-owned held light on the terrain.
    pub const LIGHTING_TERRAIN: Self = Self::v1("lighting.terrain");
    /// The entity and block-entity light of `Lighting::set_light_source`.
    pub const LIGHTING_ENTITY: Self = Self::v1("lighting.entity");
    /// The Vibrant Visuals emitter of a `LightSource` with a block.
    pub const LIGHTING_VIBRANT: Self = Self::v1("lighting.vibrant");
    /// The `ScreenView` reads of a `ScreenRender` event: kind, size, content rectangle, pointer, input mode and UI profile.
    pub const UI_SCREEN: Self = Self::v1("ui.screen");
    /// The `UiDraw` calls of a `ScreenRender` listener: fills, outlines, images, the GUI scale. Version 2
    /// (`Area::UI_DRAW.at(2)`) adds `UiDraw::item`, which draws an `ItemDrawHandle` (`Area::INVENTORY_READ.at(4)`).
    pub const UI_DRAW: Self = Self::v1("ui.draw");
    /// `Textures`: files of the mod or the game, uploaded pixels.
    pub const UI_TEXTURES: Self = Self::v1("ui.textures");
    /// `UiDraw::text_width` and `UiDraw::text`: text in the default font of the game.
    pub const UI_TEXT: Self = Self::v1("ui.text");
    /// `Chat::register_inline_icon` and `InlineIcon`: small images that a chat line shows as one character of the
    /// default font. The `ChatMessageAdded` event is offered on its own.
    pub const UI_INLINE_ICONS: Self = Self::v1("ui.inline_icons");
    /// `SkinRef::image`: the main image of a skin of a `PlayerListUpdate` or `PlayerSkinChanged` event.
    pub const NETWORK_SKIN: Self = Self::v1("network.skin");
    /// `SkinRef::animated_images`.
    pub const NETWORK_SKIN_ANIMATED: Self = Self::v1("network.skin_animated");
    /// `SkinRef::default_geometry_name` and `SkinRef::geometry_json`.
    pub const NETWORK_SKIN_GEOMETRY: Self = Self::v1("network.skin_geometry");
    /// `SkinRef::is_persona`, `SkinRef::is_premium` and `SkinRef::resource_patch`.
    pub const NETWORK_SKIN_DETAILS: Self = Self::v1("network.skin_details");
    /// `Network::set_client_chunk_cache`: the client's chunk cache status packet, edited before it is sent.
    pub const NETWORK_CHUNK_CACHE: Self = Self::v1("network.chunk_cache");
    /// `Network::protocol_version`: the network protocol number of the build.
    pub const NETWORK_PROTOCOL: Self = Self::v1("network.protocol");

    /// The same area at table version `version`: the lowest version that serves every call the mod makes. A call
    /// appended to an area's table in a later loader names its version; a mod that makes it declares
    /// `required area Area::ACTOR_READ.at(2);`. A loader whose catalog serves a lower version reports "loader too
    /// old" for the row.
    pub const fn at(self, version: u32) -> Self {
        Self { id: self.id, version }
    }

    const fn v1(id: &'static str) -> Self {
        Self { id, version: 1 }
    }

    /// Whether the area is served through the client table.
    pub(crate) fn is_client(&self) -> bool {
        self.id.starts_with("client.") || self.id.starts_with("hud.")
    }

    /// Whether the area is served through the input table.
    pub(crate) fn is_input(&self) -> bool {
        self.id.starts_with("input.")
    }

    /// Whether the area is served through the window table.
    pub(crate) fn is_window(&self) -> bool {
        self.id.starts_with("window.")
    }

    /// Whether the area is served through the actor table.
    pub(crate) fn is_actor(&self) -> bool {
        self.id.starts_with("actor.") || self.id.starts_with("player.")
    }

    /// Whether the area is served through the inventory table.
    pub(crate) fn is_inventory(&self) -> bool {
        self.id.starts_with("inventory.")
    }

    /// Whether the area is served through the lighting table.
    pub(crate) fn is_lighting(&self) -> bool {
        self.id.starts_with("lighting.")
    }

    /// Whether the area is served through the ui table.
    pub(crate) fn is_ui(&self) -> bool {
        self.id.starts_with("ui.")
    }

    /// Whether the area is served through the network table.
    pub(crate) fn is_network(&self) -> bool {
        self.id.starts_with("network.")
    }

    /// Whether the area is served through the world table.
    pub(crate) fn is_world(&self) -> bool {
        self.id.starts_with("world.")
    }

    /// Whether the area is served through the render table.
    pub(crate) fn is_render(&self) -> bool {
        self.id.starts_with("render.")
    }

    /// Whether the area is served through the camera table.
    pub(crate) fn is_camera(&self) -> bool {
        self.id.starts_with("camera.")
    }

    /// Whether the area is served through the options table.
    pub(crate) fn is_options(&self) -> bool {
        self.id.starts_with("options.")
    }

    /// Whether the area is served through the time table.
    pub(crate) fn is_time(&self) -> bool {
        self.id.starts_with("time.")
    }

    /// Whether the area is served through the overlay table.
    pub(crate) fn is_overlay(&self) -> bool {
        self.id.starts_with("overlay.")
    }
}
