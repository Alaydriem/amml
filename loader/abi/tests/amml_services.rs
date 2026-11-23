use core::ffi::c_void;

use amml_abi::{
    AmmlActorTable, AmmlBlockEntityGhost, AmmlBlockGrid, AmmlBlockHandle, AmmlCameraTable, AmmlCaptureFrame,
    AmmlCaptureRequest, AmmlClientFlag, AmmlClientTable, AmmlColorQuad, AmmlDrawSurface, AmmlHand, AmmlInputMode,
    AmmlInputTable, AmmlInventoryTable, AmmlItemSnapshot, AmmlLeaseRequest, AmmlLeaseState, AmmlLightSource,
    AmmlLightingTable, AmmlMeshBuilt, AmmlMeshGroup, AmmlMeshId, AmmlMeshStyle, AmmlMesherOptions, AmmlModifiers,
    AmmlMouseButton, AmmlMoveIntent, AmmlNetworkTable, AmmlOptionKey, AmmlOptionsTable, AmmlOverlayTable,
    AmmlPersistence, AmmlPoint, AmmlPose, AmmlProjection, AmmlRect, AmmlRenderTable, AmmlScreen, AmmlScreenKind,
    AmmlServices, AmmlShotLayers, AmmlSize, AmmlStatus, AmmlStr, AmmlTimeTable, AmmlUiProfile, AmmlUiTable, AmmlVec3,
    AmmlWindowTable, AmmlWorldInfo, AmmlWorldTable,
};

unsafe extern "C" fn view_distance_chunks(_: *const c_void, _: *mut u32) -> AmmlStatus {
    AmmlStatus::OK
}
unsafe extern "C" fn get_int(_: AmmlOptionKey, _: *mut i32) -> AmmlStatus {
    AmmlStatus::OK
}
unsafe extern "C" fn set_int(_: AmmlStr, _: AmmlOptionKey, _: i32, _: AmmlPersistence) -> AmmlStatus {
    AmmlStatus::OK
}
unsafe extern "C" fn release_transient(_: AmmlStr, _: AmmlOptionKey) -> AmmlStatus {
    AmmlStatus::OK
}
unsafe extern "C" fn current(_: *mut *const c_void) -> AmmlStatus {
    AmmlStatus::OK
}
unsafe extern "C" fn local_player(_: *const c_void, _: *mut *const c_void) -> AmmlStatus {
    AmmlStatus::OK
}
unsafe extern "C" fn flag(_: *const c_void, _: AmmlClientFlag, _: *mut u8) -> AmmlStatus {
    AmmlStatus::OK
}
unsafe extern "C" fn top_screen(_: *const c_void, _: *mut AmmlScreen) -> AmmlStatus {
    AmmlStatus::OK
}
unsafe extern "C" fn set_flag(_: AmmlStr, _: *const c_void, _: AmmlClientFlag, _: u8) -> AmmlStatus {
    AmmlStatus::OK
}
unsafe extern "C" fn open_pause(_: *const c_void) -> AmmlStatus {
    AmmlStatus::OK
}
unsafe extern "C" fn modifiers() -> AmmlModifiers {
    AmmlModifiers(0)
}
unsafe extern "C" fn is_down(_: u16, _: *mut u8) -> AmmlStatus {
    AmmlStatus::OK
}
unsafe extern "C" fn acquire(_: AmmlStr, _: *const AmmlLeaseRequest, _: *mut u64) -> AmmlStatus {
    AmmlStatus::OK
}
unsafe extern "C" fn lease_state(_: u64) -> AmmlLeaseState {
    AmmlLeaseState::ACTIVE
}
unsafe extern "C" fn key_down(_: u64, _: u16, _: *mut u64) -> AmmlStatus {
    AmmlStatus::OK
}
unsafe extern "C" fn button_down(_: u64, _: AmmlMouseButton, _: *mut u64) -> AmmlStatus {
    AmmlStatus::OK
}
unsafe extern "C" fn release_hold(_: u64) -> AmmlStatus {
    AmmlStatus::OK
}
unsafe extern "C" fn wheel(_: u64, _: f32) -> AmmlStatus {
    AmmlStatus::OK
}
unsafe extern "C" fn turn(_: u64, _: f32, _: f32) -> AmmlStatus {
    AmmlStatus::OK
}
unsafe extern "C" fn set_move(_: u64, _: *const AmmlMoveIntent) -> AmmlStatus {
    AmmlStatus::OK
}
unsafe extern "C" fn release_all(_: u64) {}
unsafe extern "C" fn end_lease(_: u64) -> AmmlStatus {
    AmmlStatus::OK
}
unsafe extern "C" fn set_ranking(_: AmmlStr, _: *const AmmlStr, _: usize) -> AmmlStatus {
    AmmlStatus::OK
}

unsafe extern "C" fn size_out(_: *mut AmmlSize) -> AmmlStatus {
    AmmlStatus::OK
}
unsafe extern "C" fn point_out(_: *mut AmmlPoint) -> AmmlStatus {
    AmmlStatus::OK
}
unsafe extern "C" fn to_buffer(_: AmmlPoint, _: *mut AmmlPoint) -> AmmlStatus {
    AmmlStatus::OK
}
unsafe extern "C" fn byte_out(_: *mut u8) -> AmmlStatus {
    AmmlStatus::OK
}
unsafe extern "C" fn dpi(_: *mut u32) -> AmmlStatus {
    AmmlStatus::OK
}
unsafe extern "C" fn request_size(_: AmmlStr, _: *const AmmlSize) -> AmmlStatus {
    AmmlStatus::OK
}
unsafe extern "C" fn replay_focus_loss() -> AmmlStatus {
    AmmlStatus::OK
}
unsafe extern "C" fn screenshot(_: AmmlStr, _: AmmlStr, _: AmmlShotLayers, _: *mut u64) -> AmmlStatus {
    AmmlStatus::OK
}
unsafe extern "C" fn open_capture(_: AmmlStr, _: *const AmmlCaptureRequest, _: *mut u64) -> AmmlStatus {
    AmmlStatus::OK
}
unsafe extern "C" fn capture_take(_: u64, _: *mut AmmlCaptureFrame) -> AmmlStatus {
    AmmlStatus::NOT_READY
}
unsafe extern "C" fn capture_give_back(_: u64, _: u64) -> AmmlStatus {
    AmmlStatus::OK
}
unsafe extern "C" fn capture_free_slots(_: u64) -> u32 {
    0
}
unsafe extern "C" fn close_capture(_: u64) -> AmmlStatus {
    AmmlStatus::OK
}
unsafe extern "C" fn present_count() -> u64 {
    0
}
unsafe extern "C" fn copy_back_buffer(_: *mut c_void) -> AmmlStatus {
    AmmlStatus::OK
}
unsafe extern "C" fn prepare_surface(_: *mut AmmlDrawSurface) -> AmmlStatus {
    AmmlStatus::OK
}

unsafe extern "C" fn is_alive(_: *const c_void, _: *mut u8) -> AmmlStatus {
    AmmlStatus::OK
}

fn actor_table() -> AmmlActorTable {
    AmmlActorTable {
        size: AmmlActorTable::SIZE_V1,
        version: 1,
        is_alive,
        position: None,
        eye_position: None,
        rotation: None,
        dimension: None,
        dimension_id: None,
        head_position: None,
        block_source: None,
        health: None,
        view_vector: None,
        aabb: None,
        game_type: None,
        permission_level: None,
        set_rotation: None,
        build_at: None,
        build_delay: None,
        max_pick_range: None,
        hit_result: None,
    }
}

unsafe extern "C" fn held_item(_: *const c_void, _: AmmlHand, _: *mut *const c_void) -> AmmlStatus {
    AmmlStatus::OK
}

unsafe extern "C" fn selected_slot(_: *const c_void, _: *mut u8) -> AmmlStatus {
    AmmlStatus::OK
}

unsafe extern "C" fn item_read(_: *const c_void, _: *const c_void, _: *mut AmmlItemSnapshot) -> AmmlStatus {
    AmmlStatus::OK
}

unsafe extern "C" fn item_name(_: *const c_void, _: *const c_void, _: *mut u8, _: usize, _: *mut usize) -> AmmlStatus {
    AmmlStatus::OK
}

fn inventory_table() -> AmmlInventoryTable {
    AmmlInventoryTable {
        size: AmmlInventoryTable::SIZE_V1,
        version: 1,
        held_item,
        selected_slot,
        item_read,
        item_name,
        item_block: None,
        slot_item: None,
        contents_open: None,
        contents_entry: None,
        contents_name: None,
        contents_close: None,
        item_draw_release: None,
        request_moves: None,
        item_change_stamp: None,
        request_pick: None,
    }
}

unsafe extern "C" fn view_kind(_: u64, _: *mut AmmlScreenKind) -> AmmlStatus {
    AmmlStatus::OK
}
unsafe extern "C" fn view_size(_: u64, _: *mut AmmlSize) -> AmmlStatus {
    AmmlStatus::OK
}
unsafe extern "C" fn view_content_rect(_: u64, _: *mut AmmlRect) -> AmmlStatus {
    AmmlStatus::OK
}
unsafe extern "C" fn view_pointer(_: u64, _: *mut AmmlPoint) -> AmmlStatus {
    AmmlStatus::OK
}
unsafe extern "C" fn view_input_mode(_: u64, _: *mut AmmlInputMode) -> AmmlStatus {
    AmmlStatus::OK
}
unsafe extern "C" fn view_ui_profile(_: u64, _: *mut AmmlUiProfile) -> AmmlStatus {
    AmmlStatus::OK
}
unsafe extern "C" fn view_container(_: u64, _: *mut u64) -> AmmlStatus {
    AmmlStatus::OK
}

fn ui_table() -> AmmlUiTable {
    AmmlUiTable {
        size: AmmlUiTable::SIZE_V1,
        version: 1,
        view_kind,
        view_size,
        view_content_rect,
        view_pointer,
        view_input_mode,
        view_ui_profile,
        view_container,
        draw_fill: None,
        draw_outline: None,
        draw_image: None,
        gui_scale: None,
        can_draw: None,
        texture_load: None,
        texture_upload_rgba: None,
        texture_ready: None,
        texture_unload: None,
        container_slot_count: None,
        container_item: None,
        container_cursor: None,
        container_action: None,
        container_swap: None,
        draw_item: None,
        text_width: None,
        draw_text: None,
        chat_text: None,
        chat_set_text: None,
        chat_icons_active: None,
        icon_register: None,
        icon_update: None,
        icon_release: None,
    }
}

fn client_table() -> AmmlClientTable {
    AmmlClientTable {
        size: AmmlClientTable::SIZE_V1,
        version: 1,
        current,
        local_player,
        flag,
        top_screen,
        set_flag,
        open_pause_screen: open_pause,
        leave_world: None,
        can_join: None,
        start_local_world: None,
        join_server: None,
        open_menu: None,
        send_command: None,
        select_hotbar_slot: None,
        note_activity: None,
        world_info: None,
        world_text: None,
        replay_app_focus_lost: None,
        in_gameplay: None,
        join_server_direct: None,
        title_screen: None,
        go_to_title: None,
    }
}

fn input_table() -> AmmlInputTable {
    AmmlInputTable {
        size: AmmlInputTable::SIZE_V2,
        version: 2,
        modifiers,
        is_down,
        acquire,
        lease_state,
        key_down,
        button_down,
        release_hold,
        wheel,
        turn,
        set_move,
        release_all,
        end_lease,
        set_ranking,
        set_coexist: Some(set_ranking),
        held_by_others: None,
    }
}

fn window_table() -> AmmlWindowTable {
    AmmlWindowTable {
        size: AmmlWindowTable::SIZE_V1,
        version: 1,
        client_size: size_out,
        buffer_size: size_out,
        cursor_pos: point_out,
        to_buffer,
        has_focus: byte_out,
        is_foreground: byte_out,
        dpi,
        request_size,
        replay_focus_loss,
        clip_cursor: None,
    }
}

fn overlay_table() -> AmmlOverlayTable {
    AmmlOverlayTable {
        size: AmmlOverlayTable::SIZE_V2,
        version: 1,
        screenshot,
        open_capture,
        capture_take,
        capture_give_back,
        capture_free_slots,
        close_capture,
        present_count,
        copy_back_buffer,
        prepare_surface: Some(prepare_surface),
    }
}

fn services(size: u32, client: *const AmmlClientTable, input: *const AmmlInputTable) -> AmmlServices {
    AmmlServices {
        size,
        version: 1,
        events: core::ptr::null(),
        core: core::ptr::null(),
        registry: core::ptr::null(),
        client,
        input,
        window: core::ptr::null(),
        overlay: core::ptr::null(),
        actor: core::ptr::null(),
        inventory: core::ptr::null(),
        lighting: core::ptr::null(),
        ui: core::ptr::null(),
        network: core::ptr::null(),
        world: core::ptr::null(),
        render: core::ptr::null(),
        camera: core::ptr::null(),
        options: core::ptr::null(),
        time: core::ptr::null(),
    }
}

// All five area tables live while the value does.
struct AreaTables {
    client: AmmlClientTable,
    input: AmmlInputTable,
    window: AmmlWindowTable,
    overlay: AmmlOverlayTable,
    actor: AmmlActorTable,
    inventory: AmmlInventoryTable,
    lighting: AmmlLightingTable,
    ui: AmmlUiTable,
    network: AmmlNetworkTable,
    world: AmmlWorldTable,
    render: AmmlRenderTable,
    camera: AmmlCameraTable,
    options: AmmlOptionsTable,
    time: AmmlTimeTable,
}

impl AreaTables {
    fn new() -> Self {
        Self {
            client: client_table(),
            input: input_table(),
            window: window_table(),
            overlay: overlay_table(),
            actor: actor_table(),
            inventory: inventory_table(),
            lighting: AmmlLightingTable { size: AmmlLightingTable::SIZE_V1, version: 1, set_light_source },
            ui: ui_table(),
            network: network_table(),
            world: AmmlWorldTable {
                size: AmmlWorldTable::SIZE_V4,
                version: 4,
                sky_darken,
                chunk: None,
                chunk_load_state: None,
                read_columns: None,
                block_type_name: None,
                biome_name: None,
                scan_surfaces: None,
                shapes_apply: None,
                shapes_max_batch: None,
                shapes_target_epoch: None,
                raycast: None,
                actors_in: None,
                block_from_state: None,
                block_rotated: None,
                block_handle_name: None,
                block_handle_item: None,
                block_at: None,
                block_info: None,
                block_built_over: None,
                block_item: None,
                block_handle_of: None,
                block_changed_state: None,
                kinds_in_box: None,
            },
            render: AmmlRenderTable {
                size: AmmlRenderTable::SIZE_V1,
                version: 1,
                canvas_camera_position,
                mesh_free,
                mesh_build: None,
                mesh_draw: None,
                canvas_clip_matrix: None,
                mesher_build: None,
                mesher_group_begin: None,
                mesher_group_append: None,
                mesher_group_finish: None,
                block_entity_create: None,
                block_entity_free: None,
                block_entity_draw: None,
                mesher_group_discard: None,
            },
            camera: AmmlCameraTable {
                size: AmmlCameraTable::SIZE_V1,
                version: 1,
                set_pose,
                rendered_position,
                projection,
                set_free: None,
                release_free: None,
                free_available: None,
                active_camera: None,
                hide_actor: None,
            },
            options: AmmlOptionsTable {
                size: AmmlOptionsTable::SIZE_V1,
                version: 1,
                view_distance_chunks,
                get_int,
                set_int,
                release_transient,
            },
            time: crate::amml_time_table::table(),
        }
    }

    // A services table of `size` that points to every area table, whatever `size` covers.
    fn services(&self, size: u32) -> AmmlServices {
        AmmlServices {
            window: &raw const self.window,
            overlay: &raw const self.overlay,
            actor: &raw const self.actor,
            inventory: &raw const self.inventory,
            lighting: &raw const self.lighting,
            ui: &raw const self.ui,
            network: &raw const self.network,
            world: &raw const self.world,
            render: &raw const self.render,
            camera: &raw const self.camera,
            options: &raw const self.options,
            time: &raw const self.time,
            ..services(size, &raw const self.client, &raw const self.input)
        }
    }
}

fn client_of(table: &AmmlServices) -> Option<*const AmmlClientTable> {
    // SAFETY: the table and the area tables it points to live for the whole test.
    unsafe { AmmlServices::client(table) }
}

fn input_of(table: &AmmlServices) -> Option<*const AmmlInputTable> {
    // SAFETY: the table and the area tables it points to live for the whole test.
    unsafe { AmmlServices::input(table) }
}

fn window_of(table: &AmmlServices) -> Option<*const AmmlWindowTable> {
    // SAFETY: the table and the area tables it points to live for the whole test.
    unsafe { AmmlServices::window(table) }
}

fn overlay_of(table: &AmmlServices) -> Option<*const AmmlOverlayTable> {
    // SAFETY: the table and the area tables it points to live for the whole test.
    unsafe { AmmlServices::overlay(table) }
}

fn actor_of(table: &AmmlServices) -> Option<*const AmmlActorTable> {
    // SAFETY: the table and the area tables it points to live for the whole test.
    unsafe { AmmlServices::actor(table) }
}

fn inventory_of(table: &AmmlServices) -> Option<*const AmmlInventoryTable> {
    // SAFETY: the table and the area tables it points to live for the whole test.
    unsafe { AmmlServices::inventory(table) }
}

unsafe extern "C" fn set_light_source(_: AmmlStr, _: *const AmmlLightSource) -> AmmlStatus {
    AmmlStatus::OK
}

fn lighting_of(table: &AmmlServices) -> Option<*const AmmlLightingTable> {
    // SAFETY: the table and the area tables it points to live for the whole test.
    unsafe { AmmlServices::lighting(table) }
}

#[test]
fn a_v5_services_table_has_no_lighting_table() {
    let areas = AreaTables::new();
    // A v5 table is only 80 bytes long: the lighting field is not part of it, whatever it holds.
    assert!(lighting_of(&areas.services(AmmlServices::SIZE_V5)).is_none());
}

#[test]
fn a_v6_services_table_gives_the_lighting_table_unless_it_is_null_or_short() {
    let mut areas = AreaTables::new();
    let table = areas.services(AmmlServices::SIZE_V6);
    assert_eq!(lighting_of(&table), Some(&raw const areas.lighting));
    assert_eq!(inventory_of(&table), Some(&raw const areas.inventory));
    let null = AmmlServices { lighting: core::ptr::null(), ..areas.services(AmmlServices::SIZE_V6) };
    assert!(lighting_of(&null).is_none());
    areas.lighting.size = AmmlLightingTable::SIZE_V1 - 8;
    assert!(lighting_of(&areas.services(AmmlServices::SIZE_V6)).is_none());
}

#[test]
fn a_v4_services_table_has_no_inventory_table() {
    let areas = AreaTables::new();
    // A v4 table is only 72 bytes long: the inventory field is not part of it, whatever it holds.
    assert!(inventory_of(&areas.services(AmmlServices::SIZE_V4)).is_none());
}

#[test]
fn a_v5_services_table_gives_the_inventory_table_unless_it_is_null_or_short() {
    let mut areas = AreaTables::new();
    let table = areas.services(AmmlServices::SIZE_V5);
    assert_eq!(inventory_of(&table), Some(&raw const areas.inventory));
    assert_eq!(actor_of(&table), Some(&raw const areas.actor));
    let null = AmmlServices { inventory: core::ptr::null(), ..areas.services(AmmlServices::SIZE_V5) };
    assert!(inventory_of(&null).is_none());
    areas.inventory.size = AmmlInventoryTable::SIZE_V1 - 8;
    assert!(inventory_of(&areas.services(AmmlServices::SIZE_V5)).is_none());
}

#[test]
fn a_v3_services_table_has_no_actor_table() {
    let areas = AreaTables::new();
    // A v3 table is only 64 bytes long: the actor field is not part of it, whatever it holds.
    let table = areas.services(AmmlServices::SIZE_V3);
    assert!(actor_of(&table).is_none());
}

#[test]
fn a_v4_services_table_gives_the_actor_table() {
    let areas = AreaTables::new();
    let table = areas.services(AmmlServices::SIZE_V4);
    assert_eq!(actor_of(&table), Some(&raw const areas.actor));
    assert_eq!(overlay_of(&table), Some(&raw const areas.overlay));
}

#[test]
fn a_v4_services_table_with_a_null_or_short_actor_table_has_none() {
    let mut areas = AreaTables::new();
    assert!(actor_of(&AmmlServices { actor: core::ptr::null(), ..areas.services(AmmlServices::SIZE_V4) }).is_none());
    areas.actor.size = AmmlActorTable::SIZE_V1 - 8;
    assert!(actor_of(&areas.services(AmmlServices::SIZE_V4)).is_none());
}

#[test]
fn a_v1_services_table_has_no_client_or_input_table() {
    let client = client_table();
    let input = input_table();
    // A v1 table is only 32 bytes long: its trailing fields are not part of it, whatever they hold.
    let table = services(AmmlServices::SIZE_V1, &raw const client, &raw const input);
    assert!(client_of(&table).is_none());
    assert!(input_of(&table).is_none());
}

#[test]
fn a_v2_services_table_gives_both_area_tables() {
    let client = client_table();
    let input = input_table();
    let table = services(AmmlServices::SIZE_V2, &raw const client, &raw const input);
    assert_eq!(client_of(&table), Some(&raw const client));
    assert_eq!(input_of(&table), Some(&raw const input));
}

#[test]
fn a_null_or_short_area_table_is_absent() {
    let mut input = input_table();
    input.size = 8;
    let table = services(AmmlServices::SIZE_V2, core::ptr::null(), &raw const input);
    assert!(client_of(&table).is_none());
    assert!(input_of(&table).is_none());
}

#[test]
fn a_services_table_larger_than_v2_still_gives_its_area_tables() {
    let client = client_table();
    let input = input_table();
    let table = services(AmmlServices::SIZE_V2 + 16, &raw const client, &raw const input);
    assert!(client_of(&table).is_some());
    assert!(input_of(&table).is_some());
}

#[test]
fn a_v2_services_table_has_no_window_or_overlay_table() {
    let areas = AreaTables::new();
    // A v2 table is only 48 bytes long: the window and overlay fields are not part of it, whatever they hold.
    let table = areas.services(AmmlServices::SIZE_V2);
    assert!(window_of(&table).is_none());
    assert!(overlay_of(&table).is_none());
    assert_eq!(client_of(&table), Some(&raw const areas.client));
    assert_eq!(input_of(&table), Some(&raw const areas.input));
}

#[test]
fn a_v3_services_table_gives_all_four_area_tables() {
    let areas = AreaTables::new();
    let table = areas.services(AmmlServices::SIZE_V3);
    assert_eq!(client_of(&table), Some(&raw const areas.client));
    assert_eq!(input_of(&table), Some(&raw const areas.input));
    assert_eq!(window_of(&table), Some(&raw const areas.window));
    assert_eq!(overlay_of(&table), Some(&raw const areas.overlay));
}

#[test]
fn a_v3_services_table_with_a_null_or_short_window_or_overlay_table_lacks_only_that_area() {
    let mut areas = AreaTables::new();
    areas.overlay.size = AmmlOverlayTable::SIZE_V1 - 8;
    let table = AmmlServices { window: core::ptr::null(), ..areas.services(AmmlServices::SIZE_V3) };
    assert!(window_of(&table).is_none());
    assert!(overlay_of(&table).is_none());
    assert!(client_of(&table).is_some());
    assert!(input_of(&table).is_some());
}

#[test]
fn a_v1_input_table_reads_with_no_coexist_function() {
    let mut table = input_table();
    table.size = AmmlInputTable::SIZE_V1;
    // SAFETY: the table lives for the call; the read covers only its first `size` bytes.
    let read = unsafe { AmmlInputTable::read(&raw const table) }.unwrap();
    assert!(read.set_coexist.is_none());
}

#[test]
fn a_v2_input_table_reads_with_its_coexist_function() {
    let table = input_table();
    // SAFETY: the table lives for the call.
    let read = unsafe { AmmlInputTable::read(&raw const table) }.unwrap();
    assert!(read.set_coexist.is_some());
}

#[test]
fn an_input_table_shorter_than_v1_does_not_read() {
    let mut table = input_table();
    table.size = AmmlInputTable::SIZE_V1 - 8;
    // SAFETY: the table lives for the call.
    assert!(unsafe { AmmlInputTable::read(&raw const table) }.is_none());
}

unsafe extern "C" fn leave_world(_: *const c_void) -> AmmlStatus {
    AmmlStatus::OK
}

#[test]
fn a_v1_client_table_reads_with_no_session_actions() {
    let mut table = client_table();
    table.leave_world = Some(leave_world);
    // SAFETY: the table lives for the call; the read covers only its first `size` bytes (the v1 size).
    let read = unsafe { AmmlClientTable::read(&raw const table) }.unwrap();
    assert!(read.leave_world.is_none());
}

unsafe extern "C" fn note_activity(_: *const c_void) -> AmmlStatus {
    AmmlStatus::OK
}

#[test]
fn a_v2_client_table_reads_with_no_note_activity() {
    let mut table = client_table();
    table.size = AmmlClientTable::SIZE_V2;
    table.note_activity = Some(note_activity);
    // SAFETY: the table lives for the call; the read covers only its first `size` bytes (the v2 size).
    let read = unsafe { AmmlClientTable::read(&raw const table) }.unwrap();
    assert!(read.note_activity.is_none());
}

#[test]
fn a_v2_client_table_reads_with_its_session_actions() {
    let mut table = client_table();
    table.size = AmmlClientTable::SIZE_V2;
    table.leave_world = Some(leave_world);
    // SAFETY: the table lives for the call.
    let read = unsafe { AmmlClientTable::read(&raw const table) }.unwrap();
    assert!(read.leave_world.is_some());
}

unsafe extern "C" fn world_info(_: *const c_void, _: *mut AmmlWorldInfo) -> AmmlStatus {
    AmmlStatus::OK
}

#[test]
fn a_v3_client_table_reads_with_no_world_info_and_a_v4_table_reads_it() {
    let mut table = client_table();
    table.world_info = Some(world_info);
    table.size = AmmlClientTable::SIZE_V3;
    // SAFETY: the table lives for the call; the read covers only its first `size` bytes (the v3 size).
    let older = unsafe { AmmlClientTable::read(&raw const table) }.unwrap();
    assert!(older.world_info.is_none());

    table.size = AmmlClientTable::SIZE_V4;
    // SAFETY: the table lives for the call.
    let newer = unsafe { AmmlClientTable::read(&raw const table) }.unwrap();
    assert!(newer.world_info.is_some());
}

unsafe extern "C" fn contents_close(_: AmmlStr, _: u64) {}

#[test]
fn a_v3_inventory_table_reads_with_no_contents_function_and_a_v4_table_reads_them() {
    let mut table = inventory_table();
    table.contents_close = Some(contents_close);
    table.size = AmmlInventoryTable::SIZE_V3;

    // SAFETY: `table` is a live table; its `size` covers the v3 fields only, so the v4 fields are not read.
    let older = unsafe { AmmlInventoryTable::read(&raw const table) }.unwrap();
    table.size = AmmlInventoryTable::SIZE_V4;
    // SAFETY: `table` is a live table whose `size` covers every field.
    let newer = unsafe { AmmlInventoryTable::read(&raw const table) }.unwrap();

    assert!(older.contents_close.is_none());
    assert!(newer.contents_close.is_some());
}

unsafe extern "C" fn request_moves(
    _: AmmlStr,
    _: *const core::ffi::c_void,
    _: *const amml_abi::AmmlInventoryMove,
    _: u32,
    _: amml_abi::AmmlRequestOptions,
    _: *mut u64,
) -> AmmlStatus {
    AmmlStatus::OK
}

#[test]
fn a_v4_inventory_table_reads_with_no_request_function_and_a_v5_table_reads_it() {
    let mut table = inventory_table();
    table.request_moves = Some(request_moves);
    table.size = AmmlInventoryTable::SIZE_V4;

    // SAFETY: `table` is a live table; its `size` covers the v4 fields only, so `request_moves` is not read.
    let older = unsafe { AmmlInventoryTable::read(&raw const table) }.unwrap();
    table.size = AmmlInventoryTable::SIZE_V5;
    // SAFETY: `table` is a live table whose `size` covers every field.
    let newer = unsafe { AmmlInventoryTable::read(&raw const table) }.unwrap();

    assert!(older.request_moves.is_none());
    assert!(newer.request_moves.is_some());
}

#[test]
fn a_v5_inventory_table_reads_with_no_change_stamp_and_a_v6_table_reads_it() {
    unsafe extern "C" fn stamp(_: *const c_void, _: *const c_void, _: *mut u64) -> AmmlStatus {
        AmmlStatus::OK
    }
    let mut table = inventory_table();
    table.item_change_stamp = Some(stamp);
    table.size = AmmlInventoryTable::SIZE_V5;

    // SAFETY: `table` is a live table; its `size` covers the v5 fields only, so `item_change_stamp` is not read.
    let older = unsafe { AmmlInventoryTable::read(&raw const table) }.unwrap();
    table.size = AmmlInventoryTable::SIZE_V6;
    // SAFETY: `table` is a live table whose `size` covers every field.
    let newer = unsafe { AmmlInventoryTable::read(&raw const table) }.unwrap();

    assert!(older.item_change_stamp.is_none());
    assert!(newer.item_change_stamp.is_some());
}

#[test]
fn a_v6_inventory_table_reads_with_no_pick_and_a_v7_table_reads_it() {
    unsafe extern "C" fn pick(_: AmmlStr, _: *const c_void, _: AmmlBlockHandle, _: u8, _: *mut u64) -> AmmlStatus {
        AmmlStatus::OK
    }
    let mut table = inventory_table();
    table.request_pick = Some(pick);
    table.size = AmmlInventoryTable::SIZE_V6;

    // SAFETY: `table` is a live table; its `size` covers the v6 fields only, so `request_pick` is not read.
    let older = unsafe { AmmlInventoryTable::read(&raw const table) }.unwrap();
    table.size = AmmlInventoryTable::SIZE_V7;
    // SAFETY: `table` is a live table whose `size` covers every field.
    let newer = unsafe { AmmlInventoryTable::read(&raw const table) }.unwrap();

    assert!(older.request_pick.is_none());
    assert!(newer.request_pick.is_some());
    assert_eq!(AmmlInventoryTable::SIZE_V7, AmmlInventoryTable::SIZE_V6 + 8);
}

fn network_table() -> AmmlNetworkTable {
    unsafe extern "C" fn entry(_: u32, _: *mut amml_abi::AmmlPlayerListEntry) -> AmmlStatus {
        AmmlStatus::OK
    }
    unsafe extern "C" fn text(_: u32, _: u8, _: *mut AmmlStr) -> AmmlStatus {
        AmmlStatus::OK
    }
    unsafe extern "C" fn skin(_: *mut [u8; 16], _: *mut u64) -> AmmlStatus {
        AmmlStatus::OK
    }
    unsafe extern "C" fn image(_: u64, _: *mut amml_abi::AmmlImageView) -> AmmlStatus {
        AmmlStatus::OK
    }
    unsafe extern "C" fn count(_: u64, _: *mut u32) -> AmmlStatus {
        AmmlStatus::OK
    }
    unsafe extern "C" fn animated(_: u64, _: u32, _: *mut u32, _: *mut amml_abi::AmmlImageView) -> AmmlStatus {
        AmmlStatus::OK
    }
    unsafe extern "C" fn flag(_: u64, _: *mut u8) -> AmmlStatus {
        AmmlStatus::OK
    }
    unsafe extern "C" fn string(_: u64, _: *mut AmmlStr) -> AmmlStatus {
        AmmlStatus::OK
    }
    AmmlNetworkTable {
        size: AmmlNetworkTable::SIZE_V1,
        version: 1,
        player_list_entry: entry,
        player_list_text: text,
        player_skin: skin,
        skin_image: image,
        skin_animated_count: count,
        skin_animated: animated,
        skin_is_persona: flag,
        skin_geometry_name: string,
        skin_geometry_json: string,
        skin_is_premium: flag,
        skin_resource_patch: string,
        protocol_version: None,
        set_client_chunk_cache: None,
    }
}

fn network_of(table: &AmmlServices) -> Option<*const AmmlNetworkTable> {
    // SAFETY: the table and the area tables it points to live for the whole test.
    unsafe { AmmlServices::network(table) }
}

#[test]
fn a_v7_services_table_has_no_network_table() {
    let areas = AreaTables::new();
    // A v7 table is only 96 bytes long: the network field is not part of it, whatever it holds.
    let table = areas.services(AmmlServices::SIZE_V7);
    assert!(network_of(&table).is_none());
}

#[test]
fn a_v8_services_table_gives_the_network_table_unless_it_is_null_or_short() {
    let mut areas = AreaTables::new();
    let table = areas.services(AmmlServices::SIZE_V8);
    assert_eq!(AmmlServices::SIZE_V8, 104);
    assert_eq!(network_of(&table), Some(&raw const areas.network));
    let null = AmmlServices { network: core::ptr::null(), ..areas.services(AmmlServices::SIZE_V8) };
    assert!(network_of(&null).is_none());
    areas.network.size = AmmlNetworkTable::SIZE_V1 - 8;
    assert!(network_of(&areas.services(AmmlServices::SIZE_V8)).is_none());
}

unsafe extern "C" fn sky_darken(_: *const c_void, _: *mut u8) -> AmmlStatus {
    AmmlStatus::OK
}

unsafe extern "C" fn canvas_camera_position(_: u64, _: *mut AmmlVec3) -> AmmlStatus {
    AmmlStatus::OK
}

unsafe extern "C" fn mesh_free(_: AmmlStr, _: AmmlMeshId) -> AmmlStatus {
    AmmlStatus::OK
}

fn render_of(table: &AmmlServices) -> Option<*const AmmlRenderTable> {
    // SAFETY: the table and the area tables it points to live for the whole test.
    unsafe { AmmlServices::render(table) }
}

#[test]
fn a_v9_services_table_has_no_render_table() {
    let areas = AreaTables::new();
    // A v9 table is only 112 bytes long: the render field is not part of it, whatever it holds.
    assert!(render_of(&areas.services(AmmlServices::SIZE_V9)).is_none());
}

#[test]
fn a_v10_services_table_gives_the_render_table_unless_it_is_null_or_short() {
    let mut areas = AreaTables::new();
    assert_eq!(AmmlServices::SIZE_V10, 120);
    assert_eq!(render_of(&areas.services(AmmlServices::SIZE_V10)), Some(&raw const areas.render));
    let null = AmmlServices { render: core::ptr::null(), ..areas.services(AmmlServices::SIZE_V10) };
    assert!(render_of(&null).is_none());
    areas.render.size = AmmlRenderTable::SIZE_V1 - 8;
    assert!(render_of(&areas.services(AmmlServices::SIZE_V10)).is_none());
}

unsafe extern "C" fn mesh_build(
    _: AmmlStr,
    _: u64,
    _: *const AmmlColorQuad,
    _: u32,
    _: AmmlMeshStyle,
    _: *mut AmmlMeshId,
) -> AmmlStatus {
    AmmlStatus::OK
}

#[test]
fn a_v1_render_table_reads_mesh_build_and_draw_as_none() {
    let mut areas = AreaTables::new();
    areas.render.mesh_build = Some(mesh_build);

    // SAFETY: the table is live and its `size` is covered; the bytes past v1 are ignored by the reader.
    let v1 = unsafe { AmmlRenderTable::read(&raw const areas.render) }.unwrap();
    areas.render.size = AmmlRenderTable::SIZE_V2;
    // SAFETY: as above, now with the v2 fields covered.
    let v2 = unsafe { AmmlRenderTable::read(&raw const areas.render) }.unwrap();

    assert!(v1.mesh_build.is_none() && v1.mesh_draw.is_none());
    assert!(v2.mesh_build.is_some() && v2.mesh_draw.is_none());
}

unsafe extern "C" fn clip_matrix(_: u64, _: *mut [[f32; 4]; 4]) -> AmmlStatus {
    AmmlStatus::OK
}

#[test]
fn a_v2_render_table_reads_the_clip_matrix_as_none() {
    let mut areas = AreaTables::new();
    areas.render.canvas_clip_matrix = Some(clip_matrix);
    areas.render.size = AmmlRenderTable::SIZE_V2;

    // SAFETY: the table is live and its `size` is covered; the bytes past v2 are ignored by the reader.
    let v2 = unsafe { AmmlRenderTable::read(&raw const areas.render) }.unwrap();
    areas.render.size = AmmlRenderTable::SIZE_V3;
    // SAFETY: as above, now with the v3 field covered.
    let v3 = unsafe { AmmlRenderTable::read(&raw const areas.render) }.unwrap();

    assert_eq!(AmmlRenderTable::SIZE_V3, 48);
    assert!(v2.canvas_clip_matrix.is_none());
    assert!(v3.canvas_clip_matrix.is_some());
}

unsafe extern "C" fn mesher_build(
    _: AmmlStr,
    _: u64,
    _: *const AmmlBlockGrid,
    _: *const AmmlMesherOptions,
    _: *mut AmmlMeshBuilt,
    _: *mut u32,
    _: u32,
) -> AmmlStatus {
    AmmlStatus::OK
}

#[test]
fn a_v3_render_table_reads_the_mesher_as_none() {
    let mut areas = AreaTables::new();
    areas.render.mesher_build = Some(mesher_build);
    areas.render.size = AmmlRenderTable::SIZE_V3;

    // SAFETY: the table is live and its `size` is covered; the bytes past v3 are ignored by the reader.
    let v3 = unsafe { AmmlRenderTable::read(&raw const areas.render) }.unwrap();
    areas.render.size = AmmlRenderTable::SIZE_V4;
    // SAFETY: as above, now with the v4 fields covered.
    let v4 = unsafe { AmmlRenderTable::read(&raw const areas.render) }.unwrap();

    assert_eq!(AmmlRenderTable::SIZE_V4, 80);
    assert!(v3.mesher_build.is_none());
    assert!(v4.mesher_build.is_some() && v4.mesher_group_begin.is_none());
}

unsafe extern "C" fn block_entity_free(_: AmmlStr, _: AmmlBlockEntityGhost) -> AmmlStatus {
    AmmlStatus::OK
}

#[test]
fn a_v4_render_table_reads_the_block_entity_calls_as_none() {
    let mut areas = AreaTables::new();
    areas.render.block_entity_free = Some(block_entity_free);
    areas.render.size = AmmlRenderTable::SIZE_V4;

    // SAFETY: the table is live and its `size` is covered; the bytes past v4 are ignored by the reader.
    let v4 = unsafe { AmmlRenderTable::read(&raw const areas.render) }.unwrap();
    areas.render.size = AmmlRenderTable::SIZE_V5;
    // SAFETY: as above, now with the v5 fields covered.
    let v5 = unsafe { AmmlRenderTable::read(&raw const areas.render) }.unwrap();

    assert_eq!(AmmlRenderTable::SIZE_V5, 104);
    assert!(v4.block_entity_free.is_none());
    assert!(v5.block_entity_free.is_some() && v5.block_entity_create.is_none());
}

unsafe extern "C" fn mesher_group_discard(_: AmmlStr, _: AmmlMeshGroup) -> AmmlStatus {
    AmmlStatus::OK
}

#[test]
fn a_v5_render_table_reads_the_group_discard_as_none() {
    let mut areas = AreaTables::new();
    areas.render.mesher_group_discard = Some(mesher_group_discard);
    areas.render.size = AmmlRenderTable::SIZE_V5;

    // SAFETY: the table is live and its `size` is covered; the bytes past v5 are ignored by the reader.
    let v5 = unsafe { AmmlRenderTable::read(&raw const areas.render) }.unwrap();
    areas.render.size = AmmlRenderTable::SIZE_V6;
    // SAFETY: as above, now with the v6 field covered.
    let v6 = unsafe { AmmlRenderTable::read(&raw const areas.render) }.unwrap();

    assert_eq!(AmmlRenderTable::SIZE_V6, AmmlRenderTable::SIZE_V5 + 8);
    assert!(v5.mesher_group_discard.is_none());
    assert!(v6.mesher_group_discard.is_some());
}

fn world_of(table: &AmmlServices) -> Option<*const AmmlWorldTable> {
    // SAFETY: the table and the area tables it points to live for the whole test.
    unsafe { AmmlServices::world(table) }
}

#[test]
fn a_v8_services_table_has_no_world_table() {
    let areas = AreaTables::new();
    // A v8 table is only 104 bytes long: the world field is not part of it, whatever it holds.
    assert!(world_of(&areas.services(AmmlServices::SIZE_V8)).is_none());
}

#[test]
fn a_v9_services_table_gives_the_world_table_unless_it_is_null_or_short() {
    let mut areas = AreaTables::new();
    assert_eq!(AmmlServices::SIZE_V9, 112);
    assert_eq!(world_of(&areas.services(AmmlServices::SIZE_V9)), Some(&raw const areas.world));
    let null = AmmlServices { world: core::ptr::null(), ..areas.services(AmmlServices::SIZE_V9) };
    assert!(world_of(&null).is_none());
    areas.world.size = AmmlWorldTable::SIZE_V1 - 8;
    assert!(world_of(&areas.services(AmmlServices::SIZE_V9)).is_none());
}

#[test]
fn a_v1_window_table_reads_clip_cursor_as_none() {
    let mut table = window_table();
    table.size = AmmlWindowTable::SIZE_V1;
    table.clip_cursor = Some(clip_cursor);

    // SAFETY: `table` is a live table whose `size` is covered; the bytes past v1 are ignored by the reader.
    let read = unsafe { AmmlWindowTable::read(&raw const table) }.unwrap();
    assert!(read.clip_cursor.is_none());
    assert_eq!(read.size, AmmlWindowTable::SIZE_V1);

    table.size = AmmlWindowTable::SIZE_V2;
    // SAFETY: as above; the size now covers the whole struct.
    let read = unsafe { AmmlWindowTable::read(&raw const table) }.unwrap();
    assert!(read.clip_cursor.is_some());
}

unsafe extern "C" fn clip_cursor(_module: AmmlStr, _clip: amml_abi::AmmlCursorClip) -> AmmlStatus {
    AmmlStatus::OK
}

unsafe extern "C" fn set_pose(_: AmmlStr, _: u64, _: *const AmmlPose) -> AmmlStatus {
    AmmlStatus::OK
}

unsafe extern "C" fn rendered_position(_: u64, _: *mut AmmlVec3) -> AmmlStatus {
    AmmlStatus::OK
}

unsafe extern "C" fn projection(_: u64, _: *mut AmmlProjection) -> AmmlStatus {
    AmmlStatus::OK
}

fn camera_of(table: &AmmlServices) -> Option<*const AmmlCameraTable> {
    // SAFETY: the table and the area tables it points to live for the whole test.
    unsafe { AmmlServices::camera(table) }
}

#[test]
fn a_v10_services_table_has_no_camera_table() {
    let areas = AreaTables::new();
    // A v10 table is only 120 bytes long: the camera field is not part of it, whatever it holds.
    assert!(camera_of(&areas.services(AmmlServices::SIZE_V10)).is_none());
}

#[test]
fn a_v11_services_table_gives_the_camera_table_unless_it_is_null_or_short() {
    let mut areas = AreaTables::new();
    assert_eq!(AmmlServices::SIZE_V11, 128);
    assert_eq!(camera_of(&areas.services(AmmlServices::SIZE_V11)), Some(&raw const areas.camera));
    let null = AmmlServices { camera: core::ptr::null(), ..areas.services(AmmlServices::SIZE_V11) };
    assert!(camera_of(&null).is_none());
    areas.camera.size = AmmlCameraTable::SIZE_V1 - 8;
    assert!(camera_of(&areas.services(AmmlServices::SIZE_V11)).is_none());
    // SAFETY: the table is live; its `size` is below v1, so the reader refuses it before reading more.
    assert!(unsafe { AmmlCameraTable::read(&raw const areas.camera) }.is_none());
}

fn options_of(table: &AmmlServices) -> Option<*const AmmlOptionsTable> {
    // SAFETY: the table and the area tables it points to live for the whole test.
    unsafe { AmmlServices::options(table) }
}

#[test]
fn services_v11_reads_no_options() {
    let areas = AreaTables::new();
    // A v11 table is only 128 bytes long: the options field is not part of it, whatever it holds.
    assert!(options_of(&areas.services(AmmlServices::SIZE_V11)).is_none());
}

#[test]
fn a_v12_services_table_gives_the_options_table_unless_it_is_null_or_short() {
    let mut areas = AreaTables::new();
    assert_eq!(AmmlServices::SIZE_V12, 136);
    assert_eq!(options_of(&areas.services(AmmlServices::SIZE_V12)), Some(&raw const areas.options));
    let null = AmmlServices { options: core::ptr::null(), ..areas.services(AmmlServices::SIZE_V12) };
    assert!(options_of(&null).is_none());
    areas.options.size = AmmlOptionsTable::SIZE_V1 - 8;
    assert!(options_of(&areas.services(AmmlServices::SIZE_V12)).is_none());
}

fn time_of(table: &AmmlServices) -> Option<*const AmmlTimeTable> {
    // SAFETY: the table and the area tables it points to live for the whole test.
    unsafe { AmmlServices::time(table) }
}

#[test]
fn services_v12_reads_no_time() {
    let mut areas = AreaTables::new();
    // A v12 table is only 136 bytes long: the time field is not part of it, whatever it holds.
    assert!(time_of(&areas.services(AmmlServices::SIZE_V12)).is_none());
    assert_eq!(AmmlServices::SIZE_V13, 144);
    assert_eq!(time_of(&areas.services(AmmlServices::SIZE_V13)), Some(&raw const areas.time));
    let null = AmmlServices { time: core::ptr::null(), ..areas.services(AmmlServices::SIZE_V13) };
    assert!(time_of(&null).is_none());
    areas.time.size = AmmlTimeTable::SIZE_V1 - 8;
    assert!(time_of(&areas.services(AmmlServices::SIZE_V13)).is_none());
}

#[test]
fn a_v1_network_table_reads_with_no_v2_function() {
    let areas = AreaTables::new();
    let mut bytes = vec![0xFFu8; core::mem::size_of::<AmmlNetworkTable>()];
    // SAFETY: `bytes` is as long as the table; the copy writes its first `SIZE_V1` bytes.
    unsafe {
        core::ptr::copy_nonoverlapping(
            (&raw const areas.network).cast::<u8>(),
            bytes.as_mut_ptr(),
            AmmlNetworkTable::SIZE_V1 as usize,
        );
    }
    bytes[..4].copy_from_slice(&AmmlNetworkTable::SIZE_V1.to_le_bytes());

    // SAFETY: the buffer holds a whole table; bytes past the claimed size are never read.
    let read = unsafe { AmmlNetworkTable::read(bytes.as_ptr().cast()) }.unwrap();
    assert!(read.protocol_version.is_none());
    assert!(read.set_client_chunk_cache.is_none());
}

#[test]
fn a_v1_overlay_table_reads_with_no_prepare_surface() {
    let areas = AreaTables::new();
    let mut bytes = vec![0xFFu8; core::mem::size_of::<AmmlOverlayTable>()];
    // SAFETY: `bytes` is as long as the table; the copy writes its first `SIZE_V1` bytes.
    unsafe {
        core::ptr::copy_nonoverlapping(
            (&raw const areas.overlay).cast::<u8>(),
            bytes.as_mut_ptr(),
            AmmlOverlayTable::SIZE_V1 as usize,
        );
    }
    bytes[..4].copy_from_slice(&AmmlOverlayTable::SIZE_V1.to_le_bytes());

    // SAFETY: the buffer holds a whole table; bytes past the claimed size are never read.
    let read = unsafe { AmmlOverlayTable::read(bytes.as_ptr().cast()) }.unwrap();
    assert!(read.prepare_surface.is_none());
    // SAFETY: the table of `areas` is a whole v2 table.
    let full = unsafe { AmmlOverlayTable::read(&raw const areas.overlay) }.unwrap();
    assert!(full.prepare_surface.is_some());
}
