use std::collections::BTreeMap;

use amml_data::{Check, EntryId, EntryKind, Resolved};
use amml_derive::testing::ImageFixture;
use amml_derive::{CheckRunner, RttiIndex};

const BASE: u64 = 0x1_4000_0000;
const CLASS: &str = ".?AVClientInstance@@";

#[test]
fn function_start_accepts_only_a_start() {
    let mut fx = ImageFixture::new(BASE);
    let f = fx.put_text(&[0x90, 0xC3]);
    fx.put_function(f, f + 2);
    let img = fx.image();
    let rtti = RttiIndex::new(&img);
    let check = Check::FunctionStart;
    assert!(CheckRunner::run(&img, &rtti, &check, EntryKind::Function, u64::from(f), &BTreeMap::new()));
    assert!(!CheckRunner::run(&img, &rtti, &check, EntryKind::Function, u64::from(f) + 1, &BTreeMap::new()));
}

#[test]
fn in_vtable_of_needs_a_slot_holding_the_value() {
    let mut fx = ImageFixture::new(BASE);
    let f = fx.put_text(&[0xC3]);
    let other = fx.put_text(&[0xC3]);
    fx.put_class(CLASS, &[f]);
    let img = fx.image();
    let rtti = RttiIndex::new(&img);
    let check = Check::InVtableOf { class: CLASS.to_string() };
    assert!(CheckRunner::run(&img, &rtti, &check, EntryKind::Function, u64::from(f), &BTreeMap::new()));
    assert!(!CheckRunner::run(&img, &rtti, &check, EntryKind::Function, u64::from(other), &BTreeMap::new()));
}

#[test]
fn range_is_inclusive_on_both_ends() {
    let mut fx = ImageFixture::new(BASE);
    fx.put_text(&[0xC3]);
    let img = fx.image();
    let rtti = RttiIndex::new(&img);
    let check = Check::Range { min: 8, max: 16 };
    assert!(CheckRunner::run(&img, &rtti, &check, EntryKind::Field, 8, &BTreeMap::new()));
    assert!(CheckRunner::run(&img, &rtti, &check, EntryKind::Field, 16, &BTreeMap::new()));
    assert!(!CheckRunner::run(&img, &rtti, &check, EntryKind::Field, 7, &BTreeMap::new()));
    assert!(!CheckRunner::run(&img, &rtti, &check, EntryKind::Field, 17, &BTreeMap::new()));
}

#[test]
fn section_accepts_only_a_value_inside_the_named_section() {
    let mut fx = ImageFixture::new(BASE);
    let code = fx.put_text(&[0xC3]);
    let data = fx.put_rdata(&[0; 16]);
    let img = fx.image();
    let rtti = RttiIndex::new(&img);
    let rdata = Check::Section { name: ".rdata".to_string() };
    assert!(CheckRunner::run(&img, &rtti, &rdata, EntryKind::Global, u64::from(data), &BTreeMap::new()));
    assert!(!CheckRunner::run(&img, &rtti, &rdata, EntryKind::Global, u64::from(code), &BTreeMap::new()));
    assert!(!CheckRunner::run(&img, &rtti, &rdata, EntryKind::Global, u64::MAX, &BTreeMap::new()));
}

#[test]
fn section_with_an_unknown_name_fails() {
    let mut fx = ImageFixture::new(BASE);
    let data = fx.put_rdata(&[0; 16]);
    let img = fx.image();
    let rtti = RttiIndex::new(&img);
    let check = Check::Section { name: ".nope".to_string() };
    assert!(!CheckRunner::run(&img, &rtti, &check, EntryKind::Global, u64::from(data), &BTreeMap::new()));
}

struct Table;

impl Table {
    // a global holding a 30 slot vtable whose slots 20..=26 hold `f`; returns the table and `f`
    fn with_function_in_slots_20_to_26() -> (amml_derive::Image, BTreeMap<EntryId, Resolved>, u32) {
        let mut fx = ImageFixture::new(BASE);
        let f = fx.put_text(&[0xC3]);
        let filler = fx.put_text(&[0xC3]);
        let slots: Vec<u32> = (0..30).map(|i| if (20..=26).contains(&i) { f } else { filler }).collect();
        let table = fx.put_class(CLASS, &slots);
        let mut resolved = BTreeMap::new();
        resolved.insert(
            EntryId("VT".to_string()),
            Resolved { kind: EntryKind::Global, value: u64::from(table), recipes_agreeing: 1 },
        );
        (fx.image(), resolved, f)
    }

    fn check(min_slot: u32, max_slot: u32) -> Check {
        Check::InVtable { vtable: EntryId("VT".to_string()), min_slot, max_slot }
    }
}

#[test]
fn in_vtable_passes_when_a_slot_in_the_range_holds_the_function() {
    let (img, resolved, f) = Table::with_function_in_slots_20_to_26();
    let rtti = RttiIndex::new(&img);
    let value = u64::from(f);
    assert!(CheckRunner::run(&img, &rtti, &Table::check(20, 26), EntryKind::Function, value, &resolved));
    assert!(CheckRunner::run(&img, &rtti, &Table::check(26, 29), EntryKind::Function, value, &resolved));
    assert!(CheckRunner::run(&img, &rtti, &Table::check(0, 29), EntryKind::Function, value, &resolved));
}

#[test]
fn in_vtable_fails_when_no_slot_in_the_range_holds_the_function() {
    let (img, resolved, f) = Table::with_function_in_slots_20_to_26();
    let rtti = RttiIndex::new(&img);
    let value = u64::from(f);
    assert!(!CheckRunner::run(&img, &rtti, &Table::check(0, 10), EntryKind::Function, value, &resolved));
    assert!(!CheckRunner::run(&img, &rtti, &Table::check(27, 29), EntryKind::Function, value, &resolved));
}

#[test]
fn in_vtable_fails_when_a_slot_up_to_max_is_not_code() {
    let (img, resolved, f) = Table::with_function_in_slots_20_to_26();
    let rtti = RttiIndex::new(&img);
    // slot 30 is past the table (the bytes after it are not a code pointer)
    assert!(!CheckRunner::run(&img, &rtti, &Table::check(20, 40), EntryKind::Function, u64::from(f), &resolved));
}

#[test]
fn in_vtable_fails_when_the_global_is_not_resolved() {
    let (img, _, f) = Table::with_function_in_slots_20_to_26();
    let rtti = RttiIndex::new(&img);
    let check = Table::check(20, 26);
    assert!(!CheckRunner::run(&img, &rtti, &check, EntryKind::Function, u64::from(f), &BTreeMap::new()));
}

struct Leaf;

impl Leaf {
    // `.text` starts 16-byte aligned. A caller with `.pdata` (`call` or `jmp` rel32 to the leaf, then `ret`), `pad`
    // filler bytes up to the next 16-byte boundary, then the leaf `xor eax,eax; ret`. Returns the image and the leaf.
    fn image(branch: u8, pad: u8, leaf_pdata: bool) -> (amml_derive::Image, u32) {
        let mut fx = ImageFixture::new(BASE);
        let caller = fx.put_text(&Self::caller_then(branch, &[pad; 10]));
        fx.put_function(caller, caller + 6);
        let leaf = fx.put_text(&[0x31, 0xC0, 0xC3]);
        assert_eq!(leaf, caller + 16);
        if leaf_pdata {
            fx.put_function(leaf, leaf + 3);
        }
        (fx.image(), leaf)
    }

    // a 16-byte block: `branch rel32` to the next 16-byte boundary, `ret`, then `tail` (10 bytes)
    fn caller_then(branch: u8, tail: &[u8; 10]) -> Vec<u8> {
        let mut bytes = vec![branch, 0x0B, 0x00, 0x00, 0x00, 0xC3];
        bytes.extend_from_slice(tail);
        bytes
    }

    fn passes(image: &amml_derive::Image, value: u32) -> bool {
        let rtti = RttiIndex::new(image);
        CheckRunner::run(image, &rtti, &Check::LeafStart, EntryKind::Function, u64::from(value), &BTreeMap::new())
    }
}

#[test]
fn leaf_start_accepts_a_called_leaf_without_pdata_after_int3_padding() {
    let (img, leaf) = Leaf::image(0xE8, 0xCC, false);
    assert!(Leaf::passes(&img, leaf));
}

#[test]
fn leaf_start_also_accepts_a_called_pdata_start() {
    let (img, leaf) = Leaf::image(0xE8, 0xCC, true);
    assert!(Leaf::passes(&img, leaf));
}

#[test]
fn leaf_start_refuses_a_value_that_no_call_reaches() {
    // a `jmp rel32` to the leaf is not a call
    let (img, leaf) = Leaf::image(0xE9, 0xCC, false);
    assert!(!Leaf::passes(&img, leaf));
}

#[test]
fn leaf_start_refuses_a_value_after_bytes_that_are_not_padding() {
    let (img, leaf) = Leaf::image(0xE8, 0x90, false);
    assert!(!Leaf::passes(&img, leaf));
}

#[test]
fn leaf_start_refuses_a_value_that_is_not_16_byte_aligned() {
    // the caller calls the code after one int3, at an odd address
    let mut fx = ImageFixture::new(BASE);
    let caller = fx.put_text(&[0xE8, 0x0C, 0x00, 0x00, 0x00, 0xC3]);
    fx.put_function(caller, caller + 6);
    let leaf = fx.put_text(&[0xCC, 0x31, 0xC0, 0xC3]) + 1;
    assert_eq!(leaf, caller + 17);
    let img = fx.image();
    assert!(!Leaf::passes(&img, leaf));
}

#[test]
fn leaf_start_refuses_a_value_inside_the_pdata_range_of_another_function() {
    let mut fx = ImageFixture::new(BASE);
    let caller = fx.put_text(&Leaf::caller_then(0xE8, &[0xCC; 10]));
    let leaf = fx.put_text(&[0x31, 0xC0, 0xC3]);
    // one range covers the caller, the padding and the leaf
    fx.put_function(caller, leaf + 3);
    let img = fx.image();
    assert!(!Leaf::passes(&img, leaf));
}

#[test]
fn leaf_start_accepts_a_called_leaf_right_after_a_pdata_range_that_ends_with_ret() {
    // the caller is 16 bytes long and ends with `ret`; the leaf follows with no padding
    let mut fx = ImageFixture::new(BASE);
    let mut caller_bytes = vec![0xE8, 0x0B, 0x00, 0x00, 0x00];
    caller_bytes.extend([0x90; 10]);
    caller_bytes.push(0xC3);
    let caller = fx.put_text(&caller_bytes);
    assert_eq!(caller_bytes.len(), 16);
    fx.put_function(caller, caller + 16);
    let leaf = fx.put_text(&[0x31, 0xC0, 0xC3]);
    let img = fx.image();
    assert!(Leaf::passes(&img, leaf));
}

#[test]
fn leaf_start_refuses_a_leaf_right_after_a_pdata_range_that_does_not_end_with_ret_or_jmp() {
    // the same layout as above, but the caller's last instruction is `nop`
    let mut fx = ImageFixture::new(BASE);
    let caller = fx.put_text(&Leaf::caller_then(0xE8, &[0x90; 10]));
    fx.put_function(caller, caller + 16);
    let leaf = fx.put_text(&[0x31, 0xC0, 0xC3]);
    let img = fx.image();
    assert!(!Leaf::passes(&img, leaf));
}

#[test]
fn leaf_start_refuses_a_value_outside_executable_code() {
    let mut fx = ImageFixture::new(BASE);
    fx.put_text(&[0xC3]);
    let data = fx.put_rdata(&[0; 32]);
    let img = fx.image();
    assert!(!Leaf::passes(&img, data));
}

#[test]
fn entry_is_passes_only_for_the_exact_value_of_a_resolved_entry() {
    let mut fx = ImageFixture::new(BASE);
    fx.put_text(&[0xC3]);
    let img = fx.image();
    let rtti = RttiIndex::new(&img);
    let mut resolved = BTreeMap::new();
    resolved.insert(EntryId("L".to_string()), Resolved { kind: EntryKind::Field, value: 1, recipes_agreeing: 2 });
    let is = |value| Check::EntryIs { entry: EntryId("L".to_string()), value };
    assert!(CheckRunner::run(&img, &rtti, &is(1), EntryKind::Field, 0x28, &resolved));
    assert!(!CheckRunner::run(&img, &rtti, &is(0), EntryKind::Field, 0x28, &resolved));
    assert!(!CheckRunner::run(&img, &rtti, &is(1), EntryKind::Field, 0x28, &BTreeMap::new()));
}

#[test]
fn leaf_start_refuses_a_called_fragment_start_of_another_function() {
    // the primary is the caller; the target is a chained fragment of it, after int3 padding
    let mut fx = ImageFixture::new(BASE);
    let caller = fx.put_text(&Leaf::caller_then(0xE8, &[0xCC; 10]));
    fx.put_function(caller, caller + 6);
    let fragment = fx.put_text(&[0x31, 0xC0, 0xC3]);
    fx.put_fragment(fragment, fragment + 3, caller);
    let img = fx.image();
    assert!(!Leaf::passes(&img, fragment));
}

#[test]
fn leaf_start_accepts_a_called_leaf_right_after_a_pdata_range_that_ends_with_jmp() {
    // `jmp r8` (indirect) and `jmp rel32` (direct) both end a range
    for tail in [vec![0x41, 0xFF, 0xE0], vec![0xE9, 0x00, 0x00, 0x00, 0x00]] {
        let mut fx = ImageFixture::new(BASE);
        let mut caller_bytes = vec![0xE8, 0x0B, 0x00, 0x00, 0x00];
        caller_bytes.extend(std::iter::repeat_n(0x90, 11 - tail.len()));
        caller_bytes.extend(&tail);
        assert_eq!(caller_bytes.len(), 16);
        let caller = fx.put_text(&caller_bytes);
        fx.put_function(caller, caller + 16);
        let leaf = fx.put_text(&[0x31, 0xC0, 0xC3]);
        let img = fx.image();
        assert!(Leaf::passes(&img, leaf), "tail {tail:02X?}");
    }
}

#[test]
fn leaf_start_refuses_a_called_target_in_a_section_that_is_not_executable() {
    // int3 padding and a confirmed `call rel32` from `.text`, but the target lies in `.rdata`; `.text` starts at
    // 0x1000, so the caller's address is known before it is written
    let mut fx = ImageFixture::new(BASE);
    let data = fx.put_rdata(&[0xCC; 32]) + 16;
    let rel = i32::try_from(i64::from(data) - 0x1005).unwrap();
    let mut bytes = vec![0xE8];
    bytes.extend_from_slice(&rel.to_le_bytes());
    bytes.push(0xC3);
    let caller = fx.put_text(&bytes);
    assert_eq!(caller, 0x1000);
    fx.put_function(caller, caller + 6);
    let img = fx.image();
    assert!(!Leaf::passes(&img, data));
}
