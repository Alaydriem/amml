use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, Ordering};

use amml_abi::{AmmlChatField, AmmlEvent, AmmlPhase, AmmlStatus, AmmlStr, ChatMessageAdded};
use amml_data::{AdapterCatalog, Availability, NeedKind};
use amml_msvc::MsvcHeap;

use crate::{
    ChatDetours, ChatEvent, ChatScope, ClientAdapter, EventHub, GuiMessageLayout, InlineIconAdapter, LoaderState,
    ScreenAdapter, ThreadNote,
};

static ADAPTER: OnceLock<ChatAdapter> = OnceLock::new();
static STARTED: AtomicBool = AtomicBool::new(false);

/// The `ui.chat_message` event: a chat line the game just built, raised before the game adds it to the chat list, and
/// the reads and edits of its strings.
///
/// The constructor of the line is hooked by `ChatDetours`, on the entry of the argument list the build has
/// (`GuiMessage.ctorShape`). The event runs inside a `ChatScope`: the `ui` table reads (`chat_text`) answer for the line as
/// the game built it, and the edits (`chat_set_text`) are kept until the last listener returned, then written to the
/// game on the game heap. A line whose layout the loader does not know (`GuiMessage.size` that no mirror has) is not
/// served.
pub struct ChatAdapter {
    state: &'static LoaderState,
    layout: GuiMessageLayout,
    thread: ThreadNote,
    apply_logged: AtomicBool,
}

impl ChatAdapter {
    pub const CHAT_MESSAGE: &'static str = ChatMessageAdded::ID;
    /// The most bytes of text one `chat_set_text` takes.
    pub const MAX_EDIT_BYTES: usize = 4096;
    // Loader hooks run before every mod hook on the same function.
    const PRIORITY: i32 = i32::MAX;

    /// Hooks the constructor of the shape of the build when the catalog row resolved and the layout fits, offers
    /// `ui.chat_message` when the hook is in, and logs one line. Works once per process: a second call hooks nothing and
    /// gives the catalog row.
    pub fn install(state: &'static LoaderState) -> Vec<Availability> {
        let resolved = |name: &str| state.table().get(&amml_data::EntryId(name.to_string())).is_some();
        let mut rows: Vec<Availability> =
            AdapterCatalog::availability(&resolved).into_iter().filter(|row| row.id == Self::CHAT_MESSAGE).collect();
        if STARTED.swap(true, Ordering::AcqRel) {
            curia::warn!("the chat adapter is already installed");
            return rows;
        }
        let layout = rows.iter().any(|row| row.shape.is_some()).then(|| GuiMessageLayout::from_state(state)).flatten();
        let hooked = layout.and_then(|layout| Self::hook(state, &rows, layout));
        if let (Some(layout), true) = (layout, hooked.is_some()) {
            let adapter = Self { state, layout, thread: ThreadNote::new(), apply_logged: AtomicBool::new(false) };
            if ADAPTER.set(adapter).is_err() {
                curia::warn!("the chat adapter is already installed");
                return rows;
            }
        }
        for row in &mut rows {
            if row.shape.is_some() && hooked.is_none() {
                row.shape = None;
                row.missing = vec!["a hook or a layout"];
            }
            match (row.shape, row.kind) {
                (Some(shape), NeedKind::Event) => {
                    state.events().offer(row.id, row.version);
                    curia::info!("event offered", { event: row.id, shape: shape });
                }
                _ => curia::info!("event not available", { event: row.id, missing: row.missing.join(", ") }),
            }
        }
        rows
    }

    // Installs the constructor hook of the shape the build has. The catalog shape of the row must be the shape the
    // value of `GuiMessage.ctorShape` names, so a build where `GuiMessage.ctor` still resolves with another argument
    // list never gets the 26.6x detour.
    fn hook(state: &'static LoaderState, rows: &[Availability], layout: GuiMessageLayout) -> Option<()> {
        let value = state.resolve("GuiMessage.ctorShape").ok()?;
        let row = rows.iter().find(|row| row.shape.is_some())?;
        if ChatDetours::catalog_shape_for(value) != row.shape {
            curia::warn!("the chat constructor shape does not match the catalog row", {
                shape: value,
                row: row.shape.unwrap_or("none")
            });
            return None;
        }
        let hook = ChatDetours::hook_for(value)?;
        let installed = hook.install(state, Self::PRIORITY);
        if installed {
            curia::info!("chat adapter", { shape: value, entry: hook.entry, size: layout.size });
        }
        installed.then_some(())
    }

    /// The installed adapter; `None` before `install`, and when chat lines are not served.
    pub fn get() -> Option<&'static ChatAdapter> {
        ADAPTER.get()
    }

    /// Runs `body` on the installed adapter; `fallback` when there is none.
    pub(crate) fn with<R>(fallback: R, body: impl FnOnce(&ChatAdapter) -> R) -> R {
        match ADAPTER.get() {
            Some(adapter) => body(adapter),
            None => fallback,
        }
    }

    fn heap() -> MsvcHeap {
        MsvcHeap::new(ClientAdapter::heap_new, ClientAdapter::heap_delete)
    }

    /// The constructor returned the line at `message`: raises `ChatMessageAdded` on this thread while a mod listens, then
    /// writes the edits the listeners asked for.
    pub(crate) fn added(&self, message: usize) {
        let hub = self.state.events();
        if !hub.has_listeners(ChatMessageAdded::ID) {
            return;
        }
        let context = ScreenAdapter::context_of(self.state, None);
        if self.thread.changed(context.thread) {
            curia::info!("chat message thread", { game: context.thread == amml_abi::AmmlThread::GAME });
        }
        InlineIconAdapter::with((), |adapter| adapter.refresh_font(context.thread == amml_abi::AmmlThread::GAME));
        // SAFETY: `message` is the line the constructor just built on this thread; the game adds it to its list only
        // after the constructor returns.
        let Some(kind) = (unsafe { self.layout.kind_of(message) }) else { return };
        let scope = ChatScope::enter(ChatEvent::new(message, self.layout, Self::heap()));
        let mut event =
            ChatMessageAdded { header: EventHub::header::<ChatMessageAdded>(AmmlPhase::AFTER), kind, _reserved: 0 };
        hub.raise_fixed(&mut event, &context);
        if let Some(done) = scope.finish() {
            self.apply(&done);
        }
    }

    // Writes the edits of `event` to the game: the full string, then the filtered one.
    fn apply(&self, event: &ChatEvent) {
        let (text, filtered) = event.edits();
        let heap = event.heap();
        for (field, edit) in [(AmmlChatField::TEXT, text), (AmmlChatField::FILTERED_TEXT, filtered)] {
            let Some(edit) = edit else { continue };
            // SAFETY: the line is the one of the event, alive until the game adds it; its strings were built by the
            // game allocator, which `heap` calls; the listeners are done, so nothing else uses it.
            let written = unsafe { event.layout().replace(event.message(), field, &heap, edit) };
            if !written && !self.apply_logged.swap(true, Ordering::Relaxed) {
                curia::warn!("a chat edit was not written", { field: field.0 });
            }
        }
    }

    /// The bytes of `field` of the line of the running event of this thread. `REFUSED` when no event runs here.
    pub(crate) fn text(&self, field: AmmlChatField) -> Result<AmmlStr, AmmlStatus> {
        let layout = self.layout;
        ChatScope::with(|event| {
            // SAFETY: the scope holds the line the constructor just built; it lives while the listeners run.
            unsafe { layout.text_of(event.message(), field) }
        })
        .ok_or(AmmlStatus::REFUSED)?
        .map(|(address, length)| AmmlStr { ptr: std::ptr::with_exposed_provenance(address), len: length })
    }

    /// Keeps `text` as the new value of `field` for the running event of this thread. `INVALID_ARGUMENT` for a field the
    /// loader does not know or that is read only; `REFUSED` for a text over `MAX_EDIT_BYTES` and when no event runs
    /// here; `NOT_AVAILABLE` for the filtered text of a line that has none.
    pub(crate) fn set_text(&self, field: AmmlChatField, text: &str) -> Result<(), AmmlStatus> {
        if field != AmmlChatField::TEXT && field != AmmlChatField::FILTERED_TEXT {
            return Err(AmmlStatus::INVALID_ARGUMENT);
        }
        if text.len() > Self::MAX_EDIT_BYTES {
            return Err(AmmlStatus::REFUSED);
        }
        let layout = self.layout;
        ChatScope::with(|event| {
            // SAFETY: the scope holds the line the constructor just built; it lives while the listeners run.
            if field == AmmlChatField::FILTERED_TEXT && !unsafe { layout.has_filtered(event.message()) } {
                return Err(AmmlStatus::NOT_AVAILABLE);
            }
            event.set(field, text);
            Ok(())
        })
        .ok_or(AmmlStatus::REFUSED)?
    }
}
