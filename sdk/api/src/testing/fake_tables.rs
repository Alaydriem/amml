#![allow(unsafe_code)]

use std::ffi::c_void;

use amml_abi::{
    AMML_API_VERSION, AmmlActorTable, AmmlAmount, AmmlApi, AmmlApplyStats, AmmlBand, AmmlBlockHandle, AmmlCameraTable,
    AmmlCaptureFrame, AmmlCaptureRequest, AmmlClientFlag, AmmlClientTable, AmmlColor, AmmlContainerAction,
    AmmlContainerEntry, AmmlCoreTable, AmmlCursorClip, AmmlDirKind, AmmlDrawSurface, AmmlEventsTable, AmmlHand,
    AmmlHitResult, AmmlInputBlocks, AmmlInputMode, AmmlInputTable, AmmlInventoryMove, AmmlInventoryTable, AmmlItemDraw,
    AmmlItemDrawFlags, AmmlItemSnapshot, AmmlLeaseMode, AmmlLeaseRequest, AmmlLeaseState, AmmlLightSource,
    AmmlLightingTable, AmmlLogLevel, AmmlMenu, AmmlMeshId, AmmlModifiers, AmmlMouseButton, AmmlMoveIntent,
    AmmlNetworkTable, AmmlOptionKey, AmmlOptionsTable, AmmlOtherLeases, AmmlOverlayTable, AmmlPoint, AmmlPose,
    AmmlProjection, AmmlRect, AmmlRegistryTable, AmmlRenderTable, AmmlRequestOptions, AmmlRotation, AmmlScreen,
    AmmlScreenKind, AmmlServices, AmmlShapeOp, AmmlShotLayers, AmmlSize, AmmlSlotAt, AmmlStartWorldOutcome, AmmlStatus,
    AmmlStr, AmmlTextAlign, AmmlTextureId, AmmlTextureSource, AmmlUiProfile, AmmlUiTable, AmmlUv, AmmlVec3,
    AmmlWindowTable, AmmlWorldInfo, AmmlWorldTable, AmmlWorldText, EventCallbackFn,
};
use amml_abi::{AmmlAabb, AmmlActorTarget, AmmlCameraKind, AmmlGameType, AmmlPermissionLevel};
use amml_abi::{AmmlBlockEntityGhost, AmmlGhostEntityOptions};
use amml_abi::{AmmlBlockGrid, AmmlMeshBuilt, AmmlMeshGroup, AmmlMesherOptions};
use amml_abi::{AmmlBlockInfo, AmmlBlockMatch, AmmlBlockPos, AmmlKindsResult, AmmlStateEntry, AmmlStateValue};
use amml_abi::{AmmlChatField, AmmlImageView, AmmlInlineIcon};
use amml_abi::{AmmlChunkLoad, AmmlColumnReadStats, AmmlColumnRequest, AmmlColumnSample, AmmlSurfaceSample};
use amml_abi::{AmmlColorQuad, AmmlMeshStyle, AmmlPersistence, AmmlTransform};
use amml_abi::{AmmlHoldRequest, AmmlTimeHold, AmmlTimeState, AmmlTimeTable};
use amml_runtime::{InjectedOp, LeaseTerms};

use crate::testing::{
    FakeActivity, FakeChatBook, FakeClient, FakeGame, FakeItem, FakeItemBook, FakeListener, FakeNetworkTable,
    FakeScreen, FakeShapes, FakeState, FakeTextureBook, FakeUi, FakeWindow,
};
use crate::{Area, InstructionKind, InventoryMove, RequestOptions, Screen, ScreenKind, StartWorldOutcome, TextureId};

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
    _world: Box<AmmlWorldTable>,
    _render: Box<AmmlRenderTable>,
    _camera: Box<AmmlCameraTable>,
    _options: Box<AmmlOptionsTable>,
    _time: Box<AmmlTimeTable>,
}

impl FakeTables {
    const TABLE_VERSION: u32 = 1;
    // The longest text a text call takes, as in the loader.
    const TEXT_MAX_BYTES: usize = 256;
    // The fake dimension handle is this far after the fake player handle.
    const DIMENSION_OFFSET: usize = 0x10;
    // The fake block source handle is this far after the fake player handle.
    const BLOCK_SOURCE_OFFSET: usize = 0x18;

    /// Tables whose services table has `services_size`: `SIZE_V4` points to every area table, `SIZE_V3` to all
    /// but the actor table, `SIZE_V2`
    /// to the client and input tables only, `SIZE_V1` to none, as a loader before them.
    pub(crate) fn new(services_size: u32) -> Self {
        Self::with_inventory(services_size, AmmlInventoryTable::SIZE_V7)
    }

    /// `new` with an inventory table of `inventory_size`: `SIZE_V4` has no `request_moves`, `SIZE_V5` no
    /// `item_change_stamp` and `SIZE_V6` no `request_pick`, as a loader before them.
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
            size: AmmlClientTable::SIZE_V8,
            version: 8,
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
            world_info: Some(Self::client_world_info),
            world_text: Some(Self::client_world_text),
            replay_app_focus_lost: Some(Self::client_replay_app_focus_lost),
            in_gameplay: Some(Self::client_in_gameplay),
            join_server_direct: Some(Self::client_join_server_direct),
            title_screen: Some(Self::client_title_screen),
            go_to_title: Some(Self::client_go_to_title),
        });
        let input = Box::new(AmmlInputTable {
            size: AmmlInputTable::SIZE_V3,
            version: 3,
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
            held_by_others: Some(Self::input_held_by_others),
        });
        let window = Box::new(AmmlWindowTable {
            size: AmmlWindowTable::SIZE_V2,
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
            clip_cursor: Some(Self::window_clip_cursor),
        });
        let overlay = Box::new(AmmlOverlayTable {
            size: AmmlOverlayTable::SIZE_V2,
            version: Self::TABLE_VERSION,
            screenshot: Self::overlay_screenshot,
            open_capture: Self::overlay_open_capture,
            capture_take: Self::overlay_capture_take,
            capture_give_back: Self::overlay_capture_give_back,
            capture_free_slots: Self::overlay_capture_free_slots,
            close_capture: Self::overlay_close_capture,
            present_count: Self::overlay_present_count,
            copy_back_buffer: Self::overlay_copy_back_buffer,
            prepare_surface: Some(Self::overlay_prepare_surface),
        });
        let actor = Box::new(AmmlActorTable {
            size: AmmlActorTable::SIZE_V6,
            version: 6,
            is_alive: Self::actor_is_alive,
            position: Some(Self::actor_position),
            eye_position: Some(Self::actor_eye_position),
            rotation: Some(Self::actor_rotation),
            dimension: Some(Self::actor_dimension),
            dimension_id: Some(Self::actor_dimension_id),
            head_position: Some(Self::actor_head_position),
            block_source: Some(Self::actor_block_source),
            health: Some(Self::actor_health),
            view_vector: Some(Self::actor_view_vector),
            aabb: Some(Self::actor_aabb),
            game_type: Some(Self::actor_game_type),
            permission_level: Some(Self::actor_permission_level),
            set_rotation: Some(Self::actor_set_rotation),
            build_at: Some(Self::actor_build_at),
            build_delay: Some(Self::actor_build_delay),
            max_pick_range: Some(Self::actor_max_pick_range),
            hit_result: Some(Self::actor_hit_result),
        });
        let v2 = services_size >= AmmlServices::SIZE_V2;
        let v3 = services_size >= AmmlServices::SIZE_V3;
        let inventory = Box::new(AmmlInventoryTable {
            size: inventory_size,
            version: match inventory_size {
                size if size >= AmmlInventoryTable::SIZE_V7 => 7,
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
            request_pick: (inventory_size >= AmmlInventoryTable::SIZE_V7).then_some(Self::inventory_request_pick),
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
        let v9 = services_size >= AmmlServices::SIZE_V9;
        let v10 = services_size >= AmmlServices::SIZE_V10;
        let v11 = services_size >= AmmlServices::SIZE_V11;
        let v12 = services_size >= AmmlServices::SIZE_V12;
        let v13 = services_size >= AmmlServices::SIZE_V13;
        let network = Box::new(FakeNetworkTable::table());
        let world = Box::new(AmmlWorldTable {
            size: AmmlWorldTable::SIZE_V9,
            version: 9,
            sky_darken: Self::world_sky_darken,
            chunk: Some(Self::world_chunk),
            chunk_load_state: Some(Self::world_chunk_load_state),
            read_columns: Some(Self::world_read_columns),
            block_type_name: Some(Self::world_block_type_name),
            biome_name: Some(Self::world_biome_name),
            scan_surfaces: Some(Self::world_scan_surfaces),
            shapes_apply: Some(Self::world_shapes_apply),
            shapes_max_batch: Some(Self::world_shapes_max_batch),
            shapes_target_epoch: Some(Self::world_shapes_target_epoch),
            raycast: Some(Self::world_raycast),
            actors_in: Some(Self::world_actors_in),
            block_from_state: Some(Self::world_block_from_state),
            block_rotated: Some(Self::world_block_rotated),
            block_handle_name: Some(Self::world_block_handle_name),
            block_handle_item: Some(Self::world_block_handle_item),
            block_at: Some(Self::world_block_at),
            block_info: Some(Self::world_block_info),
            block_built_over: Some(Self::world_block_built_over),
            block_item: Some(Self::world_block_item),
            block_handle_of: Some(Self::world_block_handle_of),
            block_changed_state: Some(Self::world_block_changed_state),
            kinds_in_box: Some(Self::world_kinds_in_box),
        });
        let render = Box::new(AmmlRenderTable {
            size: AmmlRenderTable::SIZE_V6,
            version: 6,
            canvas_camera_position: Self::render_canvas_camera_position,
            mesh_free: Self::render_mesh_free,
            mesh_build: Some(Self::render_mesh_build),
            mesh_draw: Some(Self::render_mesh_draw),
            canvas_clip_matrix: Some(Self::render_canvas_clip_matrix),
            mesher_build: Some(Self::render_mesher_build),
            mesher_group_begin: Some(Self::render_mesher_group_begin),
            mesher_group_append: Some(Self::render_mesher_group_append),
            mesher_group_finish: Some(Self::render_mesher_group_finish),
            block_entity_create: Some(Self::render_block_entity_create),
            block_entity_free: Some(Self::render_block_entity_free),
            block_entity_draw: Some(Self::render_block_entity_draw),
            mesher_group_discard: Some(Self::render_mesher_group_discard),
        });
        let camera = Box::new(AmmlCameraTable {
            size: AmmlCameraTable::SIZE_V4,
            version: 4,
            set_pose: Self::camera_set_pose,
            rendered_position: Self::camera_rendered_position,
            projection: Self::camera_projection,
            set_free: Some(Self::camera_set_free),
            release_free: Some(Self::camera_release_free),
            free_available: Some(Self::camera_free_available),
            active_camera: Some(Self::camera_active_camera),
            hide_actor: Some(Self::camera_hide_actor),
        });
        let options = Box::new(AmmlOptionsTable {
            size: AmmlOptionsTable::SIZE_V1,
            version: 1,
            view_distance_chunks: Self::options_view_distance,
            get_int: Self::options_get_int,
            set_int: Self::options_set_int,
            release_transient: Self::options_release_transient,
        });
        let time = Box::new(AmmlTimeTable {
            size: AmmlTimeTable::SIZE_V1,
            version: 1,
            hold: Self::time_hold,
            step_us: Self::time_step_us,
            release: Self::time_release,
            state: Self::time_state,
        });
        let services = Box::new(AmmlServices {
            size: services_size,
            version: if v13 {
                13
            } else if v12 {
                12
            } else if v11 {
                11
            } else if v10 {
                10
            } else if v9 {
                9
            } else if v8 {
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
            world: if v9 { &raw const *world } else { std::ptr::null() },
            render: if v10 { &raw const *render } else { std::ptr::null() },
            camera: if v11 { &raw const *camera } else { std::ptr::null() },
            options: if v12 { &raw const *options } else { std::ptr::null() },
            time: if v13 { &raw const *time } else { std::ptr::null() },
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
            _world: world,
            _render: render,
            _camera: camera,
            _options: options,
            _time: time,
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

    unsafe extern "C" fn inventory_request_pick(
        _module: AmmlStr,
        player: *const c_void,
        block: AmmlBlockHandle,
        slot: u8,
        out_id: *mut u64,
    ) -> AmmlStatus {
        let Some(state) = FakeState::current() else {
            return AmmlStatus::FAILED;
        };
        if out_id.is_null() {
            return AmmlStatus::INVALID_ARGUMENT;
        }
        let fake = match state.client_check(state.client_handle(), Area::INVENTORY_REQUEST.at(2)) {
            Ok(fake) => fake,
            Err(status) => return status,
        };
        if !fake.local_player || player.addr() != state.player_handle() {
            return AmmlStatus::REFUSED;
        }
        let Some(block) = crate::BlockHandle::from_raw(block) else {
            return AmmlStatus::INVALID_ARGUMENT;
        };
        if slot >= 9 || state.requests().running() {
            return AmmlStatus::REFUSED;
        }
        let id = state.requests().take_pick(block, slot);
        Self::write(out_id, id.0)
    }

    // The loader rule of a move: slots below 36, no swap or place of a slot with itself, no zero count.
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
        Self::actor_check_in(player, Area::ACTOR_READ.at(2))
    }

    // The fake client of a read of the local player `player` in `area`.
    fn actor_check_in(player: *const c_void, area: Area) -> Result<(std::rc::Rc<FakeState>, FakeClient), AmmlStatus> {
        let state = FakeState::current().ok_or(AmmlStatus::FAILED)?;
        let fake = state.client_check(state.client_handle(), area)?;
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

    unsafe extern "C" fn actor_head_position(player: *const c_void, out: *mut AmmlVec3) -> AmmlStatus {
        match Self::actor_check_in(player, Area::ACTOR_READ.at(3)).map(|(_, fake)| fake.head_position) {
            Ok(Some(value)) => Self::write(out, AmmlVec3 { x: value.x, y: value.y, z: value.z }),
            Ok(None) => AmmlStatus::NOT_AVAILABLE,
            Err(status) => status,
        }
    }

    // The fake block source handle sits next to the player handle; the fake gives one only for a client with a world.
    unsafe extern "C" fn actor_block_source(player: *const c_void, out: *mut *const c_void) -> AmmlStatus {
        match Self::actor_check_in(player, Area::ACTOR_READ.at(3)) {
            Ok((state, fake)) if fake.world.is_some() => {
                Self::write(out, std::ptr::with_exposed_provenance(state.player_handle() + Self::BLOCK_SOURCE_OFFSET))
            }
            Ok(_) => AmmlStatus::NOT_AVAILABLE,
            Err(status) => status,
        }
    }

    unsafe extern "C" fn actor_view_vector(player: *const c_void, out: *mut AmmlVec3) -> AmmlStatus {
        match Self::actor_check_in(player, Area::ACTOR_READ.at(4)).map(|(_, fake)| fake.view_vector) {
            Ok(Some(value)) => Self::write(out, AmmlVec3 { x: value.x, y: value.y, z: value.z }),
            Ok(None) => AmmlStatus::NOT_AVAILABLE,
            Err(status) => status,
        }
    }

    unsafe extern "C" fn actor_aabb(player: *const c_void, out: *mut AmmlAabb) -> AmmlStatus {
        match Self::actor_check_in(player, Area::ACTOR_READ.at(4)).map(|(_, fake)| fake.aabb) {
            Ok(Some(value)) => Self::write(out, AmmlAabb::from(value)),
            Ok(None) => AmmlStatus::NOT_AVAILABLE,
            Err(status) => status,
        }
    }

    unsafe extern "C" fn actor_game_type(player: *const c_void, out: *mut AmmlGameType) -> AmmlStatus {
        match Self::actor_check_in(player, Area::ACTOR_GAME_TYPE).map(|(_, fake)| fake.game_type) {
            Ok(Some(value)) => Self::write(out, AmmlGameType::from(value)),
            Ok(None) => AmmlStatus::NOT_AVAILABLE,
            Err(status) => status,
        }
    }

    unsafe extern "C" fn actor_permission_level(player: *const c_void, out: *mut AmmlPermissionLevel) -> AmmlStatus {
        match Self::actor_check_in(player, Area::ACTOR_PERMISSION).map(|(_, fake)| fake.permission_level) {
            Ok(Some(value)) => Self::write(out, AmmlPermissionLevel::from(value)),
            Ok(None) => AmmlStatus::NOT_AVAILABLE,
            Err(status) => status,
        }
    }

    // A write that is not finite gives `INVALID_ARGUMENT` and changes nothing, as the loader does.
    unsafe extern "C" fn actor_set_rotation(
        player: *const c_void,
        rotation: *const AmmlRotation,
        head_yaw: f32,
        has_head_yaw: u8,
    ) -> AmmlStatus {
        if rotation.is_null() {
            return AmmlStatus::INVALID_ARGUMENT;
        }
        // SAFETY: the caller passes a readable rotation.
        let rotation = unsafe { rotation.read() };
        let head = (has_head_yaw != 0).then_some(head_yaw);
        if !rotation.pitch.is_finite() || !rotation.yaw.is_finite() || head.is_some_and(|head| !head.is_finite()) {
            return AmmlStatus::INVALID_ARGUMENT;
        }
        match Self::actor_check_in(player, Area::ACTOR_LOOK_WRITE) {
            Ok((state, _)) => {
                state.write_rotation(crate::Rotation::from(rotation), head);
                AmmlStatus::OK
            }
            Err(status) => status,
        }
    }

    // A face that is not one of the six or a click that is not finite gives `INVALID_ARGUMENT`, as the loader does. The
    // build raises `BuildBlock` with the origin `THIS_MOD` (the fake has one mod); a cancel gives 0 and records nothing.
    unsafe extern "C" fn actor_build_at(
        _module: AmmlStr,
        player: *const c_void,
        pos: *const AmmlBlockPos,
        face: amml_abi::AmmlFace,
        click: *const AmmlVec3,
        out: *mut u8,
    ) -> AmmlStatus {
        if pos.is_null() || click.is_null() || out.is_null() {
            return AmmlStatus::INVALID_ARGUMENT;
        }
        // SAFETY: the caller passes a readable position and click point.
        let (pos, click) = unsafe { (pos.read(), click.read()) };
        if face.0 > amml_abi::AmmlFace::EAST.0 || !click.x.is_finite() || !click.y.is_finite() || !click.z.is_finite() {
            return AmmlStatus::INVALID_ARGUMENT;
        }
        match Self::actor_check_in(player, Area::PLAYER_BUILD) {
            Ok((state, _)) => {
                let built = FakeGame::raise_own_build(&state, crate::BlockPos::from(pos), crate::Face::from(face));
                if built {
                    state.record_build(crate::BlockPos::from(pos), crate::Face::from(face));
                }
                Self::write(out, u8::from(built))
            }
            Err(status) => status,
        }
    }

    unsafe extern "C" fn actor_build_delay(player: *const c_void, out: *mut u32) -> AmmlStatus {
        match Self::actor_check_in(player, Area::PLAYER_BUILD).map(|(_, fake)| fake.build_delay) {
            Ok(Some(value)) => Self::write(out, value),
            Ok(None) => AmmlStatus::NOT_AVAILABLE,
            Err(status) => status,
        }
    }

    unsafe extern "C" fn actor_hit_result(player: *const c_void, out: *mut AmmlHitResult) -> AmmlStatus {
        match Self::actor_check_in(player, Area::PLAYER_HIT_RESULT).map(|(_, fake)| fake.hit_result) {
            Ok(Some(hit)) => Self::write(out, AmmlHitResult::from(hit)),
            Ok(None) => AmmlStatus::NOT_AVAILABLE,
            Err(status) => status,
        }
    }

    unsafe extern "C" fn actor_max_pick_range(player: *const c_void, out: *mut f32) -> AmmlStatus {
        match Self::actor_check_in(player, Area::PLAYER_PICK_RANGE).map(|(_, fake)| fake.max_pick_range) {
            Ok(Some(value)) => Self::write(out, value),
            Ok(None) => AmmlStatus::NOT_AVAILABLE,
            Err(status) => status,
        }
    }

    unsafe extern "C" fn actor_health(player: *const c_void, out: *mut i32) -> AmmlStatus {
        match Self::actor_check_in(player, Area::ACTOR_HEALTH).map(|(_, fake)| fake.health) {
            Ok(Some(value)) => Self::write(out, value),
            Ok(None) => AmmlStatus::NOT_AVAILABLE,
            Err(status) => status,
        }
    }

    // The camera position of the running `WorldRendered` dispatch of `frame`, as the loader gives it.
    unsafe extern "C" fn render_canvas_camera_position(frame: u64, out: *mut AmmlVec3) -> AmmlStatus {
        let Some(state) = FakeState::current() else {
            return AmmlStatus::FAILED;
        };
        if !state.area_offered(Area::RENDER_WORLD_CANVAS.id, Area::RENDER_WORLD_CANVAS.version) {
            return AmmlStatus::NOT_AVAILABLE;
        }
        match state.render().camera_position(frame) {
            Ok(position) => Self::write(out, AmmlVec3 { x: position.x, y: position.y, z: position.z }),
            Err(status) => status,
        }
    }

    // The clip matrix of the running `WorldRendered` dispatch of `frame`, as the loader gives it (area version 3).
    unsafe extern "C" fn render_canvas_clip_matrix(frame: u64, out: *mut [[f32; 4]; 4]) -> AmmlStatus {
        let Some(state) = FakeState::current() else {
            return AmmlStatus::FAILED;
        };
        let area = Area::RENDER_WORLD_CANVAS.at(3);
        if !state.area_offered(area.id, area.version) {
            return AmmlStatus::NOT_AVAILABLE;
        }
        if out.is_null() {
            return AmmlStatus::INVALID_ARGUMENT;
        }
        match state.render().clip_matrix(frame) {
            Ok(matrix) => {
                // SAFETY: `out` is non-null and the caller promises it is writable.
                unsafe { out.write_unaligned(matrix) };
                AmmlStatus::OK
            }
            Err(status) => status,
        }
    }

    // Frees a mesh of the fake mod; a mesh of an older generation or one freed before gives `STALE`.
    unsafe extern "C" fn render_mesh_free(_module: AmmlStr, mesh: AmmlMeshId) -> AmmlStatus {
        let Some(state) = FakeState::current() else {
            return AmmlStatus::FAILED;
        };
        if !state.area_offered(Area::RENDER_WORLD_CANVAS.id, Area::RENDER_WORLD_CANVAS.version) {
            return AmmlStatus::NOT_AVAILABLE;
        }
        match state.render().free(mesh.into()) {
            Ok(()) => AmmlStatus::OK,
            Err(status) => status,
        }
    }

    // Builds a mesh of the fake mod in the running dispatch of `frame`, with the loader's checks.
    unsafe extern "C" fn render_mesh_build(
        _module: AmmlStr,
        frame: u64,
        quads: *const AmmlColorQuad,
        count: u32,
        style: AmmlMeshStyle,
        out: *mut AmmlMeshId,
    ) -> AmmlStatus {
        let Some(state) = FakeState::current() else {
            return AmmlStatus::FAILED;
        };
        let canvas = Area::RENDER_WORLD_CANVAS.at(2);
        if !state.area_offered(canvas.id, canvas.version) {
            return AmmlStatus::NOT_AVAILABLE;
        }
        if quads.is_null() || out.is_null() {
            return AmmlStatus::INVALID_ARGUMENT;
        }
        // SAFETY: the caller promises `count` quads readable at `quads` for the call.
        let quads = unsafe { std::slice::from_raw_parts(quads, count as usize) };
        match state.render().build(frame, quads, style) {
            Ok(mesh) => Self::write(out, mesh.into()),
            Err(status) => status,
        }
    }

    // Makes a ghost of the fake mod with the loader's checks (area `render.block_entities`, the gate and the world, then
    // the options).
    unsafe extern "C" fn render_block_entity_create(
        _module: AmmlStr,
        block: AmmlBlockHandle,
        nbt: *const u8,
        nbt_len: usize,
        options: *const AmmlGhostEntityOptions,
        out: *mut AmmlBlockEntityGhost,
    ) -> AmmlStatus {
        let Some(state) = FakeState::current() else {
            return AmmlStatus::FAILED;
        };
        let area = Area::RENDER_BLOCK_ENTITIES;
        if !state.area_offered(area.id, area.version) {
            return AmmlStatus::NOT_AVAILABLE;
        }
        if out.is_null() || options.is_null() || (nbt.is_null() && nbt_len != 0) {
            return AmmlStatus::INVALID_ARGUMENT;
        }
        if !state.gate().get() || state.epoch().get() == 0 {
            return AmmlStatus::NOT_READY;
        }
        // SAFETY: the caller promises readable options for the call.
        let options = unsafe { options.read() };
        let (pair, skull) = (Area::RENDER_CHEST_PAIR, Area::RENDER_SKULL_POSE);
        let offered = (state.area_offered(pair.id, pair.version), state.area_offered(skull.id, skull.version));
        match state.block_entities().create(block, nbt_len, &options, offered.0, offered.1) {
            Ok(ghost) => Self::write(out, ghost),
            Err(status) => status,
        }
    }

    unsafe extern "C" fn render_block_entity_free(_module: AmmlStr, ghost: AmmlBlockEntityGhost) -> AmmlStatus {
        let Some(state) = FakeState::current() else {
            return AmmlStatus::FAILED;
        };
        match state.block_entities().free(ghost) {
            Ok(()) => AmmlStatus::OK,
            Err(status) => status,
        }
    }

    // Draws a ghost of the fake mod inside the running `BlockEntityPass` dispatch of `frame`.
    unsafe extern "C" fn render_block_entity_draw(
        _module: AmmlStr,
        frame: u64,
        ghost: AmmlBlockEntityGhost,
        pos: *const AmmlBlockPos,
    ) -> AmmlStatus {
        let Some(state) = FakeState::current() else {
            return AmmlStatus::FAILED;
        };
        let area = Area::RENDER_BLOCK_ENTITIES;
        if !state.area_offered(area.id, area.version) {
            return AmmlStatus::NOT_AVAILABLE;
        }
        if pos.is_null() {
            return AmmlStatus::INVALID_ARGUMENT;
        }
        // SAFETY: the caller promises a readable position for the call.
        let pos = unsafe { pos.read() };
        match state.block_entities().draw(frame, ghost, [pos.x, pos.y, pos.z]) {
            Ok(()) => AmmlStatus::OK,
            Err(status) => status,
        }
    }

    // Builds a grid into a mesh of the fake mod in the running dispatch of `frame`, with the loader's checks (area
    // `render.block_mesher`, the frame, the grid); nothing is skipped.
    unsafe extern "C" fn render_mesher_build(
        _module: AmmlStr,
        frame: u64,
        grid: *const AmmlBlockGrid,
        options: *const AmmlMesherOptions,
        out: *mut AmmlMeshBuilt,
        _skipped: *mut u32,
        _skipped_cap: u32,
    ) -> AmmlStatus {
        let Some(state) = FakeState::current() else {
            return AmmlStatus::FAILED;
        };
        let mesher = Area::RENDER_BLOCK_MESHER;
        if !state.area_offered(mesher.id, mesher.version) {
            return AmmlStatus::NOT_AVAILABLE;
        }
        if grid.is_null() || options.is_null() || out.is_null() {
            return AmmlStatus::INVALID_ARGUMENT;
        }
        // SAFETY: the caller promises a readable grid for the call.
        let grid = unsafe { grid.read() };
        let slice = |pointer: *const u16, count: u32| {
            if count == 0 {
                &[][..]
            } else {
                // SAFETY: the caller promises `count` values readable at `pointer` for the call.
                unsafe { std::slice::from_raw_parts(pointer, count as usize) }
            }
        };
        let cells = slice(grid.cells, grid.cell_count);
        let palette = if grid.palette_len == 0 {
            &[][..]
        } else {
            // SAFETY: the caller promises `palette_len` handles readable at `palette` for the call.
            unsafe { std::slice::from_raw_parts(grid.palette, grid.palette_len as usize) }
        };
        match state.render().mesher_build(frame, grid.dims, cells, palette) {
            Ok((mesh, quads)) => {
                let mesh = mesh.map(AmmlMeshId::from).unwrap_or_default();
                Self::write(out, AmmlMeshBuilt { mesh, quads, skipped_count: 0 })
            }
            Err(status) => status,
        }
    }

    // Opens a group of the fake mod in the running dispatch of `frame`, with the loader's checks (area
    // `render.block_mesher`, the frame, the slot).
    unsafe extern "C" fn render_mesher_group_begin(
        _module: AmmlStr,
        frame: u64,
        slot_size: *const [u32; 3],
        out: *mut AmmlMeshGroup,
    ) -> AmmlStatus {
        let Some(state) = FakeState::current() else {
            return AmmlStatus::FAILED;
        };
        let mesher = Area::RENDER_BLOCK_MESHER;
        if !state.area_offered(mesher.id, mesher.version) {
            return AmmlStatus::NOT_AVAILABLE;
        }
        if slot_size.is_null() || out.is_null() {
            return AmmlStatus::INVALID_ARGUMENT;
        }
        // SAFETY: the caller promises a readable slot size for the call.
        let slot_size = unsafe { slot_size.read() };
        match state.render().group_begin(frame, slot_size) {
            Ok(group) => Self::write(out, group),
            Err(status) => status,
        }
    }

    // Appends a grid to a group of the fake mod in the running dispatch of `frame`, with the loader's checks.
    unsafe extern "C" fn render_mesher_group_append(
        _module: AmmlStr,
        frame: u64,
        group: AmmlMeshGroup,
        grid: *const AmmlBlockGrid,
        offset: *const AmmlBlockPos,
        options: *const AmmlMesherOptions,
    ) -> AmmlStatus {
        let Some(state) = FakeState::current() else {
            return AmmlStatus::FAILED;
        };
        let mesher = Area::RENDER_BLOCK_MESHER;
        if !state.area_offered(mesher.id, mesher.version) {
            return AmmlStatus::NOT_AVAILABLE;
        }
        if grid.is_null() || offset.is_null() || options.is_null() {
            return AmmlStatus::INVALID_ARGUMENT;
        }
        // SAFETY: the caller promises a readable grid and offset for the call.
        let (grid, offset) = unsafe { (grid.read(), offset.read()) };
        let (cells, palette) = Self::grid_slices(&grid);
        match state.render().group_append(frame, group, (grid.dims, cells, palette), [offset.x, offset.y, offset.z]) {
            Ok(()) => AmmlStatus::OK,
            Err(status) => status,
        }
    }

    // Ends a group of the fake mod into a mesh of it in the running dispatch of `frame`.
    unsafe extern "C" fn render_mesher_group_finish(
        _module: AmmlStr,
        frame: u64,
        group: AmmlMeshGroup,
        out: *mut AmmlMeshBuilt,
    ) -> AmmlStatus {
        let Some(state) = FakeState::current() else {
            return AmmlStatus::FAILED;
        };
        let mesher = Area::RENDER_BLOCK_MESHER;
        if !state.area_offered(mesher.id, mesher.version) {
            return AmmlStatus::NOT_AVAILABLE;
        }
        if out.is_null() {
            return AmmlStatus::INVALID_ARGUMENT;
        }
        match state.render().group_finish(frame, group) {
            Ok((mesh, quads)) => {
                let mesh = mesh.map(AmmlMeshId::from).unwrap_or_default();
                Self::write(out, AmmlMeshBuilt { mesh, quads, skipped_count: 0 })
            }
            Err(status) => status,
        }
    }

    // Ends a group of the fake mod without a mesh, from any frame or none.
    unsafe extern "C" fn render_mesher_group_discard(_module: AmmlStr, group: AmmlMeshGroup) -> AmmlStatus {
        let Some(state) = FakeState::current() else {
            return AmmlStatus::FAILED;
        };
        let mesher = Area::RENDER_BLOCK_MESHER;
        if !state.area_offered(mesher.id, mesher.version) {
            return AmmlStatus::NOT_AVAILABLE;
        }
        match state.render().group_discard(group) {
            Ok(()) => AmmlStatus::OK,
            Err(status) => status,
        }
    }

    // The cells and the palette of a grid the caller promises are readable for their counts.
    fn grid_slices(grid: &AmmlBlockGrid) -> (&[u16], &[AmmlBlockHandle]) {
        let cells = if grid.cell_count == 0 {
            &[][..]
        } else {
            // SAFETY: the caller promises `cell_count` cells readable at `cells` for the call.
            unsafe { std::slice::from_raw_parts(grid.cells, grid.cell_count as usize) }
        };
        let palette = if grid.palette_len == 0 {
            &[][..]
        } else {
            // SAFETY: the caller promises `palette_len` handles readable at `palette` for the call.
            unsafe { std::slice::from_raw_parts(grid.palette, grid.palette_len as usize) }
        };
        (cells, palette)
    }

    // Draws a mesh of the fake mod in the running dispatch of `frame`: records the line.
    unsafe extern "C" fn render_mesh_draw(
        _module: AmmlStr,
        frame: u64,
        mesh: AmmlMeshId,
        transform: *const AmmlTransform,
    ) -> AmmlStatus {
        let Some(state) = FakeState::current() else {
            return AmmlStatus::FAILED;
        };
        let canvas = Area::RENDER_WORLD_CANVAS.at(2);
        if !state.area_offered(canvas.id, canvas.version) {
            return AmmlStatus::NOT_AVAILABLE;
        }
        if transform.is_null() {
            return AmmlStatus::INVALID_ARGUMENT;
        }
        // SAFETY: the caller promises a readable transform for the call.
        let transform = unsafe { transform.read() };
        match state.render().draw(frame, mesh.into(), &transform) {
            Ok(()) => AmmlStatus::OK,
            Err(status) => status,
        }
    }

    // A pose of the fake mod in the running `CameraSetup` dispatch of `frame`.
    unsafe extern "C" fn camera_set_pose(_module: AmmlStr, frame: u64, pose: *const AmmlPose) -> AmmlStatus {
        let Some(state) = FakeState::current() else {
            return AmmlStatus::FAILED;
        };
        if !state.area_offered(Area::CAMERA_POSE.id, Area::CAMERA_POSE.version) {
            return AmmlStatus::NOT_AVAILABLE;
        }
        if pose.is_null() {
            return AmmlStatus::INVALID_ARGUMENT;
        }
        // SAFETY: the caller promises a readable pose for the call.
        let pose = unsafe { pose.read() };
        match state.camera().set_pose(frame, pose) {
            Ok(()) => AmmlStatus::OK,
            Err(status) => status,
        }
    }

    // The rendered position the fake camera of the running dispatch of `frame` gives.
    unsafe extern "C" fn camera_rendered_position(frame: u64, out: *mut AmmlVec3) -> AmmlStatus {
        let Some(state) = FakeState::current() else {
            return AmmlStatus::FAILED;
        };
        if !state.area_offered(Area::CAMERA_POSE.id, Area::CAMERA_POSE.version) {
            return AmmlStatus::NOT_AVAILABLE;
        }
        match state.camera().rendered(frame) {
            Ok(position) => Self::write(out, AmmlVec3::from(position)),
            Err(status) => status,
        }
    }

    // The projection the fake camera of the running After dispatch of `frame` gives.
    unsafe extern "C" fn camera_projection(frame: u64, out: *mut AmmlProjection) -> AmmlStatus {
        let Some(state) = FakeState::current() else {
            return AmmlStatus::FAILED;
        };
        if !state.area_offered(Area::CAMERA_PROJECTION.id, Area::CAMERA_PROJECTION.version) {
            return AmmlStatus::NOT_AVAILABLE;
        }
        match state.camera().projection(frame) {
            Ok(projection) => Self::write(out, AmmlProjection::from(projection)),
            Err(status) => status,
        }
    }

    // The area, the gate and a joined world of a free camera call, as the loader checks them.
    fn free_camera_check(state: &FakeState) -> Result<(), AmmlStatus> {
        if !state.area_offered(Area::CAMERA_FREE.id, Area::CAMERA_FREE.version) {
            return Err(AmmlStatus::NOT_AVAILABLE);
        }
        if !state.gate().get() || state.epoch().get() == 0 {
            return Err(AmmlStatus::NOT_READY);
        }
        Ok(())
    }

    // A `set_free` of the fake mod; on success the fake raises the instruction to the mod's listeners with origin
    // `THIS_MOD`, as the loader does from the game's handler.
    unsafe extern "C" fn camera_set_free(_module: AmmlStr, pose: *const AmmlPose) -> AmmlStatus {
        let Some(state) = FakeState::current() else {
            return AmmlStatus::FAILED;
        };
        if let Err(status) = Self::free_camera_check(&state) {
            return status;
        }
        if pose.is_null() {
            return AmmlStatus::INVALID_ARGUMENT;
        }
        // SAFETY: the caller promises a readable pose for the call.
        let pose = unsafe { pose.read() };
        match state.camera_free().set_free(pose) {
            Ok(()) => {
                FakeGame::raise_own_instruction(&state, InstructionKind::Set);
                AmmlStatus::OK
            }
            Err(status) => status,
        }
    }

    // A `release_free` of the fake mod; the clear is raised as `set_free` raises the set, when the mod held the camera.
    unsafe extern "C" fn camera_release_free(_module: AmmlStr) -> AmmlStatus {
        let Some(state) = FakeState::current() else {
            return AmmlStatus::FAILED;
        };
        if let Err(status) = Self::free_camera_check(&state) {
            return status;
        }
        match state.camera_free().release_free() {
            Ok(sent) => {
                if sent {
                    FakeGame::raise_own_instruction(&state, InstructionKind::Clear);
                }
                AmmlStatus::OK
            }
            Err(status) => status,
        }
    }

    // A `hide_actor` of the fake mod: the area as the loader checks it (any thread, no gate), then the target.
    unsafe extern "C" fn camera_hide_actor(_module: AmmlStr, target: AmmlActorTarget, on: u8) -> AmmlStatus {
        let Some(state) = FakeState::current() else {
            return AmmlStatus::FAILED;
        };
        if !state.area_offered(Area::CAMERA_HIDE_ACTOR.id, Area::CAMERA_HIDE_ACTOR.version) {
            return AmmlStatus::NOT_AVAILABLE;
        }
        if target != AmmlActorTarget::LOCAL_PLAYER {
            return AmmlStatus::INVALID_ARGUMENT;
        }
        state.camera().set_body_hidden(on != 0);
        AmmlStatus::OK
    }

    // The active camera of the fake client, when the area is offered and a world is joined.
    unsafe extern "C" fn camera_active_camera(client: *const c_void, out: *mut AmmlCameraKind) -> AmmlStatus {
        let Some(state) = FakeState::current() else {
            return AmmlStatus::FAILED;
        };
        let fake = match state.client_check(client.addr(), Area::CAMERA_ACTIVE) {
            Ok(fake) => fake,
            Err(status) => return status,
        };
        match fake.active_camera.filter(|_| fake.local_player) {
            Some(kind) => Self::write(out, kind.to_abi()),
            None => AmmlStatus::FAILED,
        }
    }

    // The gate and the offer of an options area, as the loader checks them.
    fn options_check(state: &FakeState) -> Result<(), AmmlStatus> {
        if !state.gate().get() {
            return Err(AmmlStatus::NOT_READY);
        }
        if !state.area_offered(Area::OPTIONS_INT.id, Area::OPTIONS_INT.version) {
            return Err(AmmlStatus::NOT_AVAILABLE);
        }
        Ok(())
    }

    // The view distance of the fake client.
    unsafe extern "C" fn options_view_distance(client: *const c_void, out: *mut u32) -> AmmlStatus {
        let Some(state) = FakeState::current() else {
            return AmmlStatus::FAILED;
        };
        if let Err(status) = state.client_check(client.addr(), Area::OPTIONS_VIEW_DISTANCE) {
            return status;
        }
        Self::write(out, state.options().view_distance())
    }

    unsafe extern "C" fn options_get_int(key: AmmlOptionKey, out: *mut i32) -> AmmlStatus {
        let Some(state) = FakeState::current() else {
            return AmmlStatus::FAILED;
        };
        if let Err(status) = Self::options_check(&state) {
            return status;
        }
        match state.options().get_int(key) {
            Ok(value) => Self::write(out, value),
            Err(status) => status,
        }
    }

    unsafe extern "C" fn options_set_int(
        _module: AmmlStr,
        key: AmmlOptionKey,
        value: i32,
        persistence: AmmlPersistence,
    ) -> AmmlStatus {
        let Some(state) = FakeState::current() else {
            return AmmlStatus::FAILED;
        };
        if persistence != AmmlPersistence::TRANSIENT {
            return AmmlStatus::INVALID_ARGUMENT;
        }
        if let Err(status) = Self::options_check(&state) {
            return status;
        }
        Self::status(state.options().set_int(key, value))
    }

    unsafe extern "C" fn options_release_transient(_module: AmmlStr, key: AmmlOptionKey) -> AmmlStatus {
        let Some(state) = FakeState::current() else {
            return AmmlStatus::FAILED;
        };
        if let Err(status) = Self::options_check(&state) {
            return status;
        }
        Self::status(state.options().release(key))
    }

    // The gate and the offer of the time area, as the loader checks them.
    fn time_check(state: &FakeState) -> Result<(), AmmlStatus> {
        if !state.area_offered(Area::TIME_CONTROL.id, Area::TIME_CONTROL.version) {
            return Err(AmmlStatus::NOT_AVAILABLE);
        }
        if !state.gate().get() {
            return Err(AmmlStatus::NOT_READY);
        }
        Ok(())
    }

    // A hold of the fake mod; it needs a joined world (epoch above 0).
    unsafe extern "C" fn time_hold(
        _module: AmmlStr,
        request: *const AmmlHoldRequest,
        out: *mut AmmlTimeHold,
    ) -> AmmlStatus {
        let Some(state) = FakeState::current() else {
            return AmmlStatus::FAILED;
        };
        if request.is_null() || out.is_null() {
            return AmmlStatus::INVALID_ARGUMENT;
        }
        if let Err(status) = Self::time_check(&state) {
            return status;
        }
        if state.epoch().get() == 0 {
            return AmmlStatus::NOT_READY;
        }
        // SAFETY: `request` is non-null and the caller promises it is readable.
        match state.time().hold(unsafe { request.read() }) {
            Ok(hold) => Self::write(out, hold),
            Err(status) => status,
        }
    }

    unsafe extern "C" fn time_step_us(hold: AmmlTimeHold, micros: i64) -> AmmlStatus {
        let Some(state) = FakeState::current() else {
            return AmmlStatus::FAILED;
        };
        if let Err(status) = Self::time_check(&state) {
            return status;
        }
        Self::status(state.time().step_us(hold, micros))
    }

    unsafe extern "C" fn time_release(hold: AmmlTimeHold) -> AmmlStatus {
        let Some(state) = FakeState::current() else {
            return AmmlStatus::FAILED;
        };
        Self::status(state.time().release(hold))
    }

    unsafe extern "C" fn time_state(out: *mut AmmlTimeState) -> AmmlStatus {
        let Some(state) = FakeState::current() else {
            return AmmlStatus::FAILED;
        };
        if !state.area_offered(Area::TIME_CONTROL.id, Area::TIME_CONTROL.version) {
            return AmmlStatus::NOT_AVAILABLE;
        }
        Self::write(out, state.time().state())
    }

    // Whether the fake world has the free preset.
    unsafe extern "C" fn camera_free_available(out: *mut u8) -> AmmlStatus {
        let Some(state) = FakeState::current() else {
            return AmmlStatus::FAILED;
        };
        if let Err(status) = Self::free_camera_check(&state) {
            return status;
        }
        Self::write(out, u8::from(state.camera_free().available()))
    }

    // A shapes apply of the fake mod: the area, the gate and a joined world as the loader checks them, then the fake's
    // rules (`FakeShapes::apply`).
    unsafe extern "C" fn world_shapes_apply(
        _module: AmmlStr,
        dimension: i32,
        ops: *const AmmlShapeOp,
        count: u32,
        out: *mut AmmlApplyStats,
    ) -> AmmlStatus {
        let Some(state) = FakeState::current() else {
            return AmmlStatus::FAILED;
        };
        if !state.area_offered(Area::WORLD_SHAPES.id, Area::WORLD_SHAPES.version) {
            return AmmlStatus::NOT_AVAILABLE;
        }
        if !state.gate().get() || state.epoch().get() == 0 {
            return AmmlStatus::NOT_READY;
        }
        if ops.is_null() && count != 0 {
            return AmmlStatus::INVALID_ARGUMENT;
        }
        let ops: &[AmmlShapeOp] = if count == 0 {
            &[]
        } else {
            // SAFETY: the caller promises `count` readable ops for the call.
            unsafe { std::slice::from_raw_parts(ops, count as usize) }
        };
        match state.shapes().apply(dimension, ops) {
            Ok(delivered) => {
                if !out.is_null() {
                    // SAFETY: `out` is non-null and the caller promises it is writable.
                    unsafe { out.write(AmmlApplyStats { delivered, elapsed_us: 0 }) };
                }
                AmmlStatus::OK
            }
            Err(status) => status,
        }
    }

    unsafe extern "C" fn world_shapes_max_batch() -> u32 {
        FakeShapes::MAX_BATCH as u32
    }

    // The target epoch of the fake; `NOT_AVAILABLE` without the area.
    unsafe extern "C" fn world_shapes_target_epoch(out: *mut u32) -> AmmlStatus {
        let Some(state) = FakeState::current() else {
            return AmmlStatus::FAILED;
        };
        if !state.area_offered(Area::WORLD_SHAPES.id, Area::WORLD_SHAPES.version) {
            return AmmlStatus::NOT_AVAILABLE;
        }
        match state.shapes().target_epoch() {
            Ok(epoch) => Self::write(out, epoch),
            Err(status) => status,
        }
    }

    // The sky darkness of the fake dimension; the handle must be the one `actor_dimension` gave.
    unsafe extern "C" fn world_sky_darken(dimension: *const c_void, out: *mut u8) -> AmmlStatus {
        let Some(state) = FakeState::current() else {
            return AmmlStatus::FAILED;
        };
        let fake = match state.client_check(state.client_handle(), Area::WORLD_SKY_DARKEN) {
            Ok(fake) => fake,
            Err(status) => return status,
        };
        if !fake.local_player
            || fake.dimension.is_none()
            || dimension.addr() != state.player_handle() + Self::DIMENSION_OFFSET
        {
            return AmmlStatus::REFUSED;
        }
        match fake.sky_darken {
            Some(value) => Self::write(out, value),
            None => AmmlStatus::NOT_AVAILABLE,
        }
    }

    // The fake client of a world call that takes a block source: the handle must be the one `actor_block_source` gave.
    fn world_source_check(source: *const c_void) -> Result<(std::rc::Rc<FakeState>, FakeClient), AmmlStatus> {
        Self::world_source_check_in(source, Area::WORLD_CHUNK_READ)
    }

    // As `world_source_check`, for a call of `area`.
    fn world_source_check_in(
        source: *const c_void,
        area: Area,
    ) -> Result<(std::rc::Rc<FakeState>, FakeClient), AmmlStatus> {
        let state = FakeState::current().ok_or(AmmlStatus::FAILED)?;
        let fake = state.client_check(state.client_handle(), area)?;
        if !fake.local_player
            || fake.world.is_none()
            || source.addr() != state.player_handle() + Self::BLOCK_SOURCE_OFFSET
        {
            return Err(AmmlStatus::REFUSED);
        }
        Ok((state, fake))
    }

    // The chunk a chunk handle names, of the fake world of a client with a local player.
    fn world_chunk_check(chunk: *const c_void) -> Result<crate::testing::FakeChunk, AmmlStatus> {
        let state = FakeState::current().ok_or(AmmlStatus::FAILED)?;
        let fake = state.client_check(state.client_handle(), Area::WORLD_CHUNK_READ)?;
        let position = state.chunk_of_handle(chunk.addr()).ok_or(AmmlStatus::REFUSED)?;
        if !fake.local_player {
            return Err(AmmlStatus::REFUSED);
        }
        fake.world.and_then(|world| world.chunks.get(&position).cloned()).ok_or(AmmlStatus::REFUSED)
    }

    // The chunk of the fake world at (`x`, `z`) as a handle, or null when the world has none.
    unsafe extern "C" fn world_chunk(source: *const c_void, x: i32, z: i32, out: *mut *const c_void) -> AmmlStatus {
        match Self::world_source_check(source) {
            Ok((state, fake)) => {
                let has = fake.world.is_some_and(|world| world.chunks.contains_key(&(x, z)));
                let handle =
                    if has { std::ptr::with_exposed_provenance(state.chunk_handle(x, z)) } else { std::ptr::null() };
                Self::write(out, handle)
            }
            Err(status) => status,
        }
    }

    unsafe extern "C" fn world_chunk_load_state(chunk: *const c_void, out: *mut AmmlChunkLoad) -> AmmlStatus {
        match Self::world_chunk_check(chunk) {
            Ok(chunk) => Self::write(out, chunk.load.to_abi()),
            Err(status) => status,
        }
    }

    // Copies the samples of the asked columns of a loaded chunk; marks them skipped in any other state.
    unsafe extern "C" fn world_read_columns(
        chunk: *const c_void,
        request: *const AmmlColumnRequest,
        out: *mut AmmlColumnSample,
        len: u32,
        stats: *mut AmmlColumnReadStats,
    ) -> AmmlStatus {
        if request.is_null() || out.is_null() || stats.is_null() || len != 256 {
            return AmmlStatus::INVALID_ARGUMENT;
        }
        let chunk = match Self::world_chunk_check(chunk) {
            Ok(chunk) => chunk,
            Err(status) => return status,
        };
        // SAFETY: `request` is non-null and the caller promises it is readable; `out` holds `len` (256) samples.
        let (request, out) = unsafe { (request.read(), std::slice::from_raw_parts_mut(out, 256)) };
        let state = FakeState::current();
        let offered = |area: Area| state.as_ref().is_some_and(|state| state.area_offered(area.id, area.version));
        let colors = request.flags & AmmlColumnRequest::VANILLA_COLORS != 0 && offered(Area::WORLD_MAP_COLOR);
        let biomes = offered(Area::WORLD_BIOMES);
        let mut counts = AmmlColumnReadStats::default();
        for (index, slot) in out.iter_mut().enumerate() {
            if (request.columns[index / 64] >> (index % 64)) & 1 == 0 {
                continue;
            }
            let mut sample = chunk.samples[index];
            if chunk.load != crate::ChunkLoad::Loaded || sample.skipped() {
                *slot = AmmlColumnSample::from(crate::ColumnSample {
                    flags: crate::ColumnSample::SKIPPED,
                    ..Default::default()
                });
                counts.skipped += 1;
                continue;
            }
            // The loader's rules: no colour without the area and the request flag, no biome without its area.
            if !colors {
                sample.shown_color = crate::Rgba::default();
                sample.water_color = crate::Rgba::default();
                if sample.shown != crate::BlockTypeId::NONE {
                    sample.flags |= crate::ColumnSample::NO_COLOR;
                }
            }
            if !biomes {
                sample.biome = crate::ColumnSample::NO_BIOME;
            }
            *slot = AmmlColumnSample::from(sample);
            counts.read += 1;
            if sample.retry() {
                counts.retry += 1;
            }
        }
        Self::write(stats, counts)
    }

    // The name of a block type of the fake world.
    unsafe extern "C" fn world_block_type_name(id: u32, buf: *mut u8, cap: usize, out_len: *mut usize) -> AmmlStatus {
        let Some(state) = FakeState::current() else {
            return AmmlStatus::FAILED;
        };
        let fake = match state.client_check(state.client_handle(), Area::WORLD_CHUNK_READ) {
            Ok(fake) => fake,
            Err(status) => return status,
        };
        let named = |world: &crate::testing::FakeWorld| {
            let id = crate::BlockTypeId(id);
            world.names.get(&id).cloned().or_else(|| {
                world
                    .blocks
                    .values()
                    .find(|block| block.type_id == id && !block.name.is_empty())
                    .map(|block| block.name.to_string())
            })
        };
        let Some(name) = fake.world.as_ref().and_then(named) else {
            return AmmlStatus::INVALID_ARGUMENT;
        };
        let bytes = name.as_bytes();
        let copied = bytes.len().min(cap);
        if copied > 0 && !buf.is_null() {
            // SAFETY: the caller promises `buf` is writable for `cap` bytes, and `copied <= cap`.
            unsafe { std::ptr::copy_nonoverlapping(bytes.as_ptr(), buf, copied) };
        }
        Self::write(out_len, bytes.len())
    }

    // The block of the fake world for a name and states, from `FakeWorld::states` (exact) or `default_states`; the
    // states are sorted by name and their values written as text, as the maps hold them.
    unsafe extern "C" fn world_block_from_state(
        _module: AmmlStr,
        name: AmmlStr,
        states: *const AmmlStateEntry,
        count: u32,
        out: *mut AmmlBlockMatch,
    ) -> AmmlStatus {
        let Some(state) = FakeState::current() else {
            return AmmlStatus::FAILED;
        };
        let world = match state.block_world(Area::WORLD_BLOCK_STATES) {
            Ok(world) => world,
            Err(status) => return status,
        };
        if out.is_null() || (states.is_null() && count != 0) {
            return AmmlStatus::INVALID_ARGUMENT;
        }
        // SAFETY: the caller promises `name` is valid for the call.
        let Ok(name) = (unsafe { name.as_str() }) else {
            return AmmlStatus::INVALID_ARGUMENT;
        };
        let entries: &[AmmlStateEntry] = if count == 0 {
            &[]
        } else {
            // SAFETY: the caller promises `count` entries are readable for the call.
            unsafe { std::slice::from_raw_parts(states, count as usize) }
        };
        let mut wanted = Vec::new();
        for entry in entries {
            // SAFETY: the caller promises the texts of every entry are valid for the call.
            let Ok(key) = (unsafe { entry.name.as_str() }) else {
                return AmmlStatus::INVALID_ARGUMENT;
            };
            let value = match entry.value.kind {
                AmmlStateValue::INT => entry.value.int.to_string(),
                AmmlStateValue::BOOL => (entry.value.int != 0).to_string(),
                // SAFETY: as above.
                AmmlStateValue::TEXT => match unsafe { entry.value.text.as_str() } {
                    Ok(text) => text.to_string(),
                    Err(status) => return status,
                },
                _ => return AmmlStatus::INVALID_ARGUMENT,
            };
            wanted.push((key.to_string(), value));
        }
        wanted.sort();
        let exact = world.states.get(&(name.to_string(), wanted)).copied();
        let found =
            match exact.map(|block| (block, 1)).or_else(|| world.default_states.get(name).map(|block| (*block, 0))) {
                Some((block, exact)) => AmmlBlockMatch { block: block.raw(), exact, _reserved: [0; 7] },
                None => AmmlBlockMatch { block: AmmlBlockHandle::NONE, exact: 0, _reserved: [0; 7] },
            };
        // SAFETY: `out` is non-null and the caller promises it is writable.
        unsafe { out.write(found) };
        AmmlStatus::OK
    }

    // The turned block of the fake world (`FakeWorld::rotated`, the same handle when absent); each call is recorded.
    unsafe extern "C" fn world_block_rotated(
        _module: AmmlStr,
        block: AmmlBlockHandle,
        quarters: u8,
        out: *mut AmmlBlockHandle,
    ) -> AmmlStatus {
        let Some(state) = FakeState::current() else {
            return AmmlStatus::FAILED;
        };
        let world = match state.block_world(Area::WORLD_BLOCK_STATES) {
            Ok(world) => world,
            Err(status) => return status,
        };
        let Some(handle) = crate::BlockHandle::from_raw(block) else {
            return AmmlStatus::STALE;
        };
        if out.is_null() {
            return AmmlStatus::INVALID_ARGUMENT;
        }
        let quarters = quarters % 4;
        state.note_block_rotation(handle, quarters);
        let turned = world.rotated.get(&(handle, quarters)).copied().unwrap_or(handle);
        // SAFETY: `out` is non-null and the caller promises it is writable.
        unsafe { out.write(turned.raw()) };
        AmmlStatus::OK
    }

    // The fake world and the block a block handle names, for a call of `world.blocks`.
    fn world_block_check(block: *const c_void) -> Result<(crate::testing::FakeBlock, crate::BlockPos), AmmlStatus> {
        let state = FakeState::current().ok_or(AmmlStatus::FAILED)?;
        let fake = state.client_check(state.client_handle(), Area::WORLD_BLOCKS)?;
        let pos = state.block_of_handle(block.addr()).ok_or(AmmlStatus::REFUSED)?;
        let world = fake.world.ok_or(AmmlStatus::REFUSED)?;
        Ok((world.blocks.get(&pos).copied().unwrap_or_else(|| world.air_block()), pos))
    }

    // The block of the fake world at a position, as a handle; null when the world has none there.
    unsafe extern "C" fn world_block_at(
        source: *const c_void,
        pos: *const AmmlBlockPos,
        out: *mut *const c_void,
    ) -> AmmlStatus {
        if pos.is_null() || out.is_null() {
            return AmmlStatus::INVALID_ARGUMENT;
        }
        let (state, fake) = match Self::world_source_check_in(source, Area::WORLD_BLOCKS) {
            Ok(checked) => checked,
            Err(status) => return status,
        };
        // SAFETY: `pos` is non-null and the caller promises it is readable.
        let pos = crate::BlockPos::from(unsafe { pos.read() });
        if fake.world.is_none() {
            return AmmlStatus::REFUSED;
        }
        Self::write(out, std::ptr::with_exposed_provenance(state.block_handle(pos)))
    }

    unsafe extern "C" fn world_block_info(block: *const c_void, out: *mut AmmlBlockInfo) -> AmmlStatus {
        if out.is_null() {
            return AmmlStatus::INVALID_ARGUMENT;
        }
        match Self::world_block_check(block) {
            Ok((found, _)) => Self::write(
                out,
                AmmlBlockInfo {
                    type_id: found.type_id.0,
                    is_air: u8::from(found.is_air),
                    is_interactive: u8::from(found.interactive && !found.is_air),
                    _reserved: [0; 2],
                },
            ),
            Err(status) => status,
        }
    }

    unsafe extern "C" fn world_block_built_over(
        source: *const c_void,
        block: *const c_void,
        pos: *const AmmlBlockPos,
        out: *mut u8,
    ) -> AmmlStatus {
        if pos.is_null() || out.is_null() {
            return AmmlStatus::INVALID_ARGUMENT;
        }
        if let Err(status) = Self::world_source_check_in(source, Area::WORLD_BLOCKS) {
            return status;
        }
        match Self::world_block_check(block) {
            Ok((found, _)) => Self::write(out, u8::from(found.built_over)),
            Err(status) => status,
        }
    }

    unsafe extern "C" fn world_block_item(
        block: *const c_void,
        out: *mut AmmlItemSnapshot,
        name_buf: *mut u8,
        name_cap: usize,
        name_len: *mut usize,
    ) -> AmmlStatus {
        if out.is_null() || name_len.is_null() {
            return AmmlStatus::INVALID_ARGUMENT;
        }
        let item = match Self::world_block_check(block) {
            Ok((found, _)) => found.item,
            Err(status) => return status,
        };
        let Some(item) = item else {
            return AmmlStatus::NOT_AVAILABLE;
        };
        // SAFETY: `out` is non-null and the caller promises a v1 snapshot is writable.
        unsafe { out.write(item.raw_snapshot()) };
        let name = item.type_name.as_bytes();
        let copied = name.len().min(name_cap);
        if copied > 0 && !name_buf.is_null() {
            // SAFETY: the caller promises `name_buf` is writable for `name_cap` bytes, and `copied <= name_cap`.
            unsafe { std::ptr::copy_nonoverlapping(name.as_ptr(), name_buf, copied) };
        }
        Self::write(name_len, name.len())
    }

    unsafe extern "C" fn world_block_handle_of(
        _module: AmmlStr,
        block: *const c_void,
        out: *mut AmmlBlockHandle,
    ) -> AmmlStatus {
        if out.is_null() {
            return AmmlStatus::INVALID_ARGUMENT;
        }
        match Self::world_block_check(block) {
            Ok((found, _)) => match found.handle {
                Some(handle) => Self::write(out, handle.raw()),
                None => AmmlStatus::NOT_AVAILABLE,
            },
            Err(status) => status,
        }
    }

    // The distinct block types of a box of the fake world, as the loader gives them: every chunk of the box must be in
    // `chunks` and loaded, the box must have no inverted axis and at most 65536 cells, the cells are read x, then z, then y,
    // and a cell not in `blocks` is air.
    unsafe extern "C" fn world_kinds_in_box(
        source: *const c_void,
        min: *const AmmlBlockPos,
        max: *const AmmlBlockPos,
        ids: *mut u32,
        cap: u32,
        out: *mut AmmlKindsResult,
    ) -> AmmlStatus {
        if min.is_null() || max.is_null() || out.is_null() || (ids.is_null() && cap != 0) {
            return AmmlStatus::INVALID_ARGUMENT;
        }
        let (_, fake) = match Self::world_source_check_in(source, Area::WORLD_KINDS_IN_BOX) {
            Ok(checked) => checked,
            Err(status) => return status,
        };
        let Some(world) = fake.world else {
            return AmmlStatus::REFUSED;
        };
        // SAFETY: `min` and `max` are non-null and the caller promises they are readable.
        let (min, max) = unsafe { (min.read(), max.read()) };
        let span = |low: i32, high: i32| i64::from(high) - i64::from(low) + 1;
        let (dx, dy, dz) = (span(min.x, max.x), span(min.y, max.y), span(min.z, max.z));
        if dx < 1 || dy < 1 || dz < 1 || dx.saturating_mul(dy).saturating_mul(dz) > 65536 {
            return AmmlStatus::REFUSED;
        }
        let mut result = AmmlKindsResult { kind: AmmlKindsResult::NOT_LOADED, ..AmmlKindsResult::default() };
        let loaded =
            |cx: i32, cz: i32| world.chunks.get(&(cx, cz)).is_some_and(|chunk| chunk.load == crate::ChunkLoad::Loaded);
        let chunks_loaded = (min.x >> 4..=max.x >> 4).all(|cx| (min.z >> 4..=max.z >> 4).all(|cz| loaded(cx, cz)));
        if !chunks_loaded {
            return Self::write(out, result);
        }
        let air = world.air_block().type_id;
        let mut found: Vec<u32> = Vec::new();
        for x in min.x..=max.x {
            for z in min.z..=max.z {
                for y in min.y..=max.y {
                    result.cells += 1;
                    let id = world.blocks.get(&crate::BlockPos { x, y, z }).map_or(air, |block| block.type_id).0;
                    if found.contains(&id) {
                        continue;
                    }
                    if found.len() >= cap as usize {
                        result.kind = AmmlKindsResult::TOO_MANY;
                        return Self::write(out, result);
                    }
                    found.push(id);
                }
            }
        }
        for (index, id) in found.iter().enumerate() {
            // SAFETY: `ids` is non-null and the caller promises `cap` entries are writable; `found.len() <= cap`.
            unsafe { ids.add(index).write(*id) };
        }
        result.kind = AmmlKindsResult::KINDS;
        result.count = u32::try_from(found.len()).unwrap_or(u32::MAX);
        Self::write(out, result)
    }

    // The new state of the `BlockChanged` event that runs (`FakeGame::raise_block_changed_v2`); `STALE` for any other
    // token and outside the raise.
    unsafe extern "C" fn world_block_changed_state(
        _module: AmmlStr,
        token: u64,
        out: *mut AmmlBlockHandle,
    ) -> AmmlStatus {
        let Some(state) = FakeState::current() else {
            return AmmlStatus::FAILED;
        };
        if out.is_null() {
            return AmmlStatus::INVALID_ARGUMENT;
        }
        match state.changed_state() {
            Some((running, handle)) if token != 0 && u64::from(running) == token => Self::write(out, handle.raw()),
            _ => AmmlStatus::STALE,
        }
    }

    unsafe extern "C" fn world_block_handle_name(
        block: AmmlBlockHandle,
        buf: *mut u8,
        cap: usize,
        out_len: *mut usize,
    ) -> AmmlStatus {
        let Some(state) = FakeState::current() else {
            return AmmlStatus::FAILED;
        };
        let world = match state.block_world(Area::WORLD_BLOCK_STATES) {
            Ok(world) => world,
            Err(status) => return status,
        };
        let name = crate::BlockHandle::from_raw(block).and_then(|handle| world.block_names.get(&handle).cloned());
        let Some(name) = name else {
            return AmmlStatus::STALE;
        };
        let copied = name.len().min(cap);
        if copied > 0 && !buf.is_null() {
            // SAFETY: the caller promises `buf` is writable for `cap` bytes, and `copied <= cap`.
            unsafe { std::ptr::copy_nonoverlapping(name.as_ptr(), buf, copied) };
        }
        Self::write(out_len, name.len())
    }

    unsafe extern "C" fn world_block_handle_item(
        block: AmmlBlockHandle,
        out: *mut AmmlItemSnapshot,
        name_buf: *mut u8,
        name_cap: usize,
        name_len: *mut usize,
    ) -> AmmlStatus {
        let Some(state) = FakeState::current() else {
            return AmmlStatus::FAILED;
        };
        let world = match state.block_world(Area::WORLD_BLOCK_STATES) {
            Ok(world) => world,
            Err(status) => return status,
        };
        let item = crate::BlockHandle::from_raw(block).and_then(|handle| world.block_items.get(&handle).copied());
        let Some(item) = item else {
            return AmmlStatus::STALE;
        };
        if out.is_null() {
            return AmmlStatus::INVALID_ARGUMENT;
        }
        // SAFETY: `out` is non-null and the caller promises a v1 snapshot is writable.
        unsafe { out.write(item.raw_snapshot()) };
        let name = item.type_name.as_bytes();
        let copied = name.len().min(name_cap);
        if copied > 0 && !name_buf.is_null() {
            // SAFETY: the caller promises `name_buf` is writable for `name_cap` bytes, and `copied <= name_cap`.
            unsafe { std::ptr::copy_nonoverlapping(name.as_ptr(), name_buf, copied) };
        }
        Self::write(name_len, name.len())
    }

    // The surfaces of the fake world for a column, from `FakeWorld::surfaces`: lowest first, in the range asked, at most
    // `len`. A chunk that is not there or not loaded fails, as the loader does.
    #[allow(clippy::too_many_arguments)]
    unsafe extern "C" fn world_scan_surfaces(
        source: *const c_void,
        x: i32,
        z: i32,
        y_min: i32,
        y_max: i32,
        out: *mut AmmlSurfaceSample,
        len: u32,
        out_count: *mut u32,
    ) -> AmmlStatus {
        if out_count.is_null() || (out.is_null() && len != 0) {
            return AmmlStatus::INVALID_ARGUMENT;
        }
        let (_, fake) = match Self::world_source_check_in(source, Area::WORLD_SCAN_SURFACES) {
            Ok(checked) => checked,
            Err(status) => return status,
        };
        let Some(world) = fake.world else {
            return AmmlStatus::REFUSED;
        };
        if world.chunks.get(&(x >> 4, z >> 4)).is_none_or(|chunk| chunk.load != crate::ChunkLoad::Loaded) {
            return AmmlStatus::FAILED;
        }
        let mut found: Vec<crate::SurfaceSample> = world
            .surfaces
            .get(&(x, z))
            .map(|samples| samples.iter().copied().filter(|sample| (y_min..=y_max).contains(&sample.pos.y)).collect())
            .unwrap_or_default();
        found.sort_by_key(|sample| sample.pos.y);
        found.truncate(len as usize);
        for (index, sample) in found.iter().enumerate() {
            // SAFETY: the caller promises `len` writable samples at `out`, and `index < len`.
            unsafe {
                out.add(index).write(AmmlSurfaceSample {
                    pos: amml_abi::AmmlBlockPos { x: sample.pos.x, y: sample.pos.y, z: sample.pos.z },
                    block_light: sample.block_light,
                    sky_light: sample.sky_light,
                    _reserved: [0; 2],
                });
            }
        }
        Self::write(out_count, found.len() as u32)
    }

    // The hit of the fake world for a ray with finite points (`FakeWorld::ray_hit`).
    unsafe extern "C" fn world_raycast(
        source: *const c_void,
        from: amml_abi::AmmlVec3,
        to: amml_abi::AmmlVec3,
        _options: amml_abi::AmmlRayOptions,
        out: *mut amml_abi::AmmlBlockHit,
    ) -> AmmlStatus {
        let finite = |point: &amml_abi::AmmlVec3| point.x.is_finite() && point.y.is_finite() && point.z.is_finite();
        if out.is_null() || !finite(&from) || !finite(&to) {
            return AmmlStatus::INVALID_ARGUMENT;
        }
        let (_, fake) = match Self::world_source_check_in(source, Area::WORLD_RAYCAST) {
            Ok(checked) => checked,
            Err(status) => return status,
        };
        let hit = fake.world.and_then(|world| world.ray_hit).map(amml_abi::AmmlBlockHit::from).unwrap_or_default();
        Self::write(out, hit)
    }

    // The actors of the fake world whose boxes meet `aabb` (`FakeWorld::actors`), at most `cap`.
    unsafe extern "C" fn world_actors_in(
        source: *const c_void,
        aabb: amml_abi::AmmlAabb,
        out: *mut amml_abi::AmmlActorBox,
        cap: u32,
        out_count: *mut u32,
    ) -> AmmlStatus {
        if out_count.is_null() || (out.is_null() && cap != 0) {
            return AmmlStatus::INVALID_ARGUMENT;
        }
        let (_, fake) = match Self::world_source_check_in(source, Area::WORLD_ACTORS_IN) {
            Ok(checked) => checked,
            Err(status) => return status,
        };
        let asked = crate::Aabb::from(aabb);
        let meets = |actor: &crate::ActorBox| {
            let (a, b) = (actor.aabb, asked);
            a.min.x <= b.max.x
                && a.max.x >= b.min.x
                && a.min.y <= b.max.y
                && a.max.y >= b.min.y
                && a.min.z <= b.max.z
                && a.max.z >= b.min.z
        };
        let found: Vec<crate::ActorBox> = fake
            .world
            .map(|world| world.actors.into_iter().filter(meets).take(cap as usize).collect())
            .unwrap_or_default();
        for (index, actor) in found.iter().enumerate() {
            // SAFETY: the caller promises `cap` writable boxes at `out`, and `index < cap`.
            unsafe { out.add(index).write(amml_abi::AmmlActorBox::from(*actor)) };
        }
        Self::write(out_count, found.len() as u32)
    }

    // The name of a biome of the fake world.
    unsafe extern "C" fn world_biome_name(id: u16, buf: *mut u8, cap: usize, out_len: *mut usize) -> AmmlStatus {
        let Some(state) = FakeState::current() else {
            return AmmlStatus::FAILED;
        };
        let fake = match state.client_check(state.client_handle(), Area::WORLD_BIOMES) {
            Ok(fake) => fake,
            Err(status) => return status,
        };
        let Some(name) = fake.world.and_then(|world| world.biome_names.get(&id).cloned()) else {
            return AmmlStatus::INVALID_ARGUMENT;
        };
        let bytes = name.as_bytes();
        let copied = bytes.len().min(cap);
        if copied > 0 && !buf.is_null() {
            // SAFETY: the caller promises `buf` is writable for `cap` bytes, and `copied <= cap`.
            unsafe { std::ptr::copy_nonoverlapping(bytes.as_ptr(), buf, copied) };
        }
        Self::write(out_len, bytes.len())
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

    unsafe extern "C" fn client_join_server_direct(
        _: AmmlStr,
        client: *const c_void,
        host: AmmlStr,
        port: u16,
        name: AmmlStr,
        out: *mut u64,
    ) -> AmmlStatus {
        let Some(state) = FakeState::current() else {
            return AmmlStatus::FAILED;
        };
        let fake = match state.client_check(client.addr(), Area::CLIENT_JOIN_SERVER_DIRECT) {
            Ok(fake) => fake,
            Err(status) => return status,
        };
        // SAFETY: the caller promises the views are valid for the call.
        let (Ok(host), Ok(name)) = (unsafe { host.as_str() }, unsafe { name.as_str() }) else {
            return AmmlStatus::INVALID_ARGUMENT;
        };
        if host.parse::<std::net::IpAddr>().is_err() || port == 0 {
            return AmmlStatus::INVALID_ARGUMENT;
        }
        if fake.local_player {
            return AmmlStatus::REFUSED;
        }
        let shown = if name.is_empty() { host } else { name };
        state.write_client(|_| {}, format!("join_server_direct {host}:{port} {shown}"));
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

    // The facts of the fake world: a flag for each fact the fake holds.
    unsafe extern "C" fn client_world_info(client: *const c_void, out: *mut AmmlWorldInfo) -> AmmlStatus {
        let Some(state) = FakeState::current() else {
            return AmmlStatus::FAILED;
        };
        let fake = match state.client_check(client.addr(), Area::CLIENT_WORLD_INFO) {
            Ok(fake) => fake,
            Err(status) => return status,
        };
        let Some(info) = fake.world_info.filter(|_| fake.local_player) else {
            return AmmlStatus::REFUSED;
        };
        let mut abi = AmmlWorldInfo::default();
        if let Some(seed) = info.seed {
            abi.flags |= AmmlWorldInfo::HAS_SEED;
            abi.seed = seed;
        }
        if info.storage_folder.is_some() {
            abi.flags |= AmmlWorldInfo::HAS_STORAGE;
        }
        if info.level_name.is_some() {
            abi.flags |= AmmlWorldInfo::HAS_LEVEL_NAME;
        }
        if let Some(realm) = info.realm {
            abi.flags |= AmmlWorldInfo::HAS_REALM | if realm { AmmlWorldInfo::REALM } else { 0 };
        }
        if let Some(enabled) = info.commands_enabled {
            abi.flags |= AmmlWorldInfo::HAS_COMMANDS | if enabled { AmmlWorldInfo::COMMANDS } else { 0 };
        }
        if let Some(connection) = info.connection {
            abi.flags |= AmmlWorldInfo::HAS_CONNECTION;
            abi.connection_kind = connection.kind.to_abi();
            abi.port = connection.port;
        }
        Self::write(out, abi)
    }

    // The texts of the fake world; the length is the whole text, the copy stops at `cap`.
    unsafe extern "C" fn client_world_text(
        client: *const c_void,
        field: AmmlWorldText,
        buf: *mut u8,
        cap: usize,
        out_len: *mut usize,
    ) -> AmmlStatus {
        let Some(state) = FakeState::current() else {
            return AmmlStatus::FAILED;
        };
        let fake = match state.client_check(client.addr(), Area::CLIENT_WORLD_INFO) {
            Ok(fake) => fake,
            Err(status) => return status,
        };
        let Some(info) = fake.world_info.filter(|_| fake.local_player) else {
            return AmmlStatus::REFUSED;
        };
        let text = match field {
            AmmlWorldText::STORAGE_FOLDER => info.storage_folder.map(|path| path.to_string_lossy().into_owned()),
            AmmlWorldText::LEVEL_NAME => info.level_name,
            AmmlWorldText::HOST => info.connection.map(|connection| connection.host),
            _ => return AmmlStatus::INVALID_ARGUMENT,
        };
        let Some(text) = text else {
            return AmmlStatus::NOT_AVAILABLE;
        };
        let bytes = text.as_bytes();
        let copied = bytes.len().min(cap);
        if copied > 0 && !buf.is_null() {
            // SAFETY: the caller promises `buf` is writable for `cap` bytes, and `copied <= cap`.
            unsafe { std::ptr::copy_nonoverlapping(bytes.as_ptr(), buf, copied) };
        }
        Self::write(out_len, bytes.len())
    }

    unsafe extern "C" fn client_in_gameplay(client: *const c_void, out: *mut u8) -> AmmlStatus {
        let Some(state) = FakeState::current() else {
            return AmmlStatus::FAILED;
        };
        match state.client_check(client.addr(), Area::CLIENT_IN_GAMEPLAY) {
            Ok(fake) => Self::write(out, u8::from(fake.in_gameplay)),
            Err(status) => status,
        }
    }

    unsafe extern "C" fn client_title_screen(client: *const c_void, out: *mut u8) -> AmmlStatus {
        let Some(state) = FakeState::current() else {
            return AmmlStatus::FAILED;
        };
        match state.client_check(client.addr(), Area::CLIENT_TITLE_SCREEN) {
            Ok(fake) => Self::write(out, u8::from(fake.title_screen && !fake.local_player)),
            Err(status) => status,
        }
    }

    unsafe extern "C" fn client_go_to_title(client: *const c_void) -> AmmlStatus {
        let Some(state) = FakeState::current() else {
            return AmmlStatus::FAILED;
        };
        let fake = match state.client_check(client.addr(), Area::CLIENT_GO_TO_TITLE) {
            Ok(fake) => fake,
            Err(status) => return status,
        };
        if fake.local_player {
            return AmmlStatus::REFUSED;
        }
        state.write_client(|client| client.title_screen = true, "go_to_title".to_string());
        AmmlStatus::OK
    }

    unsafe extern "C" fn client_replay_app_focus_lost(_module: AmmlStr, client: *const c_void) -> AmmlStatus {
        let Some(state) = FakeState::current() else {
            return AmmlStatus::FAILED;
        };
        if let Err(status) = state.client_check(client.addr(), Area::CLIENT_FOCUS_REPLAY) {
            return status;
        }
        state.replay_focus_loss();
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

    unsafe extern "C" fn input_held_by_others(_: AmmlStr, out: *mut AmmlOtherLeases) -> AmmlStatus {
        let Some(state) = FakeState::current() else {
            return AmmlStatus::FAILED;
        };
        let (leases, blocks) = state.input().others();
        Self::write(out, AmmlOtherLeases { leases, blocks })
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

    unsafe extern "C" fn window_clip_cursor(module: AmmlStr, clip: AmmlCursorClip) -> AmmlStatus {
        let (Some(state), Some(_)) = (FakeState::current(), Self::text(module)) else {
            return AmmlStatus::FAILED;
        };
        if let Err(status) = state.window_check(Area::WINDOW_CURSOR_CLIP) {
            return status;
        }
        let line = match clip {
            AmmlCursorClip::NONE => "clip_cursor none",
            AmmlCursorClip::CENTRE => "clip_cursor centre",
            _ => return AmmlStatus::INVALID_ARGUMENT,
        };
        state.record_window_request(line.to_string());
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

    unsafe extern "C" fn overlay_prepare_surface(out: *mut AmmlDrawSurface) -> AmmlStatus {
        let Some(state) = FakeState::current() else {
            return AmmlStatus::FAILED;
        };
        if !state.area_offered(Area::OVERLAY_SURFACE.id, Area::OVERLAY_SURFACE.version) {
            return AmmlStatus::NOT_AVAILABLE;
        }
        if out.is_null() {
            return AmmlStatus::INVALID_ARGUMENT;
        }
        match state.overlay().prepare() {
            Ok(surface) => {
                // SAFETY: `out` is non-null and the caller promises it is writable.
                unsafe { out.write(surface) };
                AmmlStatus::OK
            }
            Err(status) => status,
        }
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
