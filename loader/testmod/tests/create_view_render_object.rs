use std::ffi::c_void;
use std::sync::atomic::{AtomicUsize, Ordering};

use amml_testmod::create_view_render_object;

static SEEN_THIS: AtomicUsize = AtomicUsize::new(0);
static SEEN_RET: AtomicUsize = AtomicUsize::new(0);
static SEEN_SCREEN: AtomicUsize = AtomicUsize::new(0);
static SEEN_R9: AtomicUsize = AtomicUsize::new(0);

extern "C" fn game_create(this: usize, ret: usize, screen_ctx: usize, sub_client_id: usize) -> usize {
    SEEN_THIS.store(this, Ordering::SeqCst);
    SEEN_RET.store(ret, Ordering::SeqCst);
    SEEN_SCREEN.store(screen_ctx, Ordering::SeqCst);
    SEEN_R9.store(sub_client_id, Ordering::SeqCst);
    0xABCD_0000_1234
}

#[test]
fn the_create_view_render_object_detour_passes_every_argument_on_and_returns_the_original_rax() {
    create_view_render_object::GUARD.original().store(game_create as *mut c_void, Ordering::SeqCst);

    let result = create_view_render_object::detour(0x1000, 0x2000, 0x3000, 0xDEAD_BEEF_CAFE_0001);

    assert_eq!(result, 0xABCD_0000_1234);
    assert_eq!(SEEN_THIS.load(Ordering::SeqCst), 0x1000);
    assert_eq!(SEEN_RET.load(Ordering::SeqCst), 0x2000);
    assert_eq!(SEEN_SCREEN.load(Ordering::SeqCst), 0x3000);
    assert_eq!(SEEN_R9.load(Ordering::SeqCst), 0xDEAD_BEEF_CAFE_0001);
}
