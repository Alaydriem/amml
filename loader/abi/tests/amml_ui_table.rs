use amml_abi::{
    AmmlAmount, AmmlChatField, AmmlContainerAction, AmmlImageView, AmmlInlineIcon, AmmlInputMode, AmmlItemDraw,
    AmmlItemDrawFlags, AmmlPoint, AmmlRect, AmmlScreenKind, AmmlServices, AmmlSize, AmmlSlotAt, AmmlStatus, AmmlStr,
    AmmlTextAlign, AmmlUiProfile, AmmlUiTable,
};

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

fn services(size: u32, ui: *const AmmlUiTable) -> AmmlServices {
    AmmlServices {
        size,
        version: 7,
        events: core::ptr::null(),
        core: core::ptr::null(),
        registry: core::ptr::null(),
        client: core::ptr::null(),
        input: core::ptr::null(),
        window: core::ptr::null(),
        overlay: core::ptr::null(),
        actor: core::ptr::null(),
        inventory: core::ptr::null(),
        lighting: core::ptr::null(),
        ui,
        network: core::ptr::null(),
    }
}

fn ui_of(table: &AmmlServices) -> Option<*const AmmlUiTable> {
    // SAFETY: the services table and the ui table it points to live for the whole test.
    unsafe { AmmlServices::ui(table) }
}

#[test]
fn a_v6_services_table_has_no_ui_table() {
    let ui = ui_table();
    // A v6 table is only 88 bytes long: the ui field is not part of it, whatever it holds.
    assert!(ui_of(&services(AmmlServices::SIZE_V6, &raw const ui)).is_none());
}

#[test]
fn a_v7_services_table_gives_the_ui_table() {
    let ui = ui_table();
    assert_eq!(ui_of(&services(AmmlServices::SIZE_V7, &raw const ui)), Some(&raw const ui));
}

#[test]
fn a_v7_services_table_with_a_null_or_short_ui_table_has_none() {
    let mut ui = ui_table();
    assert!(ui_of(&services(AmmlServices::SIZE_V7, core::ptr::null())).is_none());
    ui.size = AmmlUiTable::SIZE_V1 - 8;
    assert!(ui_of(&services(AmmlServices::SIZE_V7, &raw const ui)).is_none());
}

#[test]
fn a_v1_ui_table_reads_with_no_draw_or_texture_function() {
    let ui = ui_table();

    // SAFETY: `ui` is a live table whose `size` says it holds the v1 fields only.
    let copy = unsafe { AmmlUiTable::read(&raw const ui) }.unwrap();

    assert_eq!(copy.size, AmmlUiTable::SIZE_V1);
    assert!(copy.draw_fill.is_none() && copy.draw_image.is_none() && copy.texture_load.is_none());
}

#[test]
fn a_short_or_null_ui_table_does_not_read() {
    let mut ui = ui_table();
    ui.size = AmmlUiTable::SIZE_V1 - 8;

    // SAFETY: `ui` is a live table; a null pointer is the case under test.
    unsafe {
        assert!(AmmlUiTable::read(&raw const ui).is_none());
        assert!(AmmlUiTable::read(core::ptr::null()).is_none());
    }
}

unsafe extern "C" fn container_action(_: u64, _: AmmlContainerAction, _: AmmlSlotAt, _: AmmlAmount) -> AmmlStatus {
    AmmlStatus::OK
}

#[test]
fn a_v2_ui_table_reads_with_no_container_function() {
    let mut ui = ui_table();
    ui.container_action = Some(container_action);
    ui.size = AmmlUiTable::SIZE_V2;

    // SAFETY: `ui` is a live table; its `size` covers the v2 fields only, so the container field is not read.
    let copy = unsafe { AmmlUiTable::read(&raw const ui) }.unwrap();

    assert!(copy.container_action.is_none() && copy.container_swap.is_none());
}

#[test]
fn a_v3_ui_table_reads_its_container_functions() {
    let mut ui = ui_table();
    ui.container_action = Some(container_action);
    ui.size = AmmlUiTable::SIZE_V3;

    // SAFETY: `ui` is a live table whose `size` covers every field.
    let copy = unsafe { AmmlUiTable::read(&raw const ui) }.unwrap();

    assert!(copy.container_action.is_some() && copy.container_item.is_none());
}

unsafe extern "C" fn draw_item(
    _: amml_abi::AmmlStr,
    _: u64,
    _: AmmlItemDraw,
    _: AmmlPoint,
    _: AmmlItemDrawFlags,
    _: f32,
    _: i32,
) -> AmmlStatus {
    AmmlStatus::OK
}

#[test]
fn a_v3_ui_table_reads_with_no_item_draw_and_a_v4_table_reads_it() {
    let mut ui = ui_table();
    ui.draw_item = Some(draw_item);
    ui.size = AmmlUiTable::SIZE_V3;

    // SAFETY: `ui` is a live table; its `size` covers the v3 fields only, so the item field is not read.
    let older = unsafe { AmmlUiTable::read(&raw const ui) }.unwrap();
    ui.size = AmmlUiTable::SIZE_V4;
    // SAFETY: `ui` is a live table whose `size` covers every field.
    let newer = unsafe { AmmlUiTable::read(&raw const ui) }.unwrap();

    assert!(older.draw_item.is_none());
    assert!(newer.draw_item.is_some());
}

unsafe extern "C" fn text_width(_: u64, _: amml_abi::AmmlStr, _: *mut f32) -> AmmlStatus {
    AmmlStatus::OK
}

unsafe extern "C" fn draw_text(
    _: u64,
    _: amml_abi::AmmlStr,
    _: AmmlRect,
    _: amml_abi::AmmlColor,
    _: f32,
    _: AmmlTextAlign,
) -> AmmlStatus {
    AmmlStatus::OK
}

#[test]
fn a_v4_ui_table_reads_with_no_text_function_and_a_v5_table_reads_both() {
    let mut ui = ui_table();
    ui.text_width = Some(text_width);
    ui.draw_text = Some(draw_text);
    ui.size = AmmlUiTable::SIZE_V4;

    // SAFETY: `ui` is a live table; its `size` covers the v4 fields only, so the text fields are not read.
    let older = unsafe { AmmlUiTable::read(&raw const ui) }.unwrap();
    ui.size = AmmlUiTable::SIZE_V5;
    // SAFETY: `ui` is a live table whose `size` covers every field.
    let newer = unsafe { AmmlUiTable::read(&raw const ui) }.unwrap();

    assert!(older.text_width.is_none() && older.draw_text.is_none());
    assert!(newer.text_width.is_some() && newer.draw_text.is_some());
}

#[test]
fn the_text_alignments_are_left_center_right() {
    assert_eq!((AmmlTextAlign::LEFT.0, AmmlTextAlign::CENTER.0, AmmlTextAlign::RIGHT.0), (0, 1, 2));
}

unsafe extern "C" fn chat_text(_: AmmlChatField, _: *mut AmmlStr) -> AmmlStatus {
    AmmlStatus::OK
}
unsafe extern "C" fn chat_set_text(_: AmmlStr, _: AmmlChatField, _: AmmlStr) -> AmmlStatus {
    AmmlStatus::OK
}
unsafe extern "C" fn chat_icons_active() -> u8 {
    1
}
unsafe extern "C" fn icon_register(_: AmmlStr, _: *const AmmlImageView, _: f32, _: *mut AmmlInlineIcon) -> AmmlStatus {
    AmmlStatus::OK
}
unsafe extern "C" fn icon_update(_: AmmlStr, _: AmmlInlineIcon, _: *const AmmlImageView) -> AmmlStatus {
    AmmlStatus::OK
}
unsafe extern "C" fn icon_release(_: AmmlStr, _: AmmlInlineIcon) -> AmmlStatus {
    AmmlStatus::OK
}

#[test]
fn a_v5_ui_table_reads_with_no_chat_or_icon_function_and_a_v6_table_reads_all_six() {
    let mut ui = ui_table();
    ui.chat_text = Some(chat_text);
    ui.chat_set_text = Some(chat_set_text);
    ui.chat_icons_active = Some(chat_icons_active);
    ui.icon_register = Some(icon_register);
    ui.icon_update = Some(icon_update);
    ui.icon_release = Some(icon_release);
    ui.size = AmmlUiTable::SIZE_V5;

    // SAFETY: `ui` is a live table; its `size` covers the v5 fields only, so the chat fields are not read.
    let older = unsafe { AmmlUiTable::read(&raw const ui) }.unwrap();
    ui.size = AmmlUiTable::SIZE_V6;
    // SAFETY: `ui` is a live table whose `size` covers every field.
    let newer = unsafe { AmmlUiTable::read(&raw const ui) }.unwrap();

    assert!(older.chat_text.is_none() && older.chat_set_text.is_none() && older.chat_icons_active.is_none());
    assert!(older.icon_register.is_none() && older.icon_update.is_none() && older.icon_release.is_none());
    assert!(newer.chat_text.is_some() && newer.chat_set_text.is_some() && newer.chat_icons_active.is_some());
    assert!(newer.icon_register.is_some() && newer.icon_update.is_some() && newer.icon_release.is_some());
    assert_eq!(AmmlUiTable::SIZE_V6, AmmlUiTable::SIZE_V5 + 48);
}

#[test]
fn the_chat_fields_are_username_xuid_text_filtered_text() {
    assert_eq!(
        (AmmlChatField::USERNAME.0, AmmlChatField::XUID.0, AmmlChatField::TEXT.0, AmmlChatField::FILTERED_TEXT.0),
        (1, 2, 3, 4)
    );
}
