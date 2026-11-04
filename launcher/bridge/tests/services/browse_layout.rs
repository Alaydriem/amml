use amml_launcher_bridge::{BrowseLayout, CatalogGroupView};
use amml_registry::NameRow;

fn row(id: &str, apis: &[u32]) -> NameRow {
    NameRow(id.to_string(), id.to_string(), Vec::new(), apis.to_vec(), 1)
}

fn group(id: &str, start: u32, total: u32) -> CatalogGroupView {
    CatalogGroupView { id: id.to_string(), label: id.to_string(), start, total }
}

#[test]
fn groups_follow_the_first_letter_and_count_only_rows_for_the_api() {
    let rows = [
        row("@1st/zoom", &[2]),
        row("@alaydriem/peek", &[2]),
        row("@alaydriem/old", &[1]),
        row("@alaydriem/zoom", &[2, 1]),
        row("@bob/map", &[2]),
        row("@bob/other", &[1]),
        row("@cat/x", &[2]),
    ];

    let groups = BrowseLayout::build(&rows, 2);

    assert_eq!(groups, vec![group("#", 0, 1), group("A", 1, 2), group("B", 3, 1), group("C", 4, 1)]);
}

#[test]
fn a_registry_with_no_rows_for_the_api_has_no_groups() {
    let rows = [row("@a/b", &[1])];
    assert!(BrowseLayout::build(&rows, 2).is_empty());
    assert!(BrowseLayout::build(&[], 2).is_empty());
}

#[test]
fn the_start_of_a_group_gives_the_page_that_holds_its_first_row() {
    let rows: Vec<NameRow> =
        (0..250).map(|index| row(&format!("@{}/m{index}", if index < 120 { "alpha" } else { "beta" }), &[2])).collect();

    let groups = BrowseLayout::build(&rows, 2);

    assert_eq!(groups.len(), 2);
    assert_eq!(groups[1].start, 120);
    assert_eq!(groups[1].start / 100 + 1, 2);
    assert_eq!(groups.iter().map(|group| group.total).sum::<u32>(), 250);
}
