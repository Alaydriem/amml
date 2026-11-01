use amml_data::{Check, Entry, EntryId, EntryKind, OperandKind, Recipe, RecipeSet, Step};
use amml_derive::testing::ImageFixture;
use amml_derive::{Deriver, FailureReason, LayoutGuard, LayoutRefusal, StepError};

const BASE: u64 = 0x1_4000_0000;
const CLASS: &str = ".?AVClientInstance@@";
const MARK: [u8; 4] = [0x55, 0xAA, 0x55, 0xAA];

struct Entries;

impl Entries {
    fn id(name: &str) -> EntryId {
        EntryId(name.to_string())
    }

    fn entry(name: &str, kind: EntryKind, recipes: Vec<Vec<Step>>, checks: Vec<Check>) -> Entry {
        Entry {
            id: Self::id(name),
            kind,
            msvc_name: None,
            recipes: recipes.into_iter().map(|steps| Recipe { steps }).collect(),
            checks,
            notes: String::new(),
            hooked: false,
        }
    }

    fn pattern(bytes: &str, offset: i64) -> Step {
        Step::Pattern { bytes: bytes.to_string(), offset, literal_ok: Vec::new() }
    }

    fn vtable(slot: u32) -> Step {
        Step::RttiVtable { class: CLASS.to_string(), slot: Some(slot) }
    }

    fn reference(name: &str) -> Step {
        Step::Ref { entry: Self::id(name) }
    }
}

#[test]
fn recipes_that_agree_resolve_with_their_count() {
    let mut fx = ImageFixture::new(BASE);
    let f = fx.put_text(&[0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xC3]);
    fx.put_function(f, f + 6);
    fx.put_class(CLASS, &[f]);
    let img = fx.image();
    let set = RecipeSet::new(vec![Entries::entry(
        "ClientInstance.update",
        EntryKind::Function,
        vec![vec![Entries::vtable(0)], vec![Entries::pattern("AA BB CC DD EE", 0)]],
        vec![Check::FunctionStart],
    )])
    .unwrap();
    let out = Deriver::new(&img, &set).derive().unwrap();
    let resolved = &out.data.entries[&Entries::id("ClientInstance.update")];
    assert_eq!(resolved.recipes_agreeing, 2);
    assert_eq!(resolved.value, u64::from(f));
    assert!(out.failures.is_empty());
}

#[test]
fn a_failing_recipe_does_not_hide_a_working_one() {
    let mut fx = ImageFixture::new(BASE);
    let f = fx.put_text(&[0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xC3]);
    fx.put_function(f, f + 6);
    let img = fx.image();
    let set = RecipeSet::new(vec![Entries::entry(
        "X",
        EntryKind::Function,
        vec![vec![Entries::pattern("AA BB CC DD EE", 0)], vec![Entries::pattern("01 02 03", 0)]],
        vec![],
    )])
    .unwrap();
    let out = Deriver::new(&img, &set).derive().unwrap();
    assert_eq!(out.data.entries[&Entries::id("X")].recipes_agreeing, 1);
}

#[test]
fn disagreeing_recipes_fail_and_never_pick_one() {
    let mut fx = ImageFixture::new(BASE);
    let f0 = fx.put_text(&[0xC3]);
    let f1 = fx.put_text(&[0xC3]);
    fx.put_class(CLASS, &[f0, f1]);
    let img = fx.image();
    let set = RecipeSet::new(vec![Entries::entry(
        "X",
        EntryKind::Function,
        vec![vec![Entries::vtable(1)], vec![Entries::vtable(0)]],
        vec![],
    )])
    .unwrap();
    let out = Deriver::new(&img, &set).derive().unwrap();
    assert!(matches!(&out.failures[0].reason, FailureReason::Disagree(v) if v == &vec![f0 as u64, f1 as u64]));
    assert!(!out.data.entries.contains_key(&Entries::id("X")));
}

#[test]
fn all_recipes_failing_reports_every_error() {
    let mut fx = ImageFixture::new(BASE);
    fx.put_text(&[0xC3]);
    let img = fx.image();
    let set = RecipeSet::new(vec![Entries::entry(
        "X",
        EntryKind::Function,
        vec![vec![Entries::pattern("01 02 03", 0)], vec![Entries::pattern("04 05 06", 0)]],
        vec![],
    )])
    .unwrap();
    let out = Deriver::new(&img, &set).derive().unwrap();
    assert!(matches!(&out.failures[0].reason, FailureReason::AllRecipesFailed(e) if e.len() == 2));
}

#[test]
fn a_malformed_pattern_fails_only_its_entry() {
    let mut fx = ImageFixture::new(BASE);
    let f = fx.put_text(&[0xAA, 0xBB, 0xCC, 0xC3]);
    fx.put_function(f, f + 4);
    let img = fx.image();
    let set = RecipeSet::new(vec![
        Entries::entry("Bad", EntryKind::Function, vec![vec![Entries::pattern("ZZ", 0)]], vec![]),
        Entries::entry("Good", EntryKind::Function, vec![vec![Entries::pattern("AA BB CC", 0)]], vec![]),
    ])
    .unwrap();
    let out = Deriver::new(&img, &set).derive().unwrap();
    assert_eq!(out.failures.len(), 1);
    assert!(matches!(&out.failures[0].reason,
        FailureReason::AllRecipesFailed(e) if matches!(e.as_slice(), [StepError::BadPattern(_)])));
    assert!(out.data.entries.contains_key(&Entries::id("Good")));
}

#[test]
fn a_failed_check_keeps_the_entry_out_of_the_data() {
    let mut fx = ImageFixture::new(BASE);
    let f = fx.put_text(&[0x90, 0xAA, 0xBB, 0xCC, 0xC3]);
    fx.put_function(f, f + 5);
    let img = fx.image();
    let set = RecipeSet::new(vec![Entries::entry(
        "X",
        EntryKind::Function,
        vec![vec![Entries::pattern("AA BB CC", 0)]],
        vec![Check::FunctionStart],
    )])
    .unwrap();
    let out = Deriver::new(&img, &set).derive().unwrap();
    assert!(matches!(&out.failures[0].reason, FailureReason::CheckFailed(Check::FunctionStart)));
    assert!(out.data.entries.is_empty());
}

#[test]
fn dependents_of_a_failed_entry_fail_with_dependency_failed() {
    let mut fx = ImageFixture::new(BASE);
    fx.put_text(&MARK);
    fx.put_text(&MARK);
    let img = fx.image();
    let set = RecipeSet::new(vec![
        Entries::entry("A", EntryKind::Function, vec![vec![Entries::pattern("55 AA 55 AA", 0)]], vec![]),
        Entries::entry("B", EntryKind::Function, vec![vec![Entries::reference("A")]], vec![]),
    ])
    .unwrap();
    let out = Deriver::new(&img, &set).derive().unwrap();
    assert!(out.failures.iter().any(|f| f.id == Entries::id("A")));
    assert!(out.failures.iter().any(|f| f.id == Entries::id("B")
        && matches!(&f.reason, FailureReason::DependencyFailed(a) if *a == Entries::id("A"))));
}

#[test]
fn field_offset_from_a_pattern_inside_a_referenced_function() {
    let mut fx = ImageFixture::new(BASE);
    let mut code = MARK.to_vec();
    // mov rax, [rcx + 0x5A8]; ret
    code.extend_from_slice(&[0x48, 0x8B, 0x81, 0xA8, 0x05, 0x00, 0x00, 0xC3]);
    let a = fx.put_text(&code);
    fx.put_function(a, a + code.len() as u32);
    // the same load shape in another function must not count
    let other = fx.put_text(&[0x48, 0x8B, 0x81, 0x10, 0x00, 0x00, 0x00, 0xC3]);
    fx.put_function(other, other + 8);
    let img = fx.image();
    let set = RecipeSet::new(vec![
        Entries::entry("A", EntryKind::Function, vec![vec![Entries::pattern("55 AA 55 AA", 0)]], vec![]),
        Entries::entry(
            "B",
            EntryKind::Field,
            vec![vec![
                Entries::reference("A"),
                Entries::pattern("48 8B 81 ? ? ? ?", 0),
                Step::Operand { index: 1, kind: OperandKind::Displacement },
            ]],
            vec![Check::Range { min: 0, max: 0x1000 }],
        ),
    ])
    .unwrap();
    let out = Deriver::new(&img, &set).derive().unwrap();
    assert!(out.failures.is_empty());
    assert_eq!(out.data.entries[&Entries::id("B")].value, 0x5A8);
}

#[test]
fn a_dependency_cycle_is_an_error_of_the_whole_derive() {
    let mut fx = ImageFixture::new(BASE);
    fx.put_text(&[0xC3]);
    let img = fx.image();
    let set = RecipeSet::new(vec![
        Entries::entry("A", EntryKind::Function, vec![vec![Entries::reference("B")]], vec![]),
        Entries::entry("B", EntryKind::Function, vec![vec![Entries::reference("A")]], vec![]),
    ])
    .unwrap();
    assert!(Deriver::new(&img, &set).derive().is_err());
}

#[test]
fn derive_twice_gives_identical_data() {
    let mut fx = ImageFixture::new(BASE);
    let f = fx.put_text(&[0xAA, 0xBB, 0xCC, 0xC3]);
    fx.put_function(f, f + 4);
    fx.put_class(CLASS, &[f]);
    let img = fx.image();
    let set = RecipeSet::new(vec![
        Entries::entry("A", EntryKind::Function, vec![vec![Entries::vtable(0)]], vec![]),
        Entries::entry("B", EntryKind::Function, vec![vec![Entries::pattern("AA BB CC", 0)]], vec![]),
    ])
    .unwrap();
    assert_eq!(Deriver::new(&img, &set).derive().unwrap().data, Deriver::new(&img, &set).derive().unwrap().data);
}

#[test]
fn a_two_level_dependency_chain_fails_all_the_way_down() {
    let mut fx = ImageFixture::new(BASE);
    fx.put_text(&MARK);
    fx.put_text(&MARK);
    let img = fx.image();
    let set = RecipeSet::new(vec![
        Entries::entry("A", EntryKind::Function, vec![vec![Entries::pattern("55 AA 55 AA", 0)]], vec![]),
        Entries::entry("B", EntryKind::Function, vec![vec![Entries::reference("A")]], vec![]),
        Entries::entry("C", EntryKind::Function, vec![vec![Entries::reference("B")]], vec![]),
    ])
    .unwrap();
    let out = Deriver::new(&img, &set).derive().unwrap();
    let reason = |name: &str| out.failures.iter().find(|f| f.id == Entries::id(name)).map(|f| &f.reason);
    assert!(matches!(reason("B"), Some(FailureReason::DependencyFailed(t)) if *t == Entries::id("A")));
    assert!(matches!(reason("C"), Some(FailureReason::DependencyFailed(t)) if *t == Entries::id("B")));
}

#[test]
fn an_independent_recipe_saves_an_entry_whose_other_recipe_depends_on_a_failed_entry() {
    let mut fx = ImageFixture::new(BASE);
    fx.put_text(&MARK);
    fx.put_text(&MARK);
    let f = fx.put_text(&[0xAA, 0xBB, 0xCC, 0xC3]);
    fx.put_function(f, f + 4);
    let img = fx.image();
    let set = RecipeSet::new(vec![
        Entries::entry("A", EntryKind::Function, vec![vec![Entries::pattern("55 AA 55 AA", 0)]], vec![]),
        Entries::entry(
            "B",
            EntryKind::Function,
            vec![vec![Entries::reference("A")], vec![Entries::pattern("AA BB CC", 0)]],
            vec![],
        ),
    ])
    .unwrap();
    let out = Deriver::new(&img, &set).derive().unwrap();
    let resolved = &out.data.entries[&Entries::id("B")];
    assert_eq!(resolved.recipes_agreeing, 1);
    assert_eq!(resolved.value, u64::from(f));
    assert!(out.failures.iter().all(|failure| failure.id == Entries::id("A")));
}

#[test]
fn a_resolved_entry_lists_the_recipes_that_failed() {
    let mut fx = ImageFixture::new(BASE);
    let f = fx.put_text(&[0xAA, 0xBB, 0xCC, 0xC3]);
    fx.put_function(f, f + 4);
    let img = fx.image();
    let set = RecipeSet::new(vec![Entries::entry(
        "X",
        EntryKind::Function,
        vec![vec![Entries::pattern("01 02 03", 0)], vec![Entries::pattern("AA BB CC", 0)]],
        vec![Check::FunctionStart],
    )])
    .unwrap();
    let out = Deriver::new(&img, &set).derive().unwrap();
    assert_eq!(out.degraded.len(), 1);
    assert_eq!(out.degraded[0].id, Entries::id("X"));
    assert_eq!(out.degraded[0].recipe_errors, vec![(0, StepError::NoMatch)]);
    assert!(!out.degraded[0].unverified);
}

#[test]
fn one_recipe_and_no_checks_is_unverified_but_a_check_or_a_second_recipe_is_not() {
    let mut fx = ImageFixture::new(BASE);
    let f = fx.put_text(&[0xAA, 0xBB, 0xCC, 0xC3]);
    fx.put_function(f, f + 4);
    let img = fx.image();
    let one = || vec![Entries::pattern("AA BB CC", 0)];
    let set = RecipeSet::new(vec![
        Entries::entry("Bare", EntryKind::Function, vec![one()], vec![]),
        Entries::entry("Checked", EntryKind::Function, vec![one()], vec![Check::FunctionStart]),
        Entries::entry("Twice", EntryKind::Function, vec![one(), one()], vec![]),
    ])
    .unwrap();
    let out = Deriver::new(&img, &set).derive().unwrap();
    assert_eq!(out.degraded.len(), 1);
    assert_eq!(out.degraded[0].id, Entries::id("Bare"));
    assert!(out.degraded[0].unverified && out.degraded[0].recipe_errors.is_empty());
}

#[test]
fn two_derives_of_the_same_image_give_equal_outcomes() {
    let mut fx = ImageFixture::new(BASE);
    let f = fx.put_text(&[0xAA, 0xBB, 0xCC, 0xC3]);
    fx.put_function(f, f + 4);
    let img = fx.image();
    let set = RecipeSet::new(vec![
        Entries::entry("Bad", EntryKind::Function, vec![vec![Entries::pattern("01 02 03", 0)]], vec![]),
        Entries::entry("Good", EntryKind::Function, vec![vec![Entries::pattern("AA BB CC", 0)]], vec![]),
    ])
    .unwrap();
    assert_eq!(Deriver::new(&img, &set).derive().unwrap(), Deriver::new(&img, &set).derive().unwrap());
}

#[test]
fn a_vtable_route_alone_fails_and_is_never_written() {
    let mut fx = ImageFixture::new(BASE);
    let f = fx.put_text(&[0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xC3]);
    fx.put_function(f, f + 6);
    fx.put_class(CLASS, &[f]);
    let img = fx.image();
    let route = vec![
        Step::RttiVtable { class: CLASS.to_string(), slot: None },
        Step::VtableSlot { slot: 0 },
        Step::ReadPointer,
    ];
    let set = RecipeSet::new(vec![Entries::entry(
        "X",
        EntryKind::Function,
        vec![route, vec![Entries::pattern("01 02 03", 0)]],
        vec![Check::FunctionStart],
    )])
    .unwrap();
    let out = Deriver::new(&img, &set).derive().unwrap();
    assert!(matches!(&out.failures[0].reason, FailureReason::VtableRouteOnly));
    assert!(!out.data.entries.contains_key(&Entries::id("X")));
}

#[test]
fn a_function_checked_only_by_section_fails_on_one_recipe_and_resolves_on_two_agreeing() {
    let mut fx = ImageFixture::new(BASE);
    let f = fx.put_text(&[0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xC3]);
    fx.put_function(f, f + 6);
    let img = fx.image();
    let text = || vec![Check::Section { name: ".text".to_string() }];
    let set = RecipeSet::new(vec![
        Entries::entry(
            "One",
            EntryKind::Function,
            vec![vec![Entries::pattern("AA BB CC", 0)], vec![Entries::pattern("01 02 03", 0)]],
            text(),
        ),
        Entries::entry(
            "Two",
            EntryKind::Function,
            vec![vec![Entries::pattern("AA BB CC", 0)], vec![Entries::pattern("CC DD EE", -2)]],
            text(),
        ),
    ])
    .unwrap();
    let out = Deriver::new(&img, &set).derive().unwrap();
    assert_eq!(out.failures.len(), 1);
    assert_eq!(out.failures[0].id, Entries::id("One"));
    assert_eq!(out.failures[0].reason, FailureReason::WeakChecksOnly { agreeing: 1 });
    assert!(!out.data.entries.contains_key(&Entries::id("One")));
    let resolved = &out.data.entries[&Entries::id("Two")];
    assert_eq!(resolved.recipes_agreeing, 2);
    assert_eq!(resolved.value, u64::from(f));
}

#[test]
fn a_vslot_checked_only_by_range_fails_on_one_recipe_and_resolves_on_two_agreeing() {
    let mut fx = ImageFixture::new(BASE);
    // two call sites through the same slot: mov rax,[rax+498h] and mov rax,[rcx+498h]
    fx.put_text(&[0x48, 0x8B, 0x80, 0x98, 0x04, 0x00, 0x00, 0xC3]);
    fx.put_text(&[0x48, 0x8B, 0x81, 0x98, 0x04, 0x00, 0x00, 0xC3]);
    let img = fx.image();
    let site = |bytes: &str| {
        vec![
            Entries::pattern(bytes, 0),
            Step::Operand { index: 1, kind: OperandKind::Displacement },
            Step::DisplacementToSlot,
        ]
    };
    let range = || vec![Check::Range { min: 140, max: 150 }];
    let set = RecipeSet::new(vec![
        Entries::entry("One", EntryKind::VSlot, vec![site("48 8B 80 ?? ?? ?? ?? C3"), site("01 02 03")], range()),
        Entries::entry(
            "Two",
            EntryKind::VSlot,
            vec![site("48 8B 80 ?? ?? ?? ?? C3"), site("48 8B 81 ?? ?? ?? ?? C3")],
            range(),
        ),
    ])
    .unwrap();
    let out = Deriver::new(&img, &set).derive().unwrap();
    assert_eq!(out.failures.len(), 1);
    assert_eq!(out.failures[0].id, Entries::id("One"));
    assert_eq!(out.failures[0].reason, FailureReason::WeakChecksOnly { agreeing: 1 });
    let resolved = &out.data.entries[&Entries::id("Two")];
    assert_eq!(resolved.recipes_agreeing, 2);
    assert_eq!(resolved.value, 0x498 / 8);
}

#[test]
fn a_vslot_with_no_checks_or_a_field_with_only_a_range_check_still_resolves_on_one_recipe() {
    let mut fx = ImageFixture::new(BASE);
    fx.put_text(&[0x48, 0x8B, 0x80, 0x98, 0x04, 0x00, 0x00, 0xC3]);
    let img = fx.image();
    let site = || {
        vec![
            Entries::pattern("48 8B 80 ?? ?? ?? ?? C3", 0),
            Step::Operand { index: 1, kind: OperandKind::Displacement },
        ]
    };
    let mut slot = site();
    slot.push(Step::DisplacementToSlot);
    let set = RecipeSet::new(vec![
        Entries::entry("Bare", EntryKind::VSlot, vec![slot], vec![]),
        Entries::entry("Field", EntryKind::Field, vec![site()], vec![Check::Range { min: 0, max: 0x1000 }]),
    ])
    .unwrap();
    let out = Deriver::new(&img, &set).derive().unwrap();
    assert!(out.failures.is_empty());
    assert_eq!(out.data.entries[&Entries::id("Bare")].value, 0x498 / 8);
    assert_eq!(out.data.entries[&Entries::id("Field")].value, 0x498);
}

#[test]
fn a_stronger_check_next_to_a_section_check_lets_one_recipe_resolve_a_function() {
    let mut fx = ImageFixture::new(BASE);
    let f = fx.put_text(&[0xAA, 0xBB, 0xCC, 0xC3]);
    fx.put_function(f, f + 4);
    let img = fx.image();
    let set = RecipeSet::new(vec![Entries::entry(
        "X",
        EntryKind::Function,
        vec![vec![Entries::pattern("AA BB CC", 0)]],
        vec![Check::Section { name: ".text".to_string() }, Check::FunctionStart],
    )])
    .unwrap();
    let out = Deriver::new(&img, &set).derive().unwrap();
    assert!(out.failures.is_empty());
    assert_eq!(out.data.entries[&Entries::id("X")].value, u64::from(f));
}

#[test]
fn function_start_after_function_start_of_does_not_count_so_one_recipe_is_unverified() {
    let mut fx = ImageFixture::new(BASE);
    let f = fx.put_text(&[0xAA, 0xBB, 0xCC, 0xDD, 0xC3]);
    fx.put_function(f, f + 5);
    let img = fx.image();
    let walked = || vec![Entries::pattern("CC DD", 0), Step::FunctionStartOf];
    let set = RecipeSet::new(vec![
        Entries::entry("Walked", EntryKind::Function, vec![walked()], vec![Check::FunctionStart]),
        Entries::entry(
            "Direct",
            EntryKind::Function,
            vec![vec![Entries::pattern("AA BB", 0)]],
            vec![Check::FunctionStart],
        ),
    ])
    .unwrap();
    let out = Deriver::new(&img, &set).derive().unwrap();
    assert!(out.failures.is_empty());
    assert_eq!(out.data.entries[&Entries::id("Walked")].value, u64::from(f));
    assert_eq!(out.degraded.len(), 1);
    assert_eq!(out.degraded[0].id, Entries::id("Walked"));
    assert!(out.degraded[0].unverified);
}

#[test]
fn section_and_function_start_after_function_start_of_need_two_agreeing_recipes() {
    let mut fx = ImageFixture::new(BASE);
    let f = fx.put_text(&[0xAA, 0xBB, 0xCC, 0xDD, 0xC3]);
    fx.put_function(f, f + 5);
    let img = fx.image();
    let checks = || vec![Check::Section { name: ".text".to_string() }, Check::FunctionStart];
    let walked = |bytes: &str| vec![Entries::pattern(bytes, 0), Step::FunctionStartOf];
    let set = RecipeSet::new(vec![
        Entries::entry("One", EntryKind::Function, vec![walked("CC DD")], checks()),
        Entries::entry("Two", EntryKind::Function, vec![walked("CC DD"), walked("BB CC")], checks()),
    ])
    .unwrap();
    let out = Deriver::new(&img, &set).derive().unwrap();
    assert_eq!(out.failures.len(), 1);
    assert_eq!(out.failures[0].id, Entries::id("One"));
    assert_eq!(out.failures[0].reason, FailureReason::WeakChecksOnly { agreeing: 1 });
    assert_eq!(out.data.entries[&Entries::id("Two")].recipes_agreeing, 2);
}

#[test]
fn a_global_checked_only_by_section_resolves_on_one_recipe() {
    let mut fx = ImageFixture::new(BASE);
    let f = fx.put_text(&[0xAA, 0xBB, 0xCC, 0xC3]);
    fx.put_function(f, f + 4);
    let img = fx.image();
    let set = RecipeSet::new(vec![Entries::entry(
        "G",
        EntryKind::Global,
        vec![vec![Entries::pattern("AA BB CC", 0)]],
        vec![Check::Section { name: ".text".to_string() }],
    )])
    .unwrap();
    let out = Deriver::new(&img, &set).derive().unwrap();
    assert!(out.failures.is_empty());
    assert_eq!(out.data.entries[&Entries::id("G")].value, u64::from(f));
}

#[test]
fn a_vtable_route_with_an_agreeing_pattern_resolves() {
    let mut fx = ImageFixture::new(BASE);
    let f = fx.put_text(&[0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xC3]);
    fx.put_function(f, f + 6);
    fx.put_class(CLASS, &[f]);
    let img = fx.image();
    let route = vec![
        Step::RttiVtable { class: CLASS.to_string(), slot: None },
        Step::VtableSlot { slot: 0 },
        Step::ReadPointer,
    ];
    let set = RecipeSet::new(vec![Entries::entry(
        "X",
        EntryKind::Function,
        vec![route, vec![Entries::pattern("AA BB CC DD EE", 0)]],
        vec![Check::FunctionStart],
    )])
    .unwrap();
    let out = Deriver::new(&img, &set).derive().unwrap();
    assert!(out.failures.is_empty());
    let resolved = &out.data.entries[&Entries::id("X")];
    assert_eq!(resolved.recipes_agreeing, 2);
    assert_eq!(resolved.value, u64::from(f));
}

#[test]
fn in_vtable_is_a_strong_check_and_makes_its_global_a_dependency() {
    let mut fx = ImageFixture::new(BASE);
    let f = fx.put_text(&[0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xC3]);
    fx.put_function(f, f + 6);
    fx.put_class(CLASS, &[f, f]);
    let img = fx.image();
    let in_vtable = || Check::InVtable { vtable: Entries::id("VT"), min_slot: 0, max_slot: 1 };
    let text = || Check::Section { name: ".text".to_string() };
    let one = || vec![vec![Entries::pattern("AA BB CC", 0)]];
    // "A" sorts before "VT", so the order must come from the check and not from the ids
    let set = RecipeSet::new(vec![
        Entries::entry("A", EntryKind::Function, one(), vec![text(), in_vtable()]),
        Entries::entry("B", EntryKind::Function, one(), vec![text()]),
        Entries::entry(
            "VT",
            EntryKind::Global,
            vec![vec![Step::RttiVtable { class: CLASS.to_string(), slot: None }]],
            vec![],
        ),
    ])
    .unwrap();
    let out = Deriver::new(&img, &set).derive().unwrap();
    assert_eq!(out.data.entries[&Entries::id("A")].value, u64::from(f));
    assert_eq!(out.failures.len(), 1);
    assert_eq!(out.failures[0].id, Entries::id("B"));
    assert_eq!(out.failures[0].reason, FailureReason::WeakChecksOnly { agreeing: 1 });
}

#[test]
fn an_in_vtable_check_on_a_failed_global_fails_the_entry_as_a_dependency() {
    let mut fx = ImageFixture::new(BASE);
    let f = fx.put_text(&[0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xC3]);
    fx.put_function(f, f + 6);
    let img = fx.image();
    let set = RecipeSet::new(vec![
        Entries::entry(
            "F",
            EntryKind::Function,
            vec![vec![Entries::pattern("AA BB CC", 0)]],
            vec![Check::InVtable { vtable: Entries::id("VT"), min_slot: 0, max_slot: 1 }],
        ),
        // no such class, so the global fails
        Entries::entry(
            "VT",
            EntryKind::Global,
            vec![vec![Step::RttiVtable { class: CLASS.to_string(), slot: None }]],
            vec![],
        ),
    ])
    .unwrap();
    let out = Deriver::new(&img, &set).derive().unwrap();
    let reason_of = |name: &str| out.failures.iter().find(|f| f.id == Entries::id(name)).map(|f| f.reason.clone());
    assert_eq!(reason_of("F"), Some(FailureReason::DependencyFailed(Entries::id("VT"))));
    assert!(reason_of("VT").is_some());
}

impl Entries {
    // a field `Layout` from the bytes `AA BB` (immediate of nothing: the value is the alternative index) and a field
    // `X` guarded by `entry_is Layout = guard`
    fn guarded(guard: u64) -> RecipeSet {
        let layout = Entries::entry(
            "Layout",
            EntryKind::Field,
            vec![vec![
                Step::PatternAny {
                    patterns: vec!["01 02".to_string(), "AA BB".to_string()],
                    offset: 0,
                    literal_ok: Vec::new(),
                },
                Step::Alternative,
            ]],
            vec![],
        );
        let guarded = Entries::entry(
            "X",
            EntryKind::Global,
            vec![vec![Entries::pattern("AA BB", 0)]],
            vec![Check::EntryIs { entry: Entries::id("Layout"), value: guard }],
        );
        match RecipeSet::new(vec![layout, guarded]) {
            Ok(set) => set,
            Err(error) => panic!("{error}"),
        }
    }
}

#[test]
fn entry_is_refuses_the_entry_when_the_named_field_has_another_value() {
    let mut fx = ImageFixture::new(BASE);
    let at = fx.put_text(&[0xAA, 0xBB, 0xC3]);
    let img = fx.image();
    let out = Deriver::new(&img, &Entries::guarded(1)).derive().unwrap();
    assert_eq!(out.data.entries[&Entries::id("X")].value, u64::from(at));
    let out = Deriver::new(&img, &Entries::guarded(0)).derive().unwrap();
    assert!(!out.data.entries.contains_key(&Entries::id("X")));
    assert_eq!(out.failures.len(), 1);
    assert!(matches!(&out.failures[0].reason, FailureReason::CheckFailed(Check::EntryIs { value: 0, .. })));
}

#[test]
fn entry_is_does_not_count_as_evidence_so_one_recipe_is_unverified() {
    let mut fx = ImageFixture::new(BASE);
    fx.put_text(&[0xAA, 0xBB, 0xC3]);
    let img = fx.image();
    let out = Deriver::new(&img, &Entries::guarded(1)).derive().unwrap();
    let x = out.degraded.iter().find(|row| row.id == Entries::id("X")).unwrap();
    assert!(x.unverified);
}

// what the loader asks after a derive: the value of an entry in the derived data
fn value_in(out: &amml_derive::DeriveOutcome) -> impl Fn(&EntryId) -> Option<u64> + '_ {
    |id| out.data.entries.get(id).map(|resolved| resolved.value)
}

#[test]
fn layout_guard_names_the_entry_an_entry_is_check_refused_as_not_for_this_build() {
    let mut fx = ImageFixture::new(BASE);
    fx.put_text(&[0xAA, 0xBB, 0xC3]);
    let img = fx.image();
    let set = Entries::guarded(0);
    let out = Deriver::new(&img, &set).derive().unwrap();
    let x = set.get(&Entries::id("X")).unwrap();
    assert!(matches!(&out.failures[0].reason, FailureReason::CheckFailed(Check::EntryIs { .. })));
    assert_eq!(
        LayoutGuard::refusal(x, value_in(&out)),
        Some(LayoutRefusal { guard: Entries::id("Layout"), wanted: 0, found: 1 })
    );
}

#[test]
fn layout_guard_gives_no_refusal_for_an_entry_of_this_build() {
    let mut fx = ImageFixture::new(BASE);
    fx.put_text(&[0xAA, 0xBB, 0xC3]);
    let img = fx.image();
    let set = Entries::guarded(1);
    let out = Deriver::new(&img, &set).derive().unwrap();
    assert_eq!(LayoutGuard::refusal(set.get(&Entries::id("X")).unwrap(), value_in(&out)), None);
}

#[test]
fn a_real_failure_under_a_guard_that_holds_is_missing_not_a_refusal() {
    // `Layout` matches `AA BB` (value 1) and the guard wants 1, but `X` wants `AA BB CC DD`, which is not there
    let mut fx = ImageFixture::new(BASE);
    fx.put_text(&[0xAA, 0xBB, 0xC3]);
    let img = fx.image();
    let mut entries: Vec<Entry> = Entries::guarded(1).entries().to_vec();
    for entry in &mut entries {
        if entry.id == Entries::id("X") {
            entry.recipes = vec![Recipe { steps: vec![Entries::pattern("AA BB CC DD", 0)] }];
        }
    }
    let set = RecipeSet::new(entries).unwrap();
    let out = Deriver::new(&img, &set).derive().unwrap();
    assert!(!out.data.entries.contains_key(&Entries::id("X")));
    assert_eq!(LayoutGuard::refusal(set.get(&Entries::id("X")).unwrap(), value_in(&out)), None);
}

#[test]
fn a_guard_with_no_value_is_not_a_refusal() {
    let set = Entries::guarded(0);
    let x = set.get(&Entries::id("X")).unwrap();
    assert_eq!(LayoutGuard::refusal(x, |_| None), None);
    assert_eq!(LayoutGuard::guards(x).cloned().collect::<Vec<_>>(), vec![Entries::id("Layout")]);
}
