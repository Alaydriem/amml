use amml_data::EntryKind;
use amml_runtime::{EntryTable, LayoutRefusal};

use crate::support::Fixtures;

#[test]
fn missing_is_sorted_and_counts_repeated_ids_once() {
    let wanted = [Fixtures::id("B"), Fixtures::id("A"), Fixtures::id("B"), Fixtures::id("C")];
    let mut table = EntryTable::new(&wanted);
    table.insert(Fixtures::id("A"), Fixtures::resolved(EntryKind::Function, 1), "cache");
    assert_eq!(table.missing(), vec![Fixtures::id("B"), Fixtures::id("C")]);
}

#[test]
fn the_first_value_for_an_id_wins_and_unwanted_ids_are_ignored() {
    let mut table = EntryTable::new(&[Fixtures::id("A")]);
    table.insert(Fixtures::id("A"), Fixtures::resolved(EntryKind::Function, 1), "cache");
    table.insert(Fixtures::id("A"), Fixtures::resolved(EntryKind::Function, 2), "derive");
    table.insert(Fixtures::id("Z"), Fixtures::resolved(EntryKind::Function, 3), "derive");
    assert_eq!(table.get(&Fixtures::id("A")).map(|resolved| resolved.value), Some(1));
    assert_eq!(table.source_of(&Fixtures::id("A")), Some("cache"));
    assert_eq!(table.get(&Fixtures::id("Z")), None);
    assert_eq!(table.source_of(&Fixtures::id("Z")), None);
}

#[test]
fn a_refused_id_is_not_missing_and_a_found_or_unwanted_id_is_never_refused() {
    let refusal = || LayoutRefusal { guard: Fixtures::id("L"), wanted: 0, found: 1 };
    let mut table = EntryTable::new(&[Fixtures::id("A"), Fixtures::id("B"), Fixtures::id("C")]);
    table.insert(Fixtures::id("A"), Fixtures::resolved(EntryKind::Field, 1), "cache");
    table.refuse(Fixtures::id("A"), refusal());
    table.refuse(Fixtures::id("B"), refusal());
    table.refuse(Fixtures::id("Z"), refusal());
    assert_eq!(table.missing(), vec![Fixtures::id("C")]);
    assert_eq!(table.not_for_build().keys().cloned().collect::<Vec<_>>(), vec![Fixtures::id("B")]);
    let narrowed = table.narrowed(&[Fixtures::id("B")]);
    assert!(narrowed.missing().is_empty());
    assert_eq!(narrowed.get(&Fixtures::id("A")), None);
    assert_eq!(narrowed.not_for_build().len(), 1);
}
