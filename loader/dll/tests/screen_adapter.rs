use std::ffi::c_void;
use std::sync::Mutex;

use amml::{LoaderState, ModListener, ScreenAdapter, ScreenDetours, SessionAdapter, TexturePump, UiFlags, UiScope};
use amml_abi::{
    AmmlApi, AmmlBand, AmmlEventContext, AmmlEventHeader, AmmlServices, AmmlSize, AmmlSlotAt, AmmlStatus,
    ContainerSlotHover, ContainerSlotUnhover, ScreenRender,
};
use amml_msvc::{MsvcHeap, MsvcString};
use amml_runtime::UiScopeKind;

use crate::support::{Harness, SessionGame};

const CONTROLLER: usize = 0x5000;
const VIEW: usize = 0x6000;

// What the listener saw: the view id and the status of a table read made inside the call.
static SEEN: Mutex<Vec<(u64, AmmlStatus)>> = Mutex::new(Vec::new());

unsafe extern "C" fn on_render(event: *mut AmmlEventHeader, _: *const AmmlEventContext, _: *mut c_void) -> AmmlStatus {
    // SAFETY: the loader raises `ui.screen_render` with a `ScreenRender`.
    let view = unsafe { (*event.cast::<ScreenRender>()).view };
    SEEN.lock().unwrap().push((view, size_of_view(view)));
    AmmlStatus::OK
}

unsafe extern "C" fn on_changed(_: *mut AmmlEventHeader, _: *const AmmlEventContext, _: *mut c_void) -> AmmlStatus {
    AmmlStatus::OK
}

// The status of `view_size` for `view` now.
fn size_of_view(view: u64) -> AmmlStatus {
    let state = LoaderState::get().unwrap();
    // SAFETY: the loader's tables live for the process.
    let services = unsafe { AmmlApi::services(state.api()) }.unwrap();
    // SAFETY: as above.
    let table = unsafe { &*AmmlServices::ui(services).unwrap() };
    let mut out = AmmlSize { width: 0, height: 0 };
    // SAFETY: the table is live and `out` is a live local.
    unsafe { (table.view_size)(view, &mut out) }
}

fn start() -> Harness {
    let harness = Harness::start_with_entries(&SessionGame::entries(&[]));
    SessionAdapter::install(harness.state);
    ScreenAdapter::install(harness.state);
    harness.state.areas().offer(ScreenAdapter::SCREEN, 1);
    harness.state.events().offer("ui.screen_render", 1);
    harness.state.events().offer("ui.screen_changed", 1);
    SessionGame::frame();
    SEEN.lock().unwrap().clear();
    harness
}

fn listen(harness: &Harness, event: &str, callback: amml_abi::EventCallbackFn) -> amml_runtime::ListenerId {
    let listener = ModListener { owner: 0, callback, user: 0 };
    harness.state.events().listen(event, 1, AmmlBand::NORMAL, listener).unwrap()
}

#[test]
fn a_listener_gets_one_dispatch_with_a_scope_that_ends_with_it() {
    let harness = start();
    listen(&harness, "ui.screen_render", on_render);
    assert!(ScreenDetours::views_wanted());

    ScreenAdapter::get().unwrap().dispatch_view(CONTROLLER, VIEW, 0);

    let seen = SEEN.lock().unwrap().clone();
    assert_eq!(seen.len(), 1);
    let (view, inside) = seen[0];
    assert_ne!(view, 0);
    // Inside the scope the call is past the refusal (the test adapter has no game layout, so it is not served).
    assert_eq!(inside, AmmlStatus::NOT_AVAILABLE);
    assert!(!UiScope::runs_view(view));
    assert_eq!(size_of_view(view), AmmlStatus::REFUSED);
}

#[test]
fn an_unlisten_ends_the_dispatch_and_clears_the_flag() {
    let harness = start();
    let id = listen(&harness, "ui.screen_render", on_render);
    assert!(ScreenDetours::views_wanted());

    assert!(harness.state.events().unlisten(id));

    assert!(!ScreenDetours::views_wanted());
    ScreenAdapter::get().unwrap().dispatch_view(CONTROLLER, VIEW, 0);
    assert!(SEEN.lock().unwrap().is_empty());
}

#[test]
fn the_flag_stays_while_either_event_has_a_listener() {
    let harness = start();
    let render = listen(&harness, "ui.screen_render", on_render);
    let changed = listen(&harness, "ui.screen_changed", on_changed);

    harness.state.events().unlisten(render);
    assert!(ScreenDetours::views_wanted());
    harness.state.events().unlisten(changed);
    assert!(!ScreenDetours::views_wanted());
}

#[test]
fn another_thread_has_no_scope() {
    let _scope = UiScope::enter(UiScopeKind::ScreenRender { view: 7 });

    let seen = std::thread::spawn(|| (UiScope::current(), UiScope::runs_view(7))).join().unwrap();

    assert_eq!(seen, (None, false));
    assert!(UiScope::runs_view(7));
}

#[test]
fn the_idle_word_is_zero_so_the_detour_takes_the_early_return() {
    let harness = start();
    // No listener and nothing pending: the one word the detour loads is 0.
    assert_eq!(UiFlags::load(), 0);

    let id = listen(&harness, "ui.screen_render", on_render);
    assert_eq!(UiFlags::load(), UiFlags::VIEWS);
    TexturePump::request();
    assert_eq!(UiFlags::load(), UiFlags::VIEWS | UiFlags::DUE);
    // A texture bit written by the pump does not clear the listener bit.
    TexturePump::settle(|| (false, true));
    assert_eq!(UiFlags::load(), UiFlags::VIEWS | UiFlags::WAITING);

    harness.state.events().unlisten(id);
    assert_eq!(UiFlags::load(), UiFlags::WAITING);
    TexturePump::settle(|| (false, false));
    assert_eq!(UiFlags::load(), 0);
}

// What a hover listener saw: (screen, collection, index), and what an unhover listener saw.
static HOVERED: Mutex<Vec<(u64, u32, i32)>> = Mutex::new(Vec::new());
static UNHOVERED: Mutex<Vec<(u64, u32, i32)>> = Mutex::new(Vec::new());

unsafe extern "C" fn on_hover(event: *mut AmmlEventHeader, _: *const AmmlEventContext, _: *mut c_void) -> AmmlStatus {
    // SAFETY: the loader raises `ui.container_slot_hover` with a `ContainerSlotHover`.
    let event = unsafe { &*event.cast::<ContainerSlotHover>() };
    HOVERED.lock().unwrap().push((event.screen, event.slot.collection, event.slot.index));
    AmmlStatus::OK
}

unsafe extern "C" fn on_unhover(event: *mut AmmlEventHeader, _: *const AmmlEventContext, _: *mut c_void) -> AmmlStatus {
    // SAFETY: the loader raises `ui.container_slot_unhover` with a `ContainerSlotUnhover`.
    let event = unsafe { &*event.cast::<ContainerSlotUnhover>() };
    UNHOVERED.lock().unwrap().push((event.screen, event.slot.collection, event.slot.index));
    AmmlStatus::OK
}

// A game `std::string` that holds `text` inline, as the game passes a collection name by reference.
fn name(text: &str) -> MsvcString {
    extern "C" fn no_new(_: usize) -> *mut u8 {
        std::ptr::null_mut()
    }
    extern "C" fn no_delete(_: *mut u8, _: usize) {}
    // SAFETY: a text of at most 15 bytes stays in the inline buffer, so the heap functions are never called.
    unsafe { MsvcString::new_in(&MsvcHeap::new(no_new, no_delete), text) }.unwrap()
}

fn hover_start() -> Harness {
    let harness = start();
    harness.state.events().offer("ui.container_slot_hover", 1);
    harness.state.events().offer("ui.container_slot_unhover", 1);
    harness
}

#[test]
fn a_hover_listener_sets_the_hover_bit_and_an_unlisten_clears_it() {
    let harness = hover_start();
    assert!(!ScreenDetours::hover_wanted());

    let hover = listen(&harness, "ui.container_slot_hover", on_hover);
    assert!(ScreenDetours::hover_wanted());
    assert_eq!(UiFlags::load(), UiFlags::HOVER);
    let unhover = listen(&harness, "ui.container_slot_unhover", on_unhover);

    harness.state.events().unlisten(hover);
    assert!(ScreenDetours::hover_wanted());
    harness.state.events().unlisten(unhover);
    assert!(!ScreenDetours::hover_wanted());
    assert_eq!(UiFlags::load(), 0);
}

#[test]
fn a_hover_of_a_drawn_controller_raises_the_event_with_the_mapped_slot() {
    let harness = hover_start();
    listen(&harness, "ui.container_slot_hover", on_hover);
    listen(&harness, "ui.container_slot_unhover", on_unhover);
    let adapter = ScreenAdapter::get().unwrap();
    let (screen, _) = adapter.book().note(CONTROLLER, VIEW);
    let controller = std::ptr::with_exposed_provenance_mut::<c_void>(CONTROLLER);

    let hotbar = name("hotbar_items");
    adapter.hovered(ScreenDetours::HOVER, controller, std::ptr::from_ref(&hotbar).cast(), 3);
    let armor = name("armor_items");
    adapter.hovered(ScreenDetours::HOVER, controller, std::ptr::from_ref(&armor).cast(), 1);
    let container = name("container_items");
    adapter.hovered(ScreenDetours::UNHOVER, controller, std::ptr::from_ref(&container).cast(), 26);

    assert_eq!(*HOVERED.lock().unwrap(), [(screen, AmmlSlotAt::HOTBAR, 3), (screen, AmmlSlotAt::OTHER, 1)]);
    assert_eq!(*UNHOVERED.lock().unwrap(), [(screen, AmmlSlotAt::CONTAINER, 26)]);
}

#[test]
fn a_hover_before_the_first_draw_gives_the_controller_its_screen_id() {
    let harness = hover_start();
    listen(&harness, "ui.container_slot_hover", on_hover);
    let adapter = ScreenAdapter::get().unwrap();
    let hotbar = name("hotbar_items");
    let fresh = 0x7000;

    // The game hovers the slot under a still pointer in the first frame of a screen, before its first draw.
    adapter.hovered(
        ScreenDetours::HOVER,
        std::ptr::with_exposed_provenance_mut(fresh),
        std::ptr::from_ref(&hotbar).cast(),
        4,
    );

    let seen = HOVERED.lock().unwrap().clone();
    assert_eq!(seen.len(), 1);
    assert_ne!(seen[0].0, 0);
    assert_eq!(seen[0].1, AmmlSlotAt::HOTBAR);
    let (drawn, _) = adapter.book().note(fresh, 0x8000);
    assert_eq!(drawn, seen[0].0);
}

#[test]
fn a_hover_without_a_collection_name_raises_nothing() {
    let harness = hover_start();
    listen(&harness, "ui.container_slot_hover", on_hover);
    let adapter = ScreenAdapter::get().unwrap();

    adapter.hovered(ScreenDetours::HOVER, std::ptr::with_exposed_provenance_mut(CONTROLLER), std::ptr::null(), 0);

    assert!(HOVERED.lock().unwrap().is_empty());
}

#[test]
fn a_hover_with_no_listener_raises_nothing() {
    let harness = hover_start();
    let adapter = ScreenAdapter::get().unwrap();
    adapter.book().note(CONTROLLER, VIEW);
    let hotbar = name("hotbar_items");
    let listener = listen(&harness, "ui.container_slot_hover", on_hover);
    harness.state.events().unlisten(listener);

    adapter.hovered(
        ScreenDetours::HOVER,
        std::ptr::with_exposed_provenance_mut(CONTROLLER),
        std::ptr::from_ref(&hotbar).cast(),
        2,
    );

    assert!(HOVERED.lock().unwrap().is_empty());
}
