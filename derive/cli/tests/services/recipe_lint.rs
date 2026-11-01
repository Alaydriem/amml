use amml_data::{Check, Entry, EntryId, EntryKind, LiteralOk, Recipe, RecipeSet, Step};
use amml_derive::testing::ImageFixture;
use amml_derive::{FieldKind, LintFinding, LintReport, RecipeLint};

const BASE: u64 = 0x1_4000_0000;
const CLASS: &str = ".?AVClientInstance@@";

// push rbx; sub rsp,28h; mov rax,[rsi+1A8h]; call rel32; ret; int3
const FUNCTION: [u8; 19] =
    [0x53, 0x48, 0x83, 0xEC, 0x28, 0x48, 0x8B, 0x86, 0xA8, 0x01, 0x00, 0x00, 0xE8, 0x34, 0x12, 0x00, 0x00, 0xC3, 0xCC];

struct Cases;

impl Cases {
    fn entry(kind: EntryKind, steps: Vec<Step>, checks: Vec<Check>) -> Entry {
        Entry {
            id: EntryId("E".to_string()),
            kind,
            msvc_name: None,
            recipes: vec![Recipe { steps }],
            checks,
            notes: String::new(),
            hooked: false,
        }
    }

    fn pattern(bytes: &str, kept: &[&str]) -> Step {
        let literal_ok = kept
            .iter()
            .map(|text| match LiteralOk::parse(text, false) {
                Ok(item) => item,
                Err(message) => panic!("{message}"),
            })
            .collect();
        Step::Pattern { bytes: bytes.to_string(), offset: 0, literal_ok }
    }

    // lints one entry over `FUNCTION`, which is registered as a function (without its trailing int3)
    fn lint(entry: Entry) -> LintReport {
        let mut fx = ImageFixture::new(BASE);
        let start = fx.put_text(&FUNCTION);
        fx.put_function(start, start + FUNCTION.len() as u32 - 1);
        Self::lint_image(&fx, entry)
    }

    fn lint_image(fx: &ImageFixture, entry: Entry) -> LintReport {
        let image = fx.image();
        let Ok(set) = RecipeSet::new(vec![entry]) else { panic!("the entry is invalid") };
        match RecipeLint::new(&image, &set).lint() {
            Ok(report) => report,
            Err(error) => panic!("{error}"),
        }
    }

    fn pattern_lint(bytes: &str, kept: &[&str]) -> LintReport {
        Self::lint(Self::entry(EntryKind::Function, vec![Self::pattern(bytes, kept)], Vec::new()))
    }

    // (byte index, field, instruction text, kept reason) of every fixed-byte finding
    fn bytes(report: &LintReport) -> Vec<(usize, FieldKind, String, Option<String>)> {
        report
            .entries
            .iter()
            .flat_map(|entry| &entry.findings)
            .filter_map(|finding| match finding {
                LintFinding::Byte(b) => Some((b.byte, b.field, b.instruction.clone(), b.kept.clone())),
                _ => None,
            })
            .collect()
    }
}

#[test]
fn a_literal_sub_rsp_immediate_is_an_immediate_finding() {
    let report = Cases::pattern_lint("48 83 EC 28", &[]);
    assert_eq!(Cases::bytes(&report), vec![(3, FieldKind::Immediate, "sub rsp,28h".to_string(), None)]);
    assert_eq!(report.open_findings(), 1);
    assert_eq!(report.exit_code(), 1);
}

#[test]
fn the_same_pattern_with_the_immediate_wildcarded_is_clean() {
    let report = Cases::pattern_lint("48 83 EC ?", &[]);
    assert!(Cases::bytes(&report).is_empty());
    assert_eq!(report.open_findings(), 0);
    assert_eq!(report.exit_code(), 0);
}

#[test]
fn a_literal_displacement_is_found_and_the_opcode_bytes_are_not() {
    let report = Cases::pattern_lint("8B 86 A8 01 00 00", &[]);
    let found = Cases::bytes(&report);
    let fields: Vec<_> = found.iter().map(|(byte, field, ..)| (*byte, *field)).collect();
    assert_eq!(fields, (2..=5).map(|byte| (byte, FieldKind::Displacement)).collect::<Vec<_>>());
    assert!(found.iter().all(|(_, _, text, _)| text == "mov rax,[rsi+1A8h]"));
    assert!(Cases::bytes(&Cases::pattern_lint("8B 86 ? ? ? ?", &[])).is_empty());
}

#[test]
fn a_pattern_that_starts_inside_the_displacement_still_decodes_the_whole_instruction() {
    // starts at the second byte of the disp32 of `mov rax,[rsi+1A8h]`
    let report = Cases::pattern_lint("01 00 00 E8 ? ? ? ? C3", &[]);
    let found = Cases::bytes(&report);
    assert_eq!(found.iter().map(|(byte, ..)| *byte).collect::<Vec<_>>(), vec![0, 1, 2]);
    assert!(found.iter().all(|(_, field, text, _)| *field == FieldKind::Displacement && text == "mov rax,[rsi+1A8h]"));
}

#[test]
fn a_pattern_that_starts_inside_the_instruction_with_the_field_wildcarded_is_clean() {
    let report = Cases::pattern_lint("86 ? ? ? ? E8 ? ? ? ? C3", &[]);
    assert!(Cases::bytes(&report).is_empty());
    assert_eq!(report.exit_code(), 0);
}

#[test]
fn a_literal_call_offset_is_a_branch_finding() {
    let report = Cases::pattern_lint("E8 34 12 00 00 C3", &[]);
    let found = Cases::bytes(&report);
    assert_eq!(found.iter().map(|(byte, ..)| *byte).collect::<Vec<_>>(), vec![1, 2, 3, 4]);
    assert!(found.iter().all(|(_, field, ..)| *field == FieldKind::Branch));
    assert!(Cases::bytes(&Cases::pattern_lint("E8 ? ? ? ? C3", &[])).is_empty());
}

#[test]
fn literal_ok_turns_a_finding_into_a_kept_byte() {
    let kept = ["1: stable stub", "2: stable stub", "3: stable stub", "4: stable stub"];
    let report = Cases::pattern_lint("E8 34 12 00 00 C3", &kept);
    let found = Cases::bytes(&report);
    assert_eq!(found.len(), 4);
    assert!(found.iter().all(|(.., reason)| reason.as_deref() == Some("stable stub")));
    assert_eq!(report.open_findings(), 0);
    assert_eq!(report.kept_bytes(), 4);
    assert_eq!(report.exit_code(), 0);
    let text = RecipeLint::render(&report);
    assert!(text.contains("kept: stable stub"));
    assert!(!text.contains("fixed byte"));
}

#[test]
fn a_partly_kept_pattern_still_fails() {
    let report = Cases::pattern_lint("E8 34 12 00 00 C3", &["1: stable"]);
    assert_eq!(report.open_findings(), 3);
    assert_eq!(report.kept_bytes(), 1);
    assert_eq!(report.exit_code(), 1);
}

#[test]
fn every_matching_alternative_of_pattern_any_is_linted_with_its_index() {
    let step = Step::PatternAny {
        patterns: vec!["48 83 EC 28".to_string(), "48 83 EC ? 48 8B".to_string()],
        offset: 0,
        literal_ok: Vec::new(),
    };
    let report = Cases::lint(Cases::entry(EntryKind::Function, vec![step], Vec::new()));
    let alternatives: Vec<_> = report.entries[0]
        .findings
        .iter()
        .filter_map(|finding| match finding {
            LintFinding::Byte(b) => Some((b.alternative, b.byte)),
            _ => None,
        })
        .collect();
    assert_eq!(alternatives, vec![(Some(0), 3)]);
}

#[test]
fn vtable_slot_on_a_function_entry_is_flagged() {
    let mut fx = ImageFixture::new(BASE);
    let start = fx.put_text(&FUNCTION);
    fx.put_function(start, start + FUNCTION.len() as u32 - 1);
    fx.put_class(CLASS, &[start; 25]);
    let steps = vec![
        Step::RttiVtable { class: CLASS.to_string(), slot: None },
        Step::VtableSlot { slot: 24 },
        Step::ReadPointer,
    ];
    let report = Cases::lint_image(&fx, Cases::entry(EntryKind::Function, steps, Vec::new()));
    assert_eq!(report.entries[0].findings, vec![LintFinding::LiteralSlot { recipe: 0, step: 1, slot: 24 }]);
    assert_eq!(report.exit_code(), 1);
}

#[test]
fn a_narrow_range_check_on_a_vslot_is_flagged_and_an_eight_slot_one_is_not() {
    let steps = vec![
        Cases::pattern("48 8B 86 ? ? ? ?", &[]),
        Step::Operand { index: 1, kind: amml_data::OperandKind::Displacement },
        Step::DisplacementToSlot,
    ];
    let narrow = Cases::lint(Cases::entry(EntryKind::VSlot, steps.clone(), vec![Check::Range { min: 0, max: 6 }]));
    assert!(narrow.entries[0].findings.contains(&LintFinding::NarrowRange { min: 0, max: 6 }));
    let edge = Cases::lint(Cases::entry(EntryKind::VSlot, steps, vec![Check::Range { min: 0, max: 7 }]));
    assert!(!edge.entries[0].findings.iter().any(|finding| matches!(finding, LintFinding::NarrowRange { .. })));
}

#[test]
fn a_match_outside_every_function_is_not_decodable_and_never_clean() {
    // the offset makes the value differ from the match start, so nothing proves the match starts an instruction
    let mut fx = ImageFixture::new(BASE);
    fx.put_text(&FUNCTION);
    let step = Step::Pattern { bytes: "48 83 EC ?".to_string(), offset: 1, literal_ok: Vec::new() };
    let entry = Cases::entry(EntryKind::Function, vec![step], Vec::new());
    let report = Cases::lint_image(&fx, entry);
    assert!(matches!(report.entries[0].findings.as_slice(), [LintFinding::NotDecodable { .. }]));
    assert_eq!(report.exit_code(), 1);
}

#[test]
fn a_leaf_without_pdata_whose_start_is_the_value_is_decoded_from_the_match() {
    let mut fx = ImageFixture::new(BASE);
    fx.put_text(&FUNCTION);
    let entry = Cases::entry(EntryKind::Function, vec![Cases::pattern("53 48 83 EC 28", &[])], Vec::new());
    let report = Cases::lint_image(&fx, entry);
    assert_eq!(Cases::bytes(&report), vec![(4, FieldKind::Immediate, "sub rsp,28h".to_string(), None)]);
}

#[test]
fn a_one_step_leaf_match_that_is_not_the_entry_value_is_not_decodable() {
    // the recipe's own result is the match start, but the entry fails its check, so nothing proves the start
    let mut fx = ImageFixture::new(BASE);
    fx.put_text(&FUNCTION);
    let entry =
        Cases::entry(EntryKind::Function, vec![Cases::pattern("53 48 83 EC 28", &[])], vec![Check::FunctionStart]);
    let report = Cases::lint_image(&fx, entry);
    assert!(matches!(report.entries[0].findings.as_slice(), [LintFinding::NotDecodable { .. }]));
}

#[test]
fn a_leaf_without_pdata_found_by_slot_of_is_decoded_from_the_match() {
    let mut fx = ImageFixture::new(BASE);
    let start = fx.put_text(&FUNCTION);
    fx.put_class(CLASS, &[start]);
    let table = Entry {
        id: EntryId("VT".to_string()),
        kind: EntryKind::Global,
        msvc_name: None,
        recipes: vec![Recipe { steps: vec![Step::RttiVtable { class: CLASS.to_string(), slot: None }] }],
        checks: Vec::new(),
        notes: String::new(),
        hooked: false,
    };
    let steps = vec![Cases::pattern("53 48 83 EC 28", &[]), Step::SlotOf { vtable: EntryId("VT".to_string()) }];
    let slot = Cases::entry(EntryKind::VSlot, steps, vec![Check::Range { min: 0, max: 7 }]);
    let image = fx.image();
    let Ok(set) = RecipeSet::new(vec![table, slot]) else { panic!("the entries are invalid") };
    let report = match RecipeLint::new(&image, &set).lint() {
        Ok(report) => report,
        Err(error) => panic!("{error}"),
    };
    assert_eq!(Cases::bytes(&report), vec![(4, FieldKind::Immediate, "sub rsp,28h".to_string(), None)]);
}

#[test]
fn a_recipe_that_does_not_run_is_listed_and_never_clean() {
    let report = Cases::pattern_lint("DE AD BE EF", &[]);
    assert!(matches!(report.entries[0].findings.as_slice(), [LintFinding::DidNotRun { recipe: 0, .. }]));
    assert_eq!(report.exit_code(), 1);
}

#[test]
fn the_report_groups_findings_by_entry() {
    let text = RecipeLint::render(&Cases::pattern_lint("48 83 EC 28", &[]));
    assert!(text.contains("## E (function)"));
    assert!(text.contains("fixed byte: recipe 0, step 0, byte 3 (`28`): immediate in `sub rsp,28h`"));
}

struct Raw;

impl Raw {
    // lints a one-pattern function entry over `code`, which is registered whole as a function
    fn lint(code: &[u8], pattern: &str) -> LintReport {
        let mut fx = ImageFixture::new(BASE);
        let start = fx.put_text(code);
        fx.put_function(start, start + code.len() as u32);
        Cases::lint_image(&fx, Cases::entry(EntryKind::Function, vec![Cases::pattern(pattern, &[])], Vec::new()))
    }
}

#[test]
fn both_immediates_of_an_instruction_with_two_are_found() {
    // enter 10h,0
    let report = Raw::lint(&[0xC8, 0x10, 0x00, 0x00, 0xC3], "C8 10 00 00 C3");
    let found: Vec<_> = Cases::bytes(&report).into_iter().map(|(byte, field, ..)| (byte, field)).collect();
    assert_eq!(found, vec![(1, FieldKind::Immediate), (2, FieldKind::Immediate), (3, FieldKind::Immediate)]);
}

#[test]
fn a_pattern_that_ends_inside_a_field_reports_only_its_own_bytes() {
    let report = Cases::pattern_lint("E8 34 12", &[]);
    let found: Vec<_> = Cases::bytes(&report).into_iter().map(|(byte, field, ..)| (byte, field)).collect();
    assert_eq!(found, vec![(1, FieldKind::Branch), (2, FieldKind::Branch)]);
}

#[test]
fn displacement_and_immediate_of_one_instruction_are_both_found() {
    // mov dword ptr [rcx+10h],1
    let report = Raw::lint(&[0xC7, 0x41, 0x10, 0x01, 0x00, 0x00, 0x00, 0xC3], "C7 41 10 01 00 00 00 C3");
    let found: Vec<_> = Cases::bytes(&report).into_iter().map(|(byte, field, ..)| (byte, field)).collect();
    let mut expected = vec![(2, FieldKind::Displacement)];
    expected.extend((3..=6).map(|byte| (byte, FieldKind::Immediate)));
    assert_eq!(found, expected);
}

#[test]
fn literal_ok_on_one_alternative_does_not_keep_the_same_byte_of_another() {
    let step = Step::PatternAny {
        patterns: vec!["48 83 EC 28".to_string(), "48 83 EC 28 48 8B".to_string()],
        offset: 0,
        literal_ok: vec![LiteralOk { alternative: Some(0), index: 3, reason: "frame".to_string() }],
    };
    let report = Cases::lint(Cases::entry(EntryKind::Function, vec![step], Vec::new()));
    let rows: Vec<_> = report.entries[0]
        .findings
        .iter()
        .filter_map(|finding| match finding {
            LintFinding::Byte(b) => Some((b.alternative, b.byte, b.kept.is_some())),
            _ => None,
        })
        .collect();
    assert_eq!(rows, vec![(Some(0), 3, true), (Some(1), 3, false)]);
    assert_eq!(report.open_findings(), 1);
    assert!(report.entries[0].info.is_empty());
}

#[test]
fn a_literal_ok_that_keeps_nothing_is_an_info_line_and_does_not_fail() {
    let report = Cases::pattern_lint("48 83 EC ? 48", &["0: no field here"]);
    assert_eq!(report.exit_code(), 0);
    assert_eq!(report.entries[0].info.len(), 1);
    assert!(report.entries[0].info[0].starts_with("unused literal_ok: recipe 0, step 0, `0: no field here`"));
    assert!(RecipeLint::render(&report).contains("- info: unused literal_ok"));
}

#[test]
fn a_pattern_any_alternative_that_did_not_hit_is_an_info_line() {
    let step = Step::PatternAny {
        patterns: vec!["48 83 EC ?".to_string(), "DE AD".to_string()],
        offset: 0,
        literal_ok: Vec::new(),
    };
    let report = Cases::lint(Cases::entry(EntryKind::Function, vec![step], Vec::new()));
    assert_eq!(report.exit_code(), 0);
    assert_eq!(
        report.entries[0].info,
        vec!["alternative 1 not linted (no match on this build): recipe 0, step 0".to_string()]
    );
}

#[test]
fn a_field_read_from_the_start_of_a_leaf_that_a_function_entry_gives_is_decoded() {
    let mut fx = ImageFixture::new(BASE);
    fx.put_text(&FUNCTION);
    let leaf = Entry {
        id: EntryId("Leaf".to_string()),
        ..Cases::entry(EntryKind::Function, vec![Cases::pattern("53 48 83 EC ??", &[])], Vec::new())
    };
    let read = Step::Pattern { bytes: "53 48 83 EC 28".to_string(), offset: 1, literal_ok: Vec::new() };
    let operand = Step::Operand { index: 1, kind: amml_data::OperandKind::Immediate };
    let field = Cases::entry(EntryKind::Field, vec![read, operand], vec![Check::Range { min: 0, max: 64 }]);
    let image = fx.image();
    let Ok(set) = RecipeSet::new(vec![leaf, field]) else { panic!("the entries are invalid") };
    let report = match RecipeLint::new(&image, &set).lint() {
        Ok(report) => report,
        Err(error) => panic!("{error}"),
    };
    assert_eq!(Cases::bytes(&report), vec![(4, FieldKind::Immediate, "sub rsp,28h".to_string(), None)]);
}

#[test]
fn a_field_read_from_a_leaf_that_no_function_entry_gives_is_not_decodable() {
    let mut fx = ImageFixture::new(BASE);
    fx.put_text(&FUNCTION);
    let read = Step::Pattern { bytes: "53 48 83 EC ??".to_string(), offset: 1, literal_ok: Vec::new() };
    let operand = Step::Operand { index: 1, kind: amml_data::OperandKind::Immediate };
    let field = Cases::entry(EntryKind::Field, vec![read, operand], vec![Check::Range { min: 0, max: 64 }]);
    let report = Cases::lint_image(&fx, field);
    assert!(matches!(report.entries[0].findings.as_slice(), [LintFinding::NotDecodable { .. }]));
}

// a guard field `G` (reads the displacement 1A8h from FUNCTION, or fails to match) and an entry `E` whose only
// recipe never matches, gated by `entry_is G = wanted`
fn gated_report(guard_matches: bool, wanted: u64) -> LintReport {
    let guard_bytes = if guard_matches { "48 8B 86 ? ? ? ?" } else { "EE EE EE EE" };
    let mut guard = Cases::entry(
        EntryKind::Field,
        vec![Cases::pattern(guard_bytes, &[]), Step::Operand { index: 1, kind: amml_data::OperandKind::Displacement }],
        Vec::new(),
    );
    guard.id = EntryId("G".to_string());
    let gated = Cases::entry(
        EntryKind::Function,
        vec![Cases::pattern("FF FF FF FF", &[])],
        vec![Check::EntryIs { entry: EntryId("G".to_string()), value: wanted }],
    );
    let mut fx = ImageFixture::new(BASE);
    let start = fx.put_text(&FUNCTION);
    fx.put_function(start, start + FUNCTION.len() as u32 - 1);
    let image = fx.image();
    let Ok(set) = RecipeSet::new(vec![guard, gated]) else { panic!("the entries are invalid") };
    match RecipeLint::new(&image, &set).lint() {
        Ok(report) => report,
        Err(error) => panic!("{error}"),
    }
}

#[test]
fn an_entry_whose_guard_matches_is_linted_as_before() {
    let report = gated_report(true, 0x1A8);
    assert_eq!(report.skipped(), 0);
    assert!(report.entries[1].findings.iter().any(|finding| matches!(finding, LintFinding::DidNotRun { .. })));
    assert_eq!(report.open_findings(), 1);
}

#[test]
fn an_entry_whose_guard_has_another_value_is_skipped_and_not_open() {
    let report = gated_report(true, 0x140);
    assert_eq!(report.skipped(), 1);
    assert!(report.entries[1].not_for_build.is_some());
    assert!(report.entries[1].findings.is_empty());
    assert_eq!(report.open_findings(), 0);
    assert_eq!(report.exit_code(), 0);
}

#[test]
fn an_entry_whose_guard_did_not_derive_stays_open() {
    let report = gated_report(false, 0x140);
    assert_eq!(report.skipped(), 0);
    assert!(report.entries[1].findings.iter().any(|finding| matches!(finding, LintFinding::DidNotRun { .. })));
    assert!(report.open_findings() >= 1);
}
