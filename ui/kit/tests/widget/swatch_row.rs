use amml_ui::tokens::{ColorToken, color};
use amml_ui::widget::SwatchRow;
use egui::{Color32, Rect, vec2};

use crate::support::{ClickProbe, PaintCapture};

struct Swatches;

impl Swatches {
    const COLOURS: [Color32; 6] = [
        Color32::from_rgb(0xB5, 0xE8, 0x53),
        Color32::from_rgb(0xFF, 0x7A, 0xC6),
        Color32::from_rgb(0xF0, 0xA3, 0xFF),
        Color32::from_rgb(0x8F, 0xA8, 0xFF),
        Color32::from_rgb(0xFF, 0x9E, 0x7A),
        Color32::from_rgb(0xC8, 0xB8, 0x8A),
    ];

    fn row() -> impl FnMut(&mut egui::Ui, &mut usize) -> egui::Response {
        |ui, value| SwatchRow::new(egui::Id::unique(1), value, Self::COLOURS.to_vec()).show(ui)
    }

    fn cell_centre(row: Rect, index: usize) -> egui::Pos2 {
        SwatchRow::cell(row, index).center()
    }
}

#[test]
fn clicking_a_swatch_selects_it() {
    let mut harness = ClickProbe::harness_with(0, Swatches::row());
    let row = harness.state().rect;
    ClickProbe::click_at(&mut harness, Swatches::cell_centre(row, 3));
    assert_eq!(harness.state().value, 3);
    assert_eq!(harness.state().changes, 1);
}

#[test]
fn clicking_the_selected_swatch_changes_nothing() {
    let mut harness = ClickProbe::harness_with(2, Swatches::row());
    let row = harness.state().rect;
    ClickProbe::click_at(&mut harness, Swatches::cell_centre(row, 2));
    assert_eq!(harness.state().value, 2);
    assert_eq!(harness.state().changes, 0);
}

#[test]
fn every_colour_is_painted_in_order_as_a_square() {
    let mut selected = 0;
    let row = Rect::from_min_size(egui::pos2(20.0, 20.0), vec2(400.0, SwatchRow::HEIGHT));
    let shapes = PaintCapture::shapes(|painter| {
        SwatchRow::new(egui::Id::NULL, &mut selected, Swatches::COLOURS.to_vec()).paint(painter, row);
    });
    let rects = PaintCapture::rects(&shapes);
    for (index, colour) in Swatches::COLOURS.iter().enumerate() {
        let cell = SwatchRow::cell(row, index);
        assert!(rects.iter().any(|r| r.rect == cell && r.fill == *colour), "swatch {index} is not painted at {cell:?}");
        assert_eq!(cell.size(), vec2(SwatchRow::SIDE, SwatchRow::SIDE));
    }
}

#[test]
fn only_the_selected_swatch_has_the_outline() {
    let mut selected = 1;
    let row = Rect::from_min_size(egui::pos2(20.0, 20.0), vec2(400.0, SwatchRow::HEIGHT));
    let shapes = PaintCapture::shapes(|painter| {
        SwatchRow::new(egui::Id::NULL, &mut selected, Swatches::COLOURS.to_vec()).paint(painter, row);
    });
    let outlined: Vec<Rect> = PaintCapture::rects(&shapes)
        .into_iter()
        .filter(|r| r.stroke.color == color::OnAccent::VALUE && r.stroke.width > 0.0)
        .map(|r| r.rect)
        .collect();
    assert_eq!(outlined.len(), 1, "{outlined:?}");
    // the outline sits outside the swatch, so the swatch stays the same size
    assert!(outlined[0].contains_rect(SwatchRow::cell(row, 1)));
    assert!(outlined[0].size().x > SwatchRow::SIDE);
}
