use amml_ui::state::InteractionState;
use amml_ui::tokens::{ColorToken, color};
use amml_ui::widget::{Segment, Segmented};
use egui::{Key, Rect, Shape, pos2};

use crate::support::{ClickProbe, PaintCapture, Sample};

struct SegmentPaint;

impl SegmentPaint {
    const RECT: Rect = Rect::from_min_max(pos2(20.0, 20.0), pos2(260.0, 54.0));

    fn segments(words: &[&str]) -> Vec<Segment> {
        words.iter().map(|word| Segment::Text(Sample::text(word))).collect()
    }

    // the divider segments painted (short vertical lines in line-strong)
    fn dividers(words: &[&str], selected: usize, target: usize, state: InteractionState) -> usize {
        let mut selected = selected;
        let shapes = PaintCapture::shapes(|painter| {
            Segmented::new(egui::Id::NULL, &mut selected, Self::segments(words)).paint(
                painter,
                Self::RECT,
                target,
                state,
            );
        });
        shapes
            .iter()
            .filter(|s| matches!(s, Shape::LineSegment { stroke, .. } if stroke.color == color::LineStrong::VALUE))
            .count()
    }
}

#[test]
fn divider_rule() {
    let three = ["24", "30", "60"];
    // selected last, rest: one divider, between 24 and 30
    assert_eq!(SegmentPaint::dividers(&three, 2, 0, InteractionState::Rest), 1);
    // the first segment hovered: that divider goes
    assert_eq!(SegmentPaint::dividers(&three, 2, 0, InteractionState::Hover), 0);
    let five = ["Recorded", "Sunrise", "Noon", "Sunset", "Night"];
    assert_eq!(SegmentPaint::dividers(&five, 2, 0, InteractionState::Rest), 2);
    let four = ["Recorded", "Clear", "Rain", "Thunder"];
    assert_eq!(SegmentPaint::dividers(&four, 0, 1, InteractionState::Rest), 2);
}

#[test]
fn selected_is_soft_and_semibold() {
    let mut selected = 1;
    let shapes = PaintCapture::shapes(|painter| {
        Segmented::new(egui::Id::NULL, &mut selected, SegmentPaint::segments(&["Linear", "Smooth"])).paint(
            painter,
            SegmentPaint::RECT,
            1,
            InteractionState::Rest,
        );
    });
    assert!(PaintCapture::rects(&shapes).iter().any(|r| r.fill == color::AccentSoft::VALUE));
    let smooth = shapes.iter().find_map(|s| match s {
        Shape::Text(t) if t.galley.text() == "Smooth" => Some(t.galley.job.sections[0].format.clone()),
        _ => None,
    });
    let format = smooth.unwrap_or_default();
    assert_eq!(format.color, color::AccentHover::VALUE);
    assert_eq!(format.font_id.family, amml_ui::font::FontFace::Semibold.family());
}

#[test]
fn click_selects() {
    let mut harness = ClickProbe::harness_with(0_usize, |ui, selected| {
        Segmented::new(egui::Id::unique(7), selected, SegmentPaint::segments(&["24", "30", "60"])).show(ui, 240.0)
    });
    let rect = harness.state().rect;
    let third = pos2(rect.right() - 20.0, rect.center().y);
    harness.hover_at(third);
    harness.drag_at(third);
    harness.run();
    harness.drop_at(third);
    harness.run();
    assert_eq!(harness.state().value, 2);
    assert_eq!(harness.state().changes, 1);
}

struct GridPaint;

impl GridPaint {
    const WORDS: [&'static str; 6] = ["Sunrise", "Day", "Noon", "Sunset", "Night", "Midnight"];
    const RECT: Rect = Rect::from_min_max(pos2(20.0, 20.0), pos2(380.0, 20.0 + 28.0 * 2.0 + 5.0));

    // the painted cells: each button-sized rect with its fill
    fn cells(selected: usize) -> Vec<(Rect, egui::Color32)> {
        let mut selected = selected;
        let shapes = PaintCapture::shapes(|painter| {
            Segmented::new(egui::Id::NULL, &mut selected, SegmentPaint::segments(&Self::WORDS)).columns(3).paint(
                painter,
                Self::RECT,
                0,
                InteractionState::Rest,
            );
        });
        PaintCapture::rects(&shapes).into_iter().filter(|r| r.rect.height() == 28.0).map(|r| (r.rect, r.fill)).collect()
    }
}

#[test]
fn grid_lays_the_segments_in_rows_of_columns() {
    let cells = GridPaint::cells(2);
    assert_eq!(cells.len(), 6);
    let width = (GridPaint::RECT.width() - 2.0 * 5.0) / 3.0;
    for (index, (rect, _)) in cells.iter().enumerate() {
        let (row, column) = (index / 3, index % 3);
        assert!((rect.width() - width).abs() < 0.5, "{rect:?}");
        assert!((rect.left() - (GridPaint::RECT.left() + column as f32 * (width + 5.0))).abs() < 0.5, "{rect:?}");
        assert!((rect.top() - (GridPaint::RECT.top() + row as f32 * (28.0 + 5.0))).abs() < 0.5, "{rect:?}");
    }
}

#[test]
fn grid_fills_only_the_selected_cell_with_the_accent() {
    let cells = GridPaint::cells(2);
    for (index, (_, fill)) in cells.iter().enumerate() {
        let want = if index == 2 { color::Accent::VALUE } else { color::BgField::VALUE };
        assert_eq!(*fill, want, "cell {index}");
    }
}

#[test]
fn grid_selected_word_is_on_accent_and_semibold() {
    let mut selected = 2;
    let shapes = PaintCapture::shapes(|painter| {
        Segmented::new(egui::Id::NULL, &mut selected, SegmentPaint::segments(&GridPaint::WORDS)).columns(3).paint(
            painter,
            GridPaint::RECT,
            0,
            InteractionState::Rest,
        );
    });
    let format = |word: &str| {
        shapes.iter().find_map(|s| match s {
            Shape::Text(t) if t.galley.text() == word => Some(t.galley.job.sections[0].format.clone()),
            _ => None,
        })
    };
    let noon = format("Noon").unwrap_or_default();
    assert_eq!(noon.color, color::OnAccent::VALUE);
    assert_eq!(noon.font_id.family, amml_ui::font::FontFace::Semibold.family());
    let day = format("Day").unwrap_or_default();
    assert_eq!(day.color, color::Text::VALUE);
}

#[test]
fn grid_click_in_the_second_row_selects_that_segment() {
    let mut harness = ClickProbe::harness_with(0_usize, |ui, selected| {
        Segmented::new(egui::Id::unique(7), selected, SegmentPaint::segments(&GridPaint::WORDS))
            .columns(3)
            .show(ui, 360.0)
    });
    let rect = harness.state().rect;
    assert_eq!(rect.height(), 28.0 * 2.0 + 5.0);
    let night = pos2(rect.center().x, rect.bottom() - 10.0);
    ClickProbe::click_at(&mut harness, night);
    assert_eq!(harness.state().value, 4);
    assert_eq!(harness.state().changes, 1);
}

#[test]
fn arrow_moves_selection() {
    let mut harness = ClickProbe::harness_with(0_usize, |ui, selected| {
        Segmented::new(egui::Id::unique(7), selected, SegmentPaint::segments(&["24", "30", "60"])).show(ui, 240.0)
    });
    ClickProbe::tab_and_press(&mut harness, Key::ArrowRight);
    assert_eq!(harness.state().value, 1);
}

#[test]
fn each_arrow_moves_one_segment() {
    let mut harness = ClickProbe::harness_with(0_usize, |ui, selected| {
        let segments = SegmentPaint::segments(&["24", "30", "60", "120"]);
        Segmented::new(egui::Id::unique(7), selected, segments).show(ui, 320.0)
    });
    ClickProbe::tab_and_press(&mut harness, Key::ArrowRight);
    harness.key_press(Key::ArrowRight);
    harness.run();
    assert_eq!(harness.state().value, 2);
}

#[test]
fn height_sets_the_track_height_and_the_default_is_the_field() {
    let mut row = ClickProbe::harness_with(0usize, |ui, selected| {
        Segmented::new(egui::Id::new("short"), selected, SegmentPaint::segments(&["Linear", "Smooth"]))
            .height(28.0)
            .show(ui, 240.0)
    });
    row.run();
    assert_eq!(row.state().rect.height(), 28.0);
    let mut field = ClickProbe::harness_with(0usize, |ui, selected| {
        Segmented::new(egui::Id::new("field"), selected, SegmentPaint::segments(&["Linear", "Smooth"])).show(ui, 240.0)
    });
    field.run();
    assert_eq!(field.state().rect.height(), 34.0);
}
