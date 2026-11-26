use amml_abi::{
    AmmlAmount, AmmlChatField, AmmlColor, AmmlContainerAction, AmmlDirKind, AmmlImageView, AmmlInlineIcon,
    AmmlInputMode, AmmlItemDraw, AmmlItemDrawFlags, AmmlPoint, AmmlRect, AmmlScreenKind, AmmlSize, AmmlSlotAt,
    AmmlStatus, AmmlStr, AmmlTextAlign, AmmlTextureId, AmmlTextureSource, AmmlUiProfile, AmmlUiTable, AmmlUv,
};

use std::ffi::c_void;

use crate::{
    Boundary, ChatAdapter, ContainerAdapter, ContainerTransferAmount, InlineIconAdapter, ItemDrawAdapter, LoaderState,
    ScreenAdapter, TextAdapter, TextureAdapter, UiDrawAdapter,
};

const TABLE_VERSION: u32 = 6;

pub(crate) static UI: AmmlUiTable = AmmlUiTable {
    size: AmmlUiTable::SIZE_V6,
    version: TABLE_VERSION,
    view_kind: UiTable::view_kind,
    view_size: UiTable::view_size,
    view_content_rect: UiTable::view_content_rect,
    view_pointer: UiTable::view_pointer,
    view_input_mode: UiTable::view_input_mode,
    view_ui_profile: UiTable::view_ui_profile,
    view_container: UiTable::view_container,
    draw_fill: Some(UiTable::draw_fill),
    draw_outline: Some(UiTable::draw_outline),
    draw_image: Some(UiTable::draw_image),
    gui_scale: Some(UiTable::gui_scale),
    can_draw: Some(UiTable::can_draw),
    texture_load: Some(UiTable::texture_load),
    texture_upload_rgba: Some(UiTable::texture_upload_rgba),
    texture_ready: Some(UiTable::texture_ready),
    texture_unload: Some(UiTable::texture_unload),
    container_slot_count: Some(UiTable::container_slot_count),
    container_item: Some(UiTable::container_item),
    container_cursor: Some(UiTable::container_cursor),
    container_action: Some(UiTable::container_action),
    container_swap: Some(UiTable::container_swap),
    draw_item: Some(UiTable::draw_item),
    text_width: Some(UiTable::text_width),
    draw_text: Some(UiTable::draw_text),
    chat_text: Some(UiTable::chat_text),
    chat_set_text: Some(UiTable::chat_set_text),
    chat_icons_active: Some(UiTable::chat_icons_active),
    icon_register: Some(UiTable::icon_register),
    icon_update: Some(UiTable::icon_update),
    icon_release: Some(UiTable::icon_release),
};

/// The ui table behind `AmmlServices::ui`, and its `extern "C"` functions. Each function checks its output
/// pointer (`INVALID_ARGUMENT`), then delegates to `ScreenAdapter` inside the boundary. The adapter answers only
/// for the view of a `ScreenRender` dispatch that runs now on the calling thread (`REFUSED` for any other id);
/// with no screen adapter every call gives `NOT_AVAILABLE`. The draw functions go to `UiDrawAdapter` under the same
/// rule. The texture functions check the module (`INVALID_ARGUMENT` for an unknown one, `REFUSED` for one that is not
/// live) and delegate to `TextureAdapter`, which only queues; they run on any thread. The container functions check
/// their slots and amounts (`INVALID_ARGUMENT`), then the screen adapter proves the view is the running dispatch of a
/// container screen (`REFUSED` otherwise) and `ContainerAdapter` calls the game. The text functions go to
/// `TextAdapter`, which checks the scope, the thread and the text. The chat functions go to `ChatAdapter`, which answers
/// only inside the running `ChatMessageAdded` of the calling thread (`REFUSED` otherwise); the icon functions check the
/// module like the texture functions do and go to `InlineIconAdapter`.
pub struct UiTable;

impl UiTable {
    unsafe extern "C" fn view_kind(view: u64, out: *mut AmmlScreenKind) -> AmmlStatus {
        Self::write(out, view, ScreenAdapter::view_kind)
    }

    unsafe extern "C" fn view_size(view: u64, out: *mut AmmlSize) -> AmmlStatus {
        Self::write(out, view, ScreenAdapter::view_size)
    }

    unsafe extern "C" fn view_content_rect(view: u64, out: *mut AmmlRect) -> AmmlStatus {
        Self::write(out, view, ScreenAdapter::view_content_rect)
    }

    unsafe extern "C" fn view_pointer(view: u64, out: *mut AmmlPoint) -> AmmlStatus {
        Self::write(out, view, ScreenAdapter::view_pointer)
    }

    unsafe extern "C" fn view_input_mode(view: u64, out: *mut AmmlInputMode) -> AmmlStatus {
        Self::write(out, view, ScreenAdapter::view_input_mode)
    }

    unsafe extern "C" fn view_ui_profile(view: u64, out: *mut AmmlUiProfile) -> AmmlStatus {
        Self::write(out, view, ScreenAdapter::view_ui_profile)
    }

    unsafe extern "C" fn view_container(view: u64, out_screen: *mut u64) -> AmmlStatus {
        Self::write(out_screen, view, ScreenAdapter::view_container)
    }

    // Checks `out`, runs `read` on the adapter and writes its value to `out`, all inside the boundary.
    fn write<T>(out: *mut T, view: u64, read: impl FnOnce(&ScreenAdapter, u64) -> Result<T, AmmlStatus>) -> AmmlStatus {
        Boundary::guard(|| {
            if out.is_null() {
                return AmmlStatus::INVALID_ARGUMENT;
            }
            let Some(adapter) = ScreenAdapter::get() else {
                return AmmlStatus::NOT_AVAILABLE;
            };
            match read(adapter, view) {
                Ok(value) => {
                    // SAFETY: `out` is non-null and the caller promises it is writable.
                    unsafe { out.write(value) };
                    AmmlStatus::OK
                }
                Err(status) => status,
            }
        })
    }

    unsafe extern "C" fn container_slot_count(view: u64, collection: u32, out: *mut u32) -> AmmlStatus {
        Boundary::guard(|| {
            if out.is_null() {
                return AmmlStatus::INVALID_ARGUMENT;
            }
            match Self::container(view, |_, _, kind| ContainerAdapter::slot_count(kind, collection)) {
                Ok(count) => {
                    // SAFETY: `out` is non-null and the caller promises it is writable.
                    unsafe { out.write(count) };
                    AmmlStatus::OK
                }
                Err(status) => status,
            }
        })
    }

    unsafe extern "C" fn container_item(view: u64, slot: AmmlSlotAt, out_stack: *mut *const c_void) -> AmmlStatus {
        Boundary::guard(|| {
            if out_stack.is_null() || !Self::slot_is_valid(slot) {
                return AmmlStatus::INVALID_ARGUMENT;
            }
            let stack = Self::container(view, |adapter, controller, kind| adapter.item(controller, kind, slot));
            Self::write_stack(out_stack, stack)
        })
    }

    unsafe extern "C" fn container_cursor(view: u64, out_stack: *mut *const c_void) -> AmmlStatus {
        Boundary::guard(|| {
            if out_stack.is_null() {
                return AmmlStatus::INVALID_ARGUMENT;
            }
            let stack = Self::container(view, |adapter, controller, _| adapter.cursor(controller));
            Self::write_stack(out_stack, stack)
        })
    }

    unsafe extern "C" fn container_action(
        view: u64,
        action: AmmlContainerAction,
        slot: AmmlSlotAt,
        amount: AmmlAmount,
    ) -> AmmlStatus {
        Boundary::guard(|| {
            let known = (AmmlContainerAction::AUTO_PLACE.0..=AmmlContainerAction::DROP_CURSOR.0).contains(&action.0);
            let slot_used = action != AmmlContainerAction::DROP_CURSOR;
            if !known || (slot_used && !Self::slot_is_valid(slot)) || ContainerTransferAmount::new(amount).is_none() {
                return AmmlStatus::INVALID_ARGUMENT;
            }
            if let Err(status) = ContainerAdapter::require_game_thread() {
                return status;
            }
            match Self::container(view, |adapter, controller, kind| {
                adapter.action(controller, kind, action, slot, amount)
            }) {
                Ok(()) => AmmlStatus::OK,
                Err(status) => status,
            }
        })
    }

    unsafe extern "C" fn container_swap(view: u64, a: AmmlSlotAt, b: AmmlSlotAt, out_swapped: *mut u8) -> AmmlStatus {
        Boundary::guard(|| {
            if out_swapped.is_null() || !Self::slot_is_valid(a) || !Self::slot_is_valid(b) {
                return AmmlStatus::INVALID_ARGUMENT;
            }
            if let Err(status) = ContainerAdapter::require_game_thread() {
                return status;
            }
            match Self::container(view, |adapter, controller, kind| adapter.swap(controller, kind, a, b)) {
                Ok(swapped) => {
                    // SAFETY: `out_swapped` is non-null and the caller promises it is writable.
                    unsafe { out_swapped.write(u8::from(swapped)) };
                    AmmlStatus::OK
                }
                Err(status) => status,
            }
        })
    }

    // Whether `slot` names a collection of the API and an index in the range every call takes; the adapter checks the
    // index against the slot count of the screen.
    fn slot_is_valid(slot: AmmlSlotAt) -> bool {
        slot.collection < AmmlSlotAt::OTHER && (0..=AmmlSlotAt::MAX_INDEX).contains(&slot.index)
    }

    fn write_stack(out_stack: *mut *const c_void, stack: Result<usize, AmmlStatus>) -> AmmlStatus {
        match stack {
            Ok(stack) => {
                // SAFETY: `out_stack` is non-null and the caller promises it is writable.
                unsafe { out_stack.write(std::ptr::with_exposed_provenance(stack)) };
                AmmlStatus::OK
            }
            Err(status) => status,
        }
    }

    // Runs `call` for the container view `view`: the screen adapter proves the view and its controller
    // (`NOT_AVAILABLE` without the adapter, `REFUSED` outside the dispatch of a container view), then the container
    // adapter does the call (`NOT_AVAILABLE` without its entries).
    fn container<T>(
        view: u64,
        call: impl FnOnce(&ContainerAdapter, usize, AmmlScreenKind) -> Result<T, AmmlStatus>,
    ) -> Result<T, AmmlStatus> {
        let screen = ScreenAdapter::get().ok_or(AmmlStatus::NOT_AVAILABLE)?;
        let (controller, kind) = screen.container_view(view)?;
        let adapter = ContainerAdapter::get().ok_or(AmmlStatus::NOT_AVAILABLE)?;
        call(adapter, controller, kind)
    }

    unsafe extern "C" fn draw_fill(view: u64, rect: AmmlRect, color: AmmlColor, alpha: f32) -> AmmlStatus {
        Self::draw(|adapter| adapter.fill(view, rect, color, alpha))
    }

    unsafe extern "C" fn draw_outline(
        view: u64,
        rect: AmmlRect,
        color: AmmlColor,
        alpha: f32,
        thickness: i32,
    ) -> AmmlStatus {
        Self::draw(|adapter| adapter.outline(view, rect, color, alpha, thickness))
    }

    unsafe extern "C" fn draw_image(
        view: u64,
        texture: AmmlTextureId,
        rect: AmmlRect,
        uv: AmmlUv,
        tint: AmmlColor,
        alpha: f32,
    ) -> AmmlStatus {
        Self::draw(|adapter| adapter.image(view, texture, rect, uv, tint, alpha))
    }

    unsafe extern "C" fn draw_item(
        module: AmmlStr,
        view: u64,
        draw: AmmlItemDraw,
        at: AmmlPoint,
        flags: AmmlItemDrawFlags,
        alpha: f32,
        z: i32,
    ) -> AmmlStatus {
        Boundary::guard(|| {
            // SAFETY: the caller promises `module` is valid for the call.
            let owner = match unsafe { ItemDrawAdapter::live_owner(module) } {
                Ok(owner) => owner,
                Err(status) => return status,
            };
            let drawn = ItemDrawAdapter::with(Err(AmmlStatus::NOT_AVAILABLE), |adapter| {
                adapter.draw(owner, view, draw, at, flags, alpha, z)
            });
            match drawn {
                Ok(()) => AmmlStatus::OK,
                Err(status) => status,
            }
        })
    }

    unsafe extern "C" fn text_width(view: u64, text: AmmlStr, out: *mut f32) -> AmmlStatus {
        Boundary::guard(|| {
            if out.is_null() {
                return AmmlStatus::INVALID_ARGUMENT;
            }
            // SAFETY: the caller promises `text` is valid for the call.
            let width =
                TextAdapter::with(Err(AmmlStatus::NOT_AVAILABLE), |adapter| unsafe { adapter.width(view, text) });
            match width {
                Ok(width) => {
                    // SAFETY: `out` is non-null and the caller promises it is writable.
                    unsafe { out.write(width) };
                    AmmlStatus::OK
                }
                Err(status) => status,
            }
        })
    }

    unsafe extern "C" fn draw_text(
        view: u64,
        text: AmmlStr,
        rect: AmmlRect,
        color: AmmlColor,
        alpha: f32,
        align: AmmlTextAlign,
    ) -> AmmlStatus {
        Boundary::guard(|| {
            // SAFETY: the caller promises `text` is valid for the call.
            let drawn = TextAdapter::with(Err(AmmlStatus::NOT_AVAILABLE), |adapter| unsafe {
                adapter.draw(view, text, rect, color, alpha, align)
            });
            match drawn {
                Ok(()) => AmmlStatus::OK,
                Err(status) => status,
            }
        })
    }

    unsafe extern "C" fn chat_text(field: AmmlChatField, out: *mut AmmlStr) -> AmmlStatus {
        Boundary::guard(|| {
            if out.is_null() {
                return AmmlStatus::INVALID_ARGUMENT;
            }
            match ChatAdapter::with(Err(AmmlStatus::NOT_AVAILABLE), |adapter| adapter.text(field)) {
                Ok(text) => {
                    // SAFETY: `out` is non-null and the caller promises it is writable.
                    unsafe { out.write(text) };
                    AmmlStatus::OK
                }
                Err(status) => status,
            }
        })
    }

    unsafe extern "C" fn chat_set_text(module: AmmlStr, field: AmmlChatField, text: AmmlStr) -> AmmlStatus {
        Boundary::guard(|| {
            if let Err(status) = Self::owner_of(module) {
                return status;
            }
            // The length is checked before any byte is read.
            if text.len > ChatAdapter::MAX_EDIT_BYTES {
                return AmmlStatus::REFUSED;
            }
            // SAFETY: the caller promises `text` is valid for the call.
            let text = match unsafe { text.as_str() } {
                Ok(text) => text,
                Err(status) => return status,
            };
            match ChatAdapter::with(Err(AmmlStatus::NOT_AVAILABLE), |adapter| adapter.set_text(field, text)) {
                Ok(()) => AmmlStatus::OK,
                Err(status) => status,
            }
        })
    }

    unsafe extern "C" fn chat_icons_active() -> u8 {
        Boundary::run(0, || u8::from(InlineIconAdapter::with(false, InlineIconAdapter::active)))
    }

    unsafe extern "C" fn icon_register(
        module: AmmlStr,
        image: *const AmmlImageView,
        advance: f32,
        out: *mut AmmlInlineIcon,
    ) -> AmmlStatus {
        Boundary::guard(|| {
            if out.is_null() {
                return AmmlStatus::INVALID_ARGUMENT;
            }
            let owner = match Self::owner_of(module) {
                Ok(owner) => owner,
                Err(status) => return status,
            };
            // SAFETY: the caller promises `image` is a valid view for the call.
            let registered = InlineIconAdapter::with(Err(AmmlStatus::NOT_AVAILABLE), |adapter| unsafe {
                adapter.register(owner, image, advance)
            });
            match registered {
                Ok(icon) => {
                    // SAFETY: `out` is non-null and the caller promises it is writable.
                    unsafe { out.write(icon) };
                    AmmlStatus::OK
                }
                Err(status) => status,
            }
        })
    }

    unsafe extern "C" fn icon_update(module: AmmlStr, icon: AmmlInlineIcon, image: *const AmmlImageView) -> AmmlStatus {
        Boundary::guard(|| {
            let owner = match Self::owner_of(module) {
                Ok(owner) => owner,
                Err(status) => return status,
            };
            // SAFETY: the caller promises `image` is a valid view for the call.
            let updated = InlineIconAdapter::with(Err(AmmlStatus::NOT_AVAILABLE), |adapter| unsafe {
                adapter.update(owner, icon, image)
            });
            match updated {
                Ok(()) => AmmlStatus::OK,
                Err(status) => status,
            }
        })
    }

    unsafe extern "C" fn icon_release(module: AmmlStr, icon: AmmlInlineIcon) -> AmmlStatus {
        Boundary::guard(|| {
            let owner = match Self::owner_of(module) {
                Ok(owner) => owner,
                Err(status) => return status,
            };
            match InlineIconAdapter::with(Err(AmmlStatus::NOT_AVAILABLE), |adapter| adapter.release(owner, icon)) {
                Ok(()) => AmmlStatus::OK,
                Err(status) => status,
            }
        })
    }

    // The index of the live mod `module`; an unknown one is `INVALID_ARGUMENT`, one that is not live `REFUSED`.
    fn owner_of(module: AmmlStr) -> Result<usize, AmmlStatus> {
        let state = LoaderState::get().ok_or(AmmlStatus::FAILED)?;
        // SAFETY: the caller of every table function promises `module` is valid for the call.
        let module = unsafe { module.as_str() }?;
        state.live_mod(module)
    }

    unsafe extern "C" fn gui_scale(view: u64, out: *mut f32) -> AmmlStatus {
        Boundary::guard(|| {
            if out.is_null() {
                return AmmlStatus::INVALID_ARGUMENT;
            }
            match UiDrawAdapter::with(Err(AmmlStatus::NOT_AVAILABLE), |adapter| adapter.gui_scale(view)) {
                Ok(scale) => {
                    // SAFETY: `out` is non-null and the caller promises it is writable.
                    unsafe { out.write(scale) };
                    AmmlStatus::OK
                }
                Err(status) => status,
            }
        })
    }

    unsafe extern "C" fn can_draw(view: u64) -> u8 {
        Boundary::run(0, || u8::from(UiDrawAdapter::with(false, |adapter| adapter.can_draw(view))))
    }

    // Runs a draw call of the adapter inside the boundary.
    fn draw(call: impl FnOnce(&UiDrawAdapter) -> Result<(), AmmlStatus>) -> AmmlStatus {
        Boundary::guard(|| match UiDrawAdapter::with(Err(AmmlStatus::NOT_AVAILABLE), call) {
            Ok(()) => AmmlStatus::OK,
            Err(status) => status,
        })
    }

    unsafe extern "C" fn texture_load(
        module: AmmlStr,
        source: AmmlTextureSource,
        path: AmmlStr,
        out: *mut AmmlTextureId,
    ) -> AmmlStatus {
        Boundary::guard(|| {
            if out.is_null() {
                return AmmlStatus::INVALID_ARGUMENT;
            }
            // SAFETY: the caller promises `path` is valid for the call.
            let path = match unsafe { path.as_str() } {
                Ok(path) => path,
                Err(status) => return status,
            };
            let result =
                Self::with_module(module, |adapter, owner, mod_dir| adapter.load(owner, source, path, mod_dir));
            Self::write_id(out, result)
        })
    }

    unsafe extern "C" fn texture_upload_rgba(
        module: AmmlStr,
        width: u32,
        height: u32,
        pixels: *const u8,
        len: usize,
        out: *mut AmmlTextureId,
    ) -> AmmlStatus {
        Boundary::guard(|| {
            if out.is_null() || (pixels.is_null() && len != 0) {
                return AmmlStatus::INVALID_ARGUMENT;
            }
            let bytes: &[u8] = if len == 0 {
                &[]
            } else {
                // SAFETY: the caller promises `pixels` is valid for `len` bytes for the call.
                unsafe { std::slice::from_raw_parts(pixels, len) }
            };
            let result = Self::with_module(module, |adapter, owner, _| adapter.upload(owner, width, height, bytes));
            Self::write_id(out, result)
        })
    }

    unsafe extern "C" fn texture_ready(texture: AmmlTextureId) -> u8 {
        Boundary::run(0, || u8::from(TextureAdapter::with(false, |adapter| adapter.ready(texture))))
    }

    unsafe extern "C" fn texture_unload(module: AmmlStr, texture: AmmlTextureId) -> AmmlStatus {
        Boundary::guard(|| match Self::with_module(module, |adapter, owner, _| adapter.unload(owner, texture)) {
            Ok(()) => AmmlStatus::OK,
            Err(status) => status,
        })
    }

    fn write_id(out: *mut AmmlTextureId, result: Result<AmmlTextureId, AmmlStatus>) -> AmmlStatus {
        match result {
            Ok(id) => {
                // SAFETY: `out` is non-null and the caller promises it is writable.
                unsafe { out.write(id) };
                AmmlStatus::OK
            }
            Err(status) => status,
        }
    }

    // Checks the module (an unknown one is `INVALID_ARGUMENT`, one that is not live `REFUSED`), gives the index and
    // the mod folder of the mod to `call`, and runs it on the texture adapter (`NOT_AVAILABLE` without one).
    fn with_module<T>(
        module: AmmlStr,
        call: impl FnOnce(&TextureAdapter, usize, &str) -> Result<T, AmmlStatus>,
    ) -> Result<T, AmmlStatus> {
        let state = LoaderState::get().ok_or(AmmlStatus::FAILED)?;
        // SAFETY: the caller promises `module` is valid for the call.
        let module = unsafe { module.as_str() }?;
        let owner = state.live_mod(module)?;
        // A mod without a folder has no mod files; a vanilla path and an upload need none.
        let mod_dir = state.dir(module, AmmlDirKind::MOD).ok().and_then(|dir| {
            // SAFETY: the loader keeps the folder text for the process.
            unsafe { dir.as_str() }.ok()
        });
        TextureAdapter::with(Err(AmmlStatus::NOT_AVAILABLE), |adapter| call(adapter, owner, mod_dir.unwrap_or("")))
    }
}
