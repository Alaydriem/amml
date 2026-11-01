use std::collections::BTreeMap;

use amml_data::{EntryId, EntryKind, Resolved};
use amml_runtime::{GradeReport, GradeVerdict};

use crate::support::Fixtures;

fn values(items: &[(&str, EntryKind, u64)]) -> BTreeMap<EntryId, Resolved> {
    items.iter().map(|(name, kind, value)| (Fixtures::id(name), Fixtures::resolved(*kind, *value))).collect()
}

fn report() -> GradeReport {
    let set = Fixtures::set(vec![
        Fixtures::entry("Same", EntryKind::Function, Some("same_sym")),
        Fixtures::entry("Moved", EntryKind::Function, Some("moved_sym")),
        Fixtures::entry("OnlyOurs", EntryKind::Global, Some("ours_sym")),
        Fixtures::entry("OnlyTheirs", EntryKind::Function, Some("theirs_sym")),
        Fixtures::entry("Nobody", EntryKind::Function, Some("nobody_sym")),
        Fixtures::entry("NoName", EntryKind::Function, None),
        Fixtures::entry("Slot", EntryKind::VSlot, Some("slot_sym")),
    ]);
    let derived = values(&[
        ("Same", EntryKind::Function, 0x100),
        ("Moved", EntryKind::Function, 0x200),
        ("OnlyOurs", EntryKind::Global, 0x300),
        ("NoName", EntryKind::Function, 0x400),
        ("Slot", EntryKind::VSlot, 3),
    ]);
    let levilamina = values(&[
        ("Same", EntryKind::Function, 0x100),
        ("Moved", EntryKind::Function, 0x208),
        ("OnlyTheirs", EntryKind::Function, 0x500),
    ]);
    GradeReport::compare(&set, &derived, &levilamina)
}

#[test]
fn compare_counts_each_verdict_and_grades_only_named_functions_and_globals() {
    let report = report();
    let ids: Vec<&str> = report.rows.iter().map(|row| row.id.0.as_str()).collect();
    assert_eq!(ids, vec!["Moved", "Nobody", "OnlyOurs", "OnlyTheirs", "Same"]);
    for verdict in GradeVerdict::ALL {
        assert_eq!(report.count(verdict), 1, "{}", verdict.label());
    }
    assert_eq!(report.reference_count(), 3);
    assert_eq!(
        report.summary_line(),
        "grade: reference for 3 of 5 names; match 1, mismatch 1, derived-only 1, levilamina-only 1, neither 1"
    );
}

#[test]
fn the_markdown_lists_the_counts_and_each_mismatch_with_both_values() {
    let text = report().markdown("6aa482fd-2658118eba0dbac5");
    assert!(text.contains("- build: 6aa482fd-2658118eba0dbac5"));
    assert!(text.contains("- mismatch: 1"));
    assert!(text.contains("- levilamina-only: 1"));
    let mismatch_table = text.split("## Entries").next().unwrap();
    assert!(mismatch_table.contains("| Moved | `moved_sym` | 0x200 | 0x208 |"));
    assert!(!mismatch_table.contains("| Same |"));
    assert!(text.contains("| OnlyTheirs | `theirs_sym` | - | 0x500 | levilamina-only |"));
}

#[test]
fn a_report_without_mismatches_says_none() {
    let set = Fixtures::set(vec![Fixtures::entry("Same", EntryKind::Function, Some("same_sym"))]);
    let both = values(&[("Same", EntryKind::Function, 0x100)]);
    let text = GradeReport::compare(&set, &both, &both).markdown("stem");
    assert!(text.contains("## Mismatches\n\nnone\n"));
    assert_eq!(GradeReport::file_name("stem"), "grade-stem.md");
}

#[test]
fn the_report_counts_the_names_with_a_reference_and_still_grades_the_rest() {
    let set = Fixtures::set(vec![
        Fixtures::entry("Found", EntryKind::Function, Some("found_sym")),
        Fixtures::entry("Known", EntryKind::Function, Some("known_sym")),
        Fixtures::entry("Lost", EntryKind::Function, Some("lost_sym")),
    ]);
    let derived = values(&[("Found", EntryKind::Function, 0x100), ("Known", EntryKind::Function, 0x200)]);
    let levilamina = values(&[("Known", EntryKind::Function, 0x200)]);
    let report = GradeReport::compare(&set, &derived, &levilamina);
    assert_eq!(report.reference_count(), 1);
    assert_eq!(
        report.summary_line(),
        "grade: reference for 1 of 3 names; match 1, mismatch 0, derived-only 1, levilamina-only 0, neither 1"
    );
    let text = report.markdown("stem");
    assert!(text.contains("- reference: 1 of 3 names"));
    assert!(text.contains("| Found | `found_sym` | 0x100 | - | derived-only |"));
    assert!(text.contains("| Lost | `lost_sym` | - | - | neither |"));
}

#[test]
fn a_report_with_no_reference_says_zero_of_n() {
    let set = Fixtures::set(vec![Fixtures::entry("Found", EntryKind::Function, Some("found_sym"))]);
    let derived = values(&[("Found", EntryKind::Function, 0x100)]);
    let report = GradeReport::compare(&set, &derived, &BTreeMap::new());
    assert!(report.summary_line().starts_with("grade: reference for 0 of 1 names;"));
}
