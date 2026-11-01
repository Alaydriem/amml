use std::ffi::c_void;

use amml_game::RenderCamera;

use crate::support::{FakeCamera, FakeEntries, FakeViewObject};

#[test]
fn forward_and_view_position_read_the_vectors_at_their_offsets() {
    let entries = FakeCamera::entries();
    let reader = RenderCamera::new(&entries);
    let camera = FakeCamera::new([0.0, -1.0, 0.5]);
    let view = FakeViewObject::new([12.0, 70.5, -4.0]);

    // SAFETY: the fakes are live and have the layouts the entries name.
    unsafe {
        assert_eq!(reader.forward((&raw const camera).cast::<c_void>()), Some([0.0, -1.0, 0.5]));
        assert_eq!(reader.view_position((&raw const view).cast::<c_void>()), Some([12.0, 70.5, -4.0]));
    }
}

#[test]
fn a_null_pointer_or_a_missing_entry_gives_none() {
    let entries = FakeCamera::entries();
    let reader = RenderCamera::new(&entries);
    let none = FakeEntries::build(&[]);
    let off = RenderCamera::new(&none);
    let camera = FakeCamera::new([0.0, -1.0, 0.5]);

    // SAFETY: null is allowed; the other calls stop at the missing entry before any read.
    unsafe {
        assert_eq!(reader.forward(std::ptr::null()), None);
        assert_eq!(reader.view_position(std::ptr::null()), None);
        assert_eq!(off.forward((&raw const camera).cast::<c_void>()), None);
        assert_eq!(off.view_position((&raw const camera).cast::<c_void>()), None);
    }
}
