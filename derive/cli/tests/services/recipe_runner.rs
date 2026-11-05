use std::collections::BTreeMap;

use amml_data::{EntryId, EntryKind, OperandKind, Recipe, Resolved, Step};
use amml_derive::testing::ImageFixture;
use amml_derive::{FunctionTable, Image, PatternHit, RecipeRunner, RttiIndex, Section, StepError, StringXref};

const BASE: u64 = 0x1_4000_0000;
const NEEDLE: [u8; 7] = [0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77];
const CLASS: &str = ".?AVClientInstance@@";
const NEEDLE_PATTERN: &str = "11 22 33 44 55 66 77";

struct Recipes;

impl Recipes {
    fn pattern(offset: i64) -> Recipe {
        Recipe { steps: vec![Step::Pattern { bytes: NEEDLE_PATTERN.to_string(), offset, literal_ok: Vec::new() }] }
    }

    fn resolved(name: &str, value: u32) -> (EntryId, BTreeMap<EntryId, Resolved>) {
        let id = EntryId(name.to_string());
        let mut map = BTreeMap::new();
        map.insert(id.clone(), Resolved { kind: EntryKind::Function, value: u64::from(value), recipes_agreeing: 1 });
        (id, map)
    }
}

#[test]
fn pattern_with_two_matches_fails_with_the_count_and_never_takes_the_first() {
    let mut fx = ImageFixture::new(BASE);
    fx.put_text(&NEEDLE);
    fx.put_text(&NEEDLE);
    let img = fx.image();
    let rtti = RttiIndex::new(&img);
    let xrefs = StringXref::new(&img);
    let runner = RecipeRunner::new(&img, &rtti, &xrefs);
    let result = runner.run(&Recipes::pattern(0), EntryKind::Function, &BTreeMap::new());
    assert!(matches!(result, Err(StepError::ManyMatches(2))));
}

#[test]
fn pattern_with_no_match_fails() {
    let mut fx = ImageFixture::new(BASE);
    fx.put_text(&[0xC3]);
    let img = fx.image();
    let rtti = RttiIndex::new(&img);
    let xrefs = StringXref::new(&img);
    let runner = RecipeRunner::new(&img, &rtti, &xrefs);
    let result = runner.run(&Recipes::pattern(0), EntryKind::Function, &BTreeMap::new());
    assert!(matches!(result, Err(StepError::NoMatch)));
}

#[test]
fn pattern_offset_moves_the_result() {
    let mut fx = ImageFixture::new(BASE);
    let at = fx.put_text(&NEEDLE);
    let img = fx.image();
    let rtti = RttiIndex::new(&img);
    let xrefs = StringXref::new(&img);
    let runner = RecipeRunner::new(&img, &rtti, &xrefs);
    let result = runner.run(&Recipes::pattern(3), EntryKind::Function, &BTreeMap::new());
    assert_eq!(result.unwrap(), u64::from(at) + 3);
}

#[test]
fn pattern_after_a_ref_searches_only_inside_that_function() {
    let mut fx = ImageFixture::new(BASE);
    let a = fx.put_text(&NEEDLE);
    fx.put_function(a, a + NEEDLE.len() as u32);
    let b = fx.put_text(&NEEDLE);
    fx.put_function(b, b + NEEDLE.len() as u32);
    let img = fx.image();
    let rtti = RttiIndex::new(&img);
    let xrefs = StringXref::new(&img);
    let runner = RecipeRunner::new(&img, &rtti, &xrefs);
    let (id, resolved) = Recipes::resolved("A", a);
    let recipe = Recipe {
        steps: vec![
            Step::Ref { entry: id },
            Step::Pattern { bytes: NEEDLE_PATTERN.to_string(), offset: 0, literal_ok: Vec::new() },
        ],
    };
    assert_eq!(runner.run(&recipe, EntryKind::Function, &resolved).unwrap(), u64::from(a));
}

#[test]
fn pattern_after_a_cursor_outside_every_function_fails() {
    let mut fx = ImageFixture::new(BASE);
    let a = fx.put_text(&NEEDLE);
    let img = fx.image();
    let rtti = RttiIndex::new(&img);
    let xrefs = StringXref::new(&img);
    let runner = RecipeRunner::new(&img, &rtti, &xrefs);
    let (id, resolved) = Recipes::resolved("A", a);
    let recipe = Recipe {
        steps: vec![
            Step::Ref { entry: id },
            Step::Pattern { bytes: NEEDLE_PATTERN.to_string(), offset: 0, literal_ok: Vec::new() },
        ],
    };
    let result = runner.run(&recipe, EntryKind::Function, &resolved);
    assert!(matches!(result, Err(StepError::NotInFunction(_))));
}

#[test]
fn a_ref_to_an_unresolved_entry_is_a_dependency_failure() {
    let mut fx = ImageFixture::new(BASE);
    fx.put_text(&[0xC3]);
    let img = fx.image();
    let rtti = RttiIndex::new(&img);
    let xrefs = StringXref::new(&img);
    let runner = RecipeRunner::new(&img, &rtti, &xrefs);
    let recipe = Recipe { steps: vec![Step::Ref { entry: EntryId("A".into()) }] };
    let result = runner.run(&recipe, EntryKind::Function, &BTreeMap::new());
    assert!(matches!(result, Err(StepError::DependencyFailed(id)) if id == EntryId("A".into())));
}

// mov eax, 5; ret
const MOV_IMM: [u8; 6] = [0xB8, 0x05, 0x00, 0x00, 0x00, 0xC3];
// mov rax, [rcx + 0x5A8]; ret
const MOV_DISP: [u8; 8] = [0x48, 0x8B, 0x81, 0xA8, 0x05, 0x00, 0x00, 0xC3];

#[test]
fn a_number_cannot_feed_call() {
    let mut fx = ImageFixture::new(BASE);
    let f = fx.put_text(&MOV_IMM);
    fx.put_function(f, f + 6);
    let img = fx.image();
    let rtti = RttiIndex::new(&img);
    let xrefs = StringXref::new(&img);
    let runner = RecipeRunner::new(&img, &rtti, &xrefs);
    let recipe = Recipe {
        steps: vec![
            Step::Pattern { bytes: "B8 05 00 00 00".into(), offset: 0, literal_ok: Vec::new() },
            Step::Operand { index: 1, kind: OperandKind::Immediate },
            Step::Call { nth: 0 },
        ],
    };
    let result = runner.run(&recipe, EntryKind::Function, &BTreeMap::new());
    assert!(matches!(result, Err(StepError::NotAnRva(5))));
}

#[test]
fn a_displacement_cannot_feed_pattern() {
    let mut fx = ImageFixture::new(BASE);
    let f = fx.put_text(&MOV_DISP);
    fx.put_function(f, f + 8);
    let img = fx.image();
    let rtti = RttiIndex::new(&img);
    let xrefs = StringXref::new(&img);
    let runner = RecipeRunner::new(&img, &rtti, &xrefs);
    let recipe = Recipe {
        steps: vec![
            Step::Pattern { bytes: "48 8B 81 ? ? ? ?".into(), offset: 0, literal_ok: Vec::new() },
            Step::Operand { index: 1, kind: OperandKind::Displacement },
            Step::Pattern { bytes: "C3".into(), offset: 0, literal_ok: Vec::new() },
        ],
    };
    let result = runner.run(&recipe, EntryKind::Field, &BTreeMap::new());
    assert!(matches!(result, Err(StepError::NotAnRva(0x5A8))));
}

#[test]
fn a_vslot_entry_cannot_end_at_an_rva() {
    let mut fx = ImageFixture::new(BASE);
    let f = fx.put_text(&[0xC3]);
    fx.put_class(CLASS, &[f]);
    let img = fx.image();
    let rtti = RttiIndex::new(&img);
    let xrefs = StringXref::new(&img);
    let runner = RecipeRunner::new(&img, &rtti, &xrefs);
    let recipe = Recipe { steps: vec![Step::RttiVtable { class: CLASS.into(), slot: None }] };
    let result = runner.run(&recipe, EntryKind::VSlot, &BTreeMap::new());
    assert!(matches!(result, Err(StepError::WrongResult { kind: EntryKind::VSlot, got }) if got == "rva"));
}

#[test]
fn a_field_entry_cannot_end_at_a_pattern() {
    let mut fx = ImageFixture::new(BASE);
    fx.put_text(&NEEDLE);
    let img = fx.image();
    let rtti = RttiIndex::new(&img);
    let xrefs = StringXref::new(&img);
    let runner = RecipeRunner::new(&img, &rtti, &xrefs);
    let result = runner.run(&Recipes::pattern(0), EntryKind::Field, &BTreeMap::new());
    assert!(matches!(result, Err(StepError::WrongResult { kind: EntryKind::Field, got }) if got == "rva"));
}

#[test]
fn a_function_entry_cannot_end_at_a_slot() {
    let mut fx = ImageFixture::new(BASE);
    let f = fx.put_text(&[0xC3]);
    fx.put_class(CLASS, &[f]);
    let img = fx.image();
    let rtti = RttiIndex::new(&img);
    let xrefs = StringXref::new(&img);
    let runner = RecipeRunner::new(&img, &rtti, &xrefs);
    let recipe = Recipe {
        steps: vec![Step::RttiVtable { class: CLASS.into(), slot: Some(0) }, Step::SlotIn { class: CLASS.into() }],
    };
    let result = runner.run(&recipe, EntryKind::Function, &BTreeMap::new());
    assert!(matches!(result, Err(StepError::WrongResult { kind: EntryKind::Function, got }) if got == "slot"));
}

#[test]
fn rtti_vtable_without_a_slot_is_the_vtable_rva() {
    let mut fx = ImageFixture::new(BASE);
    let f = fx.put_text(&[0xC3]);
    let vt = fx.put_class(CLASS, &[f]);
    let img = fx.image();
    let rtti = RttiIndex::new(&img);
    let xrefs = StringXref::new(&img);
    let runner = RecipeRunner::new(&img, &rtti, &xrefs);
    let recipe = Recipe { steps: vec![Step::RttiVtable { class: CLASS.into(), slot: None }] };
    assert_eq!(runner.run(&recipe, EntryKind::Global, &BTreeMap::new()).unwrap(), u64::from(vt));
}

#[test]
fn slot_in_gives_the_slot_index_of_the_cursor_function() {
    let mut fx = ImageFixture::new(BASE);
    let f0 = fx.put_text(&[0xC3]);
    let f1 = fx.put_text(&[0xC3]);
    fx.put_class(CLASS, &[f0, f1]);
    let img = fx.image();
    let rtti = RttiIndex::new(&img);
    let xrefs = StringXref::new(&img);
    let runner = RecipeRunner::new(&img, &rtti, &xrefs);
    let (id, resolved) = Recipes::resolved("A", f1);
    let recipe = Recipe { steps: vec![Step::Ref { entry: id }, Step::SlotIn { class: CLASS.into() }] };
    assert_eq!(runner.run(&recipe, EntryKind::VSlot, &resolved).unwrap(), 1);
}

#[test]
fn add_shifts_the_cursor_and_reports_overflow() {
    let mut fx = ImageFixture::new(BASE);
    let at = fx.put_text(&NEEDLE);
    let img = fx.image();
    let rtti = RttiIndex::new(&img);
    let xrefs = StringXref::new(&img);
    let runner = RecipeRunner::new(&img, &rtti, &xrefs);
    let first = Recipes::pattern(0).steps[0].clone();
    let shifted = Recipe { steps: vec![first.clone(), Step::Add { value: 4 }] };
    assert_eq!(runner.run(&shifted, EntryKind::Function, &BTreeMap::new()).unwrap(), u64::from(at) + 4);
    let below = Recipe { steps: vec![first, Step::Add { value: -0x10_0000 }] };
    let result = runner.run(&below, EntryKind::Function, &BTreeMap::new());
    assert!(matches!(result, Err(StepError::Overflow { .. })));
}

struct Xref;

impl Xref {
    const FIRST: u32 = 0x1000;
    const SECOND: u32 = 0x1010;

    // `lea rax, [rip + disp]; ret` placed at `at`, loading `string`
    fn code(at: u32, string: u32) -> [u8; 8] {
        let disp = string.wrapping_sub(at + 7).to_le_bytes();
        [0x48, 0x8D, 0x05, disp[0], disp[1], disp[2], disp[3], 0xC3]
    }

    // an image with the string "update" and one function per entry of `users`
    fn image(users: usize) -> Image {
        let mut fx = ImageFixture::new(BASE);
        let s = fx.put_rdata(b"update\0");
        for at in [Self::FIRST, Self::SECOND].into_iter().take(users) {
            assert_eq!(fx.put_text(&Self::code(at, s)), at);
            fx.put_function(at, at + 8);
        }
        fx.image()
    }

    fn run(img: &Image, nth: Option<u32>) -> Result<u64, StepError> {
        let rtti = RttiIndex::new(img);
        let xrefs = StringXref::new(img);
        let runner = RecipeRunner::new(img, &rtti, &xrefs);
        let recipe = Recipe { steps: vec![Step::StringXref { text: "update".into(), nth }] };
        runner.run(&recipe, EntryKind::Function, &BTreeMap::new())
    }
}

#[test]
fn string_xref_without_nth_takes_the_only_function() {
    let img = Xref::image(1);
    assert_eq!(Xref::run(&img, None).unwrap(), u64::from(Xref::FIRST));
}

#[test]
fn string_xref_without_nth_never_takes_the_first_of_several() {
    let img = Xref::image(2);
    assert!(matches!(Xref::run(&img, None), Err(StepError::ManyMatches(2))));
}

#[test]
fn string_xref_without_nth_and_without_a_user_is_no_match() {
    let img = Xref::image(0);
    assert!(matches!(Xref::run(&img, None), Err(StepError::NoMatch)));
}

#[test]
fn string_xref_with_nth_picks_by_address_or_reports_the_count() {
    let img = Xref::image(2);
    assert_eq!(Xref::run(&img, Some(0)).unwrap(), u64::from(Xref::FIRST));
    assert_eq!(Xref::run(&img, Some(1)).unwrap(), u64::from(Xref::SECOND));
    assert!(matches!(Xref::run(&img, Some(2)), Err(StepError::XrefIndex { nth: 2, found: 2 })));
}

#[test]
fn pattern_after_a_cursor_finds_the_match_in_a_chained_fragment() {
    let mut fx = ImageFixture::new(BASE);
    let primary = fx.put_text(&[0x90, 0xC3]);
    let fragment = fx.put_text(&NEEDLE);
    let other = fx.put_text(&NEEDLE);
    let image = fx.image();
    let table = FunctionTable::with_fragments(
        vec![(primary, primary + 2), (other, other + 7)],
        vec![(fragment, fragment + 7, primary)],
    );
    let sections: Vec<Section> = image.sections().to_vec();
    let img = Image::new(image.image_base(), sections, table, image.build());
    let rtti = RttiIndex::new(&img);
    let xrefs = StringXref::new(&img);
    let runner = RecipeRunner::new(&img, &rtti, &xrefs);
    let (id, resolved) = Recipes::resolved("A", primary);
    let recipe = Recipe {
        steps: vec![
            Step::Ref { entry: id },
            Step::Pattern { bytes: NEEDLE_PATTERN.to_string(), offset: 0, literal_ok: Vec::new() },
        ],
    };
    assert_eq!(runner.run(&recipe, EntryKind::Function, &resolved).unwrap(), u64::from(fragment));
}

struct VtableSteps;

impl VtableSteps {
    fn recipe(slot: u32) -> Recipe {
        Recipe { steps: vec![Step::RttiVtable { class: CLASS.into(), slot: None }, Step::VtableSlot { slot }] }
    }

    fn read_recipe(slot: u32) -> Recipe {
        Recipe {
            steps: vec![
                Step::RttiVtable { class: CLASS.into(), slot: None },
                Step::VtableSlot { slot },
                Step::ReadPointer,
            ],
        }
    }

    // `call qword ptr [rax + disp8]` found by pattern, then its displacement as a slot
    fn call_recipe(disp: u8) -> Recipe {
        Recipe {
            steps: vec![
                Step::Pattern { bytes: format!("FF 50 {disp:02X}"), offset: 0, literal_ok: Vec::new() },
                Step::Operand { index: 0, kind: OperandKind::Displacement },
                Step::DisplacementToSlot,
            ],
        }
    }
}

#[test]
fn vtable_slot_gives_the_rva_of_the_slot_when_every_slot_up_to_it_is_code() {
    let mut fx = ImageFixture::new(BASE);
    let f0 = fx.put_text(&[0xC3]);
    let f1 = fx.put_text(&[0xC3]);
    let vt = fx.put_class(CLASS, &[f0, f1]);
    let img = fx.image();
    let rtti = RttiIndex::new(&img);
    let xrefs = StringXref::new(&img);
    let runner = RecipeRunner::new(&img, &rtti, &xrefs);
    let result = runner.run(&VtableSteps::recipe(1), EntryKind::Function, &BTreeMap::new());
    assert_eq!(result.unwrap(), u64::from(vt) + 8);
}

#[test]
fn vtable_slot_fails_on_a_non_code_pointer_before_the_requested_slot() {
    let mut fx = ImageFixture::new(BASE);
    let f0 = fx.put_text(&[0xC3]);
    let data = fx.put_rdata(&[0; 8]);
    let f2 = fx.put_text(&[0xC3]);
    let vt = fx.put_class(CLASS, &[f0, data, f2]);
    let img = fx.image();
    let rtti = RttiIndex::new(&img);
    let xrefs = StringXref::new(&img);
    let runner = RecipeRunner::new(&img, &rtti, &xrefs);
    let result = runner.run(&VtableSteps::recipe(2), EntryKind::Function, &BTreeMap::new());
    assert_eq!(result, Err(StepError::NotCode { rva: vt + 8 }));
}

#[test]
fn vtable_slot_fails_when_the_slot_lies_outside_the_image() {
    let mut fx = ImageFixture::new(BASE);
    let f0 = fx.put_text(&[0xC3]);
    fx.put_class(CLASS, &[f0]);
    let img = fx.image();
    let rtti = RttiIndex::new(&img);
    let xrefs = StringXref::new(&img);
    let runner = RecipeRunner::new(&img, &rtti, &xrefs);
    let result = runner.run(&VtableSteps::recipe(40), EntryKind::Function, &BTreeMap::new());
    assert!(matches!(result, Err(StepError::NotCode { .. })));
}

#[test]
fn displacement_to_slot_divides_by_eight() {
    let mut fx = ImageFixture::new(BASE);
    fx.put_text(&[0xFF, 0x50, 0x58]);
    let img = fx.image();
    let rtti = RttiIndex::new(&img);
    let xrefs = StringXref::new(&img);
    let runner = RecipeRunner::new(&img, &rtti, &xrefs);
    let result = runner.run(&VtableSteps::call_recipe(0x58), EntryKind::VSlot, &BTreeMap::new());
    assert_eq!(result.unwrap(), 11);
}

#[test]
fn displacement_to_slot_rejects_an_unaligned_displacement() {
    let mut fx = ImageFixture::new(BASE);
    fx.put_text(&[0xFF, 0x50, 0x5C]);
    let img = fx.image();
    let rtti = RttiIndex::new(&img);
    let xrefs = StringXref::new(&img);
    let runner = RecipeRunner::new(&img, &rtti, &xrefs);
    let result = runner.run(&VtableSteps::call_recipe(0x5C), EntryKind::VSlot, &BTreeMap::new());
    assert_eq!(result, Err(StepError::NotSlotAligned(0x5C)));
}

#[test]
fn read_pointer_gives_the_function_a_vtable_slot_points_to() {
    let mut fx = ImageFixture::new(BASE);
    let f0 = fx.put_text(&[0xC3]);
    let f1 = fx.put_text(&[0x90, 0xC3]);
    fx.put_class(CLASS, &[f0, f1]);
    let img = fx.image();
    let rtti = RttiIndex::new(&img);
    let xrefs = StringXref::new(&img);
    let runner = RecipeRunner::new(&img, &rtti, &xrefs);
    let result = runner.run(&VtableSteps::read_recipe(1), EntryKind::Function, &BTreeMap::new());
    assert_eq!(result.unwrap(), u64::from(f1));
}

#[test]
fn read_pointer_fails_on_a_pointer_to_data() {
    let mut fx = ImageFixture::new(BASE);
    let f0 = fx.put_text(&[0xC3]);
    let data = fx.put_rdata(&[0; 8]);
    let vt = fx.put_class(CLASS, &[f0, data]);
    let img = fx.image();
    let rtti = RttiIndex::new(&img);
    let xrefs = StringXref::new(&img);
    let runner = RecipeRunner::new(&img, &rtti, &xrefs);
    let recipe = Recipe {
        steps: vec![Step::RttiVtable { class: CLASS.into(), slot: None }, Step::Add { value: 8 }, Step::ReadPointer],
    };
    let result = runner.run(&recipe, EntryKind::Function, &BTreeMap::new());
    assert_eq!(result, Err(StepError::NotCode { rva: vt + 8 }));
}

#[test]
fn vtable_slot_fails_on_the_zero_qword_that_ends_a_vtable() {
    let mut fx = ImageFixture::new(BASE);
    let f0 = fx.put_text(&[0xC3]);
    let vt = fx.put_class(CLASS, &[f0]);
    let end = fx.put_rdata(&[0; 8]);
    assert_eq!(end, vt + 8);
    let img = fx.image();
    let rtti = RttiIndex::new(&img);
    let xrefs = StringXref::new(&img);
    let runner = RecipeRunner::new(&img, &rtti, &xrefs);
    let result = runner.run(&VtableSteps::recipe(1), EntryKind::Function, &BTreeMap::new());
    assert_eq!(result, Err(StepError::NotCode { rva: vt + 8 }));
}

#[test]
fn read_pointer_fails_on_the_zero_qword_that_ends_a_vtable() {
    let mut fx = ImageFixture::new(BASE);
    let f0 = fx.put_text(&[0xC3]);
    let vt = fx.put_class(CLASS, &[f0]);
    let end = fx.put_rdata(&[0; 8]);
    assert_eq!(end, vt + 8);
    let img = fx.image();
    let rtti = RttiIndex::new(&img);
    let xrefs = StringXref::new(&img);
    let runner = RecipeRunner::new(&img, &rtti, &xrefs);
    let recipe = Recipe {
        steps: vec![Step::RttiVtable { class: CLASS.into(), slot: None }, Step::Add { value: 8 }, Step::ReadPointer],
    };
    let result = runner.run(&recipe, EntryKind::Function, &BTreeMap::new());
    assert_eq!(result, Err(StepError::NotCode { rva: vt + 8 }));
}

struct SlotOfSteps;

impl SlotOfSteps {
    const TABLE: &'static str = "T";

    // a function found by pattern, then its slot in the vtable of global `T`
    fn recipe() -> Recipe {
        Recipe {
            steps: vec![
                Step::Pattern { bytes: NEEDLE_PATTERN.to_string(), offset: 0, literal_ok: Vec::new() },
                Step::SlotOf { vtable: EntryId(Self::TABLE.to_string()) },
            ],
        }
    }

    fn table(vtable: u32) -> BTreeMap<EntryId, Resolved> {
        let mut map = BTreeMap::new();
        let id = EntryId(Self::TABLE.to_string());
        map.insert(id, Resolved { kind: EntryKind::Global, value: u64::from(vtable), recipes_agreeing: 1 });
        map
    }
}

#[test]
fn slot_of_gives_the_slot_that_holds_the_function() {
    let mut fx = ImageFixture::new(BASE);
    let f0 = fx.put_text(&[0xC3]);
    let f1 = fx.put_text(&[0xC3]);
    let f2 = fx.put_text(&[0xC3]);
    let target = fx.put_text(&NEEDLE);
    let vt = fx.put_class(CLASS, &[f0, f1, f2, target]);
    let img = fx.image();
    let rtti = RttiIndex::new(&img);
    let xrefs = StringXref::new(&img);
    let runner = RecipeRunner::new(&img, &rtti, &xrefs);
    let result = runner.run(&SlotOfSteps::recipe(), EntryKind::VSlot, &SlotOfSteps::table(vt));
    assert_eq!(result.unwrap(), 3);
}

#[test]
fn slot_of_fails_on_the_zero_qword_that_ends_a_vtable_without_a_match() {
    let mut fx = ImageFixture::new(BASE);
    let f0 = fx.put_text(&[0xC3]);
    fx.put_text(&NEEDLE);
    let vt = fx.put_class(CLASS, &[f0]);
    let img = fx.image();
    let rtti = RttiIndex::new(&img);
    let xrefs = StringXref::new(&img);
    let runner = RecipeRunner::new(&img, &rtti, &xrefs);
    let result = runner.run(&SlotOfSteps::recipe(), EntryKind::VSlot, &SlotOfSteps::table(vt));
    assert!(matches!(result, Err(StepError::NotCode { .. })));
}

#[test]
fn slot_of_gives_up_after_1024_code_slots_without_a_match() {
    let mut fx = ImageFixture::new(BASE);
    let f0 = fx.put_text(&[0xC3]);
    fx.put_text(&NEEDLE);
    let vt = fx.put_class(CLASS, &vec![f0; 1100]);
    let img = fx.image();
    let rtti = RttiIndex::new(&img);
    let xrefs = StringXref::new(&img);
    let runner = RecipeRunner::new(&img, &rtti, &xrefs);
    let result = runner.run(&SlotOfSteps::recipe(), EntryKind::VSlot, &SlotOfSteps::table(vt));
    assert!(matches!(result, Err(StepError::NotInVtable { rva, .. }) if rva > 0));
}

#[test]
fn slot_of_fails_on_a_non_code_pointer_before_the_match() {
    let mut fx = ImageFixture::new(BASE);
    let f0 = fx.put_text(&[0xC3]);
    let data = fx.put_rdata(&[0; 8]);
    let target = fx.put_text(&NEEDLE);
    let vt = fx.put_class(CLASS, &[f0, data, target]);
    let img = fx.image();
    let rtti = RttiIndex::new(&img);
    let xrefs = StringXref::new(&img);
    let runner = RecipeRunner::new(&img, &rtti, &xrefs);
    let result = runner.run(&SlotOfSteps::recipe(), EntryKind::VSlot, &SlotOfSteps::table(vt));
    assert_eq!(result, Err(StepError::NotCode { rva: vt + 8 }));
}

#[test]
fn slot_of_fails_when_the_vtable_entry_did_not_resolve() {
    let mut fx = ImageFixture::new(BASE);
    fx.put_text(&NEEDLE);
    let img = fx.image();
    let rtti = RttiIndex::new(&img);
    let xrefs = StringXref::new(&img);
    let runner = RecipeRunner::new(&img, &rtti, &xrefs);
    let result = runner.run(&SlotOfSteps::recipe(), EntryKind::VSlot, &BTreeMap::new());
    assert_eq!(result, Err(StepError::DependencyFailed(EntryId("T".to_string()))));
}

struct Alt;

impl Alt {
    fn any(patterns: &[&str]) -> Recipe {
        Recipe {
            steps: vec![Step::PatternAny {
                patterns: patterns.iter().map(|p| p.to_string()).collect(),
                offset: 0,
                literal_ok: Vec::new(),
            }],
        }
    }

    fn run(fx: &ImageFixture, recipe: &Recipe) -> Result<u64, StepError> {
        let img = fx.image();
        let rtti = RttiIndex::new(&img);
        let xrefs = StringXref::new(&img);
        RecipeRunner::new(&img, &rtti, &xrefs).run(recipe, EntryKind::Function, &BTreeMap::new())
    }

    fn start_of(fx: &ImageFixture, at: u32) -> Result<u64, StepError> {
        let recipe = Recipe { steps: vec![Step::Ref { entry: EntryId("A".into()) }, Step::FunctionStartOf] };
        let (_, resolved) = Recipes::resolved("A", at);
        let img = fx.image();
        let rtti = RttiIndex::new(&img);
        let xrefs = StringXref::new(&img);
        RecipeRunner::new(&img, &rtti, &xrefs).run(&recipe, EntryKind::Function, &resolved)
    }
}

#[test]
fn function_start_of_inside_a_primary_gives_its_start() {
    let mut fx = ImageFixture::new(BASE);
    let start = fx.put_text(&[0x90; 16]);
    fx.put_function(start, start + 16);
    assert_eq!(Alt::start_of(&fx, start + 9).unwrap(), u64::from(start));
}

#[test]
fn function_start_of_inside_a_fragment_gives_the_primary_start() {
    let mut fx = ImageFixture::new(BASE);
    let start = fx.put_text(&[0x90; 16]);
    fx.put_function(start, start + 16);
    let cold = fx.put_text(&[0x90; 16]);
    fx.put_fragment(cold, cold + 16, start);
    assert_eq!(Alt::start_of(&fx, cold + 5).unwrap(), u64::from(start));
}

#[test]
fn function_start_of_without_pdata_fails_and_does_not_walk_to_a_neighbour() {
    let mut fx = ImageFixture::new(BASE);
    let start = fx.put_text(&[0x90; 16]);
    fx.put_function(start, start + 16);
    let leaf = fx.put_text(&[0x90; 16]);
    let result = Alt::start_of(&fx, leaf + 2);
    assert!(matches!(result, Err(StepError::NotInFunction(rva)) if rva == leaf + 2));
}

#[test]
fn pattern_any_with_one_alternative_matching_gives_that_match() {
    let mut fx = ImageFixture::new(BASE);
    let at = fx.put_text(&NEEDLE);
    assert_eq!(Alt::run(&fx, &Alt::any(&["AA BB CC", NEEDLE_PATTERN])).unwrap(), u64::from(at));
}

#[test]
fn pattern_any_with_two_alternatives_on_the_same_position_counts_once() {
    let mut fx = ImageFixture::new(BASE);
    let at = fx.put_text(&NEEDLE);
    assert_eq!(Alt::run(&fx, &Alt::any(&[NEEDLE_PATTERN, "11 22 ?? 44"])).unwrap(), u64::from(at));
}

#[test]
fn pattern_any_with_alternatives_at_different_positions_fails() {
    let mut fx = ImageFixture::new(BASE);
    fx.put_text(&NEEDLE);
    fx.put_text(&[0xAA, 0xBB, 0xCC]);
    let result = Alt::run(&fx, &Alt::any(&[NEEDLE_PATTERN, "AA BB CC"]));
    assert!(matches!(result, Err(StepError::ManyMatches(2))));
}

#[test]
fn pattern_any_with_one_alternative_matching_twice_fails() {
    let mut fx = ImageFixture::new(BASE);
    fx.put_text(&NEEDLE);
    fx.put_text(&NEEDLE);
    let result = Alt::run(&fx, &Alt::any(&[NEEDLE_PATTERN, "AA BB CC"]));
    assert!(matches!(result, Err(StepError::ManyMatches(2))));
}

#[test]
fn pattern_any_with_no_match_fails() {
    let mut fx = ImageFixture::new(BASE);
    fx.put_text(&[0xC3]);
    assert!(matches!(Alt::run(&fx, &Alt::any(&["AA BB", "CC DD"])), Err(StepError::NoMatch)));
}

#[test]
fn pattern_any_after_a_ref_searches_only_inside_that_function_and_applies_the_offset() {
    let mut fx = ImageFixture::new(BASE);
    let a = fx.put_text(&NEEDLE);
    fx.put_function(a, a + NEEDLE.len() as u32);
    let b = fx.put_text(&NEEDLE);
    fx.put_function(b, b + NEEDLE.len() as u32);
    let (id, resolved) = Recipes::resolved("A", a);
    let recipe = Recipe {
        steps: vec![
            Step::Ref { entry: id },
            Step::PatternAny {
                patterns: vec![NEEDLE_PATTERN.to_string(), "AA BB".to_string()],
                offset: 2,
                literal_ok: Vec::new(),
            },
        ],
    };
    let img = fx.image();
    let rtti = RttiIndex::new(&img);
    let xrefs = StringXref::new(&img);
    let runner = RecipeRunner::new(&img, &rtti, &xrefs);
    assert_eq!(runner.run(&recipe, EntryKind::Function, &resolved).unwrap(), u64::from(a) + 2);
}

#[test]
fn a_pattern_that_matches_twice_still_fails_before_function_start_of() {
    let mut fx = ImageFixture::new(BASE);
    let a = fx.put_text(&NEEDLE);
    fx.put_function(a, a + NEEDLE.len() as u32);
    let b = fx.put_text(&NEEDLE);
    fx.put_function(b, b + NEEDLE.len() as u32);
    let recipe = Recipe {
        steps: vec![
            Step::Pattern { bytes: NEEDLE_PATTERN.to_string(), offset: 0, literal_ok: Vec::new() },
            Step::FunctionStartOf,
        ],
    };
    assert!(matches!(Alt::run(&fx, &recipe), Err(StepError::ManyMatches(2))));
}

#[test]
fn run_traced_reports_each_pattern_match_and_gives_the_same_result_as_run() {
    let mut fx = ImageFixture::new(BASE);
    let at = fx.put_text(&NEEDLE);
    let img = fx.image();
    let rtti = RttiIndex::new(&img);
    let xrefs = StringXref::new(&img);
    let runner = RecipeRunner::new(&img, &rtti, &xrefs);
    let step = Step::PatternAny {
        patterns: vec![NEEDLE_PATTERN.to_string(), "11 22 ? 44".to_string(), "AA BB".to_string()],
        offset: 2,
        literal_ok: Vec::new(),
    };
    let recipe = Recipe { steps: vec![step] };
    let mut hits = Vec::new();
    let traced = runner.run_traced(&recipe, EntryKind::Function, &BTreeMap::new(), &mut hits).unwrap();
    assert_eq!(traced, runner.run(&recipe, EntryKind::Function, &BTreeMap::new()).unwrap());
    assert_eq!(hits, vec![PatternHit { step: 0, rva: at, alternatives: vec![0, 1] }]);
}

#[test]
fn run_explained_gives_the_same_result_as_run() {
    let mut fx = ImageFixture::new(BASE);
    let at = fx.put_text(&NEEDLE);
    let img = fx.image();
    let rtti = RttiIndex::new(&img);
    let xrefs = StringXref::new(&img);
    let runner = RecipeRunner::new(&img, &rtti, &xrefs);
    let none = BTreeMap::new();
    let passing = Recipes::pattern(2);
    let mut steps = Vec::new();
    assert_eq!(
        runner.run_explained(&passing, EntryKind::Function, &none, &mut steps),
        runner.run(&passing, EntryKind::Function, &none)
    );
    assert_eq!(steps.len(), 1);
    assert_eq!(steps[0].cursor, Some(u64::from(at) + 2));
    let failing =
        Recipe { steps: vec![Step::Pattern { bytes: "DE AD".to_string(), offset: 0, literal_ok: Vec::new() }] };
    let mut steps = Vec::new();
    let explained = runner.run_explained(&failing, EntryKind::Function, &none, &mut steps);
    assert_eq!(explained, runner.run(&failing, EntryKind::Function, &none));
    assert!(explained.is_err());
    assert_eq!(steps[0].matches, Some(0));
}

impl Alt {
    // `pattern_any` then `alternative`, run as a field
    fn index_of(fx: &ImageFixture, patterns: &[&str]) -> Result<u64, StepError> {
        let mut recipe = Alt::any(patterns);
        recipe.steps.push(Step::Alternative);
        let img = fx.image();
        let rtti = RttiIndex::new(&img);
        let xrefs = StringXref::new(&img);
        RecipeRunner::new(&img, &rtti, &xrefs).run(&recipe, EntryKind::Field, &BTreeMap::new())
    }
}

#[test]
fn alternative_gives_the_index_of_the_pattern_any_alternative_that_matched() {
    let mut fx = ImageFixture::new(BASE);
    fx.put_text(&NEEDLE);
    assert_eq!(Alt::index_of(&fx, &["AA BB CC", NEEDLE_PATTERN]).unwrap(), 1);
    assert_eq!(Alt::index_of(&fx, &[NEEDLE_PATTERN, "AA BB CC"]).unwrap(), 0);
}

#[test]
fn alternative_fails_when_two_alternatives_matched_the_same_spot() {
    let mut fx = ImageFixture::new(BASE);
    fx.put_text(&NEEDLE);
    let result = Alt::index_of(&fx, &[NEEDLE_PATTERN, "11 22 ?? 44"]);
    assert!(matches!(result, Err(StepError::AmbiguousAlternative(both)) if both == vec![0, 1]));
}

struct MemberSteps;

impl MemberSteps {
    // the code is found by `bytes`, the cursor sits `offset` bytes into it
    fn recipe(bytes: &str, offset: i64, local_at: i64) -> Recipe {
        Recipe {
            steps: vec![
                Step::Pattern { bytes: bytes.to_string(), offset, literal_ok: Vec::new() },
                Step::MemberDisplacement { index: 1, local_at },
            ],
        }
    }

    fn run(code: &[u8], recipe: &Recipe) -> Result<u64, StepError> {
        let mut fx = ImageFixture::new(BASE);
        fx.put_text(code);
        let img = fx.image();
        let rtti = RttiIndex::new(&img);
        let xrefs = StringXref::new(&img);
        RecipeRunner::new(&img, &rtti, &xrefs).run(recipe, EntryKind::Field, &BTreeMap::new())
    }
}

#[test]
fn member_displacement_through_an_object_pointer_is_the_displacement() {
    // mov rax,[rsi+40h]
    let recipe = MemberSteps::recipe("48 8B 46 40", 0, -4);
    assert_eq!(MemberSteps::run(&[0x48, 0x8B, 0x46, 0x40], &recipe), Ok(0x40));
}

#[test]
fn member_displacement_of_a_local_subtracts_the_lea_of_the_local() {
    // lea rdx,[rbp-58h]; mov rcx,[rbp-18h]
    let code = [0x48, 0x8D, 0x55, 0xA8, 0x48, 0x8B, 0x4D, 0xE8];
    let recipe = MemberSteps::recipe("48 8D 55 A8 48 8B 4D E8", 4, -4);
    assert_eq!(MemberSteps::run(&code, &recipe), Ok(0x40));
}

#[test]
fn member_displacement_of_a_local_below_its_base_is_refused() {
    // lea rdx,[rbp-58h]; mov rcx,[rbp-60h]
    let code = [0x48, 0x8D, 0x55, 0xA8, 0x48, 0x8B, 0x4D, 0xA0];
    let recipe = MemberSteps::recipe("48 8D 55 A8 48 8B 4D A0", 4, -4);
    assert!(matches!(MemberSteps::run(&code, &recipe), Err(StepError::NoOperand { .. })));
}

#[test]
fn member_displacement_of_an_add_is_its_immediate() {
    // add rsi,30h
    let recipe = MemberSteps::recipe("48 83 C6 30", 0, -4);
    assert_eq!(MemberSteps::run(&[0x48, 0x83, 0xC6, 0x30], &recipe), Ok(0x30));
}

#[test]
fn member_displacement_of_an_instruction_without_a_memory_operand_is_refused() {
    // mov rax,rsi
    let recipe = MemberSteps::recipe("48 89 F0", 0, -4);
    assert!(matches!(MemberSteps::run(&[0x48, 0x89, 0xF0], &recipe), Err(StepError::NoOperand { .. })));
}

#[test]
fn vtable_holding_step_runs_in_a_recipe() {
    let mut fx = ImageFixture::new(BASE);
    let body = fx.put_text(&NEEDLE);
    fx.put_function(body, body + NEEDLE.len() as u32);
    let filler = fx.put_text(&[0xC3]);
    let mut slots = Vec::new();
    for pointer in [filler, body, filler] {
        slots.extend((BASE + u64::from(pointer)).to_le_bytes());
    }
    let mut bytes = 0u64.to_le_bytes().to_vec();
    bytes.extend(slots);
    let table = fx.put_data(&bytes) + 8;
    let site = fx.put_text(&[]);
    let mut code = vec![0x48, 0x8D, 0x05];
    code.extend(table.wrapping_sub(site + 7).to_le_bytes());
    code.extend([0x48, 0x89, 0x01, 0xC3]);
    let start = fx.put_text(&code);
    fx.put_function(start, start + code.len() as u32);
    let recipe = Recipe {
        steps: vec![
            Step::Pattern { bytes: NEEDLE_PATTERN.to_string(), offset: 0, literal_ok: Vec::new() },
            Step::FunctionStartOf,
            Step::VtableHolding,
        ],
    };
    assert_eq!(Alt::run(&fx, &recipe).unwrap(), u64::from(table));
}
