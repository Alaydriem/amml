use crate::support::Fixture;
use amml_msvc::MsvcString;
use amml_msvc_fixture::{
    fx_sizeof_string, fx_string_append_and_destroy_in_place, fx_string_delete, fx_string_eq, fx_string_new,
};

#[test]
fn reads_cpp_strings_at_the_small_and_heap_boundary() {
    for text in ["", "a", "fifteen-bytes!!", "sixteen-bytes!!!", &"w".repeat(4095), &"x".repeat(5000)] {
        // SAFETY: the probe copies `len` bytes from a live buffer.
        let p = unsafe { fx_string_new(text.as_ptr(), text.len()) };
        // SAFETY: `p` is a live std::string, which MsvcString mirrors.
        let s = unsafe { &*(p as *const MsvcString) };
        assert_eq!(s.to_str().unwrap(), text);
        // SAFETY: `p` came from fx_string_new and is not used again.
        unsafe { fx_string_delete(p) };
    }
}

#[test]
fn cpp_reads_and_destroys_strings_made_in_rust() {
    for text in ["", "fifteen-bytes!!", "sixteen-bytes!!!", &"v".repeat(4095), &"y".repeat(5000)] {
        // SAFETY: the fixture heap is the real operator new/delete.
        let mut s = unsafe { MsvcString::new_in(&Fixture::heap(), text) }.unwrap();
        // SAFETY: `s` mirrors a std::string and the text buffer is live.
        assert!(unsafe { fx_string_eq(&s as *const _ as _, text.as_ptr(), text.len()) });
        // SAFETY: C++ appends (growing past the Rust block), then destroys the string; `s` is not used again.
        unsafe { fx_string_append_and_destroy_in_place(&mut s as *mut _ as _, b"z".as_ptr(), 1) };
    }
}

#[test]
fn release_frees_what_new_in_allocated() {
    let heap = Fixture::heap();
    for text in ["", "sixteen-bytes!!!", &"v".repeat(4095), &"y".repeat(5000)] {
        // SAFETY: the fixture heap is the real operator new/delete.
        let mut s = unsafe { MsvcString::new_in(&heap, text) }.unwrap();
        // SAFETY: `s` mirrors a std::string and the text buffer is live.
        assert!(unsafe { fx_string_eq(&s as *const _ as _, text.as_ptr(), text.len()) });
        // SAFETY: `s` was built by `new_in` with this heap.
        unsafe { s.release(&heap) };
        assert_eq!(s.as_bytes(), b"");
    }
}

#[test]
fn layout_is_32_bytes() {
    // SAFETY: the probe only returns sizeof(std::string).
    assert_eq!(std::mem::size_of::<MsvcString>(), unsafe { fx_sizeof_string() });
}
