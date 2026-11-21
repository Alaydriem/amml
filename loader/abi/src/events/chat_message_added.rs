use crate::{AmmlEvent, AmmlEventHeader};

/// The game built one chat line and has not added it to the chat list yet (`ui.chat_message`, After the line is
/// built and before the game adds it, not cancellable).
///
/// `kind` is the game's message type, as it is. The strings of the line are read with `ui.chat_text` and written with
/// `ui.chat_set_text`, and only while the event runs: a read gives the line as the game built it, and a write is
/// applied after the last listener, so a listener never sees the edit of another. Raised on the thread that builds the
/// line.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ChatMessageAdded {
    pub header: AmmlEventHeader,
    pub kind: u32,
    pub _reserved: u32,
}

impl AmmlEvent for ChatMessageAdded {
    const ID: &'static str = "ui.chat_message";
}

const _: () = assert!(core::mem::size_of::<ChatMessageAdded>() == 24);
