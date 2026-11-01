use amml_game::HashedStringLayout;
use amml_msvc::{MsvcHeap, MsvcString};

#[allow(unsafe_code)]
unsafe extern "C" fn no_new(_size: usize) -> *mut u8 {
    std::ptr::null_mut()
}

#[allow(unsafe_code)]
unsafe extern "C" fn no_delete(_ptr: *mut u8, _size: usize) {}

#[test]
fn hash_of_an_empty_text_is_zero() {
    assert_eq!(HashedStringLayout::hash_of(b""), 0);
}

#[test]
fn hash_of_multiplies_then_xors_each_byte_as_the_game_does() {
    // HashedString::computeHash: hash = c ^ (0x100000001B3 * hash) from 0xCBF29CE484222325 (FNV-1, not FNV-1a)
    assert_eq!(HashedStringLayout::hash_of(b"a"), 0xaf63_bd4c_8601_b7be);
    assert_eq!(HashedStringLayout::hash_of(b"minecraft:stone"), 0x2727_739c_84aa_7913);
}

#[test]
fn hash_of_sign_extends_a_byte_above_127_as_a_signed_char() {
    assert_eq!(HashedStringLayout::hash_of(&[0xC3, 0xA9]), 0x0832_8d07_b4eb_783d);
}

#[test]
#[allow(unsafe_code)]
fn new_keeps_the_text_and_sets_its_hash() {
    let heap = MsvcHeap::new(no_new, no_delete);
    // SAFETY: a text of 15 bytes stays in the inline buffer, so the heap functions are not called
    let text = unsafe { MsvcString::new_in(&heap, "minecraft:stone") }.unwrap();

    let hashed = HashedStringLayout::new(text);

    assert_eq!(hashed.hash, 0x2727_739c_84aa_7913);
    assert_eq!(hashed.text.to_str().unwrap(), "minecraft:stone");
}
