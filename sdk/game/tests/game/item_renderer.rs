use std::ffi::c_void;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

use amml_game::{BaseActorRenderContext, EntryId, GameEntries, ItemRenderer, ItemStackBaseRef};

use crate::support::{FakeEntries, GameMemory};

static SEEN: Mutex<Vec<String>> = Mutex::new(Vec::new());
static CALLS: AtomicUsize = AtomicUsize::new(0);

// ItemRenderer::renderGuiItemNew: this, the context, the stack and the frame in the registers; x, y, foil, alpha,
// light, scale and the z order on the stack
#[allow(clippy::too_many_arguments)]
extern "C" fn render_gui_item_new(
    this: *mut c_void,
    context: *mut c_void,
    stack: *const c_void,
    frame: i32,
    x: f32,
    y: f32,
    foil: u8,
    alpha: f32,
    light: f32,
    scale: f32,
    z_order: i32,
) {
    CALLS.fetch_add(1, Ordering::SeqCst);
    SEEN.lock()
        .unwrap()
        .push(format!("{this:?} {context:?} {stack:?} {frame} {x} {y} {foil} {alpha} {light} {scale} {z_order}"));
}

extern "C" fn ctor(this: *mut c_void, _: *mut c_void, _: *mut c_void, _: *mut c_void) -> *mut c_void {
    // BaseActorRenderContext::mItemRenderer is at 0x58
    // SAFETY: the buffer is the 0xC0 bytes the wrapper allocated.
    unsafe { this.cast::<u8>().add(0x58).cast::<usize>().write(0x1713) };
    this
}

extern "C" fn dtor(_: *mut c_void) {}

fn entries(with_render: bool) -> GameEntries {
    let mut values = vec![
        (EntryId::BASE_ACTOR_RENDER_CONTEXT_CTOR, ctor as *const () as usize as u64),
        (EntryId::BASE_ACTOR_RENDER_CONTEXT_DTOR, dtor as *const () as usize as u64),
        (EntryId::BASE_ACTOR_RENDER_CONTEXT_SIZE, 0xC0),
    ];
    if with_render {
        values.push((EntryId::ITEM_RENDERER_RENDER_GUI_ITEM_NEW, render_gui_item_new as *const () as usize as u64));
    }
    FakeEntries::build(&values)
}

#[test]
fn render_gui_item_passes_each_argument_in_the_order_of_the_game_function() {
    let entries = entries(true);
    let mut stack_memory = GameMemory::new(0x80);
    let stack_ptr = stack_memory.ptr();
    // SAFETY: the fake game accepts any pointer for the screen context, the client and the game.
    let context = unsafe {
        BaseActorRenderContext::new(&entries, 0x5C5C as *mut c_void, 0xC11E as *mut c_void, 0x6A3E as *mut c_void)
    }
    .unwrap();
    let renderer = context.item_renderer().unwrap();
    // SAFETY: a live fake stack.
    let stack = unsafe { ItemStackBaseRef::from_raw(stack_ptr, &entries) }.unwrap();

    let called = renderer.render_gui_item(&context, &stack, 7, 11.5, 22.5, true, 0.5, 2.0, 300);

    assert_eq!(called, Some(()));
    // the game's light argument is fixed at 1.0, as the game's own caller passes it
    let expected = format!("{:?} {:?} {stack_ptr:?} 7 11.5 22.5 1 0.5 1 2 300", renderer.as_ptr(), context.as_ptr());
    assert_eq!(*SEEN.lock().unwrap(), [expected]);
}

#[test]
fn render_gui_item_with_its_entry_off_gives_none_and_calls_nothing() {
    let entries = entries(false);
    let mut stack_memory = GameMemory::new(0x80);
    // SAFETY: the fake game accepts any pointer.
    let context = unsafe {
        BaseActorRenderContext::new(&entries, 0x5C5C as *mut c_void, 0xC11E as *mut c_void, 0x6A3E as *mut c_void)
    }
    .unwrap();
    let renderer: ItemRenderer = context.item_renderer().unwrap();
    // SAFETY: a live fake stack.
    let stack = unsafe { ItemStackBaseRef::from_raw(stack_memory.ptr(), &entries) }.unwrap();

    assert_eq!(renderer.render_gui_item(&context, &stack, 0, 0.0, 0.0, false, 1.0, 1.0, 0), None);
    assert_eq!(CALLS.load(Ordering::SeqCst), 0);
}
