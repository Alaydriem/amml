use crate::support::Fixture;
use amml_msvc::MsvcVec;
use amml_msvc_fixture::{
    fx_sizeof_vector, fx_vec_int_delete, fx_vec_int_make, fx_vec_int_sum_and_destroy_in_place, fx_vec_vec3_delete,
    fx_vec_vec3_make, fx_vec_vec3_sum,
};

#[test]
fn reads_a_cpp_vector_and_cpp_reads_one_made_in_rust() {
    // SAFETY: the probe returns a live std::vector<int>.
    let p = unsafe { fx_vec_int_make(5) };
    // SAFETY: `p` is a live std::vector<int>, which MsvcVec<i32> mirrors.
    let v = unsafe { &*(p as *const MsvcVec<i32>) };
    assert_eq!(v.as_slice(), &[0, 1, 2, 3, 4]);
    // SAFETY: `p` came from fx_vec_int_make and is not used again.
    unsafe { fx_vec_int_delete(p) };
    // SAFETY: the fixture heap is the real operator new/delete.
    let mut mine = unsafe { MsvcVec::from_slice_in(&Fixture::heap(), &[7, 8, 9]) }.unwrap();
    // SAFETY: C++ sums then destroys the vector in place; `mine` is not used again.
    assert_eq!(unsafe { fx_vec_int_sum_and_destroy_in_place(&mut mine as *mut _ as _) }, 24);
}

#[test]
fn large_elements_match_across_the_boundary() {
    // SAFETY: the probe returns a live std::vector<std::array<double,3>>.
    let p = unsafe { fx_vec_vec3_make(3) };
    // SAFETY: `p` is a live vector of three doubles per element.
    let v = unsafe { &*(p as *const MsvcVec<[f64; 3]>) };
    assert_eq!(v.as_slice(), &[[0.0, 0.5, 1.0], [1.0, 1.5, 2.0], [2.0, 2.5, 3.0]]);
    // SAFETY: `p` came from fx_vec_vec3_make and is not used again.
    unsafe { fx_vec_vec3_delete(p) };

    let heap = Fixture::heap();
    // SAFETY: the fixture heap is the real operator new/delete.
    let mut mine = unsafe { MsvcVec::from_slice_in(&heap, &[[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]]) }.unwrap();
    // SAFETY: `mine` mirrors a live std::vector<std::array<double,3>>.
    assert_eq!(unsafe { fx_vec_vec3_sum(&mine as *const _ as _) }, 21.0);
    assert_eq!(mine.len(), 2);
    // SAFETY: `mine` was built by `from_slice_in` with this heap.
    unsafe { mine.release(&heap) };
    assert!(mine.is_empty());
}

#[test]
fn empty_vectors_read_as_empty() {
    // SAFETY: the probe returns a live empty std::vector<int>.
    let p = unsafe { fx_vec_int_make(0) };
    // SAFETY: `p` is a live std::vector<int>.
    let v = unsafe { &*(p as *const MsvcVec<i32>) };
    assert!(v.as_slice().is_empty());
    // SAFETY: `p` came from fx_vec_int_make and is not used again.
    unsafe { fx_vec_int_delete(p) };
}

#[test]
fn layout_is_three_pointers() {
    // SAFETY: the probe only returns sizeof(std::vector<int>).
    assert_eq!(std::mem::size_of::<MsvcVec<i32>>(), unsafe { fx_sizeof_vector() });
}

#[test]
fn raw_parts_reads_the_three_pointers_of_a_cpp_vector() {
    // SAFETY: the probe returns a live std::vector<int>.
    let p = unsafe { fx_vec_int_make(5) };
    // SAFETY: `p` is a live std::vector<int>, which MsvcVec<i32> mirrors.
    let v = unsafe { &*(p as *const MsvcVec<i32>) };
    let [first, last, end] = v.raw_parts();
    assert_eq!(first.cast_const(), v.as_slice().as_ptr());
    assert_eq!((last as usize - first as usize) / size_of::<i32>(), 5);
    assert!(end >= last);
    // SAFETY: `p` came from fx_vec_int_make and is not used again.
    unsafe { fx_vec_int_delete(p) };
}

#[test]
fn set_raw_parts_repoints_the_vector_and_cpp_reads_the_new_elements() {
    let heap = Fixture::heap();
    // SAFETY: the fixture heap is the real operator new/delete.
    let mut mine = unsafe { MsvcVec::<[f64; 3]>::from_slice_in(&heap, &[[1.0, 2.0, 3.0]]) }.unwrap();
    let old = mine.raw_parts();
    let mut other = [[4.0, 5.0, 6.0], [7.0, 8.0, 9.0], [0.0; 3]];
    let first = other.as_mut_ptr();

    // SAFETY: `other` outlives the use below; two of its three elements are the length, all three the capacity.
    unsafe { mine.set_raw_parts(first, first.wrapping_add(2), first.wrapping_add(3)) };

    assert_eq!(mine.len(), 2);
    assert_eq!(mine.raw_parts(), [first, first.wrapping_add(2), first.wrapping_add(3)]);
    // SAFETY: `mine` mirrors a live std::vector<std::array<double,3>> that C++ only reads.
    assert_eq!(unsafe { fx_vec_vec3_sum(&mine as *const _ as _) }, 39.0);
    // SAFETY: the old pointers are the block `from_slice_in` made with this heap; `other` is not freed.
    unsafe {
        mine.set_raw_parts(old[0], old[1], old[2]);
        mine.release(&heap);
    }
}
