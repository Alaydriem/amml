use core::ffi::c_void;

use crate::{
    AmmlAmount, AmmlChatField, AmmlColor, AmmlContainerAction, AmmlImageView, AmmlInlineIcon, AmmlInputMode,
    AmmlItemDraw, AmmlItemDrawFlags, AmmlPoint, AmmlRect, AmmlScreenKind, AmmlSize, AmmlSlotAt, AmmlStatus, AmmlStr,
    AmmlTextAlign, AmmlTextureId, AmmlTextureSource, AmmlUiProfile, AmmlUv,
};

/// Gives the kind of the screen drawn by `view`.
pub type UiViewKindFn = unsafe extern "C" fn(view: u64, out: *mut AmmlScreenKind) -> AmmlStatus;
/// Gives the size of `view` in GUI units, rounded down.
pub type UiViewSizeFn = unsafe extern "C" fn(view: u64, out: *mut AmmlSize) -> AmmlStatus;
/// Gives the content rectangle of `view` in GUI units: the panel that holds the slots of the screen.
/// `NOT_AVAILABLE` for a screen or UI profile that has none, or while the panel is hidden.
pub type UiViewContentRectFn = unsafe extern "C" fn(view: u64, out: *mut AmmlRect) -> AmmlStatus;
/// Gives the last pointer position of `view`, in GUI units.
pub type UiViewPointerFn = unsafe extern "C" fn(view: u64, out: *mut AmmlPoint) -> AmmlStatus;
/// Gives the input mode of `view`.
pub type UiViewInputModeFn = unsafe extern "C" fn(view: u64, out: *mut AmmlInputMode) -> AmmlStatus;
/// Gives the UI profile option of the primary client.
pub type UiViewUiProfileFn = unsafe extern "C" fn(view: u64, out: *mut AmmlUiProfile) -> AmmlStatus;
/// Gives the screen id of `view` in `out_screen` when its controller is a container screen, and 0 when it is not.
pub type UiViewContainerFn = unsafe extern "C" fn(view: u64, out_screen: *mut u64) -> AmmlStatus;

/// Fills `rect` of `view` with `color` at once. `REFUSED` outside the `ScreenRender` call of `view`.
pub type UiDrawFillFn = unsafe extern "C" fn(view: u64, rect: AmmlRect, color: AmmlColor, alpha: f32) -> AmmlStatus;
/// Draws the border of `rect` of `view`, `thickness` GUI units wide, at once.
pub type UiDrawOutlineFn =
    unsafe extern "C" fn(view: u64, rect: AmmlRect, color: AmmlColor, alpha: f32, thickness: i32) -> AmmlStatus;
/// Draws the part `uv` of `texture` into `rect` of `view`, multiplied by `tint` and `alpha`. A texture that is not
/// ready is skipped with `OK`. `REFUSED` while the game left images queued. The loader draws the queued images
/// after the last `ScreenRender` listener, and before any later fill or outline of the same call.
pub type UiDrawImageFn = unsafe extern "C" fn(
    view: u64,
    texture: AmmlTextureId,
    rect: AmmlRect,
    uv: AmmlUv,
    tint: AmmlColor,
    alpha: f32,
) -> AmmlStatus;
/// Gives the GUI scale of `view`: screen pixels per GUI unit.
pub type UiGuiScaleFn = unsafe extern "C" fn(view: u64, out: *mut f32) -> AmmlStatus;
/// 1 when `view` may draw images now, 0 when it may not (a view that is not running, or images the game left queued).
pub type UiCanDrawFn = unsafe extern "C" fn(view: u64) -> u8;
/// Asks for the texture `path` of `source` for the mod `module`. The loader binds it at the next UI draw, so the
/// texture is not ready when the call returns. `REFUSED` for a mod file path that leaves the mod folder.
pub type UiTextureLoadFn = unsafe extern "C" fn(
    module: AmmlStr,
    source: AmmlTextureSource,
    path: AmmlStr,
    out: *mut AmmlTextureId,
) -> AmmlStatus;
/// Uploads `width` x `height` RGBA pixels (`len` bytes, copied before the call returns) as a new texture of
/// `module`.
pub type UiTextureUploadRgbaFn = unsafe extern "C" fn(
    module: AmmlStr,
    width: u32,
    height: u32,
    pixels: *const u8,
    len: usize,
    out: *mut AmmlTextureId,
) -> AmmlStatus;
/// 1 when `texture` is current and the game gave its data, 0 otherwise.
pub type UiTextureReadyFn = unsafe extern "C" fn(texture: AmmlTextureId) -> u8;
/// Frees `texture` of `module` at the next UI draw. `STALE` for a texture of an older generation or one freed.
pub type UiTextureUnloadFn = unsafe extern "C" fn(module: AmmlStr, texture: AmmlTextureId) -> AmmlStatus;

/// Gives the number of slots of `collection` (an `AmmlSlotAt` collection) of the container screen of `view`. `REFUSED`
/// outside the `ScreenRender` call of a container view; `NOT_AVAILABLE` for the collection `OTHER` and for a screen
/// whose slot count is not known.
pub type UiContainerSlotCountFn = unsafe extern "C" fn(view: u64, collection: u32, out: *mut u32) -> AmmlStatus;
/// Lends the stack in `slot` of the container screen of `view` in `out_stack`: a stack handle for the `inventory`
/// table calls, valid in the running `ScreenRender` call only. The game's empty stack gives a null handle.
/// `INVALID_ARGUMENT` for an index outside the collection.
pub type UiContainerItemFn =
    unsafe extern "C" fn(view: u64, slot: AmmlSlotAt, out_stack: *mut *const c_void) -> AmmlStatus;
/// Lends the stack on the cursor of the container screen of `view`, as `UiContainerItemFn`.
pub type UiContainerCursorFn = unsafe extern "C" fn(view: u64, out_stack: *mut *const c_void) -> AmmlStatus;
/// Makes `action` on `slot` of the container screen of `view`, as the game's own button does. `amount` is read only
/// by `DROP_SLOT` and `DROP_CURSOR`; `slot` is ignored by `DROP_CURSOR`. `REFUSED` outside the `ScreenRender` call of a
/// container view.
pub type UiContainerActionFn =
    unsafe extern "C" fn(view: u64, action: AmmlContainerAction, slot: AmmlSlotAt, amount: AmmlAmount) -> AmmlStatus;
/// Swaps the stacks of slots `a` and `b` of the container screen of `view`. `out_swapped` is 1 when the game made the
/// swap and 0 when it refused it.
pub type UiContainerSwapFn =
    unsafe extern "C" fn(view: u64, a: AmmlSlotAt, b: AmmlSlotAt, out_swapped: *mut u8) -> AmmlStatus;

/// Draws the stack of `draw` of `module` with its top left corner at `at` in `view`, 16 GUI units wide, at once.
/// `alpha` is the transparency and `z` the layer. `REFUSED` outside the `ScreenRender` call of `view`, for a draw of
/// another module; `INVALID_ARGUMENT` for flags the loader does not know; `STALE` for a draw of an older generation or one freed.
/// Version 4 of the table.
pub type UiDrawItemFn = unsafe extern "C" fn(
    module: AmmlStr,
    view: u64,
    draw: AmmlItemDraw,
    at: AmmlPoint,
    flags: AmmlItemDrawFlags,
    alpha: f32,
    z: i32,
) -> AmmlStatus;

/// Gives the width of `text` in GUI units, in the game's default font at its normal size, in `out`. `REFUSED` outside
/// the `ScreenRender` call of `view`, for a text of more than 256 bytes and off the game thread; `INVALID_ARGUMENT`
/// for a text that is not UTF-8. Version 5 of the table.
pub type UiTextWidthFn = unsafe extern "C" fn(view: u64, text: AmmlStr, out: *mut f32) -> AmmlStatus;
/// Queues `text` in the default font, in `color` at `alpha`, in `rect` of `view` (`align` is the place of the text
/// along the width of `rect`; the text starts at the top of `rect`). The loader copies the text and draws it after
/// the last `ScreenRender` listener, over everything the listeners drew. `REFUSED` outside the `ScreenRender` call of
/// `view`, for a text of more than 256 bytes, for a text after 1024 texts in one dispatch, and off the game thread;
/// `INVALID_ARGUMENT` for a text that is not UTF-8, an `align` the loader does not know and a value that is not
/// finite. Version 5 of the table.
pub type UiDrawTextFn = unsafe extern "C" fn(
    view: u64,
    text: AmmlStr,
    rect: AmmlRect,
    color: AmmlColor,
    alpha: f32,
    align: AmmlTextAlign,
) -> AmmlStatus;

/// Gives one string of the chat line of the running `ChatMessageAdded` in `out`, as the game built it: the bytes are the
/// game's own, valid until the last listener returns, and a write of this event is not seen. `REFUSED` outside the
/// dispatch of a `ChatMessageAdded` on the calling thread; `INVALID_ARGUMENT` for a field the loader does not know;
/// `NOT_AVAILABLE` for the filtered text of a line that has none. A string that is not UTF-8, or whose size and capacity
/// are not believable, gives `INVALID_ARGUMENT`. Version 6 of the table.
pub type UiChatTextFn = unsafe extern "C" fn(field: AmmlChatField, out: *mut AmmlStr) -> AmmlStatus;
/// Replaces one string of the chat line of the running `ChatMessageAdded` of `module` with `text`. The loader copies the
/// text and writes it to the game after the last listener. `REFUSED` outside the dispatch of a `ChatMessageAdded` on the
/// calling thread and for a text of more than 4096 bytes; `INVALID_ARGUMENT` for a field the loader does not know, for
/// `USERNAME` and `XUID` (read only) and for a text that is not UTF-8; `NOT_AVAILABLE` for the filtered text of a line
/// that has none. Version 6 of the table.
pub type UiChatSetTextFn = unsafe extern "C" fn(module: AmmlStr, field: AmmlChatField, text: AmmlStr) -> AmmlStatus;
/// 1 when the default font draws inline icons now: the loader serves the area and the font is a bitmap font. 0 when it
/// does not. Version 6 of the table.
pub type UiChatIconsActiveFn = unsafe extern "C" fn() -> u8;
/// Registers `image` (at most 16 x 16 RGBA, copied before the call returns) as an inline icon of `module`, drawn
/// `advance` GUI units wide (above 0, at most 16), in `out`. `REFUSED` for an advance or an image that is too big, all
/// 240 icons in use, and a call off the game thread; `NOT_AVAILABLE` when the default font is not a bitmap font;
/// `INVALID_ARGUMENT` for a size of 0 or pixels that do not fit it. Version 6 of the table.
pub type UiIconRegisterFn = unsafe extern "C" fn(
    module: AmmlStr,
    image: *const AmmlImageView,
    advance: f32,
    out: *mut AmmlInlineIcon,
) -> AmmlStatus;
/// Replaces the image of `icon` of `module`. `STALE` for an icon of an older generation or one freed; `REFUSED` for an
/// icon of another module or an image over 16 x 16. Allowed on any thread. Version 6 of the table.
pub type UiIconUpdateFn =
    unsafe extern "C" fn(module: AmmlStr, icon: AmmlInlineIcon, image: *const AmmlImageView) -> AmmlStatus;
/// Frees `icon` of `module`; the loader blanks its cell at the next UI draw. `STALE` for an icon of an older generation
/// or one freed. Allowed on any thread. Version 6 of the table.
pub type UiIconReleaseFn = unsafe extern "C" fn(module: AmmlStr, icon: AmmlInlineIcon) -> AmmlStatus;

/// The ui functions of `AmmlServices`: the screen reads (area `ui.screen`) and the container functions (area
/// `ui.screen` at version 2), the draw functions (area `ui.draw`)
/// the texture functions (area `ui.textures`), and the chat and inline icon functions (event `ui.chat_message`, area
/// `ui.inline_icons`). Every function takes the view id of a
/// `ScreenRender` event and answers only while that event's dispatch runs on the calling thread: any other
/// view id, a retired one included, gives `REFUSED`; so does a draw function. A null output pointer gives
/// `INVALID_ARGUMENT`. The texture functions take no view and run on any thread: they only queue.
/// The table only grows: a function added later is appended, and a reader calls a function only when `size`
/// covers it.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct AmmlUiTable {
    pub size: u32,
    pub version: u32,
    pub view_kind: UiViewKindFn,
    pub view_size: UiViewSizeFn,
    pub view_content_rect: UiViewContentRectFn,
    pub view_pointer: UiViewPointerFn,
    pub view_input_mode: UiViewInputModeFn,
    pub view_ui_profile: UiViewUiProfileFn,
    pub view_container: UiViewContainerFn,
    pub draw_fill: Option<UiDrawFillFn>,
    pub draw_outline: Option<UiDrawOutlineFn>,
    pub draw_image: Option<UiDrawImageFn>,
    pub gui_scale: Option<UiGuiScaleFn>,
    pub can_draw: Option<UiCanDrawFn>,
    pub texture_load: Option<UiTextureLoadFn>,
    pub texture_upload_rgba: Option<UiTextureUploadRgbaFn>,
    pub texture_ready: Option<UiTextureReadyFn>,
    pub texture_unload: Option<UiTextureUnloadFn>,
    pub container_slot_count: Option<UiContainerSlotCountFn>,
    pub container_item: Option<UiContainerItemFn>,
    pub container_cursor: Option<UiContainerCursorFn>,
    pub container_action: Option<UiContainerActionFn>,
    pub container_swap: Option<UiContainerSwapFn>,
    pub draw_item: Option<UiDrawItemFn>,
    pub text_width: Option<UiTextWidthFn>,
    pub draw_text: Option<UiDrawTextFn>,
    pub chat_text: Option<UiChatTextFn>,
    pub chat_set_text: Option<UiChatSetTextFn>,
    pub chat_icons_active: Option<UiChatIconsActiveFn>,
    pub icon_register: Option<UiIconRegisterFn>,
    pub icon_update: Option<UiIconUpdateFn>,
    pub icon_release: Option<UiIconReleaseFn>,
}

impl AmmlUiTable {
    /// `size` of a v1 ui table. A literal, so it keeps its meaning when a later version adds fields.
    pub const SIZE_V1: u32 = 64;
    /// `size` of a v2 ui table: v1 plus the draw and texture functions. Its `version` is 2.
    pub const SIZE_V2: u32 = 136;
    /// `size` of a v3 ui table: v2 plus the container functions. Its `version` is 3.
    pub const SIZE_V3: u32 = 176;
    /// `size` of a v4 ui table: v3 plus `draw_item`. Its `version` is 4.
    pub const SIZE_V4: u32 = 184;
    /// `size` of a v5 ui table: v4 plus `text_width` and `draw_text`. Its `version` is 5.
    pub const SIZE_V5: u32 = 200;
    /// `size` of a v6 ui table: v5 plus the chat and inline icon functions. Its `version` is 6.
    pub const SIZE_V6: u32 = 248;

    /// Copies `table` as far as its own `size` covers. A function that a shorter table does not hold is `None`.
    /// A null table, or one shorter than v1, gives `None`.
    ///
    /// # Safety
    /// A non-null `table` points to a table readable for its own `size` bytes, at least 4.
    #[allow(unsafe_code)]
    pub unsafe fn read(table: *const AmmlUiTable) -> Option<AmmlUiTable> {
        if table.is_null() {
            return None;
        }
        // SAFETY: the caller guarantees the leading `size` field is readable.
        let size = unsafe { table.cast::<u32>().read_unaligned() };
        if size < Self::SIZE_V1 {
            return None;
        }
        let covered = (size as usize).min(core::mem::size_of::<AmmlUiTable>());
        let mut copy = core::mem::MaybeUninit::<AmmlUiTable>::zeroed();
        // SAFETY: the caller guarantees `covered` bytes are readable at `table`; `copy` has room for the
        // whole struct and the two do not overlap.
        unsafe { core::ptr::copy_nonoverlapping(table.cast::<u8>(), copy.as_mut_ptr().cast::<u8>(), covered) };
        // SAFETY: the copied bytes cover every v1 field, written from a valid table; every later field is an
        // `Option` of a function pointer, for which all zero bytes are `None`.
        Some(unsafe { copy.assume_init() })
    }
}

const _: () = assert!(core::mem::size_of::<AmmlUiTable>() == AmmlUiTable::SIZE_V6 as usize);
