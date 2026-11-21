use amml_abi::{AmmlInlineIcon, AmmlStr, AmmlUiTable};

use crate::{ApiError, Chat, ImageView};

/// An inline chat icon (`Chat::register_inline_icon`). Dropping it frees the icon: the loader blanks its character at its
/// next UI draw, so an icon may be dropped on any thread. `release` does the same on purpose.
///
/// An icon of a world that was left is `Stale` for `update`; dropping it is still allowed.
pub struct InlineIcon {
    table: AmmlUiTable,
    module: &'static str,
    icon: AmmlInlineIcon,
    released: bool,
}

impl InlineIcon {
    pub(crate) fn new(table: AmmlUiTable, module: &'static str, icon: AmmlInlineIcon) -> Self {
        Self { table, module, icon, released: false }
    }

    /// The character that draws the icon: a private-use character in `U+E110..=U+E1FF`.
    pub fn glyph(&self) -> char {
        char::from_u32(self.icon.glyph).unwrap_or(char::REPLACEMENT_CHARACTER)
    }

    /// Replaces the image of the icon; the advance stays. The same limits as `Chat::register_inline_icon`. `Stale` for an
    /// icon of a world that was left.
    pub fn update(&self, image: &ImageView<'_>) -> Result<(), ApiError> {
        let update = self.table.icon_update.ok_or(ApiError::NotAvailable)?;
        let raw = Chat::raw(image);
        // SAFETY: the module id, the icon and the image view (its pixels live as long as `image`) live for the call.
        #[allow(unsafe_code)]
        let status = unsafe { update(AmmlStr::from_str(self.module), self.icon, &raw) };
        ApiError::check(status)
    }

    /// Frees the icon now. Its character is blank in the chat lines that hold it.
    pub fn release(mut self) {
        self.free();
    }

    fn free(&mut self) {
        if std::mem::replace(&mut self.released, true) {
            return;
        }
        let Some(release) = self.table.icon_release else { return };
        // SAFETY: the loader checks the icon at once and only queues the free of its character; the module id lives for
        // the call.
        #[allow(unsafe_code)]
        let _ = unsafe { release(AmmlStr::from_str(self.module), self.icon) };
    }
}

impl std::fmt::Debug for InlineIcon {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("InlineIcon")
            .field("module", &self.module)
            .field("glyph", &self.icon.glyph)
            .field("generation", &self.icon.generation)
            .finish()
    }
}

impl Drop for InlineIcon {
    fn drop(&mut self) {
        self.free();
    }
}
