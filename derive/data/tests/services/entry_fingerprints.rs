use std::collections::BTreeMap;

use amml_data::{Check, DataError, Entry, EntryFingerprints, EntryId, EntryKind, Fingerprint, Recipe, RecipeSet, Step};

struct Fixture;

impl Fixture {
    fn id(name: &str) -> EntryId {
        EntryId(name.to_string())
    }

    fn pattern(bytes: &str) -> Recipe {
        Recipe { steps: vec![Step::Pattern { bytes: bytes.to_string(), offset: 0, literal_ok: Vec::new() }] }
    }

    fn entry(id: &str, kind: EntryKind, recipes: Vec<Recipe>, checks: Vec<Check>) -> Entry {
        Entry { id: Self::id(id), kind, msvc_name: None, recipes, checks, notes: String::new(), hooked: false }
    }

    fn function(id: &str, bytes: &str) -> Entry {
        Self::entry(id, EntryKind::Function, vec![Self::pattern(bytes)], Vec::new())
    }

    fn prints(entries: Vec<Entry>, version: u32) -> Result<BTreeMap<EntryId, Fingerprint>, DataError> {
        EntryFingerprints::of(&RecipeSet::new(entries).unwrap(), version)
    }

    fn print_of(entries: Vec<Entry>, id: &str) -> Fingerprint {
        Self::prints(entries, 12).unwrap()[&Self::id(id)]
    }

    // A is a global, B is checked against the vtable A, C refs B, D stands alone.
    fn chain(a_bytes: &str) -> Vec<Entry> {
        vec![
            Self::entry("A", EntryKind::Global, vec![Self::pattern(a_bytes)], Vec::new()),
            Self::entry(
                "B",
                EntryKind::Function,
                vec![Self::pattern("48 8B")],
                vec![Check::InVtable { vtable: Self::id("A"), min_slot: 0, max_slot: 4 }],
            ),
            Self::entry(
                "C",
                EntryKind::Function,
                vec![Recipe { steps: vec![Step::Ref { entry: Self::id("B") }] }],
                Vec::new(),
            ),
            Self::function("D", "90 90"),
        ]
    }
}

#[test]
fn notes_name_and_hooked_do_not_change_a_fingerprint() {
    let plain = Fixture::function("A", "48 8B ??");
    let mut annotated = plain.clone();
    annotated.notes = "something else".to_string();
    annotated.msvc_name = Some("Class::method".to_string());
    annotated.hooked = true;
    assert_eq!(Fixture::print_of(vec![plain], "A"), Fixture::print_of(vec![annotated], "A"));
}

#[test]
fn a_changed_recipe_byte_changes_the_fingerprint() {
    let wild = Fixture::print_of(vec![Fixture::function("A", "48 8B ??")], "A");
    let fixed = Fixture::print_of(vec![Fixture::function("A", "48 8B 05")], "A");
    assert_ne!(wild, fixed);
}

#[test]
fn a_changed_check_changes_the_fingerprint() {
    let ranged = |max| {
        Fixture::entry(
            "F",
            EntryKind::Field,
            vec![Recipe {
                steps: vec![
                    Step::Pattern { bytes: "48".to_string(), offset: 0, literal_ok: Vec::new() },
                    Step::Operand { index: 0, kind: amml_data::OperandKind::Displacement },
                ],
            }],
            vec![Check::Range { min: 8, max }],
        )
    };
    assert_ne!(Fixture::print_of(vec![ranged(64)], "F"), Fixture::print_of(vec![ranged(72)], "F"));
}

#[test]
fn a_dependency_change_reaches_every_dependent() {
    let before = Fixture::prints(Fixture::chain("48 8B 05"), 12).unwrap();
    let after = Fixture::prints(Fixture::chain("48 8B 06"), 12).unwrap();
    for name in ["A", "B", "C"] {
        let id = Fixture::id(name);
        assert_ne!(before[&id], after[&id], "{name} should change");
    }
    let d = Fixture::id("D");
    assert_eq!(before[&d], after[&d]);
}

#[test]
fn an_entry_is_guard_is_a_dependency() {
    let build = |bytes: &str| {
        vec![
            Fixture::entry(
                "Guard",
                EntryKind::Field,
                vec![Recipe {
                    steps: vec![
                        Step::Pattern { bytes: bytes.to_string(), offset: 0, literal_ok: Vec::new() },
                        Step::Operand { index: 0, kind: amml_data::OperandKind::Displacement },
                    ],
                }],
                Vec::new(),
            ),
            Fixture::entry(
                "Guarded",
                EntryKind::Function,
                vec![Fixture::pattern("48")],
                vec![Check::EntryIs { entry: Fixture::id("Guard"), value: 8 }],
            ),
        ]
    };
    let before = Fixture::print_of(build("48 8B"), "Guarded");
    let after = Fixture::print_of(build("48 8C"), "Guarded");
    assert_ne!(before, after);
}

#[test]
fn the_deriver_version_changes_every_fingerprint() {
    let first = Fixture::prints(Fixture::chain("48 8B 05"), 12).unwrap();
    let second = Fixture::prints(Fixture::chain("48 8B 05"), 13).unwrap();
    assert_eq!(first.len(), 4);
    for (id, print) in &first {
        assert_ne!(*print, second[id], "{id} should change");
    }
}

#[test]
fn the_fingerprint_prints_as_16_hex_digits_and_parses_back() {
    assert_eq!(Fingerprint(0xab).to_string(), "00000000000000ab");
    assert_eq!(Fingerprint::parse("00000000000000ab"), Some(Fingerprint(0xab)));
    assert_eq!(Fingerprint::parse("xyz"), None);
}
