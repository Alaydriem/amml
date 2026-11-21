use core::ffi::c_void;

use amml_abi::{
    AmmlActorTable, AmmlCaptureFrame, AmmlCaptureRequest, AmmlClientFlag, AmmlClientTable, AmmlHand, AmmlInputMode,
    AmmlInputTable, AmmlInventoryTable, AmmlItemSnapshot, AmmlLeaseRequest, AmmlLeaseState, AmmlLightSource,
    AmmlLightingTable, AmmlModifiers, AmmlMouseButton, AmmlMoveIntent, AmmlNetworkTable, AmmlOverlayTable, AmmlPoint,
    AmmlRect, AmmlScreen, AmmlScreenKind, AmmlServices, AmmlShotLayers, AmmlSize, AmmlStatus, AmmlStr, AmmlUiProfile,
    AmmlUiTable, AmmlWindowTable,
};

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
    }
}

fn overlay_table() -> AmmlOverlayTable {
    AmmlOverlayTable {
        size: AmmlOverlayTable::SIZE_V1,
        version: 1,
        screenshot,
        open_capture,
        capture_take,
        capture_give_back,
        capture_free_slots,
        close_capture,
        present_count,
        copy_back_buffer,
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
