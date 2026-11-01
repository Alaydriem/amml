use amml_derive::FunctionTable;

fn table() -> FunctionTable {
    FunctionTable::with_fragments(vec![(0x2000, 0x2040), (0x1000, 0x1100)], vec![(0x3000, 0x3010, 0x1000)])
}

#[test]
fn chained_fragment_resolves_to_the_primary_function() {
    let t = table();
    assert_eq!(t.containing(0x3008), Some((0x1000, 0x3010)));
    assert_eq!(t.containing(0x1080), Some((0x1000, 0x1100)));
    assert_eq!(t.containing(0x2000), Some((0x2000, 0x2040)));
}

#[test]
fn only_primary_entries_are_starts() {
    let t = table();
    assert!(t.is_start(0x1000) && t.is_start(0x2000));
    assert!(!t.is_start(0x3000));
}

#[test]
fn gaps_and_range_ends_are_outside_any_function() {
    let t = table();
    assert_eq!(t.containing(0x0fff), None);
    assert_eq!(t.containing(0x1100), None);
    assert_eq!(t.containing(0x2040), None);
}

#[test]
fn the_first_byte_of_a_range_is_inside_and_the_byte_before_it_is_not() {
    let t = table();
    assert_eq!(t.containing(0x1000), Some((0x1000, 0x1100)));
    assert_eq!(t.containing(0x10FF), Some((0x1000, 0x1100)));
    assert_eq!(t.containing(0x2000), Some((0x2000, 0x2040)));
    assert_eq!(t.containing(0x1FFF), None);
}

#[test]
fn addresses_between_ranges_and_past_the_last_one_are_outside() {
    let t = table();
    assert_eq!(t.containing(0x1500), None);
    assert_eq!(t.containing(0x2500), None);
    assert_eq!(t.containing(0x3010), None);
    assert_eq!(t.containing(u32::MAX), None);
}

#[test]
fn touching_ranges_split_at_the_shared_boundary() {
    let t = FunctionTable::new(vec![(0x1000, 0x1010), (0x1010, 0x1020)]);
    assert_eq!(t.containing(0x100F), Some((0x1000, 0x1010)));
    assert_eq!(t.containing(0x1010), Some((0x1010, 0x1020)));
    assert!(t.is_start(0x1010));
}

#[test]
fn an_empty_table_contains_nothing() {
    let t = FunctionTable::new(Vec::new());
    assert_eq!(t.containing(0), None);
    assert!(!t.is_start(0));
    assert!(t.ranges_of(0x1000).is_empty());
}

#[test]
fn a_fragment_start_resolves_to_its_primary_but_is_not_a_function_start() {
    let t = table();
    assert_eq!(t.containing(0x3000), Some((0x1000, 0x3010)));
    assert!(!t.is_start(0x3000));
}

#[test]
fn ranges_of_lists_the_primary_and_its_fragments_in_address_order() {
    let t = FunctionTable::with_fragments(
        vec![(0x2000, 0x2040), (0x1000, 0x1100)],
        vec![(0x4000, 0x4010, 0x1000), (0x3000, 0x3010, 0x1000)],
    );
    assert_eq!(t.ranges_of(0x1000), vec![(0x1000, 0x1100), (0x3000, 0x3010), (0x4000, 0x4010)]);
    assert_eq!(t.ranges_of(0x2000), vec![(0x2000, 0x2040)]);
    assert!(t.ranges_of(0x3000).is_empty());
}
