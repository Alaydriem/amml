// The line of a running `ChatMessageAdded` and the edits asked for.
pub(crate) struct FakeChatEvent {
    pub(crate) username: Box<str>,
    pub(crate) xuid: Box<str>,
    pub(crate) text: Box<str>,
    pub(crate) filtered_text: Option<Box<str>>,
    pub(crate) new_text: Option<String>,
    pub(crate) new_filtered_text: Option<String>,
}
