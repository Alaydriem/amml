use amml_studio_ui::component::dialog::{CoverageFilter, CoverageGroup, CoveragePanel, CoveragePanelView, CoverageRow};
use amml_studio_ui::tokens::selector::StatusColor;

use crate::glyph::words;
use crate::support::UiProbe;

fn row(id: &str, label: &str, status: StatusColor) -> CoverageRow {
    CoverageRow { id: words(id), label: words(label), status, area: words("Editor"), go: true }
}

fn view(filter: CoverageFilter) -> CoveragePanelView {
    CoveragePanelView {
        subtitle: words("Status of each capability in the real mod."),
        filter,
        groups: vec![CoverageGroup {
            title: words("A. Library and open replay"),
            rows: vec![
                row("A1", "Open Replay button", StatusColor::Works),
                row("A11", "Replay details panel", StatusColor::Mockup),
                row("A14", "Rename a replay", StatusColor::Missing),
                row("A15", "egui launcher window", StatusColor::Planned),
            ],
        }],
    }
}

fn texts(view: &CoveragePanelView, search: &str) -> Vec<String> {
    let mut search = search.to_owned();
    UiProbe::texts(&UiProbe::sized(1100.0, 900.0).shapes(|ui| CoveragePanel::new(view, &mut search).show_card(ui)))
}

#[test]
fn coverage_filter_shows_only_that_status() {
    let texts = texts(&view(CoverageFilter::Status(StatusColor::Missing)), "");
    assert!(texts.iter().any(|text| text == "Rename a replay"), "{texts:?}");
    for other in ["Open Replay button", "Replay details panel", "egui launcher window"] {
        assert!(!texts.iter().any(|text| text == other), "{other} shows: {texts:?}");
    }
}

#[test]
fn coverage_search_with_no_match() {
    let texts = texts(&view(CoverageFilter::All), "zzz");
    assert!(texts.iter().any(|text| text == "No matches"), "{texts:?}");
    assert!(texts.iter().any(|text| text == "zzz"), "the search text stays: {texts:?}");
    assert!(!texts.iter().any(|text| text == "Rename a replay"), "{texts:?}");
}

#[test]
fn coverage_filters_wrap_with_long_words() {
    let view = view(CoverageFilter::All);
    let mut search = String::new();
    let mut probe = UiProbe::pseudo(1100.0, 900.0);
    let ((), shapes) = probe.frame(Vec::new(), |ui| {
        CoveragePanel::new(&view, &mut search).show_card(ui);
    });
    let card = shapes.iter().fold(egui::Rect::NOTHING, |all, shape| match shape {
        egui::Shape::Rect(rect) if rect.blur_width == 0.0 => all.union(rect.rect),
        _ => all,
    });
    for shape in &shapes {
        if let egui::Shape::Text(text) = shape {
            let rect = text.galley.rect.translate(text.pos.to_vec2());
            assert!(rect.right() <= card.right() + 0.5, "{:?} runs past the card", text.galley.text());
        }
    }
}

#[test]
fn coverage_search_matches_an_exact_id() {
    let texts = texts(&view(CoverageFilter::All), "a14");
    assert!(texts.iter().any(|text| text == "Rename a replay"), "{texts:?}");
    assert!(!texts.iter().any(|text| text == "Open Replay button"), "{texts:?}");
}
