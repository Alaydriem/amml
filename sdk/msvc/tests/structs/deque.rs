use amml_msvc::MsvcDeque;
use amml_msvc_fixture::{
    fx_deque_int_back, fx_deque_int_delete, fx_deque_int_make, fx_deque_matrix_back, fx_deque_matrix_delete,
    fx_deque_matrix_make, fx_sizeof_deque,
};

#[test]
fn back_is_the_element_cpp_back_returns_when_the_first_block_starts_late() {
    // SAFETY: the probe returns a sizeof value.
    assert_eq!(size_of::<MsvcDeque<i32>>(), unsafe { fx_sizeof_deque() });
    for (front, back) in [(0, 1), (3, 5), (17, 2), (1, 40)] {
        // SAFETY: the probe returns a live std::deque<std::array<float,16>>.
        let deque = unsafe { fx_deque_matrix_make(front, back) };
        // SAFETY: the pointer is a live deque of 64-byte elements, which `MsvcDeque<[f32; 16]>` mirrors.
        let mirror = unsafe { &*deque.cast::<MsvcDeque<[f32; 16]>>() };
        // SAFETY: as above.
        let ours = unsafe { mirror.back() }.unwrap();
        // SAFETY: as above.
        assert_eq!(ours.cast_const().cast(), unsafe { fx_deque_matrix_back(deque) });
        assert_eq!(mirror.len(), front + back);
        // SAFETY: `ours` points to the last element of the live deque.
        assert_eq!(unsafe { (*ours)[3] }, ((front + back - 1) * 100 + 3) as f32);
        // SAFETY: the deque came from the probe and is not used again.
        unsafe { fx_deque_matrix_delete(deque) };
    }
}

#[test]
fn back_uses_the_block_size_of_small_elements() {
    for (front, back) in [(0, 1), (2, 3), (5, 11), (9, 30)] {
        // SAFETY: the probe returns a live std::deque<int>.
        let deque = unsafe { fx_deque_int_make(front, back) };
        // SAFETY: the pointer is a live deque of ints.
        let mirror = unsafe { &*deque.cast::<MsvcDeque<i32>>() };
        // SAFETY: as above.
        let ours = unsafe { mirror.back() }.unwrap();
        // SAFETY: as above.
        assert_eq!(ours.cast_const().cast(), unsafe { fx_deque_int_back(deque) });
        // SAFETY: the deque came from the probe and is not used again.
        unsafe { fx_deque_int_delete(deque) };
    }
}
