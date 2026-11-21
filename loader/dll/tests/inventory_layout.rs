use amml::{InventoryLayout, NameReader, TagReader};

use crate::support::{FakeCompound, FakeValue};

fn entries(overrides: &[(&str, Option<u64>)]) -> impl Fn(&str) -> Option<u64> {
    let mut values: Vec<(String, Option<u64>)> = [
        ("Player.playerInventory", 0x5e0),
        ("PlayerInventory.mInventory", 0xb8),
        ("PlayerInventory.mSelected", 0x10),
        ("FillingContainer.mItems", 0x198),
        ("ItemStack.size", 0x98),
        ("ItemStackBase.mItem", 0x8),
        ("ItemStackBase.mCount", 0x22),
        ("ItemStackBase.getDamageValue", 0x1_4000_1000),
        ("Item.getMaxDamage", 0x24),
        ("Item.mFullName", 0x120),
        ("HashedString.layout", 1),
    ]
    .iter()
    .map(|(name, value)| (name.to_string(), Some(*value)))
    .collect();
    for (name, value) in overrides {
        values.retain(|(known, _)| known != name);
        values.push((name.to_string(), *value));
    }
    move |name| values.iter().find(|(known, _)| known == name).and_then(|(_, value)| *value)
}

#[test]
fn the_mirror_values_give_a_layout_with_the_name_reader_of_the_build() {
    let layout = InventoryLayout::from_entries(&entries(&[])).unwrap();
    assert_eq!((layout.player_inventory, layout.container, layout.selected, layout.items), (0x5e0, 0xb8, 0x10, 0x198));
    assert_eq!(layout.names, NameReader::new(1).unwrap());
}

#[test]
fn another_itemstack_layout_or_an_unknown_name_layout_is_refused() {
    for (name, value) in [("ItemStack.size", 0xa0), ("ItemStackBase.mItem", 0x10), ("ItemStackBase.mCount", 0x23)] {
        let refused = InventoryLayout::from_entries(&entries(&[(name, Some(value))]));
        assert_eq!(refused, Err("the ItemStack layout is not the loader's mirror"), "{name}");
    }
    let refused = InventoryLayout::from_entries(&entries(&[("HashedString.layout", Some(2))]));
    assert_eq!(refused, Err("no HashedString mirror for this build"));
    let missing = InventoryLayout::from_entries(&entries(&[("Item.mFullName", None)]));
    assert_eq!(missing, Err("an inventory entry did not resolve"));
}

#[test]
fn a_slot_is_found_by_the_stride_and_refused_past_the_end_or_in_a_bad_vector() {
    let layout = InventoryLayout::from_entries(&entries(&[])).unwrap();
    let first = 0x1000;
    let last = first + 36 * 0x98;
    assert_eq!(layout.stack_at(first, last, 0), Some(first));
    assert_eq!(layout.stack_at(first, last, 8), Some(first + 8 * 0x98));
    assert_eq!(layout.stack_at(first, last, 36), None);
    assert_eq!(layout.stack_at(0, last, 0), None);
    assert_eq!(layout.stack_at(last, first, 0), None);
}

#[test]
fn the_mirror_reads_the_aux_value_and_the_user_data_pointer_of_a_stack() {
    let layout = InventoryLayout::from_entries(&entries(&[])).unwrap();
    assert_eq!((layout.aux, layout.user_data), (0x20, 0x10));
}

#[test]
fn the_inventory_table_is_v3_and_gives_slot_items() {
    let harness = crate::support::Harness::start();
    // SAFETY: the loader's tables live for the process.
    let services = unsafe { &*amml_abi::AmmlApi::services(harness.state.api()).unwrap() };
    // SAFETY: a table of v5 or later points to the loader's static inventory table.
    let inventory = unsafe { &*services.inventory };
    assert!(inventory.size >= amml_abi::AmmlInventoryTable::SIZE_V3);
    assert!(inventory.version >= 3);
}

#[test]
fn slot_index_above_35_is_invalid_argument() {
    let harness = crate::support::Harness::start();
    // SAFETY: the loader's tables live for the process.
    let services = unsafe { &*amml_abi::AmmlApi::services(harness.state.api()).unwrap() };
    // SAFETY: a table of v5 or later points to the loader's static inventory table.
    let inventory = unsafe { amml_abi::AmmlInventoryTable::read(services.inventory) }.unwrap();
    let slot_item = inventory.slot_item.unwrap();
    for index in [36u8, 37, 255] {
        let mut out: *const std::ffi::c_void = std::ptr::null();
        // SAFETY: `out` is a live local; the index is refused before the player is read.
        let status = unsafe { slot_item(std::ptr::null(), index, &mut out) };
        assert_eq!(status, amml_abi::AmmlStatus::INVALID_ARGUMENT, "{index}");
    }
    // SAFETY: a null output is refused before any write.
    let status = unsafe { slot_item(std::ptr::null(), 0, std::ptr::null_mut()) };
    assert_eq!(status, amml_abi::AmmlStatus::INVALID_ARGUMENT);
}

fn item_entry(slot: i8) -> FakeCompound {
    FakeCompound::build(vec![("Count", FakeValue::Int(1)), ("Slot", FakeValue::Byte(slot))])
}

fn reader() -> TagReader {
    TagReader::new(0x68, FakeCompound::compound_vtable()).unwrap()
}

#[test]
fn the_tag_reader_exists_only_for_the_node_index_the_mirror_fits() {
    assert!(TagReader::new(0x68, 0x1000).is_some());
    assert!(TagReader::new(0x60, 0x1000).is_none());
    assert!(TagReader::new(0x68, 0).is_none());
}

#[test]
fn a_box_list_gives_each_element_and_its_slot_and_skips_an_element_that_is_not_a_compound() {
    let tag = FakeCompound::build(vec![("Items", FakeValue::ListWithAByte(vec![item_entry(0), item_entry(26)]))]);
    let reader = reader();

    // SAFETY: a live fake laid out as the 26.60 CompoundTag.
    unsafe {
        let elements = reader.list(tag.ptr().addr(), "Items").unwrap();
        assert_eq!(elements.len(), 3);
        assert_eq!(reader.compound(elements[0]), None);
        let slots: Vec<_> =
            elements[1..].iter().map(|element| reader.byte(reader.compound(*element).unwrap(), "Slot")).collect();
        assert_eq!(slots, vec![Some(0), Some(26)]);
    }
}

#[test]
fn a_bundle_tag_gives_its_weight_and_its_contents_list() {
    let tag = FakeCompound::build(vec![
        ("bundle_weight", FakeValue::Int(40)),
        ("storage_item_component_content", FakeValue::List(vec![item_entry(2), item_entry(0)])),
    ]);
    let reader = reader();

    // SAFETY: a live fake laid out as the 26.60 CompoundTag.
    unsafe {
        assert_eq!(reader.int(tag.ptr().addr(), "bundle_weight"), Some(40));
        assert_eq!(reader.list(tag.ptr().addr(), "storage_item_component_content").map(|list| list.len()), Some(2));
    }
}

#[test]
fn a_missing_key_a_tag_of_another_type_and_a_null_tag_read_nothing() {
    let tag = FakeCompound::build(vec![("Items", FakeValue::Int(5)), ("Slot", FakeValue::Byte(3))]);
    let reader = reader();

    // SAFETY: a live fake laid out as the 26.60 CompoundTag; the null tag is read through no pointer.
    unsafe {
        assert!(reader.list(tag.ptr().addr(), "Items").is_none());
        assert!(reader.list(tag.ptr().addr(), "absent").is_none());
        assert!(reader.int(tag.ptr().addr(), "Slot").is_none());
        assert!(reader.byte(tag.ptr().addr(), "Items").is_none());
        assert!(reader.compound(0).is_none());
    }
}
