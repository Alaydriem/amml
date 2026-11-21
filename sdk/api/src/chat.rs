use amml_abi::{AmmlImageView, AmmlInlineIcon, AmmlStr, AmmlUiTable};

use crate::{ApiError, ImageView, InlineIcon};

/// The chat of the game (`Area::UI_INLINE_ICONS`): inline icons, small images that a chat line shows as one character of
/// the default font. The text of a line is read and edited with `ChatMessageAddedExt`.
///
/// An icon is a private-use character (`InlineIcon::glyph`): a mod writes it into the text of a line with
/// `ChatMessageAddedExt::set_text`, and the font draws the image in its place with the advance the mod gave.
#[derive(Clone, Copy)]
pub struct Chat {
    table: Option<AmmlUiTable>,
    module: &'static str,
}

impl Chat {
    pub(crate) fn new(table: Option<AmmlUiTable>, module: &'static str) -> Self {
        Self { table, module }
    }

    /// Registers `image` as an inline icon and gives it a character. The image is at most 16 x 16 pixels of four bytes each
    /// (red, green, blue, alpha, format `PixelFormat::Rgba8`), drawn from the top left of a 16 x 16 cell, and is copied
    /// before the call returns. `advance` is the width of the icon in GUI units, above 0 and at most 16.
    ///
    /// Call it on the game thread (`Refused` elsewhere). `NotAvailable` when the area is not served or the default font is
    /// not a bitmap font. `Refused` for an image or an advance over the limits, and when all 240 icons are in use. A
    /// released icon is blanked, and its character is given to another icon only when no unused character is left.
    ///
    /// The icons are freed at a world leave: an icon of a left world gives `Stale` and is registered again after the next
    /// join. The icons of a mod are freed when it is disabled.
    pub fn register_inline_icon(&self, image: &ImageView<'_>, advance: f32) -> Result<InlineIcon, ApiError> {
        let table = self.table.ok_or(ApiError::NotAvailable)?;
        let register = table.icon_register.ok_or(ApiError::NotAvailable)?;
        let raw = Self::raw(image);
        let mut out = AmmlInlineIcon::default();
        // SAFETY: the module id, the image view (its pixels live as long as `image`) and the output live for the call.
        #[allow(unsafe_code)]
        let status = unsafe { register(AmmlStr::from_str(self.module), &raw, advance, &mut out) };
        ApiError::check(status).map(|()| InlineIcon::new(table, self.module, out))
    }

    /// The loader view of `image`.
    pub(crate) fn raw(image: &ImageView<'_>) -> AmmlImageView {
        AmmlImageView {
            width: image.width,
            height: image.height,
            format: u32::from(image.format),
            _reserved: 0,
            pixels: image.pixels.as_ptr(),
            len: image.pixels.len(),
        }
    }
}
