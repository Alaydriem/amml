use std::ffi::c_void;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Mutex, MutexGuard, OnceLock};

use amml_game::{
    BaseActorRenderContext, ContainerScreenController, GameEntries, GameHeap, ItemStackBaseRef, OwnedItemStack,
    RectangleAreaLayout, ScreenView, StopLatch, TextMeasureDataLayout, UiColorLayout, UiRenderContext, UiScene,
};
use amml_msvc::MsvcString;
use amml_sdk::{ModRuntime, PanicText};
use windows_sys::Win32::System::Threading::GetCurrentThreadId;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_SHIFT};

use crate::{FontScope, NbtDump, PeekHover, PeekReport, Throttle};

/// What the test mod proves for the shulker box preview: the hover and unhover of container slots, the controller
/// destructor, a drawn test cell with the first stored item in the UI render call, and the wheel steps of the UI
/// scenes. It only reads, and draws in the UI render call.
pub struct PeekProbe {
    entries: OnceLock<GameEntries>,
    latch: StopLatch,
    hover: Mutex<Option<PeekHover>>,
    frames: Throttle,
    awaiting_frame: AtomicBool,
    built_logged: AtomicBool,
    render_thread: AtomicU32,
}

impl PeekProbe {
    /// Frames between two screen lines.
    const SCREEN_EVERY: u32 = 60;

    /// The side of the test cell, in GUI units.
    const CELL: f32 = 16.0;

    /// The full name part that marks a shulker box item.
    const SHULKER: &'static str = "shulker_box";

    /// The full name part that marks a bundle item.
    const BUNDLE: &'static str = "bundle";

    /// A probe with no entries; it reads nothing until `start`.
    pub const fn new() -> Self {
        Self {
            entries: OnceLock::new(),
            latch: StopLatch::new(),
            hover: Mutex::new(None),
            frames: Throttle::new(Self::SCREEN_EVERY),
            awaiting_frame: AtomicBool::new(false),
            built_logged: AtomicBool::new(false),
            render_thread: AtomicU32::new(0),
        }
    }

    /// Keeps the resolved entries and lowers the latch.
    pub fn start(&self, entries: GameEntries) {
        let _ = self.entries.set(entries);
        self.latch.lower();
    }

    /// Raises the latch: no later hook call reads or draws.
    pub fn stop(&self) {
        self.latch.raise();
    }

    fn log(message: &str) {
        if let Some(runtime) = ModRuntime::get() {
            runtime.logger().info(message);
        }
    }

    #[allow(unsafe_code)]
    fn thread_id() -> u32 {
        // SAFETY: GetCurrentThreadId has no preconditions.
        unsafe { GetCurrentThreadId() }
    }

    // Whether Shift is down now (the high bit of the key state).
    #[allow(unsafe_code)]
    fn shift_down() -> bool {
        // SAFETY: GetAsyncKeyState has no preconditions.
        let state = unsafe { GetAsyncKeyState(i32::from(VK_SHIFT)) };
        state < 0
    }

    fn hover(&self) -> MutexGuard<'_, Option<PeekHover>> {
        self.hover.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    // Runs `read`; a panic is logged once and raises the latch, so the probe stops and the game goes on.
    fn guarded(&self, read: impl FnOnce()) {
        if self.latch.raised() {
            return;
        }
        if let Err(payload) = catch_unwind(AssertUnwindSafe(read)) {
            self.latch.raise();
            if let Some(runtime) = ModRuntime::get() {
                runtime
                    .logger()
                    .error(&format!("peek probe panicked: {}; the probe stops", PanicText::of(payload.as_ref())));
            }
        }
    }

    // The stack the controller shows in the hovered slot. The collection name is rebuilt as a game string for the
    // call and released after it.
    #[allow(unsafe_code)]
    fn visual_stack<'a>(container: &ContainerScreenController<'a>, hover: &PeekHover) -> Option<ItemStackBaseRef<'a>> {
        let heap = GameHeap::msvc()?;
        // SAFETY: the heap is the game's operator new and delete.
        let mut name = unsafe { MsvcString::new_in(&heap, &hover.collection) }.ok()?;
        let stack = container.visual_item(&name, hover.slot);
        // SAFETY: the string came from this heap, and the call that read it has returned.
        unsafe { name.release(&heap) };
        stack
    }

    // Logs the `Items` list of a shulker box stack: its length and the slot of each element.
    fn log_items(stack: &ItemStackBaseRef<'_>) {
        let list = stack.user_data().and_then(|tag| tag.list("Items"));
        let slots: Option<Vec<Option<i8>>> = list
            .map(|list| (0..list.len()).map(|index| list.compound(index).and_then(|tag| tag.byte("Slot"))).collect());
        Self::log(&PeekReport::items_line(slots.as_deref()));
    }

    // The name of the collection argument of a hover call.
    #[allow(unsafe_code)]
    fn collection_name(collection: *const MsvcString) -> Option<String> {
        // SAFETY: `collection` is the std::string reference argument of the running hover call.
        let collection = unsafe { collection.as_ref() }?;
        collection.to_str().ok().map(str::to_owned)
    }

    /// After `_onContainerSlotHovered`: logs the slot, the stack the controller shows there and, for a shulker box,
    /// the `Items` list; keeps the slot as the hovered one.
    #[allow(unsafe_code)]
    pub fn on_hover(&self, controller: usize, collection: *const MsvcString, slot: i32) {
        self.guarded(|| {
            let Some(entries) = self.entries.get() else {
                return;
            };
            let Some(name) = Self::collection_name(collection) else {
                return;
            };
            let hover = PeekHover { controller, collection: name, slot };
            if self.hover().as_ref() == Some(&hover) {
                return;
            }
            // SAFETY: `controller` is the `this` of the running hover call.
            let Some(container) = (unsafe { ContainerScreenController::from_raw(controller as *mut c_void, entries) })
            else {
                return;
            };
            let stack = Self::visual_stack(&container, &hover);
            let type_name = stack
                .as_ref()
                .and_then(|stack| if stack.is_empty() { Some("empty".to_string()) } else { stack.type_name() });
            let count = stack.as_ref().and_then(|stack| stack.count());
            Self::log(&PeekReport::hover_line(controller, &hover.collection, slot, type_name.as_deref(), count));
            if let Some(stack) = stack.as_ref()
                && type_name.as_deref().is_some_and(|name| name.contains(Self::SHULKER))
            {
                Self::log_items(stack);
            }
            if let Some(stack) = stack.as_ref()
                && type_name.as_deref().is_some_and(|name| name.contains(Self::SHULKER) || name.contains(Self::BUNDLE))
            {
                let text = stack.user_data().map(|tag| NbtDump::of(&tag));
                Self::log(&format!(
                    "peek nbt: {} = {}",
                    type_name.as_deref().unwrap_or("?"),
                    text.as_deref().unwrap_or("no user data")
                ));
            }
            *self.hover() = Some(hover);
            self.built_logged.store(false, Ordering::Relaxed);
        });
    }

    /// After `_onContainerSlotUnhovered`: logs the slot and forgets it when it is the hovered one.
    pub fn on_unhover(&self, controller: usize, collection: *const MsvcString, slot: i32) {
        self.guarded(|| {
            let Some(name) = Self::collection_name(collection) else {
                return;
            };
            let mut hover = self.hover();
            let tracked = hover
                .as_ref()
                .is_some_and(|held| held.controller == controller && held.collection == name && held.slot == slot);
            if tracked {
                *hover = None;
            }
            drop(hover);
            Self::log(&PeekReport::unhover_line(controller, &name, slot, tracked));
        });
    }

    /// Before `ScreenController::~ScreenController`: logs the controller, forgets the hover when it belongs to it and
    /// marks that the next render must draw nothing for it.
    pub fn on_controller_dtor(&self, controller: usize) {
        self.guarded(|| {
            let mut hover = self.hover();
            let tracked = hover.as_ref().is_some_and(|held| held.controller == controller);
            if tracked {
                *hover = None;
            }
            drop(hover);
            if tracked {
                self.awaiting_frame.store(true, Ordering::Relaxed);
            }
            Self::log(&PeekReport::dtor_line(controller, tracked));
        });
    }

    /// After `ScreenView::render`: on a container screen logs the GUI size and the pointer, and with a shulker box
    /// hovered draws the test cell with its first stored item and the count.
    #[allow(unsafe_code)]
    pub fn on_render(&self, view: usize, ctx: usize) {
        self.render_thread.store(Self::thread_id(), Ordering::Relaxed);
        self.guarded(|| {
            let Some(entries) = self.entries.get() else {
                return;
            };
            let after_dtor = self.awaiting_frame.swap(false, Ordering::Relaxed);
            // SAFETY: `view` is the `this` of the running render call.
            let Some(view) = (unsafe { ScreenView::from_raw(view as *mut c_void, entries) }) else {
                return;
            };
            // SAFETY: `ctx` is the UIRenderContext argument of the running render call.
            let Some(ui) = (unsafe { UiRenderContext::from_raw(ctx as *mut c_void, entries) }) else {
                return;
            };
            let container = view.controller().and_then(|controller| controller.container());
            if container.is_some() && self.frames.hit() {
                Self::log(&PeekReport::screen_line(view.size(), view.pointer()));
            }
            let drew = container.as_ref().and_then(|container| self.draw(entries, &view, &ui, container)).is_some();
            if after_dtor {
                Self::log(&PeekReport::after_dtor_line(drew));
            }
        });
    }

    /// After `UIScene::handleDirection`: for a wheel step logs the step, the Shift key, the threads, whether the
    /// scene's controller is a container screen and whether it is the controller of the tracked hover. The tracked
    /// controller is only compared by address, never read.
    #[allow(unsafe_code)]
    pub fn on_direction(&self, scene: usize, direction: u8, step: [f32; 2]) {
        if direction != UiScene::DIRECTION_SCROLL {
            return;
        }
        self.guarded(|| {
            let Some(entries) = self.entries.get() else {
                return;
            };
            // SAFETY: `scene` is the `this` of the running handleDirection call.
            let controller = unsafe { UiScene::from_raw(scene as *mut c_void, entries) }
                .and_then(|scene| scene.screen_view())
                .and_then(|view| view.controller());
            let container = controller.as_ref().and_then(|controller| controller.is_container_screen());
            let address = controller.as_ref().map(|controller| controller.address());
            let hovered = address.is_some() && self.hover().as_ref().map(|held| held.controller) == address;
            Self::log(&PeekReport::scroll_line(
                step,
                Self::shift_down(),
                Self::thread_id(),
                self.render_thread.load(Ordering::Relaxed),
                container,
                hovered,
            ));
        });
    }

    // Draws the test cell for the hovered slot, when it holds a shulker box with a stored item. `None` when nothing
    // was drawn.
    #[allow(unsafe_code)]
    fn draw(
        &self,
        entries: &GameEntries,
        view: &ScreenView<'_>,
        ui: &UiRenderContext<'_>,
        container: &ContainerScreenController<'_>,
    ) -> Option<()> {
        let hover = self.hover().clone()?;
        if container.address() != hover.controller {
            return None;
        }
        let stack = Self::visual_stack(container, &hover)?;
        if !stack.type_name().is_some_and(|name| name.contains(Self::SHULKER)) {
            return None;
        }
        let items = stack.user_data()?.list("Items")?;
        let owned = (0..items.len()).find_map(|index| items.compound(index).and_then(OwnedItemStack::from_tag))?;
        let base = owned.as_base();
        let [x, y] = view.pointer()?;
        ui.fill_rect(
            RectangleAreaLayout::new(x, x + Self::CELL, y, y + Self::CELL),
            UiColorLayout::rgb(0.1, 0.1, 0.1),
            0.9,
        )?;
        let context = BaseActorRenderContext::for_ui(ui)?;
        let renderer = context.item_renderer()?;
        let client = ui.client()?;
        let holder = client.local_player().map_or(std::ptr::null_mut(), |player| player.as_ptr());
        // SAFETY: `holder` is null or the live local player.
        let frame = unsafe { base.animation_frame(holder) }.unwrap_or(0);
        renderer.render_gui_item(&context, &base, frame, x, y, false, 1.0, 1.0, 0)?;
        // the game reads the text at the flush, so it lives until after it
        let text = format!("x{}", base.count().unwrap_or(0));
        let mut text_drawn = false;
        // SAFETY: the render call's client is a live ClientInstance of the build `entries` was resolved for.
        if let Some(font) = unsafe { FontScope::open(entries, client.as_ptr()) }
            && let Some(font_ptr) = font.font()
        {
            // SAFETY: the font lives while `font` does, and `text` lives past the flush below.
            let queued = unsafe {
                ui.draw_text(
                    font_ptr,
                    RectangleAreaLayout::new(x + Self::CELL + 2.0, x + Self::CELL + 42.0, y + 4.0, y + Self::CELL),
                    &text,
                    UiColorLayout::rgb(1.0, 1.0, 1.0),
                    1.0,
                    0,
                    TextMeasureDataLayout::new(1.0, true),
                )
            };
            text_drawn = queued.is_some() && ui.flush_text(0.016).is_some();
        }
        if !self.built_logged.swap(true, Ordering::Relaxed) {
            Self::log(&PeekReport::built_line(base.type_name().as_deref(), base.count()));
            Self::log(&PeekReport::drew_line([x, y], &text, text_drawn));
        }
        Some(())
    }
}

impl Default for PeekProbe {
    fn default() -> Self {
        Self::new()
    }
}
