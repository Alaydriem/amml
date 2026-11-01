use amml_game::{CompoundTagRef, EntryId, GameEntries};

use crate::support::{FakeCompound, FakeEntries, FakeValue};

fn entries() -> GameEntries {
    FakeEntries::build(&[(EntryId::COMPOUND_TAG_NODE_VARIANT_INDEX, 0x68)])
}

fn slot(value: i8) -> FakeCompound {
    FakeCompound::build(vec![("Slot", FakeValue::Byte(value)), ("Count", FakeValue::Byte(1))])
}

fn wrap<'a>(tag: &FakeCompound, entries: &'a GameEntries) -> CompoundTagRef<'a> {
    // SAFETY: a live fake laid out as the 26.60 CompoundTag.
    unsafe { CompoundTagRef::from_raw(tag.ptr(), entries) }.unwrap()
}

#[test]
fn list_on_a_tag_whose_items_is_a_list_of_two_compounds_has_length_two() {
    let entries = entries();
    let tag =
        FakeCompound::build(vec![("Items", FakeValue::List(vec![slot(3), slot(4)])), ("Damage", FakeValue::Int(5))]);

    assert_eq!(wrap(&tag, &entries).list("Items").map(|list| list.len()), Some(2));
}

#[test]
fn list_is_none_when_items_is_a_byte_an_int_or_missing() {
    let entries = entries();
    let byte = FakeCompound::build(vec![("Items", FakeValue::Byte(1))]);
    let int = FakeCompound::build(vec![("Items", FakeValue::Int(1))]);
    let missing = FakeCompound::build(vec![("Other", FakeValue::List(vec![]))]);
    let empty = FakeCompound::build(vec![]);

    assert!(wrap(&byte, &entries).list("Items").is_none());
    assert!(wrap(&int, &entries).list("Items").is_none());
    assert!(wrap(&missing, &entries).list("Items").is_none());
    assert!(wrap(&empty, &entries).list("Items").is_none());
}

#[test]
fn byte_gives_the_value_with_its_sign_and_none_for_a_missing_key_or_another_type() {
    let entries = entries();
    let tag = FakeCompound::build(vec![
        ("Slot", FakeValue::Byte(-3)),
        ("Count", FakeValue::Byte(26)),
        ("Damage", FakeValue::Int(7)),
        ("Items", FakeValue::List(vec![])),
    ]);
    let tag = wrap(&tag, &entries);

    assert_eq!(tag.byte("Slot"), Some(-3));
    assert_eq!(tag.byte("Count"), Some(26));
    assert_eq!(tag.byte("Damage"), None);
    assert_eq!(tag.byte("Items"), None);
    assert_eq!(tag.byte("Missing"), None);
}

#[test]
fn int_gives_the_value_with_its_sign_and_none_for_a_byte_tag_or_a_missing_key() {
    let entries = entries();
    let tag = FakeCompound::build(vec![
        ("selected_item", FakeValue::Int(-1)),
        ("Damage", FakeValue::Int(70_000)),
        ("Count", FakeValue::Byte(3)),
    ]);
    let tag = wrap(&tag, &entries);

    assert_eq!(tag.int("selected_item"), Some(-1));
    assert_eq!(tag.int("Damage"), Some(70_000));
    assert_eq!(tag.int("Count"), None);
    assert_eq!(tag.int("Missing"), None);
}

#[test]
fn a_tag_without_the_variant_index_entry_or_with_another_offset_is_refused() {
    let tag = FakeCompound::build(vec![("Slot", FakeValue::Byte(1))]);

    let none = FakeEntries::build(&[]);
    // SAFETY: a live fake; with no entry the wrapper reads nothing.
    assert!(unsafe { CompoundTagRef::from_raw(tag.ptr(), &none) }.is_none());
    let moved = FakeEntries::build(&[(EntryId::COMPOUND_TAG_NODE_VARIANT_INDEX, 0x70)]);
    // SAFETY: as above.
    assert!(unsafe { CompoundTagRef::from_raw(tag.ptr(), &moved) }.is_none());
    let on = entries();
    // SAFETY: a null pointer is allowed and gives None.
    assert!(unsafe { CompoundTagRef::from_raw(std::ptr::null_mut(), &on) }.is_none());
}
