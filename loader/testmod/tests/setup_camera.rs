use std::ffi::c_void;
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

use amml_testmod::setup_camera;

static SEEN_THIS: AtomicUsize = AtomicUsize::new(0);
static SEEN_CAMERA: AtomicUsize = AtomicUsize::new(0);
static SEEN_FLOAT_BITS: AtomicU32 = AtomicU32::new(0);

extern "C" fn game_setup_camera(this: usize, camera: usize, value: f32) {
    SEEN_THIS.store(this, Ordering::SeqCst);
    SEEN_CAMERA.store(camera, Ordering::SeqCst);
    SEEN_FLOAT_BITS.store(value.to_bits(), Ordering::SeqCst);
}

#[test]
fn the_setup_camera_detour_passes_the_float_and_both_pointers_to_the_original_unchanged() {
    setup_camera::GUARD.original().store(game_setup_camera as *mut c_void, Ordering::SeqCst);

    for bits in [0x3f8c_cccd_u32, 0x7fc0_1234, 0x8000_0001] {
        setup_camera::detour(0x1000, 0x2000, f32::from_bits(bits));

        assert_eq!(SEEN_FLOAT_BITS.load(Ordering::SeqCst), bits);
        assert_eq!(SEEN_THIS.load(Ordering::SeqCst), 0x1000);
        assert_eq!(SEEN_CAMERA.load(Ordering::SeqCst), 0x2000);
    }
}
