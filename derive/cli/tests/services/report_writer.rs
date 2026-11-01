use std::time::Duration;

use amml_data::{BuildData, BuildId, Entry, EntryId, EntryKind, Recipe, RecipeSet, Resolved, Step};
use amml_derive::{Degraded, DeriveOutcome, Failure, FailureReason, ReportWriter, StepError};

struct Cases;

impl Cases {
    fn id(name: &str) -> EntryId {
        EntryId(name.to_string())
    }

    fn entry(name: &str, steps: Vec<Step>, notes: &str) -> Entry {
        Entry {
            id: Self::id(name),
            kind: EntryKind::Function,
            msvc_name: None,
            recipes: vec![Recipe { steps }],
            checks: Vec::new(),
            notes: notes.to_string(),
            hooked: false,
        }
    }

    fn pattern() -> Step {
        Step::Pattern { bytes: "AA".to_string(), offset: 0, literal_ok: Vec::new() }
    }

    fn resolved(value: u64) -> Resolved {
        Resolved { kind: EntryKind::Function, value, recipes_agreeing: 1 }
    }

    fn data(values: &[(&str, u64)]) -> BuildData {
        let mut data = BuildData::new(BuildId::new(1, 2));
        for (name, value) in values {
            data.entries.insert(Self::id(name), Self::resolved(*value));
        }
        data
    }

    fn secs(n: u64) -> Duration {
        Duration::from_secs(n)
    }
}

#[test]
fn percentage_counts_dependency_failures_as_failures() {
    let set = RecipeSet::new(vec![
        Cases::entry("A", vec![Cases::pattern()], "first line\nsecond line"),
        Cases::entry("B", vec![Step::Ref { entry: Cases::id("A") }], ""),
        Cases::entry("C", vec![Cases::pattern()], ""),
        Cases::entry("D", vec![Cases::pattern()], ""),
    ])
    .unwrap();
    let failures = vec![
        Failure::new(Cases::id("B"), FailureReason::DependencyFailed(Cases::id("A"))),
        Failure::new(Cases::id("A"), FailureReason::AllRecipesFailed(vec![StepError::NoMatch])),
    ];
    let out = DeriveOutcome::new(Cases::data(&[("C", 1), ("D", 2)]), failures, Vec::new());
    let text = ReportWriter::render(&ReportWriter::new(&out, &set, Cases::secs(3), true, None));
    assert!(text.contains("derived 2/4 (50.00%)"));
    assert!(text.contains("| B | dependency failed: A |"));
    assert!(text.contains("deterministic: yes"));
    assert!(text.contains("elapsed: 3.0s"));
    assert!(text.contains("| A | all recipes failed: no match | first line |"));
    assert!(!text.contains("second line"));
    assert!(text.find("| A |").unwrap() < text.find("| B |").unwrap());
}

#[test]
fn moved_entries_are_listed_only_when_the_value_changed() {
    let set = RecipeSet::new(vec![
        Cases::entry("A", vec![Cases::pattern()], ""),
        Cases::entry("B", vec![Cases::pattern()], ""),
    ])
    .unwrap();
    let previous = Cases::data(&[("A", 0x10), ("B", 0x20)]);
    let out = DeriveOutcome::new(Cases::data(&[("A", 0x10), ("B", 0x30)]), Vec::new(), Vec::new());
    let text = ReportWriter::render(&ReportWriter::new(&out, &set, Cases::secs(1), false, Some(&previous)));
    assert!(text.contains("| B | 0x20 | 0x30 |"));
    assert!(!text.contains("| A | 0x10 |"));
    assert!(text.contains("deterministic: no"));
}

#[test]
fn entries_missing_from_the_previous_data_are_not_moved() {
    let set = RecipeSet::new(vec![Cases::entry("A", vec![Cases::pattern()], "")]).unwrap();
    let previous = Cases::data(&[("Z", 0x10)]);
    let out = DeriveOutcome::new(Cases::data(&[("A", 0x10)]), Vec::new(), Vec::new());
    let report = ReportWriter::new(&out, &set, Cases::secs(1), true, Some(&previous));
    assert!(report.moved.is_empty());
    assert!(!ReportWriter::render(&report).contains("Moved"));
}

#[test]
fn degraded_and_unverified_entries_have_their_own_sections() {
    let set = RecipeSet::new(vec![
        Cases::entry("A", vec![Cases::pattern()], ""),
        Cases::entry("B", vec![Cases::pattern()], ""),
    ])
    .unwrap();
    let degraded = vec![
        Degraded::new(Cases::id("B"), Vec::new(), true),
        Degraded::new(Cases::id("A"), vec![(1, StepError::NoMatch)], false),
    ];
    let out = DeriveOutcome::new(Cases::data(&[("A", 1), ("B", 2)]), Vec::new(), degraded);
    let text = ReportWriter::render(&ReportWriter::new(&out, &set, Cases::secs(1), true, None));
    let degraded_at = text.find("## Degraded").unwrap();
    let unverified_at = text.find("## Unverified").unwrap();
    assert!(degraded_at < unverified_at);
    assert!(text[degraded_at..unverified_at].contains("| A | 1 | no match |"));
    assert!(!text[degraded_at..unverified_at].contains("| B |"));
    assert!(text[unverified_at..].contains("- B"));
    assert!(!text[unverified_at..].contains("- A"));
}

#[test]
fn clean_runs_show_none_in_both_sections() {
    let set = RecipeSet::new(vec![Cases::entry("A", vec![Cases::pattern()], "")]).unwrap();
    let out = DeriveOutcome::new(Cases::data(&[("A", 1)]), Vec::new(), Vec::new());
    let text = ReportWriter::render(&ReportWriter::new(&out, &set, Cases::secs(1), true, None));
    assert!(text.contains("## Degraded\n\nnone\n"));
    assert!(text.contains("## Unverified\n\nnone\n"));
}

#[test]
fn the_target_line_says_whether_95_percent_was_reached() {
    let entries: Vec<Entry> = (0..20).map(|n| Cases::entry(&format!("E{n}"), vec![Cases::pattern()], "")).collect();
    let set = RecipeSet::new(entries).unwrap();
    let names: Vec<String> = (0..20).map(|n| format!("E{n}")).collect();
    let values = |count: usize| -> Vec<(&str, u64)> { names.iter().take(count).map(|n| (n.as_str(), 1)).collect() };
    let met = DeriveOutcome::new(Cases::data(&values(19)), Vec::new(), Vec::new());
    let missed = DeriveOutcome::new(Cases::data(&values(18)), Vec::new(), Vec::new());
    assert!(
        ReportWriter::render(&ReportWriter::new(&met, &set, Cases::secs(1), true, None))
            .contains("- target: 95% (met)")
    );
    assert!(
        ReportWriter::render(&ReportWriter::new(&missed, &set, Cases::secs(1), true, None))
            .contains("- target: 95% (not met)")
    );
}

#[test]
fn a_run_that_is_not_deterministic_exits_with_its_own_code() {
    let set = RecipeSet::new(vec![Cases::entry("A", vec![Cases::pattern()], "")]).unwrap();
    let out = DeriveOutcome::new(Cases::data(&[("A", 1)]), Vec::new(), Vec::new());
    assert_eq!(ReportWriter::new(&out, &set, Cases::secs(1), true, None).exit_code(), 0);
    assert_eq!(ReportWriter::new(&out, &set, Cases::secs(1), false, None).exit_code(), 3);
}
