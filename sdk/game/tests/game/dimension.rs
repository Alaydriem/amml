use amml_game::{Dimension, EntryId};

use crate::support::{FakeDimension, FakeEntries};

#[test]
fn a_dimension_id_the_mod_does_not_know_passes_through_as_a_raw_i32() {
    let entries = FakeEntries::build(&[(EntryId::DIMENSION_GET_DIMENSION_ID, FakeDimension::ID_SLOT)]);

    for id in [0, 1, 2, 7, 1000, -3] {
        let mut fake = FakeDimension::new(id);
        // SAFETY: the pointer is a live fake with the vtable slot the entry names.
        let dimension = unsafe { Dimension::from_raw(fake.ptr(), &entries) }.unwrap();
        assert_eq!(dimension.id(), Some(id));
    }
}

#[test]
fn a_null_pointer_gives_none() {
    let entries = FakeEntries::build(&[(EntryId::DIMENSION_GET_DIMENSION_ID, FakeDimension::ID_SLOT)]);

    // SAFETY: null is allowed.
    assert!(unsafe { Dimension::from_raw(std::ptr::null_mut(), &entries) }.is_none());
}

#[test]
fn a_missing_slot_entry_keeps_the_dimension_but_gives_no_id() {
    let without_slot = FakeEntries::build(&[]);
    let mut fake = FakeDimension::new(7);

    // SAFETY: the entry is missing, so the fake vtable is never read.
    let dimension = unsafe { Dimension::from_raw(fake.ptr(), &without_slot) }.unwrap();

    assert_eq!(dimension.as_ptr(), fake.ptr());
    assert_eq!(dimension.id(), None);
}
