use amml_launcher_core::NameSearch;
use amml_registry::{NameRow, NamesFile};

fn names() -> NamesFile {
    let row = |id: &str, title: &str, tag: &str| {
        NameRow(id.to_string(), title.to_string(), vec![tag.to_string()], vec![2], 1)
    };
    NamesFile {
        schema: 2,
        rows: vec![
            row("@alaydriem/zoom", "Zoom", "camera"),
            row("@alaydriem/peek", "Peek", "zoom"),
            row("@alaydriem/minimap", "Minimap", "map"),
        ],
    }
}

#[test]
fn id_beats_title_beats_tag() {
    let hits = NameSearch::search(&names(), "zoom");
    let found: Vec<(&str, u32)> = hits.iter().map(|hit| (hit.id.as_str(), hit.score)).collect();
    assert_eq!(found, vec![("@alaydriem/zoom", 300), ("@alaydriem/peek", 100)]);
    assert_eq!((hits[0].api, hits[0].page), (2, 1));

    let by_title =
        NamesFile { schema: 2, rows: vec![NameRow("@a/x".to_string(), "Fly Zoom".to_string(), vec![], vec![3, 2], 4)] };
    let hit = &NameSearch::search(&by_title, "zoom")[0];
    assert_eq!((hit.score, hit.api, hit.page), (200, 3, 4));
}

#[test]
fn matching_ignores_case_and_an_empty_query_finds_nothing() {
    assert_eq!(NameSearch::search(&names(), "ZOOM").len(), 2);
    assert!(NameSearch::search(&names(), "").is_empty());
    assert!(NameSearch::search(&names(), "   ").is_empty());
}

#[test]
fn equal_scores_are_sorted_by_id() {
    let rows =
        ["@b/mapa", "@a/mapb"].iter().map(|id| NameRow(id.to_string(), "T".to_string(), vec![], vec![2], 1)).collect();
    let hits = NameSearch::search(&NamesFile { schema: 2, rows }, "map");
    assert_eq!(hits.iter().map(|hit| hit.id.as_str()).collect::<Vec<_>>(), vec!["@a/mapb", "@b/mapa"]);
}
