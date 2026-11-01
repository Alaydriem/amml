use std::ffi::c_void;

use amml_game::{EntryId, Projection};

use crate::support::FakeEntries;

/// A camera whose projection stack, at 0x80, is a one-block deque that holds one matrix.
#[repr(C)]
struct FakeCamera {
    pad: [u64; 16],
    deque: [usize; 5],
}

fn read(matrix: &mut [f32; 16]) -> Option<(f32, f32)> {
    let mut map = [matrix as *mut [f32; 16]];
    let mut camera = FakeCamera { pad: [0; 16], deque: [0, map.as_mut_ptr() as usize, 1, 0, 1] };
    let entries = FakeEntries::build(&[(EntryId::CAMERA_FORWARD, 0x118)]);
    // SAFETY: a live fake camera with the layout the entries name.
    unsafe { Projection::read(&entries, (&raw mut camera).cast::<c_void>()) }
}

#[test]
fn reads_fov_and_aspect_from_a_perspective_matrix() {
    let fov: f32 = 70.0;
    let aspect: f32 = 16.0 / 9.0;
    let focal = 1.0 / (fov.to_radians() / 2.0).tan();
    let mut matrix = [0.0f32; 16];
    matrix[0] = focal / aspect;
    matrix[5] = focal;
    matrix[10] = -1.0;
    matrix[11] = -1.0;

    let (read_fov, read_aspect) = read(&mut matrix).unwrap();
    assert!((read_fov - 70.0).abs() < 1e-4, "{read_fov}");
    assert!((read_aspect - aspect).abs() < 1e-4, "{read_aspect}");
}

#[test]
fn a_zero_matrix_gives_zero() {
    assert_eq!(read(&mut [0.0; 16]), Some((0.0, 0.0)));
}

#[test]
fn a_missing_or_moved_anchor_or_a_null_camera_gives_none() {
    let missing = FakeEntries::build(&[]);
    let moved = FakeEntries::build(&[(EntryId::CAMERA_FORWARD, 0x128)]);
    let full = FakeEntries::build(&[(EntryId::CAMERA_FORWARD, 0x118)]);
    // SAFETY: the reads stop at the missing or moved anchor or the null pointer, before any read of the camera.
    unsafe {
        assert_eq!(Projection::read(&missing, 0x1000 as *const c_void), None);
        assert_eq!(Projection::read(&moved, 0x1000 as *const c_void), None);
        assert_eq!(Projection::read(&full, std::ptr::null()), None);
    }
}
