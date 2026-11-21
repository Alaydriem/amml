#![allow(unsafe_code)]

use std::ffi::c_void;

use amml_abi::{
    AMML_API_VERSION, AmmlActorTable, AmmlAmount, AmmlApi, AmmlBand, AmmlBlockHandle, AmmlCaptureFrame,
    AmmlCaptureRequest, AmmlClientFlag, AmmlClientTable, AmmlColor, AmmlContainerAction, AmmlContainerEntry,
    AmmlCoreTable, AmmlDirKind, AmmlEventsTable, AmmlHand, AmmlInputBlocks, AmmlInputMode, AmmlInputTable,
    AmmlInventoryMove, AmmlInventoryTable, AmmlItemDraw, AmmlItemDrawFlags, AmmlItemSnapshot, AmmlLeaseMode,
    AmmlLeaseRequest, AmmlLeaseState, AmmlLightSource, AmmlLightingTable, AmmlLogLevel, AmmlMenu, AmmlModifiers,
    AmmlMouseButton, AmmlMoveIntent, AmmlNetworkTable, AmmlOverlayTable, AmmlPoint, AmmlRect, AmmlRegistryTable,
    AmmlRequestOptions, AmmlRotation, AmmlScreen, AmmlScreenKind, AmmlServices, AmmlShotLayers, AmmlSize, AmmlSlotAt,
    AmmlStartWorldOutcome, AmmlStatus, AmmlStr, AmmlTextAlign, AmmlTextureId, AmmlTextureSource, AmmlUiProfile,
    AmmlUiTable, AmmlUv, AmmlVec3, AmmlWindowTable, EventCallbackFn,
};
use amml_abi::{AmmlChatField, AmmlImageView, AmmlInlineIcon};
use amml_runtime::{InjectedOp, LeaseTerms};

use crate::testing::{
    FakeActivity, FakeChatBook, FakeClient, FakeItem, FakeItemBook, FakeListener, FakeNetworkTable, FakeScreen,
    FakeState, FakeTextureBook, FakeUi, FakeWindow,
};
use crate::{Area, InventoryMove, RequestOptions, Screen, ScreenKind, StartWorldOutcome, TextureId};

/// A real v3 loader table whose functions call into the `FakeState` of the calling thread. The
/// tables are boxed so the pointers between them stay valid while this value lives.
pub(crate) struct FakeTables {
    api: Box<AmmlApi>,
    _services: Box<AmmlServices>,
    _events: Box<AmmlEventsTable>,
    _core: Box<AmmlCoreTable>,
    _registry: Box<AmmlRegistryTable>,
    _client: Box<AmmlClientTable>,
    _input: Box<AmmlInputTable>,
    _window: Box<AmmlWindowTable>,
    _overlay: Box<AmmlOverlayTable>,
    _actor: Box<AmmlActorTable>,
    _inventory: Box<AmmlInventoryTable>,
    _lighting: Box<AmmlLightingTable>,
    _ui: Box<AmmlUiTable>,
    _network: Box<AmmlNetworkTable>,
}

impl FakeTables {
    const TABLE_VERSION: u32 = 1;
    // The longest text a text call takes, as in the loader.
    const TEXT_MAX_BYTES: usize = 256;
    // The fake dimension handle is this far after the fake player handle.
    const DIMENSION_OFFSET: usize = 0x10;

    /// Tables whose services table has `services_size`: `SIZE_V4` points to every area table, `SIZE_V3` to all
    /// but the actor table, `SIZE_V2`
    /// to the client and input tables only, `SIZE_V1` to none, as a loader before them.
    pub(crate) fn new(services_size: u32) -> Self {
        Self::with_inventory(services_size, AmmlInventoryTable::SIZE_V6)
    }

    /// `new` with an inventory table of `inventory_size`: `SIZE_V4` has no `request_moves` and `SIZE_V5` no
    /// `item_change_stamp`, as a loader before them.
    pub(crate) fn with_inventory(services_size: u32, inventory_size: u32) -> Self {
        let events = Box::new(AmmlEventsTable {
            size: std::mem::size_of::<AmmlEventsTable>() as u32,
            version: Self::TABLE_VERSION,
            listen: Self::listen,
            unlisten: Self::unlisten,
        });
        let core = Box::new(AmmlCoreTable {
            size: AmmlCoreTable::SIZE_V1,
            version: Self::TABLE_VERSION,
            log: Self::log_fields,
            dir: Self::dir,
            available: Self::available,
            game_alive: Self::game_alive,
        });
        let registry = Box::new(AmmlRegistryTable {
            size: std::mem::size_of::<AmmlRegistryTable>() as u32,
            version: Self::TABLE_VERSION,
            provide: Self::provide,
            require: Self::require,
        });
        let client = Box::new(AmmlClientTable {
            size: AmmlClientTable::SIZE_V3,
            version: 2,
            current: Self::client_current,
            local_player: Self::client_local_player,
            flag: Self::client_flag,
            top_screen: Self::client_top_screen,
            set_flag: Self::client_set_flag,
            open_pause_screen: Self::client_open_pause_screen,
            leave_world: Some(Self::client_leave_world),
            can_join: Some(Self::client_can_join),
            start_local_world: Some(Self::client_start_local_world),
            join_server: Some(Self::client_join_server),
            open_menu: Some(Self::client_open_menu),
            send_command: Some(Self::client_send_command),
            select_hotbar_slot: Some(Self::client_select_hotbar_slot),
            note_activity: Some(Self::client_note_activity),
        });
        let input = Box::new(AmmlInputTable {
            size: AmmlInputTable::SIZE_V2,
            version: 2,
            modifiers: Self::input_modifiers,
            is_down: Self::input_is_down,
            acquire: Self::input_acquire,
            lease_state: Self::input_lease_state,
            key_down: Self::input_key_down,
            button_down: Self::input_button_down,
            release_hold: Self::input_release_hold,
            wheel: Self::input_wheel,
            turn: Self::input_turn,
            set_move: Self::input_set_move,
            release_all: Self::input_release_all,
            end_lease: Self::input_end_lease,
            set_ranking: Self::input_set_ranking,
            set_coexist: Some(Self::input_set_coexist),
        });
        let window = Box::new(AmmlWindowTable {
            size: AmmlWindowTable::SIZE_V1,
            version: Self::TABLE_VERSION,
            client_size: Self::window_client_size,
            buffer_size: Self::window_buffer_size,
            cursor_pos: Self::window_cursor_pos,
            to_buffer: Self::window_to_buffer,
            has_focus: Self::window_has_focus,
            is_foreground: Self::window_has_focus,
            dpi: Self::window_dpi,
            request_size: Self::window_request_size,
            replay_focus_loss: Self::window_replay_focus_loss,
        });
        let overlay = Box::new(AmmlOverlayTable {
            size: AmmlOverlayTable::SIZE_V1,
            version: Self::TABLE_VERSION,
            screenshot: Self::overlay_screenshot,
            open_capture: Self::overlay_open_capture,
            capture_take: Self::overlay_capture_take,
            capture_give_back: Self::overlay_capture_give_back,
            capture_free_slots: Self::overlay_capture_free_slots,
            close_capture: Self::overlay_close_capture,
            present_count: Self::overlay_present_count,
            copy_back_buffer: Self::overlay_copy_back_buffer,
        });
        let actor = Box::new(AmmlActorTable {
            size: AmmlActorTable::SIZE_V2,
            version: Self::TABLE_VERSION,
            is_alive: Self::actor_is_alive,
            position: Some(Self::actor_position),
            eye_position: Some(Self::actor_eye_position),
            rotation: Some(Self::actor_rotation),
            dimension: Some(Self::actor_dimension),
            dimension_id: Some(Self::actor_dimension_id),
        });
        let v2 = services_size >= AmmlServices::SIZE_V2;
        let v3 = services_size >= AmmlServices::SIZE_V3;
        let inventory = Box::new(AmmlInventoryTable {
            size: inventory_size,
            version: match inventory_size {
                size if size >= AmmlInventoryTable::SIZE_V6 => 6,
                size if size >= AmmlInventoryTable::SIZE_V5 => 5,
                _ => 4,
            },
            held_item: Self::inventory_held_item,
            selected_slot: Self::inventory_selected_slot,
            item_read: Self::inventory_item_read,
            item_name: Self::inventory_item_name,
            item_block: Some(Self::inventory_item_block),
            slot_item: Some(Self::inventory_slot_item),
            contents_open: Some(Self::inventory_contents_open),
            contents_entry: Some(Self::inventory_contents_entry),
            contents_name: Some(Self::inventory_contents_name),
            contents_close: Some(Self::inventory_contents_close),
            item_draw_release: Some(Self::inventory_item_draw_release),
            request_moves: (inventory_size >= AmmlInventoryTable::SIZE_V5).then_some(Self::inventory_request_moves),
            item_change_stamp: (inventory_size >= AmmlInventoryTable::SIZE_V6)
                .then_some(Self::inventory_item_change_stamp as amml_abi::InventoryItemChangeStampFn),
        });
        let lighting = Box::new(AmmlLightingTable {
            size: AmmlLightingTable::SIZE_V1,
            version: Self::TABLE_VERSION,
            set_light_source: Self::lighting_set_light_source,
        });
        let ui = Box::new(AmmlUiTable {
            size: AmmlUiTable::SIZE_V6,
            version: 6,
            view_kind: Self::ui_view_kind,
            view_size: Self::ui_view_size,
            view_content_rect: Self::ui_view_content_rect,
            view_pointer: Self::ui_view_pointer,
            view_input_mode: Self::ui_view_input_mode,
            view_ui_profile: Self::ui_view_ui_profile,
            view_container: Self::ui_view_container,
            draw_fill: Some(Self::ui_draw_fill),
            draw_outline: Some(Self::ui_draw_outline),
            draw_image: Some(Self::ui_draw_image),
            gui_scale: Some(Self::ui_gui_scale),
            can_draw: Some(Self::ui_can_draw),
            texture_load: Some(Self::ui_texture_load),
            texture_upload_rgba: Some(Self::ui_texture_upload_rgba),
            texture_ready: Some(Self::ui_texture_ready),
            texture_unload: Some(Self::ui_texture_unload),
            container_slot_count: Some(Self::ui_container_slot_count),
            container_item: Some(Self::ui_container_item),
            container_cursor: Some(Self::ui_container_cursor),
            container_action: Some(Self::ui_container_action),
            container_swap: Some(Self::ui_container_swap),
            draw_item: Some(Self::ui_draw_item),
            text_width: Some(Self::ui_text_width),
            draw_text: Some(Self::ui_draw_text),
            chat_text: Some(Self::ui_chat_text),
            chat_set_text: Some(Self::ui_chat_set_text),
            chat_icons_active: Some(Self::ui_chat_icons_active),
            icon_register: Some(Self::ui_icon_register),
            icon_update: Some(Self::ui_icon_update),
            icon_release: Some(Self::ui_icon_release),
        });
        let v4 = services_size >= AmmlServices::SIZE_V4;
        let v5 = services_size >= AmmlServices::SIZE_V5;
        let v6 = services_size >= AmmlServices::SIZE_V6;
        let v7 = services_size >= AmmlServices::SIZE_V7;
        let v8 = services_size >= AmmlServices::SIZE_V8;
        let network = Box::new(FakeNetworkTable::table());
        let services = Box::new(AmmlServices {
            size: services_size,
            version: if v8 {
                8
            } else if v7 {
                7
            } else if v6 {
                6
            } else if v5 {
                5
            } else if v4 {
                4
            } else if v3 {
                3
            } else if v2 {
                2
            } else {
                1
            },
            events: &raw const *events,
            core: &raw const *core,
            registry: &raw const *registry,
            client: if v2 { &raw const *client } else { std::ptr::null() },
            input: if v2 { &raw const *input } else { std::ptr::null() },
            window: if v3 { &raw const *window } else { std::ptr::null() },
            overlay: if v3 { &raw const *overlay } else { std::ptr::null() },
            actor: if v4 { &raw const *actor } else { std::ptr::null() },
            inventory: if v5 { &raw const *inventory } else { std::ptr::null() },
            lighting: if v6 { &raw const *lighting } else { std::ptr::null() },
            ui: if v7 { &raw const *ui } else { std::ptr::null() },
            network: if v8 { &raw const *network } else { std::ptr::null() },
        });
        let api = Box::new(AmmlApi {
            size: AmmlApi::SIZE_V3,
            version: AMML_API_VERSION,
            module_base: 0,
            resolve: Self::resolve,
            hook: Self::hook,
            unhook: Self::unhook,
            log: Self::log,
            heap_new: Self::heap_new,
            heap_delete: Self::heap_delete,
            mod_dir: Self::mod_dir,
            services: &raw const *services,
        });
        Self {
            api,
            _services: services,
            _events: events,
            _core: core,
            _registry: registry,
            _client: client,
            _input: input,
            _window: window,
            _overlay: overlay,
            _actor: actor,
            _inventory: inventory,
            _lighting: lighting,
            _ui: ui,
            _network: network,
        }
    }

    pub(crate) fn api(&self) -> *const AmmlApi {
        &raw const *self.api
    }

    // Reads a view for the call; `None` for invalid text.
    fn text<'a>(view: AmmlStr) -> Option<&'a str> {
        // SAFETY: the caller of every table function promises its views are valid for the call.
        unsafe { view.as_str() }.ok()
    }

    unsafe extern "C" fn resolve(_: AmmlStr, _: *mut u64) -> AmmlStatus {
        AmmlStatus::MISSING_ENTRY
    }

    unsafe extern "C" fn hook(
        _: *const c_void,
        _: *const c_void,
        _: i32,
        _: *mut *const c_void,
        _: *mut u64,
    ) -> AmmlStatus {
        AmmlStatus::FAILED
    }

    unsafe extern "C" fn unhook(_: u64) -> AmmlStatus {
        AmmlStatus::FAILED
    }

    unsafe extern "C" fn log(_: AmmlStr, level: AmmlLogLevel, message: AmmlStr) {
        if let (Some(state), Some(message)) = (FakeState::current(), Self::text(message)) {
            state.log(level, message, &[]);
        }
    }

    unsafe extern "C" fn heap_new(_: usize) -> *mut u8 {
        std::ptr::null_mut()
    }

    unsafe extern "C" fn heap_delete(_: *mut u8, _: usize) {}

    unsafe extern "C" fn mod_dir(_: AmmlStr, kind: u8, out: *mut AmmlStr) -> AmmlStatus {
        // SAFETY: the same contract as `dir`.
        unsafe { Self::dir(AmmlStr::EMPTY, AmmlDirKind(kind), out) }
    }

    unsafe extern "C" fn listen(
        event: AmmlStr,
        version: u32,
        band: AmmlBand,
        required: u8,
        callback: Option<EventCallbackFn>,
        user: *mut c_void,
        handle: *mut u64,
    ) -> AmmlStatus {
        let Some(state) = FakeState::current() else {
            return AmmlStatus::FAILED;
        };
        let (Some(callback), Some(event)) = (callback, Self::text(event)) else {
            return AmmlStatus::INVALID_ARGUMENT;
        };
        if handle.is_null() {
            return AmmlStatus::INVALID_ARGUMENT;
        }
        let listener = FakeListener { callback, user: user.expose_provenance() };
        match state.listen(event, version, band.0, required != 0, listener) {
            Ok(id) => {
                // SAFETY: `handle` is non-null and the caller promises it is writable.
                unsafe { handle.write(id) };
                AmmlStatus::OK
            }
            Err(status) => status,
        }
    }

    unsafe extern "C" fn unlisten(handle: u64) -> AmmlStatus {
        FakeState::current().map_or(AmmlStatus::FAILED, |state| state.unlisten(handle))
    }

    unsafe extern "C" fn log_fields(
        _: AmmlStr,
        level: AmmlLogLevel,
        message: AmmlStr,
        keys: *const AmmlStr,
        values: *const AmmlStr,
        count: usize,
    ) {
        let (Some(state), Some(message)) = (FakeState::current(), Self::text(message)) else {
            return;
        };
        let mut fields = Vec::new();
        if count > 0 && !keys.is_null() && !values.is_null() {
            // SAFETY: the caller promises both arrays hold `count` views valid for the call.
            let (keys, values) =
                unsafe { (std::slice::from_raw_parts(keys, count), std::slice::from_raw_parts(values, count)) };
            for (key, value) in keys.iter().zip(values) {
                if let (Some(key), Some(value)) = (Self::text(*key), Self::text(*value)) {
                    fields.push((key, value));
                }
            }
        }
        state.log(level, message, &fields);
    }

    unsafe extern "C" fn dir(_: AmmlStr, kind: AmmlDirKind, out: *mut AmmlStr) -> AmmlStatus {
        let Some(state) = FakeState::current() else {
            return AmmlStatus::FAILED;
        };
        if out.is_null() {
            return AmmlStatus::INVALID_ARGUMENT;
        }
        match state.dir(kind) {
            Some(view) => {
                // SAFETY: `out` is non-null and the caller promises it is writable.
                unsafe { out.write(view) };
                AmmlStatus::OK
            }
            None => AmmlStatus::NOT_AVAILABLE,
        }
    }

    unsafe extern "C" fn available(event: AmmlStr, version: u32) -> AmmlStatus {
        let (Some(state), Some(event)) = (FakeState::current(), Self::text(event)) else {
            return AmmlStatus::FAILED;
        };
        if state.bus().is_available(event, version) || state.area_offered(event, version) {
            AmmlStatus::OK
        } else {
            AmmlStatus::NOT_AVAILABLE
        }
    }

    unsafe extern "C" fn game_alive() -> u8 {
        FakeState::current().map_or(0, |state| u8::from(state.gate().get()))
    }

    unsafe extern "C" fn provide(name: AmmlStr, major: u16, minor: u16, table: *const c_void) -> AmmlStatus {
        let (Some(state), Some(name)) = (FakeState::current(), Self::text(name)) else {
            return AmmlStatus::FAILED;
        };
        if table.is_null() {
            return AmmlStatus::INVALID_ARGUMENT;
        }
        state.provide(name, major, minor, table.expose_provenance())
    }

    unsafe extern "C" fn require(
        name: AmmlStr,
        major: u16,
        min_minor: u16,
        out_table: *mut *const c_void,
        out_minor: *mut u16,
    ) -> AmmlStatus {
        let (Some(state), Some(name)) = (FakeState::current(), Self::text(name)) else {
            return AmmlStatus::FAILED;
        };
        if out_table.is_null() || out_minor.is_null() {
            return AmmlStatus::INVALID_ARGUMENT;
        }
        match state.require(name, major, min_minor) {
            Some((table, minor)) => {
                // SAFETY: both outputs are non-null and the caller promises they are writable.
                unsafe {
                    out_table.write(std::ptr::with_exposed_provenance::<c_void>(table));
                    out_minor.write(minor);
                }
                AmmlStatus::OK
            }
            None => AmmlStatus::NOT_AVAILABLE,
        }
    }

    // Writes `value` to a non-null `out`; `INVALID_ARGUMENT` for a null one.
    fn write<T>(out: *mut T, value: T) -> AmmlStatus {
        if out.is_null() {
            return AmmlStatus::INVALID_ARGUMENT;
        }
        // SAFETY: `out` is non-null and the caller of every table function promises it is writable.
        unsafe { out.write(value) };
        AmmlStatus::OK
    }

    fn status(result: Result<(), AmmlStatus>) -> AmmlStatus {
        match result {
            Ok(()) => AmmlStatus::OK,
            Err(status) => status,
        }
    }

    unsafe extern "C" fn client_current(out: *mut *const c_void) -> AmmlStatus {
        let Some(state) = FakeState::current() else {
            return AmmlStatus::FAILED;
        };
        if !state.gate().get() {
            return AmmlStatus::NOT_READY;
        }
        Self::write(out, std::ptr::with_exposed_provenance(state.current_client()))
    }

    unsafe extern "C" fn client_local_player(client: *const c_void, out: *mut *const c_void) -> AmmlStatus {
        let Some(state) = FakeState::current() else {
            return AmmlStatus::FAILED;
        };
        match state.client_check(client.addr(), Area::CLIENT_READ) {
            Ok(fake) => {
                let player = if fake.local_player { state.player_handle() } else { 0 };
                Self::write(out, std::ptr::with_exposed_provenance(player))
            }
            Err(status) => status,
        }
    }

    unsafe extern "C" fn client_flag(client: *const c_void, flag: AmmlClientFlag, out: *mut u8) -> AmmlStatus {
        let Some(state) = FakeState::current() else {
            return AmmlStatus::FAILED;
        };
        let (area, read): (Area, fn(&FakeClient) -> Option<bool>) = match flag {
            AmmlClientFlag::WORLD_ACTIVE => (Area::CLIENT_READ, |fake| Some(fake.world_active)),
            AmmlClientFlag::SHOWING_MENU => (Area::CLIENT_READ, |fake| Some(fake.showing_menu)),
            AmmlClientFlag::HIDE_HUD => (Area::CLIENT_HUD, |fake| Some(fake.hide_hud)),
            AmmlClientFlag::MOUSE_CAPTURED => (Area::CLIENT_MOUSE, |fake| fake.mouse_captured),
            _ => return AmmlStatus::INVALID_ARGUMENT,
        };
        match state.client_check(client.addr(), area) {
            Ok(fake) => match read(&fake) {
                Some(value) => Self::write(out, u8::from(value)),
                None => AmmlStatus::NOT_AVAILABLE,
            },
            Err(status) => status,
        }
    }

    unsafe extern "C" fn client_top_screen(client: *const c_void, out: *mut AmmlScreen) -> AmmlStatus {
        let Some(state) = FakeState::current() else {
            return AmmlStatus::FAILED;
        };
        match state.client_check(client.addr(), Area::CLIENT_TOP_SCREEN) {
            Ok(fake) => Self::write(out, AmmlScreen(fake.top_screen as u8)),
            Err(status) => status,
        }
    }

    unsafe extern "C" fn client_set_flag(
        _: AmmlStr,
        client: *const c_void,
        flag: AmmlClientFlag,
        value: u8,
    ) -> AmmlStatus {
        let Some(state) = FakeState::current() else {
            return AmmlStatus::FAILED;
        };
        let value = value != 0;
        let area = match flag {
            AmmlClientFlag::HIDE_HUD => Area::CLIENT_HUD,
            AmmlClientFlag::MOUSE_CAPTURED => Area::CLIENT_MOUSE,
            _ => return AmmlStatus::INVALID_ARGUMENT,
        };
        if let Err(status) = state.client_check(client.addr(), area) {
            return status;
        }
        if flag == AmmlClientFlag::HIDE_HUD {
            state.write_client(|fake| fake.hide_hud = value, format!("hide_hud={value}"));
        } else {
            state.write_client(|fake| fake.mouse_captured = Some(value), format!("mouse_captured={value}"));
        }
        AmmlStatus::OK
    }

    unsafe extern "C" fn client_open_pause_screen(client: *const c_void) -> AmmlStatus {
        let Some(state) = FakeState::current() else {
            return AmmlStatus::FAILED;
        };
        let fake = match state.client_check(client.addr(), Area::CLIENT_PAUSE) {
            Ok(fake) => fake,
            Err(status) => return status,
        };
        if !fake.pause_on_focus_lost {
            return AmmlStatus::REFUSED;
        }
        state.write_client(|fake| fake.top_screen = Screen::Pause, "open_pause_screen".to_string());
        AmmlStatus::OK
    }

    // The primary client's local player is the only handle it reads; the area must be offered.
    // The local player check of an inventory call: the client checks of `inventory.read`, then `player` must be the
    // fake's local player.
    fn inventory_check(state: &FakeState, player: *const c_void) -> Result<(), AmmlStatus> {
        let fake = state.client_check(state.client_handle(), Area::INVENTORY_READ)?;
        if !fake.local_player || player.addr() != state.player_handle() {
            return Err(AmmlStatus::REFUSED);
        }
        Ok(())
    }

    // The held stack for `stack`, a handle the fake gave; `REFUSED` when the fake holds no stack in that hand now.
    fn held_stack(state: &FakeState, player: *const c_void, stack: *const c_void) -> Result<FakeItem, AmmlStatus> {
        if player.is_null() {
            return Self::lent_stack(state, stack);
        }
        Self::inventory_check(state, player)?;
        let held = if stack.addr() == state.stack_handle() {
            state.inventory().held
        } else if stack.addr() == state.off_stack_handle() {
            state.lighting().off_hand()
        } else if state.ui().slot_lend_valid(stack.addr()) {
            state.slot_of_handle(stack.addr()).and_then(|index| state.inventory().slots[index])
        } else {
            None
        };
        held.ok_or(AmmlStatus::REFUSED)
    }

    // The stack of a container screen the loader lent in the running `ScreenRender` call, read with a null player.
    fn lent_stack(state: &FakeState, stack: *const c_void) -> Result<FakeItem, AmmlStatus> {
        if !state.area_offered(Area::INVENTORY_READ.id, Area::INVENTORY_READ.version) {
            return Err(AmmlStatus::NOT_AVAILABLE);
        }
        if !state.gate().get() {
            return Err(AmmlStatus::NOT_READY);
        }
        state.ui().item_at_handle(stack.addr()).ok_or(AmmlStatus::REFUSED)
    }

    unsafe extern "C" fn inventory_item_block(
        module: AmmlStr,
        player: *const c_void,
        stack: *const c_void,
        out_emission: *mut u8,
        out_block: *mut AmmlBlockHandle,
    ) -> AmmlStatus {
        let Some(state) = FakeState::current() else {
            return AmmlStatus::FAILED;
        };
        // SAFETY: the caller promises `module` is valid for the call.
        if unsafe { module.as_str() }.is_err() || out_emission.is_null() {
            return AmmlStatus::INVALID_ARGUMENT;
        }
        let item = match Self::held_stack(&state, player, stack) {
            Ok(item) => item,
            Err(status) => return status,
        };
        let epoch = state.epoch().get();
        if epoch == 0 {
            return AmmlStatus::NOT_READY;
        }
        let (block, emission) = state.lighting().block_of(&item, epoch);
        // SAFETY: `out_emission` is non-null and the caller promises it is writable.
        unsafe { out_emission.write(emission) };
        Self::write(out_block, block)
    }

    unsafe extern "C" fn lighting_set_light_source(module: AmmlStr, source: *const AmmlLightSource) -> AmmlStatus {
        let Some(state) = FakeState::current() else {
            return AmmlStatus::FAILED;
        };
        // SAFETY: the caller promises `module` is valid for the call.
        if unsafe { module.as_str() }.is_err() {
            return AmmlStatus::INVALID_ARGUMENT;
        }
        if !state.area_offered(Area::LIGHTING_TERRAIN.id, Area::LIGHTING_TERRAIN.version) {
            return AmmlStatus::NOT_AVAILABLE;
        }
        if !state.gate().get() {
            return AmmlStatus::NOT_READY;
        }
        if state.lighting().refused() {
            return AmmlStatus::REFUSED;
        }
        let light = if source.is_null() {
            None
        } else {
            // SAFETY: a non-null `source` is a live light source for the call, as the caller promises.
            let light = unsafe { source.read() };
            let finite = [light.position.x, light.position.y, light.position.z].iter().all(|c| c.is_finite());
            if light.size < AmmlLightSource::SIZE_V1 || !(1..=15).contains(&light.level) || !finite {
                return AmmlStatus::INVALID_ARGUMENT;
            }
            if light.has_block != 0 && !state.lighting().is_current(light.block, state.epoch().get()) {
                return AmmlStatus::STALE;
            }
            Some(light)
        };
        state.lighting().set_light(light);
        AmmlStatus::OK
    }

    // The loader checks before a ui read: the output, the area, the gate and the running `ScreenRender` view.
    // Gives the screen of that view.
    fn ui_screen(view: u64, out_is_null: bool) -> Result<FakeScreen, AmmlStatus> {
        if out_is_null {
            return Err(AmmlStatus::INVALID_ARGUMENT);
        }
        let Some(state) = FakeState::current() else {
            return Err(AmmlStatus::FAILED);
        };
        if !state.area_offered(Area::UI_SCREEN.id, Area::UI_SCREEN.version) {
            return Err(AmmlStatus::NOT_AVAILABLE);
        }
        if !state.gate().get() {
            return Err(AmmlStatus::NOT_READY);
        }
        state.ui().screen_of(view).ok_or(AmmlStatus::REFUSED)
    }

    // Writes the field `read` gives of the screen of `view` to `out`; a field that is `None` is `NOT_AVAILABLE`.
    fn ui_write<T>(view: u64, out: *mut T, read: impl FnOnce(FakeScreen) -> Option<T>) -> AmmlStatus {
        match Self::ui_screen(view, out.is_null()).map(read) {
            Ok(Some(value)) => Self::write(out, value),
            Ok(None) => AmmlStatus::NOT_AVAILABLE,
            Err(status) => status,
        }
    }

    unsafe extern "C" fn ui_view_kind(view: u64, out: *mut AmmlScreenKind) -> AmmlStatus {
        Self::ui_write(view, out, |screen| Some(AmmlScreenKind::from(screen.kind)))
    }

    unsafe extern "C" fn ui_view_size(view: u64, out: *mut AmmlSize) -> AmmlStatus {
        Self::ui_write(view, out, |screen| screen.size.map(AmmlSize::from))
    }

    unsafe extern "C" fn ui_view_content_rect(view: u64, out: *mut AmmlRect) -> AmmlStatus {
        Self::ui_write(view, out, |screen| screen.content_rect.map(AmmlRect::from))
    }

    unsafe extern "C" fn ui_view_pointer(view: u64, out: *mut AmmlPoint) -> AmmlStatus {
        Self::ui_write(view, out, |screen| screen.pointer.map(AmmlPoint::from))
    }

    unsafe extern "C" fn ui_view_input_mode(view: u64, out: *mut AmmlInputMode) -> AmmlStatus {
        Self::ui_write(view, out, |screen| screen.input_mode.map(AmmlInputMode::from))
    }

    unsafe extern "C" fn ui_view_ui_profile(view: u64, out: *mut AmmlUiProfile) -> AmmlStatus {
        Self::ui_write(view, out, |screen| screen.ui_profile.map(AmmlUiProfile::from))
    }

    unsafe extern "C" fn ui_view_container(view: u64, out_screen: *mut u64) -> AmmlStatus {
        let screen_id = FakeState::current().and_then(|state| state.ui().screen_id_of(view)).unwrap_or(0);
        Self::ui_write(view, out_screen, |screen| Some(if screen.kind == ScreenKind::Other { 0 } else { screen_id }))
    }

    // The loader checks before a container call: the area at version 2, the gate, the running `ScreenRender` view and
    // that its screen is a container screen. Gives the state and the screen of that view.
    fn ui_container_check(view: u64) -> Result<(std::rc::Rc<FakeState>, FakeScreen), AmmlStatus> {
        let Some(state) = FakeState::current() else {
            return Err(AmmlStatus::FAILED);
        };
        let area = Area::UI_SCREEN.at(2);
        if !state.area_offered(area.id, area.version) {
            return Err(AmmlStatus::NOT_AVAILABLE);
        }
        if !state.gate().get() {
            return Err(AmmlStatus::NOT_READY);
        }
        let screen = state.ui().screen_of(view).ok_or(AmmlStatus::REFUSED)?;
        if screen.kind == ScreenKind::Other {
            return Err(AmmlStatus::REFUSED);
        }
        Ok((state, screen))
    }

    // Whether `slot` names a collection of the API and an index in the range every call takes.
    fn slot_is_valid(slot: AmmlSlotAt) -> bool {
        slot.collection < AmmlSlotAt::OTHER && (0..=AmmlSlotAt::MAX_INDEX).contains(&slot.index)
    }

    fn collection_name(collection: u32) -> &'static str {
        match collection {
            AmmlSlotAt::CONTAINER => "container",
            AmmlSlotAt::HOTBAR => "hotbar",
            _ => "inventory",
        }
    }

    fn slot_text(slot: AmmlSlotAt) -> String {
        format!("{} {}", Self::collection_name(slot.collection), slot.index)
    }

    unsafe extern "C" fn ui_container_slot_count(view: u64, collection: u32, out: *mut u32) -> AmmlStatus {
        if out.is_null() {
            return AmmlStatus::INVALID_ARGUMENT;
        }
        match Self::ui_container_check(view) {
            Ok((_, screen)) => match screen.slot_count(collection).and_then(|count| u32::try_from(count).ok()) {
                Some(count) => Self::write(out, count),
                None => AmmlStatus::NOT_AVAILABLE,
            },
            Err(status) => status,
        }
    }

    // The stack handle the loader lends for a container stack needs `inventory.read` at version 3.
    fn ui_stack_check(state: &FakeState) -> Result<(), AmmlStatus> {
        let area = Area::INVENTORY_READ.at(3);
        if state.area_offered(area.id, area.version) { Ok(()) } else { Err(AmmlStatus::NOT_AVAILABLE) }
    }

    unsafe extern "C" fn ui_container_item(view: u64, slot: AmmlSlotAt, out_stack: *mut *const c_void) -> AmmlStatus {
        if out_stack.is_null() || !Self::slot_is_valid(slot) {
            return AmmlStatus::INVALID_ARGUMENT;
        }
        let result = Self::ui_container_check(view).and_then(|(state, screen)| {
            Self::ui_stack_check(&state)?;
            let item = screen.slot(slot.collection, slot.index).ok_or(AmmlStatus::INVALID_ARGUMENT)?;
            let handle = item.map_or(0, |_| FakeUi::slot_handle(slot.collection, slot.index));
            state.ui().lend(handle);
            Ok(handle)
        });
        match result {
            Ok(handle) => Self::write(out_stack, std::ptr::with_exposed_provenance(handle)),
            Err(status) => status,
        }
    }

    unsafe extern "C" fn ui_container_cursor(view: u64, out_stack: *mut *const c_void) -> AmmlStatus {
        if out_stack.is_null() {
            return AmmlStatus::INVALID_ARGUMENT;
        }
        let result = Self::ui_container_check(view).and_then(|(state, screen)| {
            Self::ui_stack_check(&state)?;
            let handle = screen.cursor.map_or(0, |_| FakeUi::cursor_handle());
            state.ui().lend(handle);
            Ok(handle)
        });
        match result {
            Ok(handle) => Self::write(out_stack, std::ptr::with_exposed_provenance(handle)),
            Err(status) => status,
        }
    }

    unsafe extern "C" fn ui_container_action(
        view: u64,
        action: AmmlContainerAction,
        slot: AmmlSlotAt,
        amount: AmmlAmount,
    ) -> AmmlStatus {
        let known = (AmmlContainerAction::AUTO_PLACE.0..=AmmlContainerAction::DROP_CURSOR.0).contains(&action.0);
        let slot_used = action != AmmlContainerAction::DROP_CURSOR;
        let amount_text = match amount.kind {
            AmmlAmount::KIND_WHOLE => Some("whole".to_string()),
            AmmlAmount::KIND_ONE => Some("one".to_string()),
            AmmlAmount::KIND_COUNT if amount.count > 0 => Some(amount.count.to_string()),
            _ => None,
        };
        if !known || (slot_used && !Self::slot_is_valid(slot)) || amount_text.is_none() {
            return AmmlStatus::INVALID_ARGUMENT;
        }
        let amount_text = amount_text.unwrap_or_default();
        let amount = match amount.kind {
            AmmlAmount::KIND_ONE => crate::Amount::One,
            AmmlAmount::KIND_COUNT => crate::Amount::Count(amount.count),
            _ => crate::Amount::Whole,
        };
        let result = Self::ui_container_check(view).and_then(|(state, screen)| {
            if slot_used && screen.slot(slot.collection, slot.index).is_none() {
                return Err(AmmlStatus::INVALID_ARGUMENT);
            }
            let line = match action {
                AmmlContainerAction::AUTO_PLACE => format!("auto_place {}", Self::slot_text(slot)),
                AmmlContainerAction::TAKE_ALL => format!("take_all {}", Self::slot_text(slot)),
                AmmlContainerAction::PLACE_ALL => format!("place_all {}", Self::slot_text(slot)),
                AmmlContainerAction::DROP_SLOT => format!("drop_slot {} {amount_text}", Self::slot_text(slot)),
                _ => format!("drop_cursor {amount_text}"),
            };
            state.ui().record_action(line);
            state.ui().change(view, |screen| match action {
                AmmlContainerAction::AUTO_PLACE => screen.auto_place(slot),
                AmmlContainerAction::TAKE_ALL => screen.take_all(slot),
                AmmlContainerAction::PLACE_ALL => screen.place_all(slot),
                AmmlContainerAction::DROP_SLOT => screen.drop_slot(slot, amount),
                _ => screen.drop_cursor(amount),
            });
            Ok(())
        });
        Self::status(result)
    }

    unsafe extern "C" fn ui_container_swap(
        view: u64,
        a: AmmlSlotAt,
        b: AmmlSlotAt,
        out_swapped: *mut u8,
    ) -> AmmlStatus {
        if out_swapped.is_null() || !Self::slot_is_valid(a) || !Self::slot_is_valid(b) {
            return AmmlStatus::INVALID_ARGUMENT;
        }
        let result = Self::ui_container_check(view).and_then(|(state, screen)| {
            for slot in [a, b] {
                if screen.slot(slot.collection, slot.index).is_none() {
                    return Err(AmmlStatus::INVALID_ARGUMENT);
                }
            }
            state.ui().record_action(format!("swap {} {}", Self::slot_text(a), Self::slot_text(b)));
            Ok(state.ui().change(view, |screen| screen.swap(a, b)).unwrap_or(false))
        });
        match result {
            Ok(swapped) => Self::write(out_swapped, u8::from(swapped)),
            Err(status) => status,
        }
    }

    // The loader checks before a draw call: the area, the gate and the running `ScreenRender` view. Gives the state
    // and the screen of that view.
    fn ui_draw_check(view: u64) -> Result<(std::rc::Rc<FakeState>, FakeScreen), AmmlStatus> {
        let Some(state) = FakeState::current() else {
            return Err(AmmlStatus::FAILED);
        };
        if !state.area_offered(Area::UI_DRAW.id, Area::UI_DRAW.version) {
            return Err(AmmlStatus::NOT_AVAILABLE);
        }
        if !state.gate().get() {
            return Err(AmmlStatus::NOT_READY);
        }
        let screen = state.ui().screen_of(view).ok_or(AmmlStatus::REFUSED)?;
        Ok((state, screen))
    }

    fn finite(values: &[f32]) -> Result<(), AmmlStatus> {
        if values.iter().all(|value| value.is_finite()) { Ok(()) } else { Err(AmmlStatus::INVALID_ARGUMENT) }
    }

    unsafe extern "C" fn ui_draw_fill(view: u64, rect: AmmlRect, color: AmmlColor, alpha: f32) -> AmmlStatus {
        let result = Self::ui_draw_check(view).and_then(|(state, _)| {
            Self::finite(&[rect.x, rect.y, rect.width, rect.height, color.r, color.g, color.b, alpha])?;
            state.ui().record_draw(format!("fill {} {} {} {}", rect.x, rect.y, rect.width, rect.height));
            Ok(())
        });
        Self::status(result)
    }

    unsafe extern "C" fn ui_draw_outline(
        view: u64,
        rect: AmmlRect,
        color: AmmlColor,
        alpha: f32,
        thickness: i32,
    ) -> AmmlStatus {
        let result = Self::ui_draw_check(view).and_then(|(state, _)| {
            Self::finite(&[rect.x, rect.y, rect.width, rect.height, color.r, color.g, color.b, alpha])?;
            if thickness < 1 {
                return Err(AmmlStatus::INVALID_ARGUMENT);
            }
            state.ui().record_draw(format!("outline {} {} {} {}", rect.x, rect.y, rect.width, rect.height));
            Ok(())
        });
        Self::status(result)
    }

    unsafe extern "C" fn ui_draw_image(
        view: u64,
        texture: AmmlTextureId,
        rect: AmmlRect,
        uv: AmmlUv,
        tint: AmmlColor,
        alpha: f32,
    ) -> AmmlStatus {
        let result = Self::ui_draw_check(view).and_then(|(state, screen)| {
            Self::finite(&[
                rect.x,
                rect.y,
                rect.width,
                rect.height,
                uv.u0,
                uv.v0,
                uv.u1,
                uv.v1,
                tint.r,
                tint.g,
                tint.b,
                alpha,
            ])?;
            if !screen.can_draw {
                return Err(AmmlStatus::REFUSED);
            }
            let texture = TextureId::from(texture);
            if !state.textures().is_current(texture) {
                return Err(AmmlStatus::STALE);
            }
            if state.textures().is_ready(texture) {
                state
                    .ui()
                    .record_draw(format!("image {} {} {} {} {}", texture.id, rect.x, rect.y, rect.width, rect.height));
            }
            Ok(())
        });
        Self::status(result)
    }

    unsafe extern "C" fn ui_gui_scale(view: u64, out: *mut f32) -> AmmlStatus {
        if out.is_null() {
            return AmmlStatus::INVALID_ARGUMENT;
        }
        match Self::ui_draw_check(view).map(|(_, screen)| screen.gui_scale) {
            Ok(Some(scale)) => Self::write(out, scale),
            Ok(None) => AmmlStatus::NOT_AVAILABLE,
            Err(status) => status,
        }
    }

    unsafe extern "C" fn ui_can_draw(view: u64) -> u8 {
        u8::from(Self::ui_draw_check(view).is_ok_and(|(_, screen)| screen.can_draw))
    }

    // The loader checks before a text call: the text area, the gate and the running `ScreenRender` view. The fake
    // reads no more than the loader does: a text over the byte limit is refused before it is read.
    fn ui_text_check(view: u64, text: AmmlStr) -> Result<(std::rc::Rc<FakeState>, String), AmmlStatus> {
        let Some(state) = FakeState::current() else {
            return Err(AmmlStatus::FAILED);
        };
        if !state.area_offered(Area::UI_TEXT.id, Area::UI_TEXT.version) {
            return Err(AmmlStatus::NOT_AVAILABLE);
        }
        if !state.gate().get() {
            return Err(AmmlStatus::NOT_READY);
        }
        state.ui().screen_of(view).ok_or(AmmlStatus::REFUSED)?;
        if text.len > Self::TEXT_MAX_BYTES {
            return Err(AmmlStatus::REFUSED);
        }
        // SAFETY: the caller of every table function promises `text` is valid for the call.
        let text = unsafe { text.as_str() }?;
        Ok((state, text.to_string()))
    }

    // The width of the fake font: 6 units a character.
    fn text_units(text: &str) -> f32 {
        text.chars().count() as f32 * 6.0
    }

    unsafe extern "C" fn ui_text_width(view: u64, text: AmmlStr, out: *mut f32) -> AmmlStatus {
        if out.is_null() {
            return AmmlStatus::INVALID_ARGUMENT;
        }
        match Self::ui_text_check(view, text) {
            Ok((_, text)) => Self::write(out, Self::text_units(&text)),
            Err(status) => status,
        }
    }

    unsafe extern "C" fn ui_draw_text(
        view: u64,
        text: AmmlStr,
        rect: AmmlRect,
        color: AmmlColor,
        alpha: f32,
        align: AmmlTextAlign,
    ) -> AmmlStatus {
        let result = Self::ui_text_check(view, text).and_then(|(state, text)| {
            Self::finite(&[rect.x, rect.y, rect.width, rect.height, color.r, color.g, color.b, alpha])?;
            if align.0 > AmmlTextAlign::RIGHT.0 {
                return Err(AmmlStatus::INVALID_ARGUMENT);
            }
            state.ui().record_draw(format!("text \"{text}\" {} {}", rect.x, rect.y));
            Ok(())
        });
        Self::status(result)
    }

    // The loader checks before a texture call: the module and the area. Gives the state.
    fn textures_check(module: AmmlStr) -> Result<std::rc::Rc<FakeState>, AmmlStatus> {
        let Some(state) = FakeState::current() else {
            return Err(AmmlStatus::FAILED);
        };
        // SAFETY: the caller of every table function promises `module` is valid for the call.
        unsafe { module.as_str() }?;
        if !state.area_offered(Area::UI_TEXTURES.id, Area::UI_TEXTURES.version) {
            return Err(AmmlStatus::NOT_AVAILABLE);
        }
        Ok(state)
    }

    unsafe extern "C" fn ui_texture_load(
        module: AmmlStr,
        source: AmmlTextureSource,
        path: AmmlStr,
        out: *mut AmmlTextureId,
    ) -> AmmlStatus {
        if out.is_null() {
            return AmmlStatus::INVALID_ARGUMENT;
        }
        let result = Self::textures_check(module).and_then(|state| {
            // SAFETY: the caller promises `path` is valid for the call.
            let path = unsafe { path.as_str() }?;
            if path.trim().is_empty() {
                return Err(AmmlStatus::INVALID_ARGUMENT);
            }
            let line = match source {
                AmmlTextureSource::MOD_FILE => {
                    let leaves = path.starts_with(['/', '\\'])
                        || path.contains(':')
                        || path.split(['/', '\\']).any(|part| part == "..");
                    if leaves {
                        return Err(AmmlStatus::REFUSED);
                    }
                    format!("mod_file {path}")
                }
                AmmlTextureSource::VANILLA => format!("vanilla {path}"),
                _ => return Err(AmmlStatus::INVALID_ARGUMENT),
            };
            Ok(AmmlTextureId::from(state.textures().load(line)))
        });
        match result {
            Ok(texture) => Self::write(out, texture),
            Err(status) => status,
        }
    }

    unsafe extern "C" fn ui_texture_upload_rgba(
        module: AmmlStr,
        width: u32,
        height: u32,
        pixels: *const u8,
        len: usize,
        out: *mut AmmlTextureId,
    ) -> AmmlStatus {
        if out.is_null() || (pixels.is_null() && len != 0) {
            return AmmlStatus::INVALID_ARGUMENT;
        }
        let result = Self::textures_check(module).and_then(|state| {
            let wanted = u64::from(width) * u64::from(height) * 4;
            if width == 0 || height == 0 || u64::try_from(len) != Ok(wanted) {
                return Err(AmmlStatus::INVALID_ARGUMENT);
            }
            Ok(AmmlTextureId::from(state.textures().upload(width, height)))
        });
        match result {
            Ok(texture) => Self::write(out, texture),
            Err(status) => status,
        }
    }

    unsafe extern "C" fn ui_chat_text(field: AmmlChatField, out: *mut AmmlStr) -> AmmlStatus {
        if out.is_null() {
            return AmmlStatus::INVALID_ARGUMENT;
        }
        let Some(state) = FakeState::current() else { return AmmlStatus::FAILED };
        match state.chat().text(field) {
            Ok((ptr, len)) => Self::write(out, AmmlStr { ptr, len }),
            Err(status) => status,
        }
    }

    unsafe extern "C" fn ui_chat_set_text(module: AmmlStr, field: AmmlChatField, text: AmmlStr) -> AmmlStatus {
        let Some(state) = FakeState::current() else { return AmmlStatus::FAILED };
        // SAFETY: the caller of every table function promises `module` is valid for the call.
        if let Err(status) = unsafe { module.as_str() } {
            return status;
        }
        // SAFETY: the caller promises `text` is valid for the call.
        let text = match unsafe { text.as_str() } {
            Ok(text) => text,
            Err(status) => return status,
        };
        Self::status(state.chat().set_text(field, text))
    }

    unsafe extern "C" fn ui_chat_icons_active() -> u8 {
        let active = FakeState::current().is_some_and(|state| {
            state.area_offered(Area::UI_INLINE_ICONS.id, Area::UI_INLINE_ICONS.version) && state.chat().bitmap()
        });
        u8::from(active)
    }

    // The checks of an icon call: the module and the area.
    fn icons_check(module: AmmlStr) -> Result<std::rc::Rc<FakeState>, AmmlStatus> {
        let Some(state) = FakeState::current() else {
            return Err(AmmlStatus::FAILED);
        };
        // SAFETY: the caller of every table function promises `module` is valid for the call.
        unsafe { module.as_str() }?;
        if !state.area_offered(Area::UI_INLINE_ICONS.id, Area::UI_INLINE_ICONS.version) {
            return Err(AmmlStatus::NOT_AVAILABLE);
        }
        Ok(state)
    }

    // The image rules of the loader: RGBA8, pixels that fit the size (`INVALID_ARGUMENT`), at most 16 by 16 (`REFUSED`).
    fn icon_image(image: *const AmmlImageView) -> Result<(), AmmlStatus> {
        if image.is_null() {
            return Err(AmmlStatus::INVALID_ARGUMENT);
        }
        // SAFETY: `image` is non-null and the caller promises it is readable.
        let view = unsafe { image.read() };
        let wanted = u64::from(view.width) * u64::from(view.height) * 4;
        if view.format != 4
            || (view.pixels.is_null() && view.len != 0)
            || view.width == 0
            || view.height == 0
            || u64::try_from(view.len) != Ok(wanted)
        {
            return Err(AmmlStatus::INVALID_ARGUMENT);
        }
        if view.width > 16 || view.height > 16 {
            return Err(AmmlStatus::REFUSED);
        }
        Ok(())
    }

    unsafe extern "C" fn ui_icon_register(
        module: AmmlStr,
        image: *const AmmlImageView,
        advance: f32,
        out: *mut AmmlInlineIcon,
    ) -> AmmlStatus {
        if out.is_null() {
            return AmmlStatus::INVALID_ARGUMENT;
        }
        let result = Self::icons_check(module).and_then(|state| {
            Self::icon_image(image)?;
            if !(advance > 0.0 && advance <= 16.0) {
                return Err(AmmlStatus::REFUSED);
            }
            state.chat().register()
        });
        match result {
            Ok(icon) => Self::write(out, icon),
            Err(status) => status,
        }
    }

    unsafe extern "C" fn ui_icon_update(
        module: AmmlStr,
        icon: AmmlInlineIcon,
        image: *const AmmlImageView,
    ) -> AmmlStatus {
        Self::status(Self::icons_check(module).and_then(|state| {
            state.chat().check(icon)?;
            Self::icon_image(image)
        }))
    }

    unsafe extern "C" fn ui_icon_release(module: AmmlStr, icon: AmmlInlineIcon) -> AmmlStatus {
        // A thread that holds no fake only queues the free, as the loader does from any thread.
        if FakeState::current().is_none() {
            return Self::status(FakeChatBook::release_off_thread(icon));
        }
        Self::status(Self::icons_check(module).and_then(|state| state.chat().release(icon)))
    }

    unsafe extern "C" fn ui_texture_ready(texture: AmmlTextureId) -> u8 {
        let ready = FakeState::current().is_some_and(|state| {
            state.area_offered(Area::UI_TEXTURES.id, Area::UI_TEXTURES.version)
                && state.textures().is_ready(TextureId::from(texture))
        });
        u8::from(ready)
    }

    unsafe extern "C" fn ui_texture_unload(module: AmmlStr, texture: AmmlTextureId) -> AmmlStatus {
        let texture = TextureId::from(texture);
        // A thread that holds no fake only queues the free, as the loader does from any thread.
        if FakeState::current().is_none() {
            return Self::status(FakeTextureBook::unload_off_thread(texture));
        }
        Self::status(Self::textures_check(module).and_then(|state| state.textures().unload(texture)))
    }

    // The checks of a contents call: the module, the area at version 4 and the stack.
    fn contents_check(
        module: AmmlStr,
        player: *const c_void,
        stack: *const c_void,
    ) -> Result<(std::rc::Rc<FakeState>, FakeItem), AmmlStatus> {
        let Some(state) = FakeState::current() else {
            return Err(AmmlStatus::FAILED);
        };
        // SAFETY: the caller of every table function promises `module` is valid for the call.
        unsafe { module.as_str() }?;
        let item = Self::held_stack(&state, player, stack)?;
        let area = Area::INVENTORY_READ.at(4);
        if !state.area_offered(area.id, area.version) {
            return Err(AmmlStatus::NOT_AVAILABLE);
        }
        Ok((state, item))
    }

    unsafe extern "C" fn inventory_contents_open(
        module: AmmlStr,
        player: *const c_void,
        stack: *const c_void,
        out_contents: *mut u64,
        out_len: *mut u32,
    ) -> AmmlStatus {
        if out_contents.is_null() || out_len.is_null() {
            return AmmlStatus::INVALID_ARGUMENT;
        }
        // SAFETY: the caller promises `module` is valid for the call.
        let Ok(text) = (unsafe { module.as_str() }) else { return AmmlStatus::INVALID_ARGUMENT };
        let result = Self::contents_check(module, player, stack).and_then(|(state, item)| match item.contents {
            Some(entries) => state.items().open(text, entries),
            None => Ok((0, 0)),
        });
        match result {
            Ok((contents, len)) => {
                // SAFETY: both outputs are non-null and the caller promises they are writable.
                unsafe {
                    out_contents.write(contents);
                    out_len.write(len);
                }
                AmmlStatus::OK
            }
            Err(status) => status,
        }
    }

    unsafe extern "C" fn inventory_contents_entry(
        module: AmmlStr,
        contents: u64,
        index: u32,
        out: *mut AmmlContainerEntry,
    ) -> AmmlStatus {
        let Some(state) = FakeState::current() else {
            return AmmlStatus::FAILED;
        };
        if out.is_null() {
            return AmmlStatus::INVALID_ARGUMENT;
        }
        // SAFETY: the caller promises `module` is valid for the call.
        let Ok(text) = (unsafe { module.as_str() }) else { return AmmlStatus::INVALID_ARGUMENT };
        match state.items().entry(text, contents, index) {
            Ok(entry) => Self::write(out, entry),
            Err(status) => status,
        }
    }

    unsafe extern "C" fn inventory_contents_name(
        module: AmmlStr,
        contents: u64,
        index: u32,
        buf: *mut u8,
        cap: usize,
        out_len: *mut usize,
    ) -> AmmlStatus {
        let Some(state) = FakeState::current() else {
            return AmmlStatus::FAILED;
        };
        if out_len.is_null() || (buf.is_null() && cap != 0) {
            return AmmlStatus::INVALID_ARGUMENT;
        }
        // SAFETY: the caller promises `module` is valid for the call.
        let Ok(text) = (unsafe { module.as_str() }) else { return AmmlStatus::INVALID_ARGUMENT };
        match state.items().name(text, contents, index) {
            Ok(name) => {
                let name = name.as_bytes();
                let copied = name.len().min(cap);
                if copied > 0 {
                    // SAFETY: the caller promises `buf` is writable for `cap` bytes, and `copied <= cap`.
                    unsafe { std::ptr::copy_nonoverlapping(name.as_ptr(), buf, copied) };
                }
                Self::write(out_len, name.len())
            }
            Err(status) => status,
        }
    }

    unsafe extern "C" fn inventory_contents_close(module: AmmlStr, contents: u64) {
        // SAFETY: the caller promises `module` is valid for the call.
        if let (Some(state), Ok(text)) = (FakeState::current(), unsafe { module.as_str() }) {
            state.items().close(text, contents);
        }
    }

    unsafe extern "C" fn inventory_item_draw_release(module: AmmlStr, draw: AmmlItemDraw) -> AmmlStatus {
        // A thread that holds no fake only queues the free, as the loader does from any thread.
        let Some(state) = FakeState::current() else {
            return Self::status(FakeItemBook::release_off_thread(draw));
        };
        // SAFETY: the caller of every table function promises `module` is valid for the call.
        if let Err(status) = unsafe { module.as_str() } {
            return status;
        }
        Self::status(state.items().release(draw))
    }

    unsafe extern "C" fn ui_draw_item(
        module: AmmlStr,
        view: u64,
        draw: AmmlItemDraw,
        at: AmmlPoint,
        flags: AmmlItemDrawFlags,
        alpha: f32,
        _z: i32,
    ) -> AmmlStatus {
        let result = Self::ui_draw_check(view).and_then(|(state, _)| {
            // SAFETY: the caller promises `module` is valid for the call.
            unsafe { module.as_str() }?;
            let area = Area::UI_DRAW.at(2);
            if !state.area_offered(area.id, area.version) {
                return Err(AmmlStatus::NOT_AVAILABLE);
            }
            if flags.has_unknown_bits() {
                return Err(AmmlStatus::INVALID_ARGUMENT);
            }
            Self::finite(&[at.x, at.y, alpha])?;
            let name = state.items().name_of(draw)?;
            state.ui().record_draw(format!("item {name} {} {} {}", at.x, at.y, flags.0));
            Ok(())
        });
        Self::status(result)
    }

    unsafe extern "C" fn inventory_held_item(
        player: *const c_void,
        hand: AmmlHand,
        out: *mut *const c_void,
    ) -> AmmlStatus {
        let Some(state) = FakeState::current() else {
            return AmmlStatus::FAILED;
        };
        if let Err(status) = Self::inventory_check(&state, player) {
            return status;
        }
        match hand {
            AmmlHand::MAIN => {
                let stack = if state.inventory().held.is_some() { state.stack_handle() } else { 0 };
                Self::write(out, std::ptr::with_exposed_provenance(stack))
            }
            AmmlHand::OFF => {
                let stack = if state.lighting().off_hand().is_some() { state.off_stack_handle() } else { 0 };
                Self::write(out, std::ptr::with_exposed_provenance(stack))
            }
            _ => AmmlStatus::INVALID_ARGUMENT,
        }
    }

    unsafe extern "C" fn inventory_request_moves(
        _module: AmmlStr,
        player: *const c_void,
        moves: *const AmmlInventoryMove,
        count: u32,
        options: AmmlRequestOptions,
        out_id: *mut u64,
    ) -> AmmlStatus {
        let Some(state) = FakeState::current() else {
            return AmmlStatus::FAILED;
        };
        if out_id.is_null() || (moves.is_null() && count != 0) {
            return AmmlStatus::INVALID_ARGUMENT;
        }
        let fake = match state.client_check(state.client_handle(), Area::INVENTORY_REQUEST) {
            Ok(fake) => fake,
            Err(status) => return status,
        };
        if !fake.local_player || player.addr() != state.player_handle() {
            return AmmlStatus::REFUSED;
        }
        if count == 0 || count as usize > crate::Inventory::MAX_MOVES {
            return AmmlStatus::INVALID_ARGUMENT;
        }
        // SAFETY: the caller promises `moves` holds `count` moves.
        let raw = unsafe { std::slice::from_raw_parts(moves, count as usize) };
        let mut decoded = Vec::with_capacity(raw.len());
        for item in raw {
            match InventoryMove::try_from(*item) {
                Ok(item) if Self::move_fits(&item) => decoded.push(item),
                Ok(_) => return AmmlStatus::INVALID_ARGUMENT,
                Err(status) => return status,
            }
        }
        if state.requests().running() {
            return AmmlStatus::REFUSED;
        }
        if options.hidden_open != 0 && fake.top_screen != Screen::Hud {
            return AmmlStatus::REFUSED;
        }
        // A predicted swap changes the slots on the client, so the loader forgets the stacks it lent.
        if options.predict != 0 && decoded.iter().any(|item| matches!(item, InventoryMove::Swap { .. })) {
            state.ui().forget();
        }
        let id = state.requests().take(decoded, RequestOptions::from(options));
        Self::write(out_id, id.0)
    }

    // The loader's argument rule of a move: slots below 36, no swap or place of a slot with itself, no zero count.
    fn move_fits(item: &InventoryMove) -> bool {
        match *item {
            InventoryMove::Swap { a, b } => a < 36 && b < 36 && a != b,
            InventoryMove::Place { from, to, count } => from < 36 && to < 36 && from != to && count > 0,
            InventoryMove::Drop { from, count } => from < 36 && count > 0,
        }
    }

    unsafe extern "C" fn inventory_slot_item(player: *const c_void, index: u8, out: *mut *const c_void) -> AmmlStatus {
        let Some(state) = FakeState::current() else {
            return AmmlStatus::FAILED;
        };
        if out.is_null() || usize::from(index) >= 36 {
            return AmmlStatus::INVALID_ARGUMENT;
        }
        let fake = match state.client_check(state.client_handle(), Area::INVENTORY_READ.at(3)) {
            Ok(fake) => fake,
            Err(status) => return status,
        };
        if !fake.local_player || player.addr() != state.player_handle() {
            return AmmlStatus::REFUSED;
        }
        let index = usize::from(index);
        let stack = if state.inventory().slots[index].is_some() { state.slot_stack_handle(index) } else { 0 };
        state.ui().lend_slot(stack);
        Self::write(out, std::ptr::with_exposed_provenance(stack))
    }

    unsafe extern "C" fn inventory_selected_slot(player: *const c_void, out: *mut u8) -> AmmlStatus {
        let Some(state) = FakeState::current() else {
            return AmmlStatus::FAILED;
        };
        if let Err(status) = Self::inventory_check(&state, player) {
            return status;
        }
        match state.inventory().selected_slot {
            Some(slot) => Self::write(out, slot),
            None => AmmlStatus::NOT_AVAILABLE,
        }
    }

    // The change stamp of a stack `player` holds or one lent in a running dispatch: the `FakeItem::change_stamp`,
    // `NOT_AVAILABLE` when it is `None`, as on a build whose tags the loader cannot read.
    unsafe extern "C" fn inventory_item_change_stamp(
        player: *const c_void,
        stack: *const c_void,
        out: *mut u64,
    ) -> AmmlStatus {
        let Some(state) = FakeState::current() else {
            return AmmlStatus::FAILED;
        };
        if out.is_null() {
            return AmmlStatus::INVALID_ARGUMENT;
        }
        match Self::held_stack(&state, player, stack).map(|item| item.change_stamp) {
            Ok(Some(stamp)) => {
                // SAFETY: `out` is non-null and the caller promises it is writable for a u64.
                unsafe { out.write(stamp) };
                AmmlStatus::OK
            }
            Ok(None) => AmmlStatus::NOT_AVAILABLE,
            Err(status) => status,
        }
    }

    unsafe extern "C" fn inventory_item_read(
        player: *const c_void,
        stack: *const c_void,
        out: *mut AmmlItemSnapshot,
    ) -> AmmlStatus {
        let Some(state) = FakeState::current() else {
            return AmmlStatus::FAILED;
        };
        match Self::held_stack(&state, player, stack) {
            Ok(item) => Self::write(out, item.raw_snapshot()),
            Err(status) => status,
        }
    }

    unsafe extern "C" fn inventory_item_name(
        player: *const c_void,
        stack: *const c_void,
        buf: *mut u8,
        cap: usize,
        out_len: *mut usize,
    ) -> AmmlStatus {
        let Some(state) = FakeState::current() else {
            return AmmlStatus::FAILED;
        };
        match Self::held_stack(&state, player, stack) {
            Ok(item) => {
                let name = item.type_name.as_bytes();
                let copied = name.len().min(cap);
                if copied > 0 && !buf.is_null() {
                    // SAFETY: the caller promises `buf` is writable for `cap` bytes, and `copied <= cap`.
                    unsafe { std::ptr::copy_nonoverlapping(name.as_ptr(), buf, copied) };
                }
                Self::write(out_len, name.len())
            }
            Err(status) => status,
        }
    }

    unsafe extern "C" fn actor_is_alive(player: *const c_void, out: *mut u8) -> AmmlStatus {
        let Some(state) = FakeState::current() else {
            return AmmlStatus::FAILED;
        };
        let fake = match state.client_check(state.client_handle(), Area::ACTOR_READ) {
            Ok(fake) => fake,
            Err(status) => return status,
        };
        if !fake.local_player || player.addr() != state.player_handle() {
            return AmmlStatus::REFUSED;
        }
        match fake.player_alive {
            Some(alive) => Self::write(out, u8::from(alive)),
            None => AmmlStatus::NOT_AVAILABLE,
        }
    }

    // The fake client of a read of the local player `player` in area `actor.read` version 2.
    fn actor_check(player: *const c_void) -> Result<(std::rc::Rc<FakeState>, FakeClient), AmmlStatus> {
        let state = FakeState::current().ok_or(AmmlStatus::FAILED)?;
        let fake = state.client_check(state.client_handle(), Area::ACTOR_READ.at(2))?;
        if !fake.local_player || player.addr() != state.player_handle() {
            return Err(AmmlStatus::REFUSED);
        }
        Ok((state, fake))
    }

    unsafe extern "C" fn actor_position(player: *const c_void, out: *mut AmmlVec3) -> AmmlStatus {
        match Self::actor_check(player).map(|(_, fake)| fake.position) {
            Ok(Some(value)) => Self::write(out, AmmlVec3 { x: value.x, y: value.y, z: value.z }),
            Ok(None) => AmmlStatus::NOT_AVAILABLE,
            Err(status) => status,
        }
    }

    unsafe extern "C" fn actor_eye_position(player: *const c_void, out: *mut AmmlVec3) -> AmmlStatus {
        match Self::actor_check(player).map(|(_, fake)| fake.eye_position) {
            Ok(Some(value)) => Self::write(out, AmmlVec3 { x: value.x, y: value.y, z: value.z }),
            Ok(None) => AmmlStatus::NOT_AVAILABLE,
            Err(status) => status,
        }
    }

    unsafe extern "C" fn actor_rotation(player: *const c_void, out: *mut AmmlRotation) -> AmmlStatus {
        match Self::actor_check(player).map(|(_, fake)| fake.rotation) {
            Ok(Some(value)) => Self::write(out, AmmlRotation { yaw: value.yaw, pitch: value.pitch }),
            Ok(None) => AmmlStatus::NOT_AVAILABLE,
            Err(status) => status,
        }
    }

    // The fake dimension handle sits next to the player handle.
    unsafe extern "C" fn actor_dimension(player: *const c_void, out: *mut *const c_void) -> AmmlStatus {
        match Self::actor_check(player) {
            Ok((state, fake)) if fake.dimension.is_some() => {
                Self::write(out, std::ptr::with_exposed_provenance(state.player_handle() + Self::DIMENSION_OFFSET))
            }
            Ok(_) => AmmlStatus::NOT_AVAILABLE,
            Err(status) => status,
        }
    }

    unsafe extern "C" fn actor_dimension_id(dimension: *const c_void, out: *mut i32) -> AmmlStatus {
        let Some(state) = FakeState::current() else {
            return AmmlStatus::FAILED;
        };
        let player = std::ptr::with_exposed_provenance::<c_void>(state.player_handle());
        match Self::actor_check(player) {
            Ok((state, fake)) => match fake.dimension {
                Some(id) if dimension.addr() == state.player_handle() + Self::DIMENSION_OFFSET => Self::write(out, id),
                Some(_) => AmmlStatus::REFUSED,
                None => AmmlStatus::NOT_AVAILABLE,
            },
            Err(status) => status,
        }
    }

    unsafe extern "C" fn client_leave_world(client: *const c_void) -> AmmlStatus {
        let Some(state) = FakeState::current() else {
            return AmmlStatus::FAILED;
        };
        match state.client_check(client.addr(), Area::CLIENT_LEAVE_WORLD) {
            Ok(fake) if !fake.local_player => AmmlStatus::REFUSED,
            Ok(_) => {
                state.write_client(|fake| fake.local_player = false, "leave_world".to_string());
                AmmlStatus::OK
            }
            Err(status) => status,
        }
    }

    unsafe extern "C" fn client_can_join(client: *const c_void, out: *mut u8) -> AmmlStatus {
        let Some(state) = FakeState::current() else {
            return AmmlStatus::FAILED;
        };
        match state.client_check(client.addr(), Area::CLIENT_CAN_JOIN) {
            Ok(fake) => Self::write(out, u8::from(fake.can_join)),
            Err(status) => status,
        }
    }

    unsafe extern "C" fn client_start_local_world(
        client: *const c_void,
        level_id: AmmlStr,
        out: *mut AmmlStartWorldOutcome,
    ) -> AmmlStatus {
        let Some(state) = FakeState::current() else {
            return AmmlStatus::FAILED;
        };
        let fake = match state.client_check(client.addr(), Area::CLIENT_START_LOCAL_WORLD) {
            Ok(fake) => fake,
            Err(status) => return status,
        };
        // SAFETY: the caller promises the view is valid for the call.
        let Ok(level_id) = (unsafe { level_id.as_str() }) else {
            return AmmlStatus::INVALID_ARGUMENT;
        };
        if level_id.is_empty() {
            return AmmlStatus::INVALID_ARGUMENT;
        }
        if fake.local_player {
            return AmmlStatus::REFUSED;
        }
        let Some(answer) = fake.start_world_answer else {
            return AmmlStatus::NOT_READY;
        };
        state.write_client(|_| {}, format!("start_local_world {level_id}"));
        Self::write(out, Self::start_outcome(answer))
    }

    fn start_outcome(outcome: StartWorldOutcome) -> AmmlStartWorldOutcome {
        match outcome {
            StartWorldOutcome::Started => AmmlStartWorldOutcome::STARTED,
            StartWorldOutcome::NotInLevelList => AmmlStartWorldOutcome::NOT_IN_LEVEL_LIST,
            StartWorldOutcome::StartupInProgress => AmmlStartWorldOutcome::STARTUP_IN_PROGRESS,
            StartWorldOutcome::SavedInNewerVersion => AmmlStartWorldOutcome::SAVED_IN_NEWER_VERSION,
            StartWorldOutcome::PlatformLockedContent => AmmlStartWorldOutcome::PLATFORM_LOCKED_CONTENT,
            StartWorldOutcome::NotFoundInFileSystem => AmmlStartWorldOutcome::NOT_FOUND_IN_FILE_SYSTEM,
            StartWorldOutcome::SlimCacheData => AmmlStartWorldOutcome::SLIM_CACHE_DATA,
            StartWorldOutcome::OtherRefusal => AmmlStartWorldOutcome::OTHER_REFUSAL,
        }
    }

    unsafe extern "C" fn client_join_server(
        _: AmmlStr,
        client: *const c_void,
        host: AmmlStr,
        port: u16,
        _: AmmlStr,
        out: *mut u64,
    ) -> AmmlStatus {
        let Some(state) = FakeState::current() else {
            return AmmlStatus::FAILED;
        };
        let fake = match state.client_check(client.addr(), Area::CLIENT_JOIN_SERVER) {
            Ok(fake) => fake,
            Err(status) => return status,
        };
        // SAFETY: the caller promises the view is valid for the call.
        let Ok(host) = (unsafe { host.as_str() }) else {
            return AmmlStatus::INVALID_ARGUMENT;
        };
        if host.is_empty() || port == 0 {
            return AmmlStatus::INVALID_ARGUMENT;
        }
        if fake.local_player {
            return AmmlStatus::REFUSED;
        }
        state.write_client(|_| {}, format!("join_server {host}:{port}"));
        Self::write(out, state.next_join())
    }

    unsafe extern "C" fn client_open_menu(client: *const c_void, menu: AmmlMenu) -> AmmlStatus {
        let Some(state) = FakeState::current() else {
            return AmmlStatus::FAILED;
        };
        let fake = match state.client_check(client.addr(), Area::CLIENT_OPEN_MENU) {
            Ok(fake) => fake,
            Err(status) => return status,
        };
        if menu != AmmlMenu::SERVERS {
            return AmmlStatus::INVALID_ARGUMENT;
        }
        if fake.local_player {
            return AmmlStatus::REFUSED;
        }
        state.write_client(|_| {}, "open_menu servers".to_string());
        AmmlStatus::OK
    }

    unsafe extern "C" fn client_send_command(client: *const c_void, text: AmmlStr) -> AmmlStatus {
        let Some(state) = FakeState::current() else {
            return AmmlStatus::FAILED;
        };
        let fake = match state.client_check(client.addr(), Area::CLIENT_SEND_COMMAND) {
            Ok(fake) => fake,
            Err(status) => return status,
        };
        // SAFETY: the caller promises the view is valid for the call.
        let Ok(text) = (unsafe { text.as_str() }) else {
            return AmmlStatus::INVALID_ARGUMENT;
        };
        if text.is_empty() {
            return AmmlStatus::INVALID_ARGUMENT;
        }
        if !fake.local_player || text.len() > crate::Client::MAX_COMMAND {
            return AmmlStatus::REFUSED;
        }
        state.write_client(|_| {}, format!("send_command {text}"));
        AmmlStatus::OK
    }

    unsafe extern "C" fn client_select_hotbar_slot(client: *const c_void, slot: u8) -> AmmlStatus {
        let Some(state) = FakeState::current() else {
            return AmmlStatus::FAILED;
        };
        let fake = match state.client_check(client.addr(), Area::HUD_SELECT_HOTBAR) {
            Ok(fake) => fake,
            Err(status) => return status,
        };
        if !fake.local_player || slot >= crate::Hud::HOTBAR_SLOTS {
            return AmmlStatus::REFUSED;
        }
        state.write_client(|_| {}, format!("select_hotbar_slot {slot}"));
        AmmlStatus::OK
    }

    unsafe extern "C" fn client_note_activity(client: *const c_void) -> AmmlStatus {
        let Some(state) = FakeState::current() else {
            return AmmlStatus::FAILED;
        };
        let fake = match state.client_check(client.addr(), Area::CLIENT_ACTIVITY) {
            Ok(fake) => fake,
            Err(status) => return status,
        };
        match fake.activity {
            FakeActivity::Ok => {
                state.write_client(|_| {}, "note_activity".to_string());
                AmmlStatus::OK
            }
            FakeActivity::Refused => AmmlStatus::REFUSED,
            FakeActivity::NotAvailable => AmmlStatus::NOT_AVAILABLE,
        }
    }

    unsafe extern "C" fn input_modifiers() -> AmmlModifiers {
        FakeState::current().map_or(AmmlModifiers(0), |state| state.input().modifiers())
    }

    unsafe extern "C" fn input_is_down(key: u16, out: *mut u8) -> AmmlStatus {
        let Some(state) = FakeState::current() else {
            return AmmlStatus::FAILED;
        };
        if key >= 0x100 {
            return AmmlStatus::INVALID_ARGUMENT;
        }
        match state.input().is_down(key) {
            Some(down) => Self::write(out, u8::from(down)),
            None => AmmlStatus::NOT_READY,
        }
    }

    unsafe extern "C" fn input_acquire(
        module: AmmlStr,
        request: *const AmmlLeaseRequest,
        out_lease: *mut u64,
    ) -> AmmlStatus {
        let (Some(state), Some(module)) = (FakeState::current(), Self::text(module)) else {
            return AmmlStatus::FAILED;
        };
        if request.is_null() || out_lease.is_null() {
            return AmmlStatus::INVALID_ARGUMENT;
        }
        // SAFETY: `request` is non-null and the caller promises a v1 request whose `passthrough` holds
        // `passthrough_count` keys valid for the call.
        let terms = match unsafe { Self::terms(request) } {
            Ok(terms) => terms,
            Err(status) => return status,
        };
        match state.input().acquire(module, terms) {
            Ok(lease) => Self::write(out_lease, lease),
            Err(status) => status,
        }
    }

    /// The terms of a request, under the loader's checks.
    ///
    /// # Safety
    /// `request` is non-null and readable for its `size`; with a count, `passthrough` holds
    /// `passthrough_count` keys valid for the call.
    unsafe fn terms(request: *const AmmlLeaseRequest) -> Result<LeaseTerms, AmmlStatus> {
        // SAFETY: the caller guarantees the leading `size` is readable.
        let size = unsafe { request.cast::<u32>().read_unaligned() };
        if size < AmmlLeaseRequest::SIZE_V1 {
            return Err(AmmlStatus::INVALID_ARGUMENT);
        }
        // SAFETY: `size` covers the whole v1 request.
        let request = unsafe { request.read_unaligned() };
        if request.mode != AmmlLeaseMode::SHARED && request.mode != AmmlLeaseMode::EXCLUSIVE {
            return Err(AmmlStatus::INVALID_ARGUMENT);
        }
        let passthrough = if request.passthrough_count == 0 {
            Vec::new()
        } else if request.passthrough.is_null() {
            return Err(AmmlStatus::INVALID_ARGUMENT);
        } else {
            // SAFETY: the caller guarantees `passthrough` holds `passthrough_count` keys.
            unsafe { std::slice::from_raw_parts(request.passthrough, request.passthrough_count) }.to_vec()
        };
        Ok(LeaseTerms {
            mode: request.mode,
            blocks: AmmlInputBlocks(request.blocks.0 & AmmlInputBlocks::FULL.0),
            passthrough,
            preempt: request.preempt != 0,
        })
    }

    unsafe extern "C" fn input_lease_state(lease: u64) -> AmmlLeaseState {
        FakeState::current().map_or(AmmlLeaseState::LOST, |state| state.input().state(lease))
    }

    unsafe extern "C" fn input_key_down(lease: u64, key: u16, out_hold: *mut u64) -> AmmlStatus {
        let Some(state) = FakeState::current() else {
            return AmmlStatus::FAILED;
        };
        if out_hold.is_null() || key == 0 || key >= 0x100 {
            return AmmlStatus::INVALID_ARGUMENT;
        }
        let pressed = state
            .inject_check(Area::KEY_INJECT)
            .and_then(|()| state.input().press(lease, InjectedOp::Key { key, down: true }));
        match pressed {
            Ok(hold) => Self::write(out_hold, hold),
            Err(status) => status,
        }
    }

    unsafe extern "C" fn input_button_down(lease: u64, button: AmmlMouseButton, out_hold: *mut u64) -> AmmlStatus {
        let Some(state) = FakeState::current() else {
            return AmmlStatus::FAILED;
        };
        if out_hold.is_null() || button.0 > AmmlMouseButton::X2.0 {
            return AmmlStatus::INVALID_ARGUMENT;
        }
        let pressed = state
            .inject_check(Area::MOUSE_INJECT)
            .and_then(|()| state.input().press(lease, InjectedOp::Button { button, down: true }));
        match pressed {
            Ok(hold) => Self::write(out_hold, hold),
            Err(status) => status,
        }
    }

    unsafe extern "C" fn input_release_hold(hold: u64) -> AmmlStatus {
        let Some(state) = FakeState::current() else {
            return AmmlStatus::FAILED;
        };
        state.input().release_hold(hold);
        AmmlStatus::OK
    }

    unsafe extern "C" fn input_wheel(lease: u64, notches: f32) -> AmmlStatus {
        let Some(state) = FakeState::current() else {
            return AmmlStatus::FAILED;
        };
        if !notches.is_finite() {
            return AmmlStatus::INVALID_ARGUMENT;
        }
        Self::status(state.inject_check(Area::MOUSE_INJECT).and_then(|()| state.input().wheel(lease, notches)))
    }

    unsafe extern "C" fn input_turn(lease: u64, yaw: f32, pitch: f32) -> AmmlStatus {
        let Some(state) = FakeState::current() else {
            return AmmlStatus::FAILED;
        };
        if !yaw.is_finite() || !pitch.is_finite() {
            return AmmlStatus::INVALID_ARGUMENT;
        }
        Self::status(state.inject_check(Area::TURN).and_then(|()| state.input().turn(lease, yaw, pitch)))
    }

    unsafe extern "C" fn input_set_move(lease: u64, intent: *const AmmlMoveIntent) -> AmmlStatus {
        let Some(state) = FakeState::current() else {
            return AmmlStatus::FAILED;
        };
        // SAFETY: a non-null `intent` points to a move intent the caller keeps valid for the call.
        let intent = (!intent.is_null()).then(|| unsafe { intent.read_unaligned() });
        if intent.is_some_and(|intent| !intent.forward.is_finite() || !intent.left.is_finite()) {
            return AmmlStatus::INVALID_ARGUMENT;
        }
        Self::status(state.inject_check(Area::MOVE).and_then(|()| state.input().set_move(lease, intent)))
    }

    unsafe extern "C" fn input_release_all(lease: u64) {
        if let Some(state) = FakeState::current() {
            state.input().release_all(lease);
        }
    }

    unsafe extern "C" fn input_end_lease(lease: u64) -> AmmlStatus {
        let Some(state) = FakeState::current() else {
            return AmmlStatus::FAILED;
        };
        state.input().end(lease);
        AmmlStatus::OK
    }

    unsafe extern "C" fn input_set_ranking(_: AmmlStr, ids: *const AmmlStr, count: usize) -> AmmlStatus {
        let Some(state) = FakeState::current() else {
            return AmmlStatus::FAILED;
        };
        // SAFETY: the caller promises `ids` holds `count` views valid for the call.
        match unsafe { Self::ids(ids, count) } {
            Some(ranked) => Self::status(state.input().set_ranking(ranked)),
            None => AmmlStatus::INVALID_ARGUMENT,
        }
    }

    unsafe extern "C" fn input_set_coexist(_: AmmlStr, ids: *const AmmlStr, count: usize) -> AmmlStatus {
        let Some(state) = FakeState::current() else {
            return AmmlStatus::FAILED;
        };
        if !count.is_multiple_of(2) {
            return AmmlStatus::INVALID_ARGUMENT;
        }
        // SAFETY: the caller promises `ids` holds `count` views valid for the call.
        match unsafe { Self::ids(ids, count) } {
            Some(ids) => {
                let pairs =
                    ids.as_chunks::<2>().0.iter().map(|[first, second]| (first.clone(), second.clone())).collect();
                Self::status(state.input().set_coexist(pairs))
            }
            None => AmmlStatus::INVALID_ARGUMENT,
        }
    }

    // The `count` mod ids in `ids`; `None` for a null array with a count or a view that is not text.
    //
    // Safety: a non-null `ids` holds `count` views valid for the call.
    unsafe fn ids(ids: *const AmmlStr, count: usize) -> Option<Vec<String>> {
        if count > 0 && ids.is_null() {
            return None;
        }
        let views: &[AmmlStr] = if count == 0 {
            &[]
        } else {
            // SAFETY: `ids` is non-null and the caller promises it holds `count` views.
            unsafe { std::slice::from_raw_parts(ids, count) }
        };
        views.iter().map(|view| Self::text(*view).map(str::to_string)).collect()
    }

    // Runs one window read in `area` and writes its value to `out`.
    fn window_read<T>(out: *mut T, area: Area, read: impl FnOnce(&FakeWindow) -> Result<T, AmmlStatus>) -> AmmlStatus {
        let Some(state) = FakeState::current() else {
            return AmmlStatus::FAILED;
        };
        match state.window_check(area).and_then(|window| read(&window)) {
            Ok(value) => Self::write(out, value),
            Err(status) => status,
        }
    }

    unsafe extern "C" fn window_client_size(out: *mut AmmlSize) -> AmmlStatus {
        Self::window_read(out, Area::WINDOW_READ, |window| Ok(window.client.into()))
    }

    unsafe extern "C" fn window_buffer_size(out: *mut AmmlSize) -> AmmlStatus {
        Self::window_read(out, Area::WINDOW_READ, |window| {
            let known = window.buffer.width != 0 && window.buffer.height != 0;
            if known { Ok(window.buffer.into()) } else { Err(AmmlStatus::NOT_READY) }
        })
    }

    unsafe extern "C" fn window_cursor_pos(out: *mut AmmlPoint) -> AmmlStatus {
        Self::window_read(out, Area::WINDOW_READ, |window| Ok(window.cursor.into()))
    }

    unsafe extern "C" fn window_to_buffer(client: AmmlPoint, out: *mut AmmlPoint) -> AmmlStatus {
        Self::window_read(out, Area::WINDOW_READ, |window| {
            let (from, to) = (window.client, window.buffer);
            if from.width == 0 || from.height == 0 || to.width == 0 || to.height == 0 {
                return Err(AmmlStatus::NOT_READY);
            }
            Ok(AmmlPoint {
                x: client.x * to.width as f32 / from.width as f32,
                y: client.y * to.height as f32 / from.height as f32,
            })
        })
    }

    unsafe extern "C" fn window_has_focus(out: *mut u8) -> AmmlStatus {
        Self::window_read(out, Area::WINDOW_READ, |window| Ok(u8::from(window.focus)))
    }

    unsafe extern "C" fn window_dpi(out: *mut u32) -> AmmlStatus {
        Self::window_read(out, Area::WINDOW_READ, |window| Ok(window.dpi))
    }

    unsafe extern "C" fn window_request_size(module: AmmlStr, size: *const AmmlSize) -> AmmlStatus {
        let (Some(state), Some(_)) = (FakeState::current(), Self::text(module)) else {
            return AmmlStatus::FAILED;
        };
        if let Err(status) = state.window_check(Area::WINDOW_REQUEST_SIZE) {
            return status;
        }
        // SAFETY: a non-null `size` points to a size the caller keeps valid for the call.
        let size = (!size.is_null()).then(|| unsafe { size.read_unaligned() });
        let line = match size {
            Some(size) if (1..=65535).contains(&size.width) && (1..=65535).contains(&size.height) => {
                format!("request_size {}x{}", size.width, size.height)
            }
            Some(_) => return AmmlStatus::INVALID_ARGUMENT,
            None => "request_size none".to_string(),
        };
        state.record_window_request(line);
        AmmlStatus::OK
    }

    unsafe extern "C" fn window_replay_focus_loss() -> AmmlStatus {
        let Some(state) = FakeState::current() else {
            return AmmlStatus::FAILED;
        };
        if let Err(status) = state.window_check(Area::WINDOW_READ) {
            return status;
        }
        state.record_window_request("replay_focus_loss".to_string());
        AmmlStatus::OK
    }

    unsafe extern "C" fn overlay_screenshot(
        module: AmmlStr,
        path: AmmlStr,
        layers: AmmlShotLayers,
        out_id: *mut u64,
    ) -> AmmlStatus {
        let (Some(state), Some(_), Some(path)) = (FakeState::current(), Self::text(module), Self::text(path)) else {
            return AmmlStatus::FAILED;
        };
        if !state.area_offered(Area::OVERLAY_SCREENSHOT.id, Area::OVERLAY_SCREENSHOT.version) {
            return AmmlStatus::NOT_AVAILABLE;
        }
        if out_id.is_null() {
            return AmmlStatus::INVALID_ARGUMENT;
        }
        match state.overlay().screenshot(path, layers, state.gate().get()) {
            Ok(id) => Self::write(out_id, id),
            Err(status) => status,
        }
    }

    unsafe extern "C" fn overlay_open_capture(
        module: AmmlStr,
        request: *const AmmlCaptureRequest,
        out_stream: *mut u64,
    ) -> AmmlStatus {
        let (Some(state), Some(_)) = (FakeState::current(), Self::text(module)) else {
            return AmmlStatus::FAILED;
        };
        if !state.area_offered(Area::OVERLAY_CAPTURE.id, Area::OVERLAY_CAPTURE.version) {
            return AmmlStatus::NOT_AVAILABLE;
        }
        if request.is_null() || out_stream.is_null() {
            return AmmlStatus::INVALID_ARGUMENT;
        }
        // SAFETY: `request` is non-null and the caller promises a readable v1 request.
        let request = unsafe { request.read_unaligned() };
        match state.overlay().open(&request, state.gate().get()) {
            Ok(stream) => Self::write(out_stream, stream),
            Err(status) => status,
        }
    }

    // The fake state when the capture area is offered; `NOT_AVAILABLE` otherwise.
    fn capture_state() -> Result<std::rc::Rc<FakeState>, AmmlStatus> {
        let state = FakeState::current().ok_or(AmmlStatus::FAILED)?;
        if !state.area_offered(Area::OVERLAY_CAPTURE.id, Area::OVERLAY_CAPTURE.version) {
            return Err(AmmlStatus::NOT_AVAILABLE);
        }
        Ok(state)
    }

    unsafe extern "C" fn overlay_capture_take(stream: u64, out: *mut AmmlCaptureFrame) -> AmmlStatus {
        if out.is_null() {
            return AmmlStatus::INVALID_ARGUMENT;
        }
        match Self::capture_state().and_then(|state| state.overlay().take(stream)) {
            Ok(frame) => Self::write(out, frame),
            Err(status) => status,
        }
    }

    unsafe extern "C" fn overlay_capture_give_back(stream: u64, present_index: u64) -> AmmlStatus {
        Self::status(Self::capture_state().and_then(|state| state.overlay().give_back(stream, present_index)))
    }

    unsafe extern "C" fn overlay_capture_free_slots(stream: u64) -> u32 {
        Self::capture_state().map_or(0, |state| state.overlay().free_slots(stream))
    }

    unsafe extern "C" fn overlay_close_capture(stream: u64) -> AmmlStatus {
        Self::status(Self::capture_state().and_then(|state| state.overlay().close(stream)))
    }

    unsafe extern "C" fn overlay_present_count() -> u64 {
        FakeState::current()
            .filter(|state| state.area_offered(Area::OVERLAY_SURFACE.id, Area::OVERLAY_SURFACE.version))
            .map_or(0, |state| state.overlay().present_count())
    }

    unsafe extern "C" fn overlay_copy_back_buffer(dest: *mut c_void) -> AmmlStatus {
        let Some(state) = FakeState::current() else {
            return AmmlStatus::FAILED;
        };
        if !state.area_offered(Area::OVERLAY_SURFACE.id, Area::OVERLAY_SURFACE.version) {
            return AmmlStatus::NOT_AVAILABLE;
        }
        if dest.is_null() {
            return AmmlStatus::INVALID_ARGUMENT;
        }
        AmmlStatus::REFUSED
    }
}
