use amml_abi::{AmmlChatField, ChatMessageAdded};

use crate::lent_chat::LentChat;
use crate::{ApiError, ChatKind};

/// Typed reads and the edits of a `ChatMessageAdded` (`ui.chat_message`): the game built one chat line and has not added it
/// to the chat list yet.
///
/// A read gives the line as the game built it: an edit made by a listener, the listener itself included, is not seen by
/// `text`. The edits are written to the game after the last listener returned, on the game heap, and the latest edit of
/// a field wins. The reads and the edits work only on the event of a running listener call, on its thread: a copy of the
/// event, or one kept past its call, reads `None` and refuses every edit.
pub trait ChatMessageAddedExt {
    /// The type of the line, as the game gives it.
    fn kind(&self) -> ChatKind;

    /// The name of the sender; `Some("")` for a line with no sender. `None` outside the call, and for text that is not
    /// UTF-8.
    fn username(&self) -> Option<&str>;

    /// The Xbox user id of the author; `Some("")` when the line has none. `None` as for `username`.
    fn xuid(&self) -> Option<&str>;

    /// The whole line as the game shows it: `<name> message`, or the message alone when there is no name. `None` as for
    /// `username`.
    fn text(&self) -> Option<&str>;

    /// The whole line the game shows when the player filters chat; `None` when the line has none, and as for
    /// `username`.
    fn filtered_text(&self) -> Option<&str>;

    /// Replaces the whole line. `Refused` for a text of more than 4096 bytes and outside the call.
    fn set_text(&self, text: &str) -> Result<(), ApiError>;

    /// Replaces the filtered line. `NotAvailable` when the line has none. `Refused` as for `set_text`.
    fn set_filtered_text(&self, text: &str) -> Result<(), ApiError>;

    /// Whether the default font draws inline icons now (`Chat::register_inline_icon`): the loader serves
    /// `Area::UI_INLINE_ICONS`, the font is a bitmap font and the page of the icons is uploaded and ready. A mod that writes icon characters into a line checks it
    /// first.
    fn inline_icons(&self) -> bool;
}

impl ChatMessageAddedExt for ChatMessageAdded {
    fn kind(&self) -> ChatKind {
        ChatKind(self.kind)
    }

    fn username(&self) -> Option<&str> {
        LentChat::read(self, AmmlChatField::USERNAME)
    }

    fn xuid(&self) -> Option<&str> {
        LentChat::read(self, AmmlChatField::XUID)
    }

    fn text(&self) -> Option<&str> {
        LentChat::read(self, AmmlChatField::TEXT)
    }

    fn filtered_text(&self) -> Option<&str> {
        LentChat::read(self, AmmlChatField::FILTERED_TEXT)
    }

    fn set_text(&self, text: &str) -> Result<(), ApiError> {
        LentChat::write(self, AmmlChatField::TEXT, text)
    }

    fn set_filtered_text(&self, text: &str) -> Result<(), ApiError> {
        LentChat::write(self, AmmlChatField::FILTERED_TEXT, text)
    }

    fn inline_icons(&self) -> bool {
        let Some(lent) = LentChat::find(std::ptr::from_ref(self)) else { return false };
        let Some(active) = lent.table.chat_icons_active else { return false };
        // SAFETY: the loader function takes no arguments.
        #[allow(unsafe_code)]
        let active = unsafe { active() };
        active != 0
    }
}
