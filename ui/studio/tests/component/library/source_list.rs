use amml_studio_ui::component::library::{SourceList, SourceListAction, SourceListView, SourceRow, SourceRowKind};
use amml_ui::tokens::{ColorToken, color};
use egui::{Shape, vec2};

use crate::glyph::words;
use crate::support::UiProbe;

struct ListSample;

impl ListSample {
    const WIDTH: f32 = 519.0;

    fn view() -> SourceListView {
        SourceListView {
            title: words("SOURCES"),
            sources: vec![
                SourceRow { name: words("BVC recordings"), kind: SourceRowKind::Automatic { tag: words("auto") } },
                SourceRow { name: words("D:\\Captures\\Minecraft"), kind: SourceRowKind::Folder },
                SourceRow { name: words("E:\\Clips"), kind: SourceRowKind::Folder },
            ],
            add: words("Add folder"),
        }
    }

    fn show(ui: &mut egui::Ui, view: &SourceListView) -> Option<SourceListAction> {
        SourceList::new(view).show(ui)
    }

    fn shapes(view: &SourceListView) -> Vec<Shape> {
        UiProbe::new(Self::WIDTH).shapes(|ui| Self::show(ui, view))
    }
}

#[test]
fn the_only_text_is_the_view_text() {
    let shapes = ListSample::shapes(&ListSample::view());
    let mut painted = UiProbe::words(&shapes);
    painted.sort();
    let mut expected: Vec<String> =
        ["SOURCES", "BVC recordings", "auto", "D:\\Captures\\Minecraft", "E:\\Clips", "Add folder"]
            .map(String::from)
            .to_vec();
    expected.sort();
    assert_eq!(painted, expected);
}

#[test]
fn source_list_adds_and_removes() {
    let view = ListSample::view();
    let mut probe = UiProbe::new(ListSample::WIDTH);
    let shapes = probe.shapes(|ui| ListSample::show(ui, &view));

    let Some(add) = UiProbe::text_rect(&shapes, "Add folder") else { panic!("no add button") };
    let action = probe.click(add.center(), |ui| ListSample::show(ui, &view));
    assert_eq!(action, Some(SourceListAction::Add));

    // the remove cell is 24 wide at the right end of a folder row
    for (name, index) in [("D:\\Captures\\Minecraft", 1), ("E:\\Clips", 2)] {
        let Some(row) = UiProbe::text_rect(&shapes, name) else { panic!("no row {name}") };
        let cell = egui::pos2(ListSample::WIDTH - 12.0, row.center().y);
        let mut probe = UiProbe::new(ListSample::WIDTH);
        probe.shapes(|ui| ListSample::show(ui, &view));
        let action = probe.click(cell, |ui| ListSample::show(ui, &view));
        assert_eq!(action, Some(SourceListAction::Remove(index)), "{name}");
    }
}

#[test]
fn the_automatic_source_cannot_be_removed() {
    let view = ListSample::view();
    let mut probe = UiProbe::new(ListSample::WIDTH);
    let shapes = probe.shapes(|ui| ListSample::show(ui, &view));
    let Some(row) = UiProbe::text_rect(&shapes, "BVC recordings") else { panic!("no row") };
    let cell = egui::pos2(ListSample::WIDTH - 12.0, row.center().y);
    assert_eq!(probe.click(cell, |ui| ListSample::show(ui, &view)), None);
}

#[test]
fn the_automatic_tag_is_the_accent_hover_colour_and_the_rows_are_24_high() {
    let shapes = ListSample::shapes(&ListSample::view());
    let ink = shapes.iter().find_map(|s| match s {
        Shape::Text(t) if t.galley.text() == "auto" => Some(t.galley.job.sections[0].format.color),
        _ => None,
    });
    assert_eq!(ink, Some(color::AccentHover::VALUE));
    let Some(first) = UiProbe::text_rect(&shapes, "D:\\Captures\\Minecraft") else { panic!("no row") };
    let Some(second) = UiProbe::text_rect(&shapes, "E:\\Clips") else { panic!("no row") };
    // a 24 high row and a 6 gap between rows
    assert!((second.center().y - first.center().y - 30.0).abs() <= 1.0, "{first:?} {second:?}");
}

#[test]
fn the_height_follows_the_rows_it_shows() {
    let view = ListSample::view();
    let (size, _) = UiProbe::new(ListSample::WIDTH).frame(Vec::new(), |ui| {
        ListSample::show(ui, &view);
        ui.min_rect().size()
    });
    assert_eq!(size, vec2(ListSample::WIDTH, SourceList::height(3)));
    assert!(SourceList::height(3) > SourceList::height(2));
}
