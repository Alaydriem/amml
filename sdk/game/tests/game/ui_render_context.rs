use std::ffi::c_void;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

use amml_game::{
    CaretMeasureDataLayout, EntryId, GameEntries, RectangleAreaLayout, TextMeasureDataLayout, UiColorLayout,
    UiRenderContext,
};

use crate::support::{FakeEntries, FakeVirtual};

const DRAW_TEXT_SLOT: u64 = 5;
const FLUSH_TEXT_SLOT: u64 = 7;
const DRAW_RECTANGLE_SLOT: u64 = 15;
const FILL_RECTANGLE_SLOT: u64 = 16;

// every argument the fakes see, as text, in the order of the call
static SEEN: Mutex<Vec<String>> = Mutex::new(Vec::new());
static CALLS: AtomicUsize = AtomicUsize::new(0);
static FLUSH_OPTIONAL: AtomicUsize = AtomicUsize::new(usize::MAX);
static FLUSH_DELTA: AtomicU32 = AtomicU32::new(0);

fn note(line: String) {
    CALLS.fetch_add(1, Ordering::SeqCst);
    SEEN.lock().unwrap().push(line);
}

extern "C" fn fill_rectangle(
    this: *mut c_void,
    area: *const RectangleAreaLayout,
    color: *const UiColorLayout,
    alpha: f32,
) {
    // SAFETY: the wrapper passes pointers to live values.
    let (area, color) = unsafe { (&*area, &*color) };
    note(format!(
        "fill {:?} {:?} {:?} {:?} {:?} {:?} {:?}",
        this,
        [area.x0, area.x1, area.y0, area.y1],
        color.r,
        color.g,
        color.b,
        color.a,
        alpha
    ));
}

extern "C" fn draw_rectangle(
    this: *mut c_void,
    area: *const RectangleAreaLayout,
    color: *const UiColorLayout,
    alpha: f32,
    thickness: i32,
) {
    // SAFETY: the wrapper passes pointers to live values.
    let (area, color) = unsafe { (&*area, &*color) };
    note(format!(
        "draw {:?} {:?} {:?} {:?} {:?} {:?} {:?}",
        this,
        [area.x0, area.x1, area.y0, area.y1],
        [color.r, color.g, color.b],
        color.a,
        alpha,
        thickness,
        0
    ));
}

#[repr(C)]
struct StringView {
    data: *const u8,
    len: usize,
}

#[allow(clippy::too_many_arguments)]
extern "C" fn draw_text(
    this: *mut c_void,
    font: *mut c_void,
    area: *const RectangleAreaLayout,
    text: *const StringView,
    color: *const UiColorLayout,
    alpha: f32,
    alignment: u32,
    measure: *const TextMeasureDataLayout,
    caret: *const CaretMeasureDataLayout,
) {
    // SAFETY: the wrapper passes pointers to live values and a string view of live bytes.
    let (area, text, color, measure, caret) = unsafe { (&*area, &*text, &*color, &*measure, &*caret) };
    // SAFETY: the view points at the caller's text.
    let text = unsafe { std::str::from_utf8(std::slice::from_raw_parts(text.data, text.len)) }.unwrap();
    note(format!(
        "text {:?} {:?} {:?} {text} {:?} {:?} {alignment} {} {} {}",
        this,
        font,
        [area.x0, area.x1, area.y0, area.y1],
        [color.r, color.g, color.b],
        alpha,
        measure.font_size,
        measure.render_shadow,
        caret.should_render
    ));
}

extern "C" fn flush_text(_: *mut c_void, delta_time: f32, optional: u64) {
    CALLS.fetch_add(1, Ordering::SeqCst);
    FLUSH_DELTA.store(delta_time.to_bits(), Ordering::SeqCst);
    FLUSH_OPTIONAL.store(optional as usize, Ordering::SeqCst);
}

fn entries() -> GameEntries {
    FakeEntries::build(&[
        (EntryId::MINECRAFT_UI_RENDER_CONTEXT_M_SCREEN_CONTEXT, 0x10),
        (EntryId::MINECRAFT_UI_RENDER_CONTEXT_FILL_RECTANGLE, FILL_RECTANGLE_SLOT),
        (EntryId::MINECRAFT_UI_RENDER_CONTEXT_DRAW_RECTANGLE, DRAW_RECTANGLE_SLOT),
        (EntryId::MINECRAFT_UI_RENDER_CONTEXT_DRAW_TEXT, DRAW_TEXT_SLOT),
        (EntryId::MINECRAFT_UI_RENDER_CONTEXT_FLUSH_TEXT, FLUSH_TEXT_SLOT),
    ])
}

fn context() -> FakeVirtual {
    let mut context = FakeVirtual::new(
        0x130,
        &[
            (FILL_RECTANGLE_SLOT, fill_rectangle as *const () as usize),
            (DRAW_RECTANGLE_SLOT, draw_rectangle as *const () as usize),
            (DRAW_TEXT_SLOT, draw_text as *const () as usize),
            (FLUSH_TEXT_SLOT, flush_text as *const () as usize),
        ],
    );
    // MinecraftUIRenderContext::mClient 0x08, mScreenContext 0x10
    context.put::<usize>(0x08, 0xC11E_0000);
    context.put::<usize>(0x10, 0x5C5C_0000);
    context
}

#[test]
fn fill_rect_passes_the_area_the_color_and_the_alpha_in_the_slots_registers() {
    let entries = entries();
    let mut fake = context();
    let ptr = fake.ptr();
    // SAFETY: a live fake whose vtable holds the slots.
    let ctx = unsafe { UiRenderContext::from_raw(ptr, &entries) }.unwrap();

    let called =
        ctx.fill_rect(RectangleAreaLayout::new(1.0, 10.0, 2.0, 20.0), UiColorLayout::rgb(0.25, 0.5, 0.75), 0.5);

    assert_eq!(called, Some(()));
    let expected = format!("fill {ptr:?} [1.0, 10.0, 2.0, 20.0] 0.25 0.5 0.75 1.0 0.5");
    assert_eq!(*SEEN.lock().unwrap(), [expected]);
}

#[test]
fn draw_rect_passes_the_thickness_after_the_alpha() {
    let entries = entries();
    let mut fake = context();
    let ptr = fake.ptr();
    // SAFETY: a live fake whose vtable holds the slots.
    let ctx = unsafe { UiRenderContext::from_raw(ptr, &entries) }.unwrap();

    ctx.draw_rect(RectangleAreaLayout::new(3.0, 30.0, 4.0, 40.0), UiColorLayout::rgb(1.0, 0.0, 0.0), 0.75, 2);

    let expected = format!("draw {ptr:?} [3.0, 30.0, 4.0, 40.0] [1.0, 0.0, 0.0] 1.0 0.75 2 0");
    assert_eq!(*SEEN.lock().unwrap(), [expected]);
}

#[test]
fn draw_text_passes_the_font_the_area_a_string_view_the_color_and_the_stack_arguments() {
    let entries = entries();
    let mut fake = context();
    let ptr = fake.ptr();
    // SAFETY: a live fake whose vtable holds the slots.
    let ctx = unsafe { UiRenderContext::from_raw(ptr, &entries) }.unwrap();
    let font = 0xF047usize as *mut c_void;
    let text = String::from("x26");

    // SAFETY: the text lives past the end of the test, which is past the (absent) flush.
    let called = unsafe {
        ctx.draw_text(
            font,
            RectangleAreaLayout::new(5.0, 50.0, 6.0, 60.0),
            &text,
            UiColorLayout::rgb(1.0, 1.0, 1.0),
            0.5,
            2,
            TextMeasureDataLayout::new(1.0, true),
        )
    };

    assert_eq!(called, Some(()));
    let expected = format!("text {ptr:?} {font:?} [5.0, 50.0, 6.0, 60.0] x26 [1.0, 1.0, 1.0] 0.5 2 1 true false");
    assert_eq!(*SEEN.lock().unwrap(), [expected]);
}

#[test]
fn flush_text_passes_the_delta_and_an_empty_optional() {
    let entries = entries();
    let mut fake = context();
    // SAFETY: a live fake whose vtable holds the slots.
    let ctx = unsafe { UiRenderContext::from_raw(fake.ptr(), &entries) }.unwrap();

    assert_eq!(ctx.flush_text(0.125), Some(()));

    assert_eq!(f32::from_bits(FLUSH_DELTA.load(Ordering::SeqCst)), 0.125);
    assert_eq!(FLUSH_OPTIONAL.load(Ordering::SeqCst), 0);
}

#[test]
fn client_and_screen_context_are_read_from_the_context() {
    let entries = entries();
    let mut fake = context();
    // SAFETY: a live fake.
    let ctx = unsafe { UiRenderContext::from_raw(fake.ptr(), &entries) }.unwrap();

    assert_eq!(ctx.client().map(|client| client.as_ptr() as usize), Some(0xC11E_0000));
    assert_eq!(ctx.screen_context().map(|screen| screen as usize), Some(0x5C5C_0000));

    fake.put::<usize>(0x08, 0);
    // SAFETY: a live fake.
    let ctx = unsafe { UiRenderContext::from_raw(fake.ptr(), &entries) }.unwrap();
    assert!(ctx.client().is_none());
}

#[test]
fn with_the_entries_off_every_call_gives_none_and_calls_nothing() {
    let entries = FakeEntries::build(&[]);
    let mut fake = context();
    // SAFETY: a live fake; with no entry the vtable is never read.
    let ctx = unsafe { UiRenderContext::from_raw(fake.ptr(), &entries) }.unwrap();

    assert_eq!(
        ctx.fill_rect(RectangleAreaLayout::new(0.0, 1.0, 0.0, 1.0), UiColorLayout::rgb(0.0, 0.0, 0.0), 1.0),
        None
    );
    assert_eq!(
        ctx.draw_rect(RectangleAreaLayout::new(0.0, 1.0, 0.0, 1.0), UiColorLayout::rgb(0.0, 0.0, 0.0), 1.0, 1),
        None
    );
    assert_eq!(ctx.flush_text(0.0), None);
    assert!(ctx.client().is_none());
    assert!(ctx.screen_context().is_none());
    assert_eq!(CALLS.load(Ordering::SeqCst), 0);
}
