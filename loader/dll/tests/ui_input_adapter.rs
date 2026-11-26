use std::ffi::c_void;
use std::hint::black_box;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU8, Ordering};

use amml::{LoaderState, ModListener, ScreenAdapter, SessionAdapter, UiFlags, UiInputAdapter};
use amml_abi::{
    AmmlBand, AmmlEventContext, AmmlEventHeader, AmmlInputMode, AmmlStatus, ItemHovertext, UiPointer, UiScroll,
};
use amml_data::{EntryKind, Resolved};

use crate::support::{Harness, SessionGame};

// The offsets of the stand-in view and scene.
const CONTROLLER_AT: usize = 0x40;
const IDS_AT: usize = 0x38;
const POINTER_AT: usize = 0x20;
const MODE_AT: usize = 0x80;
const SCENE_VIEW_AT: usize = 0x40;
const CONTROLLER: usize = 0x5000;
const OTHER_CONTROLLER: usize = 0x5100;
// The pointer button ids of the stand-in views.
const PRIMARY: u32 = 100;
const SECOND: u32 = 101;

// What the stand-in game functions got.
static BUTTONS: Mutex<Vec<(u32, u8)>> = Mutex::new(Vec::new());
static DIRECTIONS: Mutex<Vec<(u8, f32)>> = Mutex::new(Vec::new());
static HOVERTEXTS: Mutex<Vec<&'static str>> = Mutex::new(Vec::new());
static GLOBAL: [u64; 2] = [0; 2];

#[inline(never)]
extern "C" fn button_stand_in(this: *mut c_void, id: u32, state: u8, impact: u8) {
    black_box((this, impact));
    BUTTONS.lock().unwrap().push((id, black_box(state)));
}

#[inline(never)]
extern "C" fn direction_stand_in(this: *mut c_void, direction: u8, x: f32, y: f32, impact: u8) {
    black_box((this, x, impact));
    DIRECTIONS.lock().unwrap().push((black_box(direction), y));
}

#[inline(never)]
extern "C" fn override_stand_in(this: *mut c_void, stack: *const c_void, level: *mut c_void, text: *mut c_void, b: u8) {
    black_box((this, stack, level, text, b));
    HOVERTEXTS.lock().unwrap().push(black_box("override"));
}

#[inline(never)]
extern "C" fn base_stand_in(this: *mut c_void, stack: *const c_void, level: *mut c_void, text: *mut c_void, b: u8) {
    black_box((this, stack, level, text, b));
    HOVERTEXTS.lock().unwrap().push(black_box("base"));
}

fn entries() -> Vec<(&'static str, Resolved)> {
    let function = |address: usize| Resolved {
        kind: EntryKind::Function,
        value: address as u64 - Harness::BASE,
        recipes_agreeing: 2,
    };
    let field = |value| Resolved { kind: EntryKind::Field, value, recipes_agreeing: 2 };
    let mut all = SessionGame::entries(&[]);
    all.extend([
        ("ScreenView.handleButtonEvent", function(button_stand_in as *const () as usize)),
        ("ScreenView.mPointerButtonIds", field(IDS_AT as u64)),
        ("ScreenView.mPointerLocationPrevious", field(POINTER_AT as u64)),
        ("ScreenView.mInputMode", field(MODE_AT as u64)),
        ("ScreenView.mController", field(CONTROLLER_AT as u64)),
        ("UIScene.handleDirection", function(direction_stand_in as *const () as usize)),
        ("UIScene.mScreenView", field(SCENE_VIEW_AT as u64)),
        (
            "UIScene.vtable",
            Resolved {
                kind: EntryKind::Global,
                value: std::ptr::from_ref(&GLOBAL) as usize as u64 - Harness::BASE,
                recipes_agreeing: 2,
            },
        ),
        ("ShulkerBoxBlockItem.appendFormattedHovertext", function(override_stand_in as *const () as usize)),
        ("Item.appendFormattedHovertext", function(base_stand_in as *const () as usize)),
    ]);
    all
}

fn start() -> Harness {
    let harness = Harness::start_with_entries(&entries());
    SessionAdapter::install(harness.state);
    ScreenAdapter::install(harness.state);
    let rows = UiInputAdapter::install(harness.state);
    assert!(rows.iter().all(|row| row.shape.is_some()), "{rows:?}");
    SessionGame::frame();
    BUTTONS.lock().unwrap().clear();
    DIRECTIONS.lock().unwrap().clear();
    HOVERTEXTS.lock().unwrap().clear();
    harness
}

fn listen(harness: &Harness, event: &str, callback: amml_abi::EventCallbackFn) -> amml_runtime::ListenerId {
    let listener = ModListener { owner: 0, callback, user: 0 };
    harness.state.events().listen(event, 1, AmmlBand::NORMAL, listener).unwrap()
}

// A stand-in `ScreenView`: the controller, the two pointer button ids, the pointer position and the input mode.
struct View(&'static mut [u64; 128]);

impl View {
    fn new(controller: usize, mode: u32) -> Self {
        let memory = Box::leak(Box::new([0u64; 128]));
        memory[CONTROLLER_AT / 8] = controller as u64;
        memory[IDS_AT / 8] = u64::from(PRIMARY) | (u64::from(SECOND) << 32);
        memory[POINTER_AT / 8] = u64::from(3.0f32.to_bits()) | (u64::from(4.0f32.to_bits()) << 32);
        memory[MODE_AT / 8] = u64::from(mode);
        Self(memory)
    }

    fn address(&self) -> *mut c_void {
        std::ptr::from_ref(&*self.0).cast_mut().cast()
    }
}

// The hooked game function `ScreenView.handleButtonEvent`, as the game calls it.
fn press(view: &View, id: u32, pressed: bool) {
    let function: extern "C" fn(*mut c_void, u32, u8, u8) = black_box(button_stand_in);
    function(view.address(), id, u8::from(pressed), 0);
}

fn direction(scene: *mut c_void, id: u8, y: f32) {
    let function: extern "C" fn(*mut c_void, u8, f32, f32, u8) = black_box(direction_stand_in);
    function(scene, id, 0.0, y, 0);
}

fn hovertext() {
    let function: extern "C" fn(*mut c_void, *const c_void, *mut c_void, *mut c_void, u8) =
        black_box(override_stand_in);
    function(std::ptr::null_mut(), std::ptr::null(), std::ptr::null_mut(), std::ptr::null_mut(), 0);
}

// What the pointer listener saw, and what it does: 0 passes, 1 cancels every down, 2 cancels every event.
// The screen, view, pointer, pressed, input mode, x, y and whether the event was cancelled on entry.
type Seen = (u64, u64, u8, u8, u8, f32, f32, bool);
static SEEN: Mutex<Vec<Seen>> = Mutex::new(Vec::new());
static ACTION: AtomicU8 = AtomicU8::new(0);

unsafe extern "C" fn on_pointer(event: *mut AmmlEventHeader, _: *const AmmlEventContext, _: *mut c_void) -> AmmlStatus {
    // SAFETY: the loader raises `input.ui_pointer` with a `UiPointer`.
    let event = unsafe { &mut *event.cast::<UiPointer>() };
    SEEN.lock().unwrap().push((
        event.screen,
        event.view,
        event.pointer,
        event.pressed,
        event.input_mode,
        event.x,
        event.y,
        event.header.cancelled != 0,
    ));
    let action = ACTION.load(Ordering::SeqCst);
    if action == 2 || (action == 1 && event.pressed != 0) {
        event.header.cancelled = 1;
    }
    // A listener that edits the loader's fields changes nothing for the next listener.
    event.pointer = 9;
    event.x = -1.0;
    AmmlStatus::OK
}

unsafe extern "C" fn on_pointer_again(
    event: *mut AmmlEventHeader,
    _: *const AmmlEventContext,
    _: *mut c_void,
) -> AmmlStatus {
    // SAFETY: the loader raises `input.ui_pointer` with a `UiPointer`.
    let event = unsafe { &*event.cast::<UiPointer>() };
    SEEN.lock().unwrap().push((0, 0, event.pointer, 7, 0, event.x, 0.0, false));
    AmmlStatus::OK
}

#[test]
fn a_cancelled_pointer_down_does_not_reach_the_game_and_its_up_stays_cancelled() {
    let harness = start();
    ACTION.store(1, Ordering::SeqCst);
    listen(&harness, "input.ui_pointer", on_pointer);
    let view = View::new(CONTROLLER, 1);

    press(&view, PRIMARY, true);
    press(&view, PRIMARY, false);

    assert!(BUTTONS.lock().unwrap().is_empty());
    let seen = SEEN.lock().unwrap().clone();
    assert_eq!(seen.len(), 2);
    let (screen, view_id, pointer, pressed, mode, x, y, cancelled) = seen[0];
    assert_ne!(screen, 0);
    assert_ne!(view_id, 0);
    assert_eq!((pointer, pressed, mode, x, y, cancelled), (0, 1, AmmlInputMode::MOUSE.0, 3.0, 4.0, false));
    // The up of the cancelled down is raised already cancelled.
    assert_eq!((seen[1].2, seen[1].3, seen[1].7), (0, 0, true));
    assert_eq!(seen[1].1, view_id);
}

#[test]
fn the_second_pointer_and_other_ids() {
    let harness = start();
    ACTION.store(1, Ordering::SeqCst);
    listen(&harness, "input.ui_pointer", on_pointer);
    let view = View::new(CONTROLLER, 3);

    // The second pointer goes through with its own index; the listener cancels every down.
    press(&view, SECOND, false);
    // An id that is not a pointer button goes to the game with no event.
    press(&view, 5, true);
    press(&view, 0, true);

    assert_eq!(*BUTTONS.lock().unwrap(), [(SECOND, 0), (5, 1), (0, 1)]);
    let seen = SEEN.lock().unwrap().clone();
    assert_eq!(seen.len(), 1);
    assert_eq!((seen[0].2, seen[0].3, seen[0].4), (1, 0, AmmlInputMode::GAME_PAD.0));
}

#[test]
fn a_new_down_clears_an_up_held_back_by_a_lost_release() {
    let harness = start();
    ACTION.store(1, Ordering::SeqCst);
    listen(&harness, "input.ui_pointer", on_pointer);
    let view = View::new(CONTROLLER, 1);

    // The first down is cancelled and its up never comes (a focus change). The next down passes.
    press(&view, PRIMARY, true);
    ACTION.store(0, Ordering::SeqCst);
    press(&view, PRIMARY, true);
    press(&view, PRIMARY, false);

    assert_eq!(*BUTTONS.lock().unwrap(), [(PRIMARY, 1), (PRIMARY, 0)]);
}

#[test]
fn a_cancel_of_an_up_whose_down_reached_the_game_is_ignored() {
    let harness = start();
    ACTION.store(0, Ordering::SeqCst);
    listen(&harness, "input.ui_pointer", on_pointer);
    let view = View::new(CONTROLLER, 1);

    press(&view, PRIMARY, true);
    ACTION.store(2, Ordering::SeqCst);
    press(&view, PRIMARY, false);

    assert_eq!(*BUTTONS.lock().unwrap(), [(PRIMARY, 1), (PRIMARY, 0)]);
}

#[test]
fn every_view_of_the_stack_gets_its_own_event() {
    let harness = start();
    ACTION.store(0, Ordering::SeqCst);
    listen(&harness, "input.ui_pointer", on_pointer);
    let first = View::new(CONTROLLER, 1);
    let second = View::new(OTHER_CONTROLLER, 1);

    press(&first, PRIMARY, true);
    press(&second, PRIMARY, true);

    let seen = SEEN.lock().unwrap().clone();
    assert_eq!(seen.len(), 2);
    assert_ne!(seen[0].0, seen[1].0);
    assert_ne!(seen[0].1, seen[1].1);
    assert_eq!(BUTTONS.lock().unwrap().len(), 2);
}

#[test]
fn the_loaders_fields_are_restored_after_every_listener() {
    let harness = start();
    ACTION.store(0, Ordering::SeqCst);
    listen(&harness, "input.ui_pointer", on_pointer);
    listen(&harness, "input.ui_pointer", on_pointer_again);
    let view = View::new(CONTROLLER, 1);

    press(&view, PRIMARY, true);

    let seen = SEEN.lock().unwrap().clone();
    assert_eq!(seen.len(), 2);
    assert_eq!((seen[1].2, seen[1].5), (0, 3.0));
}

#[test]
fn a_pointer_without_a_listener_costs_one_word_and_reaches_the_game() {
    let harness = start();
    assert_eq!(UiFlags::load(), 0);
    let view = View::new(CONTROLLER, 1);
    press(&view, PRIMARY, true);
    assert_eq!(*BUTTONS.lock().unwrap(), [(PRIMARY, 1)]);

    let id = listen(&harness, "input.ui_pointer", on_pointer);
    assert_eq!(UiFlags::load(), UiFlags::POINTER);
    harness.state.events().unlisten(id);
    assert_eq!(UiFlags::load(), 0);
    press(&view, PRIMARY, false);
    assert_eq!(BUTTONS.lock().unwrap().len(), 2);
    assert!(SEEN.lock().unwrap().is_empty());
}

// What the scroll listener saw: the screen and the notches; it cancels a negative step.
static SCROLLS: Mutex<Vec<(u64, i32)>> = Mutex::new(Vec::new());

unsafe extern "C" fn on_scroll(event: *mut AmmlEventHeader, _: *const AmmlEventContext, _: *mut c_void) -> AmmlStatus {
    // SAFETY: the loader raises `input.ui_scroll` with a `UiScroll`.
    let event = unsafe { &mut *event.cast::<UiScroll>() };
    SCROLLS.lock().unwrap().push((event.screen, event.notches));
    if event.notches < 0 {
        event.header.cancelled = 1;
    }
    AmmlStatus::OK
}

// A stand-in scene whose `ScreenView` pointer is at `SCENE_VIEW_AT`.
fn scene(view: Option<&View>) -> *mut c_void {
    let memory = Box::leak(Box::new([0u64; 16]));
    memory[SCENE_VIEW_AT / 8] = view.map_or(0, |view| view.address() as u64);
    std::ptr::from_mut(memory).cast()
}

#[test]
fn a_scroll_step_raises_one_event_and_a_cancel_eats_it() {
    let harness = start();
    listen(&harness, "input.ui_scroll", on_scroll);
    let view = View::new(CONTROLLER, 1);
    let scene = scene(Some(&view));

    direction(scene, 2, -0.9375);
    direction(scene, 2, 0.9448819);

    // Only the positive step reached the game.
    assert_eq!(*DIRECTIONS.lock().unwrap(), [(2, 0.9448819)]);
    let seen = SCROLLS.lock().unwrap().clone();
    assert_eq!(seen.len(), 2);
    assert_ne!(seen[0].0, 0);
    assert_eq!(seen[0].0, seen[1].0);
    assert_eq!((seen[0].1, seen[1].1), (-1, 1));
    // The screen id is the one a pointer event of the same controller gets.
    assert_eq!(ScreenAdapter::get().unwrap().book().screen_of_controller(CONTROLLER), Some(seen[0].0));
}

#[test]
fn other_directions_and_no_step_raise_nothing() {
    let harness = start();
    SCROLLS.lock().unwrap().clear();
    listen(&harness, "input.ui_scroll", on_scroll);
    let scene = scene(None);

    direction(scene, 0, -1.0);
    direction(scene, 2, 0.0);
    direction(scene, 2, f32::NAN);
    // A scene with no view still raises, with no screen.
    direction(scene, 2, 1.0);

    assert_eq!(DIRECTIONS.lock().unwrap().len(), 4);
    assert_eq!(*SCROLLS.lock().unwrap(), [(0, 1)]);
}

// What the hover text listener does: 0 leaves the lines, 1 hides them, 2 writes the loader's fields.
static HIDE: AtomicU8 = AtomicU8::new(0);
static KINDS: Mutex<Vec<u32>> = Mutex::new(Vec::new());

unsafe extern "C" fn on_hovertext(
    event: *mut AmmlEventHeader,
    _: *const AmmlEventContext,
    _: *mut c_void,
) -> AmmlStatus {
    // SAFETY: the loader raises `ui.item_hovertext` with an `ItemHovertext`.
    let event = unsafe { &mut *event.cast::<ItemHovertext>() };
    KINDS.lock().unwrap().push(event.kind);
    match HIDE.load(Ordering::SeqCst) {
        1 => event.contents_lines = 0,
        2 => {
            event.kind = 9;
            event._reserved = [1; 3];
        }
        _ => {}
    }
    AmmlStatus::OK
}

#[test]
fn hovertext_hide_runs_the_base_function_in_place_of_the_override() {
    let harness = start();
    HIDE.store(1, Ordering::SeqCst);
    listen(&harness, "ui.item_hovertext", on_hovertext);

    hovertext();

    assert_eq!(*HOVERTEXTS.lock().unwrap(), ["base"]);
    assert_eq!(*KINDS.lock().unwrap(), [ItemHovertext::CONTAINER_ITEM]);
}

#[test]
fn hovertext_without_a_hide_runs_the_override() {
    let harness = start();
    HIDE.store(0, Ordering::SeqCst);
    assert_eq!(UiFlags::load(), 0);
    hovertext();
    let id = listen(&harness, "ui.item_hovertext", on_hovertext);
    assert_eq!(UiFlags::load(), UiFlags::HOVERTEXT);
    hovertext();
    harness.state.events().unlisten(id);
    assert_eq!(UiFlags::load(), 0);

    assert_eq!(*HOVERTEXTS.lock().unwrap(), ["override", "override"]);
}

#[test]
fn a_listener_cannot_change_the_kind_for_the_next_one() {
    let harness = start();
    HIDE.store(2, Ordering::SeqCst);
    listen(&harness, "ui.item_hovertext", on_hovertext);
    listen(&harness, "ui.item_hovertext", on_hovertext);

    hovertext();

    assert_eq!(*KINDS.lock().unwrap(), [ItemHovertext::CONTAINER_ITEM, ItemHovertext::CONTAINER_ITEM]);
    assert_eq!(*HOVERTEXTS.lock().unwrap(), ["override"]);
}

#[test]
fn the_rows_are_offered_when_their_hooks_are_in() {
    let harness = start();

    let state: &LoaderState = harness.state;
    for id in ["input.ui_pointer", "input.ui_scroll", "ui.item_hovertext"] {
        assert!(state.events().is_available(id, 1), "{id}");
    }
}

#[test]
fn a_hold_back_does_not_outlive_the_last_listener() {
    let harness = start();
    ACTION.store(1, Ordering::SeqCst);
    let first = listen(&harness, "input.ui_pointer", on_pointer);
    let view = View::new(CONTROLLER, 1);

    press(&view, PRIMARY, true);
    harness.state.events().unlisten(first);
    // No listener: the next press reaches the game without the gate.
    press(&view, PRIMARY, false);
    press(&view, PRIMARY, true);
    ACTION.store(0, Ordering::SeqCst);
    listen(&harness, "input.ui_pointer", on_pointer);
    press(&view, PRIMARY, false);

    assert_eq!(*BUTTONS.lock().unwrap(), [(PRIMARY, 0), (PRIMARY, 1), (PRIMARY, 0)]);
}
