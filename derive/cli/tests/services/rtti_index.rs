use amml_derive::testing::ImageFixture;
use amml_derive::{RttiIndex, StepError};

const BASE: u64 = 0x1_4000_0000;
const CLASS: &str = ".?AVClientInstance@@";

#[test]
fn finds_vtable_and_slot_targets() {
    let mut fx = ImageFixture::new(BASE);
    let f0 = fx.put_text(&[0xC3]);
    let f1 = fx.put_text(&[0xC3]);
    let vt = fx.put_class(CLASS, &[f0, f1]);
    let img = fx.image();
    let idx = RttiIndex::new(&img);
    assert_eq!(idx.vtable(CLASS).unwrap(), vt);
    assert_eq!(idx.slot_target(&img, CLASS, 1).unwrap(), f1);
    assert_eq!(idx.slot_of(&img, CLASS, f1).unwrap(), 1);
    assert!(idx.in_vtable(&img, CLASS, f0));
}

#[test]
fn slot_past_the_end_is_out_of_range() {
    let mut fx = ImageFixture::new(BASE);
    let f0 = fx.put_text(&[0xC3]);
    let f1 = fx.put_text(&[0xC3]);
    fx.put_class(CLASS, &[f0, f1]);
    let img = fx.image();
    let idx = RttiIndex::new(&img);
    assert!(matches!(idx.slot_target(&img, CLASS, 2), Err(StepError::SlotOutOfRange { slot: 2, .. })));
}

#[test]
fn function_outside_the_vtable_is_not_in_it() {
    let mut fx = ImageFixture::new(BASE);
    let f0 = fx.put_text(&[0xC3]);
    let other = fx.put_text(&[0xC3]);
    fx.put_class(CLASS, &[f0]);
    let img = fx.image();
    let idx = RttiIndex::new(&img);
    assert!(matches!(idx.slot_of(&img, CLASS, other), Err(StepError::NotInVtable { .. })));
    assert!(!idx.in_vtable(&img, CLASS, other));
}

#[test]
fn unknown_class_is_not_found() {
    let mut fx = ImageFixture::new(BASE);
    let f0 = fx.put_text(&[0xC3]);
    fx.put_class(CLASS, &[f0]);
    let img = fx.image();
    let idx = RttiIndex::new(&img);
    assert!(matches!(idx.vtable(".?AVMissing@@"), Err(StepError::ClassNotFound(_))));
}

#[test]
fn two_primary_vtables_for_one_name_is_ambiguous() {
    let mut fx = ImageFixture::new(BASE);
    let f0 = fx.put_text(&[0xC3]);
    fx.put_class(CLASS, &[f0]);
    fx.put_class(CLASS, &[f0]);
    let img = fx.image();
    let idx = RttiIndex::new(&img);
    assert!(matches!(idx.vtable(CLASS), Err(StepError::ClassAmbiguous(_))));
}

#[test]
fn classes_are_indexed_independently() {
    let mut fx = ImageFixture::new(BASE);
    let f0 = fx.put_text(&[0xC3]);
    let f1 = fx.put_text(&[0xC3]);
    let a = fx.put_class(".?AUA@@", &[f0]);
    let b = fx.put_class(".?AVB@@", &[f1]);
    let img = fx.image();
    let idx = RttiIndex::new(&img);
    assert_eq!(idx.vtable(".?AUA@@").unwrap(), a);
    assert_eq!(idx.vtable(".?AVB@@").unwrap(), b);
}

#[test]
fn locators_with_a_bad_signature_offset_or_self_are_ignored() {
    let mut fx = ImageFixture::new(BASE);
    let f0 = fx.put_text(&[0xC3]);
    fx.put_class_with_locator(".?AVSig@@", &[f0], 0, 0, 0);
    fx.put_class_with_locator(".?AVOffset@@", &[f0], 1, 8, 0);
    fx.put_class_with_locator(".?AVSelf@@", &[f0], 1, 0, 4);
    let img = fx.image();
    let idx = RttiIndex::new(&img);
    for class in [".?AVSig@@", ".?AVOffset@@", ".?AVSelf@@"] {
        assert!(matches!(idx.vtable(class), Err(StepError::ClassNotFound(_))));
    }
}

#[test]
fn non_code_value_after_a_slot_ends_the_vtable() {
    let mut fx = ImageFixture::new(BASE);
    let f0 = fx.put_text(&[0xC3]);
    let f1 = fx.put_text(&[0xC3]);
    let data = fx.put_rdata(&[0; 8]);
    fx.put_class(CLASS, &[f0, data, f1]);
    let img = fx.image();
    let idx = RttiIndex::new(&img);
    assert_eq!(idx.slot_target(&img, CLASS, 0).unwrap(), f0);
    assert!(matches!(idx.slot_target(&img, CLASS, 1), Err(StepError::SlotOutOfRange { .. })));
    assert!(!idx.in_vtable(&img, CLASS, f1));
}

#[test]
fn function_in_two_slots_is_an_ambiguous_slot_but_still_in_the_vtable() {
    let mut fx = ImageFixture::new(BASE);
    let f0 = fx.put_text(&[0xC3]);
    let f1 = fx.put_text(&[0xC3]);
    fx.put_class(CLASS, &[f0, f1, f0]);
    let img = fx.image();
    let idx = RttiIndex::new(&img);
    assert!(matches!(idx.slot_of(&img, CLASS, f0), Err(StepError::AmbiguousSlot { rva, .. }) if rva == f0));
    assert_eq!(idx.slot_of(&img, CLASS, f1).unwrap(), 1);
    assert!(idx.in_vtable(&img, CLASS, f0));
}

#[test]
fn stray_reference_to_a_locator_is_not_a_second_vtable() {
    let mut fx = ImageFixture::new(BASE);
    let f0 = fx.put_text(&[0xC3]);
    let vt = fx.put_class(CLASS, &[f0]);
    let col_pointer = fx.image().read_u64(vt - 8).unwrap();
    fx.put_rdata(&col_pointer.to_le_bytes());
    let img = fx.image();
    let idx = RttiIndex::new(&img);
    assert_eq!(idx.vtable(CLASS).unwrap(), vt);
}

#[test]
fn name_inside_a_longer_string_is_not_a_type_descriptor() {
    let mut fx = ImageFixture::new(BASE);
    let f0 = fx.put_text(&[0xC3]);
    fx.put_class("x.?AVInner@@", &[f0]);
    let img = fx.image();
    let idx = RttiIndex::new(&img);
    assert!(matches!(idx.vtable(".?AVInner@@"), Err(StepError::ClassNotFound(_))));
}
