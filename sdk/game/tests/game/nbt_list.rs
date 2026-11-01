use amml_game::{CompoundTagRef, EntryId, GameEntries, NbtList};

use crate::support::{FakeCompound, FakeEntries, FakeValue};

fn entries() -> GameEntries {
    FakeEntries::build(&[(EntryId::COMPOUND_TAG_NODE_VARIANT_INDEX, 0x68)])
}

fn item(slot: i8) -> FakeCompound {
    FakeCompound::build(vec![("Slot", FakeValue::Byte(slot))])
}

fn items<'a>(tag: &FakeCompound, entries: &'a GameEntries) -> NbtList<'a> {
    // SAFETY: a live fake laid out as the 26.60 CompoundTag.
    let tag = unsafe { CompoundTagRef::from_raw(tag.ptr(), entries) }.unwrap();
    tag.list("Items").unwrap()
}

#[test]
fn compound_gives_each_element_and_its_slot() {
    let entries = entries();
    let tag = FakeCompound::build(vec![("Items", FakeValue::List(vec![item(0), item(26)]))]);
    let list = items(&tag, &entries);

    assert_eq!(list.len(), 2);
    assert_eq!(list.compound(0).and_then(|compound| compound.byte("Slot")), Some(0));
    assert_eq!(list.compound(1).and_then(|compound| compound.byte("Slot")), Some(26));
}

#[test]
fn compound_is_none_past_the_end() {
    let entries = entries();
    let tag = FakeCompound::build(vec![("Items", FakeValue::List(vec![item(0)]))]);
    let list = items(&tag, &entries);

    assert!(list.compound(1).is_none());
    assert!(list.compound(usize::MAX).is_none());
}

#[test]
fn an_element_that_is_not_a_compound_is_none_and_the_next_one_still_reads() {
    let entries = entries();
    let tag = FakeCompound::build(vec![("Items", FakeValue::ListWithAByte(vec![item(9)]))]);
    let list = items(&tag, &entries);

    assert_eq!(list.len(), 2);
    assert!(list.compound(0).is_none());
    assert_eq!(list.compound(1).and_then(|compound| compound.byte("Slot")), Some(9));
}

#[test]
fn an_empty_list_has_length_zero() {
    let entries = entries();
    let tag = FakeCompound::build(vec![("Items", FakeValue::List(vec![]))]);
    let list = items(&tag, &entries);

    assert_eq!(list.len(), 0);
    assert!(list.compound(0).is_none());
}
