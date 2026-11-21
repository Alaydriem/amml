use std::ffi::c_void;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

use amml::{
    DispatchFinish, DrawCalls, ModLoader, ScreenAdapter, SessionAdapter, TextAdapter, TextCalls, TextureAdapter,
    TextureCalls, UiDrawAdapter, UiScope,
};
use amml_abi::{
    AmmlAmount, AmmlApi, AmmlColor, AmmlContainerAction, AmmlInputMode, AmmlPoint, AmmlRect, AmmlScreenKind,
    AmmlServices, AmmlSize, AmmlSlotAt, AmmlStatus, AmmlStr, AmmlTextAlign, AmmlTextureId, AmmlTextureSource,
    AmmlUiProfile, AmmlUiTable, AmmlUv,
};
use amml_runtime::{ModRegistry, UiScopeKind};

use crate::support::{EventsMod, Harness, SessionGame};

const CONTROLLER: usize = 0x5000;
const VIEW: usize = 0x6000;

// A harness with the session adapter over the stand-in game and one loaded mod.
fn start() -> Harness {
    let harness = Harness::start_with_entries(&SessionGame::entries(&[]));
    SessionAdapter::install(harness.state);
    let plain = EventsMod::build("plain", &[]);
    harness.install_mod("events", "events", &plain, "events.dll", 3);
    let scan = ModRegistry::new(ModRegistry::MAX_MODS).scan(&harness.mods()).unwrap();
    assert!(ModLoader::new(harness.state).load_all(&scan.mods).iter().all(|report| report.is_ok()));
    harness
}

// The ui table, through the services table as a mod reaches it.
fn table(harness: &Harness) -> &'static AmmlUiTable {
    // SAFETY: the loader's tables live for the process.
    let services = unsafe { AmmlApi::services(harness.state.api()) }.unwrap();
    // SAFETY: as above.
    let table = unsafe { AmmlServices::ui(services) }.unwrap();
    // SAFETY: as above.
    unsafe { &*table }
}

// Every call of the table on `view`, with a live output each.
fn calls(table: &AmmlUiTable, view: u64) -> [AmmlStatus; 7] {
    let mut kind = AmmlScreenKind::OTHER;
    let mut size = AmmlSize { width: 0, height: 0 };
    let mut rect = AmmlRect { x: 0.0, y: 0.0, width: 0.0, height: 0.0 };
    let mut point = AmmlPoint { x: 0.0, y: 0.0 };
    let mut mode = AmmlInputMode::UNKNOWN;
    let mut profile = AmmlUiProfile::CLASSIC;
    let mut container = 0u64;
    // SAFETY: the table is live and every output is a live local.
    unsafe {
        [
            (table.view_kind)(view, &mut kind),
            (table.view_size)(view, &mut size),
            (table.view_content_rect)(view, &mut rect),
            (table.view_pointer)(view, &mut point),
            (table.view_input_mode)(view, &mut mode),
            (table.view_ui_profile)(view, &mut profile),
            (table.view_container)(view, &mut container),
        ]
    }
}

#[test]
fn null_out_is_invalid_argument() {
    let harness = start();
    let table = table(&harness);
    // SAFETY: the table is live; a null output is the case under test.
    unsafe {
        assert_eq!((table.view_kind)(1, std::ptr::null_mut()), AmmlStatus::INVALID_ARGUMENT);
        assert_eq!((table.view_size)(1, std::ptr::null_mut()), AmmlStatus::INVALID_ARGUMENT);
        assert_eq!((table.view_content_rect)(1, std::ptr::null_mut()), AmmlStatus::INVALID_ARGUMENT);
        assert_eq!((table.view_pointer)(1, std::ptr::null_mut()), AmmlStatus::INVALID_ARGUMENT);
        assert_eq!((table.view_input_mode)(1, std::ptr::null_mut()), AmmlStatus::INVALID_ARGUMENT);
        assert_eq!((table.view_ui_profile)(1, std::ptr::null_mut()), AmmlStatus::INVALID_ARGUMENT);
        assert_eq!((table.view_container)(1, std::ptr::null_mut()), AmmlStatus::INVALID_ARGUMENT);
    }
}

#[test]
fn no_adapter_is_not_available() {
    let harness = start();
    SessionGame::frame();

    assert_eq!(calls(table(&harness), 1), [AmmlStatus::NOT_AVAILABLE; 7]);
}

#[test]
fn an_adapter_whose_area_is_not_offered_is_not_available() {
    let harness = start();
    ScreenAdapter::install(harness.state);
    SessionGame::frame();

    assert_eq!(calls(table(&harness), 1), [AmmlStatus::NOT_AVAILABLE; 7]);
}

#[test]
fn before_the_gate_is_not_ready() {
    let harness = start();
    ScreenAdapter::install(harness.state);
    harness.state.areas().offer(ScreenAdapter::SCREEN, 1);

    assert_eq!(calls(table(&harness), 1), [AmmlStatus::NOT_READY; 7]);
}

#[test]
fn outside_the_screen_render_scope_every_ui_call_is_refused() {
    let harness = start();
    ScreenAdapter::install(harness.state);
    harness.state.areas().offer(ScreenAdapter::SCREEN, 1);
    SessionGame::frame();
    let (_, view) = ScreenAdapter::get().unwrap().book().note(CONTROLLER, VIEW);

    assert_eq!(calls(table(&harness), view), [AmmlStatus::REFUSED; 7]);
    // A scope of another view does not open this one.
    let _other = UiScope::enter(UiScopeKind::ScreenRender { view: view + 1 });
    assert_eq!(calls(table(&harness), view), [AmmlStatus::REFUSED; 7]);
}

#[test]
fn a_retired_view_is_refused() {
    let harness = start();
    ScreenAdapter::install(harness.state);
    harness.state.areas().offer(ScreenAdapter::SCREEN, 1);
    SessionGame::frame();
    let book = ScreenAdapter::get().unwrap().book();
    let (_, view) = book.note(CONTROLLER, VIEW);
    book.retire(CONTROLLER);

    let _scope = UiScope::enter(UiScopeKind::ScreenRender { view });

    assert_eq!(calls(table(&harness), view), [AmmlStatus::REFUSED; 7]);
}

// The module of the mod that `start` loads.
const MODULE: &str = "@events/events";

// What the fake render context saw, one line per call.
static SEEN: Mutex<Vec<String>> = Mutex::new(Vec::new());

fn seen() -> Vec<String> {
    SEEN.lock().unwrap().clone()
}

unsafe extern "C" fn fake_fill(_: usize, area: *const [f32; 4], color: *const [f32; 4], alpha: f32) {
    // SAFETY: the adapter passes live arrays.
    let (area, color) = unsafe { (area.read(), color.read()) };
    SEEN.lock().unwrap().push(format!("fill {area:?} {color:?} {alpha}"));
}

unsafe extern "C" fn fake_outline(_: usize, area: *const [f32; 4], _: *const [f32; 4], alpha: f32, thickness: i32) {
    // SAFETY: the adapter passes a live array.
    let area = unsafe { area.read() };
    SEEN.lock().unwrap().push(format!("outline {area:?} {alpha} {thickness}"));
}

unsafe extern "C" fn fake_image(
    _: usize,
    data: usize,
    position: *const [f32; 2],
    size: *const [f32; 2],
    uv: *const [f32; 2],
    uv_size: *const [f32; 2],
    _: u8,
) {
    // SAFETY: the adapter passes live arrays.
    let (position, size, uv, uv_size) = unsafe { (position.read(), size.read(), uv.read(), uv_size.read()) };
    SEEN.lock().unwrap().push(format!("image {data:#x} {position:?} {size:?} {uv:?} {uv_size:?}"));
}

unsafe extern "C" fn fake_flush(_: usize, color: *const [f32; 4], alpha: f32, _: *const std::ffi::c_void) {
    // SAFETY: the adapter passes a live array.
    let color = unsafe { color.read() };
    SEEN.lock().unwrap().push(format!("flush {color:?} {alpha}"));
}

unsafe extern "C" fn fake_gui_scale(_: usize) -> f32 {
    2.0
}

// A fake render context: an object whose first word is a vtable of the fake draw functions, with a queue of images
// (begin and end at +98h) and a screen context that holds a GUI data pointer.
struct FakeContext {
    object: Box<[usize; 32]>,
    _vtable: Box<[usize; 8]>,
    _screen: Box<[usize; 4]>,
}

impl FakeContext {
    const QUEUE: usize = 0x98;

    fn new(queued: bool) -> Self {
        let mut vtable = Box::new([0usize; 8]);
        vtable[1] = fake_fill as *const () as usize;
        vtable[2] = fake_outline as *const () as usize;
        vtable[3] = fake_image as *const () as usize;
        vtable[4] = fake_flush as *const () as usize;
        vtable[5] = fake_draw_text as *const () as usize;
        vtable[6] = fake_flush_text as *const () as usize;
        let mut screen = Box::new([0usize; 4]);
        screen[1] = 0x7000;
        let mut object = Box::new([0usize; 32]);
        object[0] = vtable.as_ptr() as usize;
        object[1] = screen.as_ptr() as usize;
        if queued {
            object[Self::QUEUE / 8] = 0x100;
            object[Self::QUEUE / 8 + 1] = 0x130;
        }
        Self { object, _vtable: vtable, _screen: screen }
    }

    fn address(&self) -> usize {
        self.object.as_ptr() as usize
    }
}

fn draw_calls() -> DrawCalls {
    DrawCalls {
        fill_slot: 1,
        outline_slot: 2,
        image_slot: 3,
        flush_slot: 4,
        screen_context: 8,
        gui_data: 8,
        gui_scale: fake_gui_scale as *const () as usize,
        queued_images: Some(FakeContext::QUEUE),
    }
}

fn texture_calls() -> TextureCalls {
    TextureCalls {
        resource_location_ctor: 1,
        image_buffer_ctor: 1,
        image_buffer_words: 6,
        buffer_deleter: 0,
        buffer_data: 8,
        buffer_size: 16,
        upload: 1,
        group_get: 1,
        unload: 1,
        group_base: 0x18,
        get_texture_group_slot: 1,
        context_get_texture_slot: 1,
    }
}

const RECT: AmmlRect = AmmlRect { x: 1.0, y: 2.0, width: 10.0, height: 20.0 };
const WHITE: AmmlColor = AmmlColor { r: 1.0, g: 1.0, b: 1.0 };
const UV: AmmlUv = AmmlUv { u0: 0.0, v0: 0.0, u1: 1.0, v1: 1.0 };
const NO_TEXTURE: AmmlTextureId = AmmlTextureId { id: 1, generation: 0, _reserved: 0 };

fn draws(table: &AmmlUiTable, view: u64) -> [AmmlStatus; 4] {
    let mut scale = 0.0f32;
    // SAFETY: the table is live and every output is a live local.
    unsafe {
        [
            (table.draw_fill.unwrap())(view, RECT, WHITE, 1.0),
            (table.draw_outline.unwrap())(view, RECT, WHITE, 1.0, 1),
            (table.draw_image.unwrap())(view, NO_TEXTURE, RECT, UV, WHITE, 1.0),
            (table.gui_scale.unwrap())(view, &mut scale),
        ]
    }
}

fn can_draw(table: &AmmlUiTable, view: u64) -> u8 {
    // SAFETY: the table is live.
    unsafe { (table.can_draw.unwrap())(view) }
}

#[test]
fn draw_outside_scope_is_refused() {
    let harness = start();
    UiDrawAdapter::install_with(harness.state, draw_calls());
    SessionGame::frame();
    let table = table(&harness);

    assert_eq!(draws(table, 5), [AmmlStatus::REFUSED; 4]);
    assert_eq!(can_draw(table, 5), 0);
    // A scope of another view does not open this one.
    let context = FakeContext::new(false);
    let _other = UiScope::enter_with_context(UiScopeKind::ScreenRender { view: 6 }, context.address());
    assert_eq!(draws(table, 5), [AmmlStatus::REFUSED; 4]);
    assert!(seen().is_empty());
}

#[test]
fn draw_without_the_area_or_before_the_gate_is_not_available_or_not_ready() {
    let harness = start();
    assert_eq!(draws(table(&harness), 5), [AmmlStatus::NOT_AVAILABLE; 4]);

    UiDrawAdapter::install_with(harness.state, draw_calls());
    assert_eq!(draws(table(&harness), 5), [AmmlStatus::NOT_READY; 4]);
}

#[test]
fn a_fill_and_an_outline_reach_the_context_of_the_running_scope() {
    let harness = start();
    UiDrawAdapter::install_with(harness.state, draw_calls());
    SessionGame::frame();
    let table = table(&harness);
    let context = FakeContext::new(false);
    let _scope = UiScope::enter_with_context(UiScopeKind::ScreenRender { view: 7 }, context.address());

    let mut scale = 0.0f32;
    // SAFETY: the table is live and every output is a live local.
    let (fill, outline, status) = unsafe {
        (
            (table.draw_fill.unwrap())(7, RECT, WHITE, 0.5),
            (table.draw_outline.unwrap())(7, RECT, WHITE, 1.0, 2),
            (table.gui_scale.unwrap())(7, &mut scale),
        )
    };

    assert_eq!((fill, outline, status), (AmmlStatus::OK, AmmlStatus::OK, AmmlStatus::OK));
    assert_eq!(scale, 2.0);
    assert_eq!(
        seen(),
        vec![
            "fill [1.0, 11.0, 2.0, 22.0] [1.0, 1.0, 1.0, 1.0] 0.5".to_string(),
            "outline [1.0, 11.0, 2.0, 22.0] 1 2".to_string(),
        ]
    );
}

#[test]
fn image_when_can_draw_is_false_is_refused() {
    let harness = start();
    UiDrawAdapter::install_with(harness.state, draw_calls());
    SessionGame::frame();
    let table = table(&harness);
    let context = FakeContext::new(true);
    let _scope = UiScope::enter_with_context(UiScopeKind::ScreenRender { view: 7 }, context.address());

    assert_eq!(can_draw(table, 7), 0);
    // SAFETY: the table is live.
    let status = unsafe { (table.draw_image.unwrap())(7, NO_TEXTURE, RECT, UV, WHITE, 1.0) };
    assert_eq!(status, AmmlStatus::REFUSED);
    // A fill does not depend on the queue.
    // SAFETY: the table is live.
    assert_eq!(unsafe { (table.draw_fill.unwrap())(7, RECT, WHITE, 1.0) }, AmmlStatus::OK);
    assert!(seen().iter().all(|line| !line.starts_with("flush")));
}

#[test]
fn an_image_with_no_texture_adapter_is_not_available() {
    let harness = start();
    UiDrawAdapter::install_with(harness.state, draw_calls());
    SessionGame::frame();
    let table = table(&harness);
    let context = FakeContext::new(false);
    let _scope = UiScope::enter_with_context(UiScopeKind::ScreenRender { view: 7 }, context.address());
    assert_eq!(can_draw(table, 7), 1);

    // SAFETY: the table is live.
    let status = unsafe { (table.draw_image.unwrap())(7, NO_TEXTURE, RECT, UV, WHITE, 1.0) };

    assert_eq!(status, AmmlStatus::NOT_AVAILABLE);
}

fn load(table: &AmmlUiTable, module: &str, source: AmmlTextureSource, path: &str) -> (AmmlStatus, AmmlTextureId) {
    let mut out = AmmlTextureId { id: 0, generation: 0, _reserved: 0 };
    // SAFETY: the table is live; the views and the output live for the call.
    let status =
        unsafe { (table.texture_load.unwrap())(AmmlStr::from_str(module), source, AmmlStr::from_str(path), &mut out) };
    (status, out)
}

#[test]
fn texture_load_needs_a_module() {
    let harness = start();
    TextureAdapter::install_with(harness.state, texture_calls());
    let table = table(&harness);

    assert_eq!(load(table, "", AmmlTextureSource::VANILLA, "textures/ui/a").0, AmmlStatus::INVALID_ARGUMENT);
    assert_eq!(
        load(table, "@nobody/nothing", AmmlTextureSource::VANILLA, "textures/ui/a").0,
        AmmlStatus::INVALID_ARGUMENT
    );
    // SAFETY: the table is live; a null output is the case under test.
    let null_out = unsafe {
        (table.texture_load.unwrap())(
            AmmlStr::from_str(MODULE),
            AmmlTextureSource::VANILLA,
            AmmlStr::from_str("a"),
            std::ptr::null_mut(),
        )
    };
    assert_eq!(null_out, AmmlStatus::INVALID_ARGUMENT);
}

#[test]
fn a_texture_call_without_the_adapter_is_not_available() {
    let harness = start();

    assert_eq!(load(table(&harness), MODULE, AmmlTextureSource::VANILLA, "a").0, AmmlStatus::NOT_AVAILABLE);
}

#[test]
fn textures_queue_from_any_thread_and_use_after_a_world_leave_is_stale() {
    let harness = start();
    TextureAdapter::install_with(harness.state, texture_calls());
    let table = table(&harness);

    let (status, id) = load(table, MODULE, AmmlTextureSource::VANILLA, "textures/ui/a.png");
    assert_eq!(status, AmmlStatus::OK);
    assert_ne!(id.id, 0);
    // SAFETY: the table is live.
    assert_eq!(unsafe { (table.texture_ready.unwrap())(id) }, 0);

    // A mod file path that leaves the mod folder is refused; a wrong upload size is invalid.
    assert_eq!(load(table, MODULE, AmmlTextureSource::MOD_FILE, "../x").0, AmmlStatus::REFUSED);
    let mut out = AmmlTextureId { id: 0, generation: 0, _reserved: 0 };
    let pixels = [0u8; 4];
    // SAFETY: the table is live; the pixels and the output live for the call.
    let upload =
        unsafe { (table.texture_upload_rgba.unwrap())(AmmlStr::from_str(MODULE), 2, 2, pixels.as_ptr(), 4, &mut out) };
    assert_eq!(upload, AmmlStatus::INVALID_ARGUMENT);

    // From another thread, an unload only queues.
    let unload = std::thread::scope(|scope| {
        scope
            .spawn(|| {
                // SAFETY: the table is live.
                unsafe { (table.texture_unload.unwrap())(AmmlStr::from_str(MODULE), id) }
            })
            .join()
            .unwrap()
    });
    assert_eq!(unload, AmmlStatus::OK);
    // SAFETY: the table is live.
    assert_eq!(unsafe { (table.texture_unload.unwrap())(AmmlStr::from_str(MODULE), id) }, AmmlStatus::STALE);

    let (_, second) = load(table, MODULE, AmmlTextureSource::VANILLA, "textures/ui/b");
    TextureAdapter::get().unwrap().world_left();
    // SAFETY: the table is live.
    assert_eq!(unsafe { (table.texture_unload.unwrap())(AmmlStr::from_str(MODULE), second) }, AmmlStatus::STALE);
    // SAFETY: the table is live.
    assert_eq!(unsafe { (table.texture_ready.unwrap())(second) }, 0);
}

// A texture the book holds and has marked ready, whose `BedrockTextureData` is at `data`. Keeps the fake `TexturePtr`
// alive through the returned box.
fn ready_texture(data: usize) -> (AmmlTextureId, Box<[usize; 4]>) {
    let adapter = TextureAdapter::get().unwrap();
    let key = adapter.book().load(0, amml_runtime::TextureSourceKind::Vanilla, "textures/ui/a", "").unwrap();
    let held = Box::new([data, 0, 0, 0]);
    assert!(adapter.book().bound(key, held.as_ptr() as usize));
    adapter.book().mark_ready(key);
    (AmmlTextureId { id: key.id, generation: key.generation, _reserved: 0 }, held)
}

#[test]
fn images_of_one_texture_are_queued_and_flushed_once_after_the_last_call() {
    let harness = start();
    UiDrawAdapter::install_with(harness.state, draw_calls());
    TextureAdapter::install_with(harness.state, texture_calls());
    SessionGame::frame();
    let table = table(&harness);
    let (texture, _held) = ready_texture(0x9000);
    let context = FakeContext::new(false);
    let _scope = UiScope::enter_with_context(UiScopeKind::ScreenRender { view: 7 }, context.address());
    let tint = AmmlColor { r: 0.5, g: 0.5, b: 0.5 };

    // SAFETY: the table is live.
    let statuses = unsafe {
        [
            (table.draw_image.unwrap())(7, texture, RECT, UV, tint, 1.0),
            (table.draw_image.unwrap())(7, texture, RECT, UV, tint, 1.0),
            (table.draw_fill.unwrap())(7, RECT, WHITE, 1.0),
            (table.draw_image.unwrap())(7, texture, RECT, UV, tint, 1.0),
        ]
    };
    UiDrawAdapter::get().unwrap().finish(context.address());

    assert_eq!(statuses, [AmmlStatus::OK; 4]);
    let lines = seen();
    assert_eq!(lines.len(), 6, "{lines:?}");
    assert!(lines[0].starts_with("image 0x9000 [1.0, 2.0] [10.0, 20.0] [0.0, 0.0] [1.0, 1.0]"), "{lines:?}");
    assert!(lines[1].starts_with("image 0x9000"), "{lines:?}");
    // The fill flushes the queue first, so the order on the screen is the order of the calls.
    assert_eq!(lines[2], "flush [0.5, 0.5, 0.5, 1.0] 1");
    assert!(lines[3].starts_with("fill"), "{lines:?}");
    assert!(lines[4].starts_with("image 0x9000"), "{lines:?}");
    assert_eq!(lines[5], "flush [0.5, 0.5, 0.5, 1.0] 1");
}

#[test]
fn a_panic_inside_a_dispatch_still_draws_the_queued_images() {
    let harness = start();
    UiDrawAdapter::install_with(harness.state, draw_calls());
    TextureAdapter::install_with(harness.state, texture_calls());
    SessionGame::frame();
    let table = table(&harness);
    let (texture, _held) = ready_texture(0x9000);
    let context = FakeContext::new(false);
    let _scope = UiScope::enter_with_context(UiScopeKind::ScreenRender { view: 7 }, context.address());
    let tint = AmmlColor { r: 0.5, g: 0.5, b: 0.5 };

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _finish = DispatchFinish::new(context.address());
        // SAFETY: the table is live.
        unsafe { (table.draw_image.unwrap())(7, texture, RECT, UV, tint, 1.0) };
        panic!("a loader bug inside the dispatch");
    }));

    assert!(result.is_err());
    let lines = seen();
    assert_eq!(lines.len(), 2, "{lines:?}");
    assert!(lines[0].starts_with("image 0x9000"), "{lines:?}");
    assert_eq!(lines[1], "flush [0.5, 0.5, 0.5, 1.0] 1");
}

const FIRST_SLOT: AmmlSlotAt = AmmlSlotAt { collection: AmmlSlotAt::HOTBAR, index: 3 };

// Every container call on `view` with a slot at `slot`: the five calls of the table, in order.
fn container_calls(table: &AmmlUiTable, view: u64, slot: AmmlSlotAt) -> [AmmlStatus; 5] {
    let (mut count, mut stack, mut swapped) = (0u32, std::ptr::null::<std::ffi::c_void>(), 0u8);
    // SAFETY: the table is live and every output is a live local.
    unsafe {
        [
            (table.container_slot_count.unwrap())(view, slot.collection, &mut count),
            (table.container_item.unwrap())(view, slot, &mut stack),
            (table.container_cursor.unwrap())(view, &mut stack),
            (table.container_action.unwrap())(view, AmmlContainerAction::AUTO_PLACE, slot, AmmlAmount::WHOLE),
            (table.container_swap.unwrap())(view, slot, slot, &mut swapped),
        ]
    }
}

#[test]
fn the_table_serves_the_container_functions() {
    let harness = start();
    let table = table(&harness);

    assert!(table.size >= AmmlUiTable::SIZE_V3);
    assert!(table.container_slot_count.is_some() && table.container_swap.is_some());
}

#[test]
fn action_outside_a_container_view_is_refused() {
    let harness = start();
    ScreenAdapter::install(harness.state);
    harness.state.areas().offer(ScreenAdapter::SCREEN, 2);
    SessionGame::frame();
    let (_, view) = ScreenAdapter::get().unwrap().book().note(CONTROLLER, VIEW);
    let table = table(&harness);

    assert_eq!(container_calls(table, view, FIRST_SLOT), [AmmlStatus::REFUSED; 5]);
    // A scope of another view does not open this one.
    let _other = UiScope::enter(UiScopeKind::ScreenRender { view: view + 1 });
    assert_eq!(container_calls(table, view, FIRST_SLOT), [AmmlStatus::REFUSED; 5]);
}

#[test]
fn container_calls_without_the_adapter_are_not_available() {
    let harness = start();
    SessionGame::frame();

    assert_eq!(container_calls(table(&harness), 1, FIRST_SLOT), [AmmlStatus::NOT_AVAILABLE; 5]);
}

#[test]
fn slot_index_out_of_range_is_invalid_argument() {
    let harness = start();
    let table = table(&harness);
    let negative = AmmlSlotAt { collection: AmmlSlotAt::CONTAINER, index: -1 };
    let huge = AmmlSlotAt { collection: AmmlSlotAt::CONTAINER, index: AmmlSlotAt::MAX_INDEX + 1 };
    let other = AmmlSlotAt { collection: AmmlSlotAt::OTHER, index: 0 };

    // The slot is checked before the view, so a bad slot is invalid whatever the view is.
    for slot in [negative, huge, other] {
        let statuses = container_calls(table, 1, slot);
        assert_eq!(statuses[1], AmmlStatus::INVALID_ARGUMENT, "{slot:?}");
        assert_eq!(statuses[3], AmmlStatus::INVALID_ARGUMENT, "{slot:?}");
        assert_eq!(statuses[4], AmmlStatus::INVALID_ARGUMENT, "{slot:?}");
    }
    // The cursor and the slot count take no slot, so they are not invalid for a bad slot.
    assert_ne!(container_calls(table, 1, negative)[2], AmmlStatus::INVALID_ARGUMENT);
}

#[test]
fn a_bad_action_amount_or_output_is_invalid_argument() {
    let harness = start();
    let table = table(&harness);
    let bad_amount = AmmlAmount { kind: 9, count: 0, _reserved: [0; 2] };
    let zero_count = AmmlAmount::count(0);
    let mut swapped = 0u8;

    // SAFETY: the table is live; null outputs and bad values are the cases under test.
    unsafe {
        let drop_slot = AmmlContainerAction::DROP_SLOT;
        assert_eq!(
            (table.container_action.unwrap())(1, drop_slot, FIRST_SLOT, bad_amount),
            AmmlStatus::INVALID_ARGUMENT
        );
        assert_eq!(
            (table.container_action.unwrap())(1, drop_slot, FIRST_SLOT, zero_count),
            AmmlStatus::INVALID_ARGUMENT
        );
        assert_eq!(
            (table.container_action.unwrap())(1, AmmlContainerAction(99), FIRST_SLOT, AmmlAmount::WHOLE),
            AmmlStatus::INVALID_ARGUMENT
        );
        assert_eq!((table.container_slot_count.unwrap())(1, 0, std::ptr::null_mut()), AmmlStatus::INVALID_ARGUMENT);
        assert_eq!((table.container_item.unwrap())(1, FIRST_SLOT, std::ptr::null_mut()), AmmlStatus::INVALID_ARGUMENT);
        assert_eq!((table.container_cursor.unwrap())(1, std::ptr::null_mut()), AmmlStatus::INVALID_ARGUMENT);
        assert_eq!(
            (table.container_swap.unwrap())(1, FIRST_SLOT, FIRST_SLOT, std::ptr::null_mut()),
            AmmlStatus::INVALID_ARGUMENT
        );
        assert_eq!(
            (table.container_swap.unwrap())(1, FIRST_SLOT, AmmlSlotAt { collection: 0, index: -2 }, &mut swapped),
            AmmlStatus::INVALID_ARGUMENT
        );
    }
}

#[test]
fn moves_are_refused_off_the_game_thread() {
    let harness = start();
    ScreenAdapter::install(harness.state);
    harness.state.areas().offer(ScreenAdapter::SCREEN, 2);
    SessionGame::frame();
    let (_, view) = ScreenAdapter::get().unwrap().book().note(CONTROLLER, VIEW);
    let table = table(&harness);
    let on_thread = |view: u64| {
        let _scope = UiScope::enter(UiScopeKind::ScreenRender { view });
        // SAFETY: the table is live and every output is a live local.
        unsafe {
            let mut swapped = 0u8;
            [
                (table.container_action.unwrap())(view, AmmlContainerAction::TAKE_ALL, FIRST_SLOT, AmmlAmount::WHOLE),
                (table.container_swap.unwrap())(view, FIRST_SLOT, FIRST_SLOT, &mut swapped),
            ]
        }
    };

    // On the game thread the call passes the thread check and stops at the missing game layout.
    assert_eq!(on_thread(view), [AmmlStatus::NOT_AVAILABLE; 2]);
    // The same call, with a running scope, from another thread is refused for the thread.
    let other = std::thread::scope(|scope| scope.spawn(|| on_thread(view)).join().unwrap());
    assert_eq!(other, [AmmlStatus::REFUSED; 2]);
}

// The text calls. The stand-in game keeps the view of each queued text, as the game does, and reads the bytes only
// at the flush.
// The font, the address and the length of the text, and the area of one queued text.
type QueuedText = (usize, usize, usize, [f32; 4]);
static QUEUED: Mutex<Vec<QueuedText>> = Mutex::new(Vec::new());
static DESTROYED: AtomicUsize = AtomicUsize::new(0);
static FONT_ID: AtomicUsize = AtomicUsize::new(1);
// The addresses the font handle stand-in hands out: the repository, the default font.
static FONT_PARTS: Mutex<[usize; 2]> = Mutex::new([0; 2]);

unsafe extern "C" fn fake_draw_text(
    _: usize,
    font: usize,
    area: *const [f32; 4],
    view: *const [usize; 2],
    color: *const [f32; 4],
    alpha: f32,
    align: u32,
    measure: *const [u32; 4],
    caret: *const [u32; 2],
) {
    // SAFETY: the adapter passes live arrays.
    let (area, view, color, measure, caret) =
        unsafe { (area.read(), view.read(), color.read(), measure.read(), caret.read()) };
    assert_eq!((color, alpha, align), ([1.0, 0.5, 0.25, 1.0], 0.75, 0));
    assert_eq!((measure, caret), ([1.0f32.to_bits(), 0, 1, 0], [0, 0]));
    QUEUED.lock().unwrap().push((font, view[0], view[1], area));
}

unsafe extern "C" fn fake_flush_text(_: usize, delta: f32, maximum_lifetime: u64) {
    let mut queued = QUEUED.lock().unwrap();
    for (font, data, len, area) in queued.drain(..) {
        // SAFETY: the loader keeps the copy of the text alive until the flush.
        let text = unsafe { std::slice::from_raw_parts(data as *const u8, len) };
        SEEN.lock().unwrap().push(format!("text {:?} {font:#x} {area:?}", String::from_utf8_lossy(text)));
    }
    SEEN.lock().unwrap().push(format!("flush text {delta} {maximum_lifetime}"));
}

unsafe extern "C" fn fake_font_handle(_: usize, out: *mut [u64; 10]) -> usize {
    let [repository, default_font] = *FONT_PARTS.lock().unwrap();
    let mut handle = [0u64; 10];
    handle[5] = repository as u64;
    handle[6] = default_font as u64;
    handle[8] = FONT_ID.load(Ordering::SeqCst) as u64;
    // SAFETY: the adapter passes a buffer of the size of a `FontHandle`.
    unsafe { out.write(handle) };
    out as usize
}

unsafe extern "C" fn fake_font_destroy(_: *mut [u64; 10]) {
    DESTROYED.fetch_add(1, Ordering::SeqCst);
}

unsafe extern "C" fn fake_line_length(_: usize, view: *const [usize; 2], size: f32, _: u8) -> i32 {
    assert_eq!(size, 1.0);
    // SAFETY: the adapter passes a live view.
    let len = unsafe { view.read() }[1];
    // A character of the stand-in font is 6 units wide, and the texts of the tests are ASCII.
    i32::try_from(len * 6).unwrap()
}

unsafe extern "C" fn no_player(_: *mut c_void) -> *mut c_void {
    std::ptr::null_mut()
}

unsafe extern "C" fn no_join(_: *mut c_void, _: u64, _: u64, _: u64, _: u64, _: u64, _: u64, _: u64) {}

// The stand-in game objects of the text calls: a client whose vtable has `getFontHandle` at slot 5, two fonts of
// the repository and a default font that share one vtable with `getLineLength` at slot 6.
struct FakeText {
    client: Box<[usize; 4]>,
    fonts: Box<[usize; 4]>,
    default_font: Box<[usize; 2]>,
    _repository: Box<[usize; 12]>,
    _client_vtable: Box<[usize; 8]>,
    _font_vtable: Box<[usize; 8]>,
}

impl FakeText {
    fn new() -> Self {
        let mut font_vtable = Box::new([0usize; 8]);
        font_vtable[6] = fake_line_length as *const () as usize;
        let vtable = font_vtable.as_ptr().addr();
        let mut fonts = Box::new([0usize; 4]);
        fonts[0] = Box::leak(Box::new([vtable, 0usize])).as_ptr().addr();
        fonts[2] = Box::leak(Box::new([vtable, 0usize])).as_ptr().addr();
        let default_font = Box::new([vtable, 0usize]);
        // The repository holds `mLoadedFonts` at 0x40: begin and end, two 16-byte elements.
        let mut repository = Box::new([0usize; 12]);
        repository[8] = fonts.as_ptr().addr();
        repository[9] = fonts.as_ptr().addr() + 32;
        *FONT_PARTS.lock().unwrap() = [repository.as_ptr().addr(), default_font.as_ptr().addr()];
        let mut client_vtable = Box::new([0usize; 8]);
        client_vtable[0] = no_player as *const () as usize;
        client_vtable[1] = no_join as *const () as usize;
        client_vtable[5] = fake_font_handle as *const () as usize;
        let mut client = Box::new([0usize; 4]);
        client[0] = client_vtable.as_ptr().addr();
        Self {
            client,
            fonts,
            default_font,
            _repository: repository,
            _client_vtable: client_vtable,
            _font_vtable: font_vtable,
        }
    }

    // The address of the font at `index` of the repository.
    fn repository_font(&self, index: usize) -> usize {
        self.fonts[index * 2]
    }

    fn default_font(&self) -> usize {
        self.default_font.as_ptr().addr()
    }
}

fn text_calls() -> TextCalls {
    TextCalls {
        draw_slot: 5,
        flush_slot: 6,
        get_font_handle_slot: 5,
        line_length_slot: 6,
        font_dtor: fake_font_destroy as *const () as usize,
        font_handle_size: 0x50,
        default_font: 0x30,
        loaded_fonts: 0x40,
    }
}

// A harness with the text adapter, the game gate open and the stand-in client as the primary client.
fn start_text(game: &FakeText) -> Harness {
    let harness = start();
    TextAdapter::install_with(harness.state, text_calls());
    SessionGame::frame();
    SessionGame::update_raw(game.client.as_ptr().cast_mut().cast(), 1, 0);
    harness
}

fn text_width(table: &AmmlUiTable, view: u64, text: &str) -> (AmmlStatus, f32) {
    let mut out = -1.0f32;
    // SAFETY: the table is live, the text and the output live across the call.
    let status = unsafe { (table.text_width.unwrap())(view, AmmlStr::from_str(text), &mut out) };
    (status, out)
}

fn draw_text(table: &AmmlUiTable, view: u64, text: &str, align: AmmlTextAlign) -> AmmlStatus {
    let color = AmmlColor { r: 1.0, g: 0.5, b: 0.25 };
    // SAFETY: the table is live and the text lives across the call.
    unsafe { (table.draw_text.unwrap())(view, AmmlStr::from_str(text), RECT, color, 0.75, align) }
}

#[test]
fn text_outside_scope_is_refused() {
    let game = FakeText::new();
    let harness = start_text(&game);
    let table = table(&harness);

    assert_eq!(text_width(table, 5, "ab"), (AmmlStatus::REFUSED, -1.0));
    assert_eq!(draw_text(table, 5, "ab", AmmlTextAlign::LEFT), AmmlStatus::REFUSED);
    // A scope of another view does not open this one.
    let context = FakeContext::new(false);
    let _other = UiScope::enter_with_context(UiScopeKind::ScreenRender { view: 6 }, context.address());
    assert_eq!(text_width(table, 5, "ab"), (AmmlStatus::REFUSED, -1.0));
    assert_eq!(draw_text(table, 5, "ab", AmmlTextAlign::LEFT), AmmlStatus::REFUSED);
    assert!(seen().is_empty() && QUEUED.lock().unwrap().is_empty());
}

#[test]
fn text_over_256_bytes_is_refused() {
    let game = FakeText::new();
    let harness = start_text(&game);
    let table = table(&harness);
    let context = FakeContext::new(false);
    let _scope = UiScope::enter_with_context(UiScopeKind::ScreenRender { view: 7 }, context.address());
    let long = "x".repeat(257);
    let limit = "x".repeat(256);

    assert_eq!(text_width(table, 7, &long), (AmmlStatus::REFUSED, -1.0));
    assert_eq!(draw_text(table, 7, &long, AmmlTextAlign::LEFT), AmmlStatus::REFUSED);
    assert_eq!(text_width(table, 7, &limit), (AmmlStatus::OK, 1536.0));
    assert_eq!(draw_text(table, 7, &limit, AmmlTextAlign::LEFT), AmmlStatus::OK);
}

#[test]
fn text_calls_without_the_adapter_are_not_available() {
    let harness = start();
    SessionGame::frame();
    let table = table(&harness);

    assert_eq!(text_width(table, 7, "ab"), (AmmlStatus::NOT_AVAILABLE, -1.0));
    assert_eq!(draw_text(table, 7, "ab", AmmlTextAlign::LEFT), AmmlStatus::NOT_AVAILABLE);
    // SAFETY: the table is live; a null output is the case under test.
    let status = unsafe { (table.text_width.unwrap())(7, AmmlStr::from_str("ab"), std::ptr::null_mut()) };
    assert_eq!(status, AmmlStatus::INVALID_ARGUMENT);
}

#[test]
fn bad_text_arguments_are_invalid_argument() {
    let game = FakeText::new();
    let harness = start_text(&game);
    let table = table(&harness);
    let context = FakeContext::new(false);
    let _scope = UiScope::enter_with_context(UiScopeKind::ScreenRender { view: 7 }, context.address());
    let bytes = [0xffu8, 0xfe];
    let not_utf8 = AmmlStr { ptr: bytes.as_ptr(), len: bytes.len() };
    let color = AmmlColor { r: 1.0, g: 0.5, b: 0.25 };
    let mut out = 0.0f32;

    // SAFETY: the table is live; the views and outputs live across the calls.
    let statuses = unsafe {
        [
            (table.text_width.unwrap())(7, not_utf8, &mut out),
            (table.draw_text.unwrap())(7, not_utf8, RECT, color, 1.0, AmmlTextAlign::LEFT),
            (table.draw_text.unwrap())(7, AmmlStr::from_str("a"), RECT, color, f32::NAN, AmmlTextAlign::LEFT),
            (table.draw_text.unwrap())(7, AmmlStr::from_str("a"), RECT, color, 1.0, AmmlTextAlign(3)),
            (table.text_width.unwrap())(7, AmmlStr { ptr: std::ptr::null(), len: 2 }, &mut out),
        ]
    };

    assert_eq!(statuses, [AmmlStatus::INVALID_ARGUMENT; 5]);
    assert!(QUEUED.lock().unwrap().is_empty());
}

#[test]
fn text_is_copied_queued_and_drawn_once_after_the_last_listener() {
    let game = FakeText::new();
    let harness = start_text(&game);
    let table = table(&harness);
    let context = FakeContext::new(false);
    let _scope = UiScope::enter_with_context(UiScopeKind::ScreenRender { view: 7 }, context.address());
    let mut owned = *b"ab";

    let width = text_width(table, 7, "abc");
    let statuses = [
        draw_text(table, 7, std::str::from_utf8(&owned).unwrap(), AmmlTextAlign::LEFT),
        draw_text(table, 7, "ab", AmmlTextAlign::CENTER),
        draw_text(table, 7, "ab", AmmlTextAlign::RIGHT),
        draw_text(table, 7, "", AmmlTextAlign::LEFT),
    ];
    // The mod reuses its buffer after the call: the queued text is the copy.
    owned = *b"ZZ";
    let drawn_before_finish = seen();
    TextAdapter::get().unwrap().finish(context.address());
    TextAdapter::get().unwrap().finish(context.address());

    assert_eq!(owned, *b"ZZ");
    assert_eq!(width, (AmmlStatus::OK, 18.0));
    assert_eq!(statuses, [AmmlStatus::OK; 4]);
    assert!(drawn_before_finish.is_empty());
    let font = format!("{:#x}", game.repository_font(1));
    // The text is 12 units wide in a rectangle 10 wide at x 1: left starts at 1, center at 0, right at -1.
    assert_eq!(
        seen(),
        vec![
            format!("text \"ab\" {font} [1.0, 13.0, 2.0, 10.0]"),
            format!("text \"ab\" {font} [0.0, 12.0, 2.0, 10.0]"),
            format!("text \"ab\" {font} [-1.0, 11.0, 2.0, 10.0]"),
            "flush text 0 0".to_string(),
        ]
    );
    assert_eq!(DESTROYED.load(Ordering::SeqCst), 1);
}

#[test]
fn a_font_id_out_of_range_picks_the_default_font() {
    let game = FakeText::new();
    FONT_ID.store(2, Ordering::SeqCst);
    let harness = start_text(&game);
    let table = table(&harness);
    let context = FakeContext::new(false);
    let _scope = UiScope::enter_with_context(UiScopeKind::ScreenRender { view: 7 }, context.address());

    assert_eq!(draw_text(table, 7, "ab", AmmlTextAlign::LEFT), AmmlStatus::OK);
    TextAdapter::get().unwrap().finish(context.address());

    assert!(seen()[0].contains(&format!("{:#x}", game.default_font())));
}

#[test]
fn a_dispatch_that_drew_no_text_does_not_flush_or_open_a_font() {
    let game = FakeText::new();
    let harness = start_text(&game);
    let _ = table(&harness);
    let context = FakeContext::new(false);

    TextAdapter::get().unwrap().finish(context.address());

    assert!(seen().is_empty());
    assert_eq!(DESTROYED.load(Ordering::SeqCst), 0);
}

#[test]
fn text_off_the_game_thread_is_refused() {
    let game = FakeText::new();
    let harness = start_text(&game);
    let table = table(&harness);
    let context = FakeContext::new(false);

    let statuses = std::thread::scope(|scope| {
        scope
            .spawn(|| {
                let _scope = UiScope::enter_with_context(UiScopeKind::ScreenRender { view: 7 }, context.address());
                (text_width(table, 7, "ab").0, draw_text(table, 7, "ab", AmmlTextAlign::LEFT))
            })
            .join()
            .unwrap()
    });

    assert_eq!(statuses, (AmmlStatus::REFUSED, AmmlStatus::REFUSED));
    assert!(QUEUED.lock().unwrap().is_empty());
}
