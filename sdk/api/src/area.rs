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
    /// `Client::open_menu`.
    pub const CLIENT_OPEN_MENU: Self = Self::v1("client.open_menu");
    /// `Client::send_command`.
    pub const CLIENT_SEND_COMMAND: Self = Self::v1("client.send_command");
    /// `Hud::select_hotbar_slot`, served through the client table.
    pub const HUD_SELECT_HOTBAR: Self = Self::v1("hud.select_hotbar");
    /// `Client::note_activity`: the game's any-input handler, with no input event.
    pub const CLIENT_ACTIVITY: Self = Self::v1("client.activity");
    /// `LocalPlayer::is_alive`. Later reads are appended to the same table at a higher version.
    pub const ACTOR_READ: Self = Self::v1("actor.read");
    /// `Inventory::request_moves` and `Inventory::request_swap`, with the `InventoryRequestDone` event.
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
        self.id.starts_with("actor.")
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

    /// Whether the area is served through the overlay table.
    pub(crate) fn is_overlay(&self) -> bool {
        self.id.starts_with("overlay.")
    }
}
