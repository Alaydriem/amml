use std::marker::PhantomData;

use amml_abi::{AmmlStr, AmmlTextAlign, AmmlUiTable};

use crate::{ApiError, Color, ItemDrawFlags, ItemDrawHandle, Point, Rect, TextAlign, TextureId, Tint, Uv, ViewId};

/// The draw calls of a `ScreenRender` listener (`Area::UI_DRAW`), as `ScreenRenderExt::draw` gives them, for the call
/// of the listener that got the event. A fill or an outline is drawn at once; an image is queued and drawn after the
/// last listener, or before the next fill or outline, so the order of the calls is the order on the screen.
/// Positions and sizes are in GUI units. The loader refuses a call made after the listener returned, or on another
/// thread.
pub struct UiDraw<'f> {
    table: AmmlUiTable,
    view: ViewId,
    _event: PhantomData<&'f ()>,
}

impl<'f> UiDraw<'f> {
    pub(crate) fn new(table: AmmlUiTable, view: ViewId) -> Self {
        Self { table, view, _event: PhantomData }
    }

    /// Fills `rect` with `color` at `alpha`.
    pub fn fill_rect(&self, rect: Rect, color: Color, alpha: f32) -> Result<(), ApiError> {
        let fill = self.table.draw_fill.ok_or(ApiError::NotAvailable)?;
        // SAFETY: the loader checks the view id before it draws.
        #[allow(unsafe_code)]
        let status = unsafe { fill(self.view.0, rect.into(), color.into(), alpha) };
        ApiError::check(status)
    }

    /// Draws the border of `rect`, `thickness` GUI units wide (at least 1), in `color` at `alpha`.
    pub fn outline_rect(&self, rect: Rect, color: Color, alpha: f32, thickness: i32) -> Result<(), ApiError> {
        let outline = self.table.draw_outline.ok_or(ApiError::NotAvailable)?;
        // SAFETY: as for `fill_rect`.
        #[allow(unsafe_code)]
        let status = unsafe { outline(self.view.0, rect.into(), color.into(), alpha, thickness) };
        ApiError::check(status)
    }

    /// Draws the part `uv` of `texture` into `rect`, multiplied by `tint`. A texture that is not ready is skipped and
    /// the call gives `Ok`. `Refused` while the game left images queued (`can_draw` is false). A stale texture gives
    /// `Stale`.
    pub fn image(&self, texture: TextureId, rect: Rect, uv: Uv, tint: Tint) -> Result<(), ApiError> {
        let image = self.table.draw_image.ok_or(ApiError::NotAvailable)?;
        // SAFETY: as for `fill_rect`.
        #[allow(unsafe_code)]
        let status =
            unsafe { image(self.view.0, texture.into(), rect.into(), uv.into(), tint.color.into(), tint.alpha) };
        ApiError::check(status)
    }

    /// Draws the item stack of `handle` with its top left corner at `at`, 16 GUI units wide, at once, as the game
    /// draws an item in a slot. `alpha` is the transparency and `z` the layer; `flags` can add the glint. The loader
    /// draws the images queued before it first, so the order of the calls is the order on the screen. A handle of a
    /// world that was left gives `Stale`; a handle of another mod gives `Refused`; flags the loader does not know give
    /// `Loader(INVALID_ARGUMENT)`. Needs `Area::UI_DRAW` and, for the
    /// handle, `Area::INVENTORY_READ.at(4)`.
    pub fn item(
        &self,
        handle: &ItemDrawHandle,
        at: Point,
        flags: ItemDrawFlags,
        alpha: f32,
        z: i32,
    ) -> Result<(), ApiError> {
        let draw = self.table.draw_item.ok_or(ApiError::NotAvailable)?;
        let (module, id) = handle.raw();
        // SAFETY: as for `fill_rect`; the module id lives for the call.
        #[allow(unsafe_code)]
        let status = unsafe { draw(AmmlStr::from_str(module), self.view.0, id, at.into(), flags.into(), alpha, z) };
        ApiError::check(status)
    }

    /// The width of `text` in GUI units, in the default font of the game. `None` when the loader cannot measure it:
    /// no `Area::UI_TEXT`, a text of more than 256 bytes, or a font the loader cannot reach.
    pub fn text_width(&self, text: &str) -> Option<f32> {
        let width = self.table.text_width?;
        let mut out = 0.0f32;
        // SAFETY: as for `fill_rect`; the text and `out` live for the call.
        #[allow(unsafe_code)]
        let status = unsafe { width(self.view.0, AmmlStr::from_str(text), &mut out) };
        status.is_ok().then_some(out)
    }

    /// Queues `text` in the default font of the game, in `color` at `alpha`. The text starts at the top of `rect`;
    /// `align` places it along the width of `rect`. The text is one line high; `rect.height` is not read. The
    /// loader copies the text and draws it after the last listener, over everything the listeners drew. A text of
    /// more than 256 bytes, or a text after 1024 texts in the same dispatch, gives `Refused`. Both text calls need
    /// the game thread. Needs `Area::UI_TEXT`.
    pub fn text(&self, text: &str, rect: Rect, color: Color, alpha: f32, align: TextAlign) -> Result<(), ApiError> {
        let draw = self.table.draw_text.ok_or(ApiError::NotAvailable)?;
        // SAFETY: as for `fill_rect`; the text lives for the call.
        #[allow(unsafe_code)]
        let status = unsafe {
            draw(self.view.0, AmmlStr::from_str(text), rect.into(), color.into(), alpha, AmmlTextAlign::from(align))
        };
        ApiError::check(status)
    }

    /// The GUI scale: screen pixels per GUI unit. `None` when the loader cannot read it.
    pub fn gui_scale(&self) -> Option<f32> {
        let scale = self.table.gui_scale?;
        let mut out = 0.0f32;
        // SAFETY: as for `fill_rect`; `out` is a live local.
        #[allow(unsafe_code)]
        let status = unsafe { scale(self.view.0, &mut out) };
        status.is_ok().then_some(out)
    }

    /// Whether `image` may queue an image now: false while the game itself left images queued in this draw.
    pub fn can_draw(&self) -> bool {
        let Some(can_draw) = self.table.can_draw else { return false };
        // SAFETY: as for `fill_rect`.
        #[allow(unsafe_code)]
        let can = unsafe { can_draw(self.view.0) };
        can != 0
    }
}
