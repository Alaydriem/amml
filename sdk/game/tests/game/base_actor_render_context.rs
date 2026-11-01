use std::ffi::c_void;
use std::sync::atomic::{AtomicUsize, Ordering};

use amml_game::{BaseActorRenderContext, EntryId, GameEntries, UiRenderContext};

use crate::support::{FakeEntries, FakeVirtual};

static CTOR_ARGUMENTS: [AtomicUsize; 4] =
    [AtomicUsize::new(0), AtomicUsize::new(0), AtomicUsize::new(0), AtomicUsize::new(0)];
static CTOR_CALLS: AtomicUsize = AtomicUsize::new(0);
static DTOR_CALLS: AtomicUsize = AtomicUsize::new(0);
static DTOR_THIS: AtomicUsize = AtomicUsize::new(0);

// BaseActorRenderContext(ScreenContext&, IClientInstance&, IMinecraftGame&): this, screen context, client, game
extern "C" fn ctor(this: *mut c_void, screen: *mut c_void, client: *mut c_void, game: *mut c_void) -> *mut c_void {
    CTOR_CALLS.fetch_add(1, Ordering::SeqCst);
    for (slot, value) in CTOR_ARGUMENTS.iter().zip([this, screen, client, game]) {
        slot.store(value as usize, Ordering::SeqCst);
    }
    // SAFETY: the buffer is the 0xC0 bytes the wrapper allocated; mItemRenderer is at 0x58
    unsafe { this.cast::<u8>().add(0x58).cast::<usize>().write(0x1713) };
    this
}

extern "C" fn dtor(this: *mut c_void) {
    DTOR_CALLS.fetch_add(1, Ordering::SeqCst);
    DTOR_THIS.store(this as usize, Ordering::SeqCst);
}

extern "C" fn get_minecraft_game(_: *mut c_void) -> *mut c_void {
    0x6A3E as *mut c_void
}

fn entries(size: u64) -> GameEntries {
    FakeEntries::build(&[
        (EntryId::BASE_ACTOR_RENDER_CONTEXT_CTOR, ctor as *const () as usize as u64),
        (EntryId::BASE_ACTOR_RENDER_CONTEXT_DTOR, dtor as *const () as usize as u64),
        (EntryId::BASE_ACTOR_RENDER_CONTEXT_SIZE, size),
        (EntryId::CLIENT_INSTANCE_GET_MINECRAFT_GAME, 78),
        (EntryId::MINECRAFT_UI_RENDER_CONTEXT_M_SCREEN_CONTEXT, 0x10),
    ])
}

#[test]
fn new_calls_the_ctor_on_a_buffer_of_the_entry_size_and_drop_calls_the_dtor_once() {
    let entries = entries(0xC0);

    // SAFETY: the fake game accepts any pointer.
    let context = unsafe {
        BaseActorRenderContext::new(&entries, 0x5C5C as *mut c_void, 0xC11E as *mut c_void, 0x6A3E as *mut c_void)
    }
    .unwrap();

    let this = context.as_ptr() as usize;
    assert_eq!(this % 16, 0);
    assert_eq!(CTOR_CALLS.load(Ordering::SeqCst), 1);
    let seen: Vec<usize> = CTOR_ARGUMENTS.iter().map(|value| value.load(Ordering::SeqCst)).collect();
    assert_eq!(seen, [this, 0x5C5C, 0xC11E, 0x6A3E]);
    assert_eq!(context.item_renderer().map(|renderer| renderer.as_ptr() as usize), Some(0x1713));
    assert_eq!(DTOR_CALLS.load(Ordering::SeqCst), 0);

    drop(context);

    assert_eq!(DTOR_CALLS.load(Ordering::SeqCst), 1);
    assert_eq!(DTOR_THIS.load(Ordering::SeqCst), this);
}

#[test]
fn a_size_too_small_for_the_item_renderer_field_or_absurd_gives_none_and_calls_nothing() {
    for size in [0u64, 0x58, 0x5F, 1 << 20] {
        let entries = entries(size);

        // SAFETY: the fake game accepts any pointer.
        let context = unsafe {
            BaseActorRenderContext::new(&entries, 0x5C5C as *mut c_void, 0xC11E as *mut c_void, 0x6A3E as *mut c_void)
        };

        assert!(context.is_none(), "{size:#x}");
    }
    assert_eq!(CTOR_CALLS.load(Ordering::SeqCst), 0);
}

#[test]
fn new_with_a_null_client_or_screen_context_gives_none() {
    let entries = entries(0xC0);

    // SAFETY: null arguments are refused before any call.
    assert!(
        unsafe {
            BaseActorRenderContext::new(&entries, std::ptr::null_mut(), 0x10 as *mut c_void, 0x10 as *mut c_void)
        }
        .is_none()
    );
    // SAFETY: as above.
    assert!(
        unsafe {
            BaseActorRenderContext::new(&entries, 0x10 as *mut c_void, std::ptr::null_mut(), 0x10 as *mut c_void)
        }
        .is_none()
    );
    // SAFETY: as above.
    assert!(
        unsafe {
            BaseActorRenderContext::new(&entries, 0x10 as *mut c_void, 0x10 as *mut c_void, std::ptr::null_mut())
        }
        .is_none()
    );
    assert_eq!(CTOR_CALLS.load(Ordering::SeqCst), 0);
}

#[test]
fn for_ui_takes_the_screen_context_and_client_of_the_ui_context_and_the_game_of_the_client() {
    let entries = entries(0xC0);
    let mut client = FakeVirtual::new(0x20, &[(78, get_minecraft_game as *const () as usize)]);
    let mut ui = FakeVirtual::new(0x130, &[]);
    ui.put::<usize>(0x08, client.ptr() as usize);
    ui.put::<usize>(0x10, 0x5C5C);
    // SAFETY: a live fake context.
    let ui = unsafe { UiRenderContext::from_raw(ui.ptr(), &entries) }.unwrap();

    let context = BaseActorRenderContext::for_ui(&ui).unwrap();

    let seen: Vec<usize> = CTOR_ARGUMENTS.iter().map(|value| value.load(Ordering::SeqCst)).collect();
    assert_eq!(seen, [context.as_ptr() as usize, 0x5C5C, client.ptr() as usize, 0x6A3E]);
}
