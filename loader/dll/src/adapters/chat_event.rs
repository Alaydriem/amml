use amml_abi::AmmlChatField;
use amml_msvc::MsvcHeap;

use crate::GuiMessageLayout;

/// The state of one running `ChatMessageAdded`: the line it is about and the edits the listeners asked for. It lives
/// in a `ChatScope` and ends with it.
///
/// An edit is kept as text until the last listener returned; the adapter then writes it to the game. The latest edit
/// of a field wins.
pub struct ChatEvent {
    message: usize,
    layout: GuiMessageLayout,
    heap: MsvcHeap,
    text: Option<String>,
    filtered_text: Option<String>,
}

impl ChatEvent {
    pub fn new(message: usize, layout: GuiMessageLayout, heap: MsvcHeap) -> Self {
        Self { message, layout, heap, text: None, filtered_text: None }
    }

    /// The address of the `GuiMessage` the event is about.
    pub fn message(&self) -> usize {
        self.message
    }

    pub fn layout(&self) -> GuiMessageLayout {
        self.layout
    }

    pub fn heap(&self) -> MsvcHeap {
        self.heap
    }

    /// Keeps `text` as the new value of `field`. False for a field that is not `TEXT` or `FILTERED_TEXT`.
    pub fn set(&mut self, field: AmmlChatField, text: &str) -> bool {
        match field {
            AmmlChatField::TEXT => self.text = Some(text.to_string()),
            AmmlChatField::FILTERED_TEXT => self.filtered_text = Some(text.to_string()),
            _ => return false,
        }
        true
    }

    /// The edits asked for, as `(text, filtered text)`.
    pub fn edits(&self) -> (Option<&str>, Option<&str>) {
        (self.text.as_deref(), self.filtered_text.as_deref())
    }
}
