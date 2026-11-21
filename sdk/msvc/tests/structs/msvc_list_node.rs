use amml_msvc::{MsvcListNode, SharedPtr};
use amml_msvc_fixture::{fx_sizeof_umap, fx_umap_delete, fx_umap_head, fx_umap_make};

// The value of an `unordered_map<int, shared_ptr<int>>` node: the key, padded to 8, then the shared pointer.
#[repr(C)]
struct Pair {
    key: i32,
    _pad: u32,
    value: SharedPtr<i32>,
}

#[test]
fn walks_every_element_of_an_unordered_map_once() {
    // SAFETY: the probe returns a live map with keys 0, 10, 20.
    let map = unsafe { fx_umap_make(3) };
    // SAFETY: `map` is live; its head is the list sentinel.
    let head = unsafe { fx_umap_head(map) }.cast::<MsvcListNode<Pair>>().cast_const();
    let mut seen = Vec::new();
    // SAFETY: the head is a live node.
    let mut node = unsafe { (*head).next() };
    while node != head && seen.len() < 16 {
        // SAFETY: an element node of the live map.
        let pair = unsafe { (*node).value() };
        seen.push((pair.key, pair.value.get().copied()));
        // SAFETY: as above.
        node = unsafe { (*node).next() };
    }
    seen.sort_unstable();
    assert_eq!(seen, vec![(0, Some(0)), (10, Some(100)), (20, Some(200))]);
    // SAFETY: `map` came from fx_umap_make and is not used again.
    unsafe { fx_umap_delete(map) };
}

#[test]
fn the_head_pointer_is_the_second_qword_of_the_map() {
    // SAFETY: the probe returns a live map.
    let map = unsafe { fx_umap_make(2) };
    // SAFETY: as above.
    let head = unsafe { fx_umap_head(map) } as usize;
    // SAFETY: the map object is readable for its size.
    let words = unsafe { std::slice::from_raw_parts(map.cast::<usize>(), fx_sizeof_umap() / 8) };
    assert_eq!(words.iter().position(|word| *word == head), Some(1));
    // SAFETY: `map` came from fx_umap_make and is not used again.
    unsafe { fx_umap_delete(map) };
}
