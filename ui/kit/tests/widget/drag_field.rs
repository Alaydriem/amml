use amml_ui::state::InteractionState;
use amml_ui::tokens::{ColorToken, color};
use amml_ui::widget::{DragField, DragUnit, FieldLook};
use egui::{Event, Key, Rect, Shape, pos2, vec2};

use crate::support::{ClickProbe, PaintCapture, Sample, ShapeText};

struct DragHarness;

impl DragHarness {
    fn type_and_enter(harness: &mut egui_kittest::Harness<'static, crate::support::ClickLog<f64>>, text: &str) {
        ClickProbe::click(harness);
        // the whole value is selected, so typing replaces it
        harness.event(Event::Text(text.into()));
        harness.run();
        harness.key_press(Key::Enter);
        harness.run();
    }
}

#[test]
fn drag_changes_by_step_and_clamps() {
    let mut harness = ClickProbe::harness_with(10.0_f64, |ui, value| {
        DragField::new(egui::Id::unique(1), value).step(0.5).range(0.0..=20.0, Sample::text("Max 20")).show(ui, 120.0)
    });
    let start = harness.state().rect.center();
    harness.hover_at(start);
    harness.drag_at(start);
    harness.run();
    harness.hover_at(start + vec2(8.0, 0.0));
    harness.run();
    harness.drop_at(start + vec2(8.0, 0.0));
    harness.run();
    assert!((harness.state().value - 14.0).abs() < 1e-9, "{}", harness.state().value);
    // a long drag stops at the range end
    harness.hover_at(start);
    harness.drag_at(start);
    harness.run();
    harness.hover_at(start + vec2(100.0, 0.0));
    harness.run();
    harness.drop_at(start + vec2(100.0, 0.0));
    harness.run();
    assert_eq!(harness.state().value, 20.0);
}

#[test]
fn typed_value_commits_on_enter() {
    let mut harness =
        ClickProbe::harness_with(1.0_f64, |ui, value| DragField::new(egui::Id::unique(1), value).show(ui, 120.0));
    DragHarness::type_and_enter(&mut harness, "2.5");
    assert_eq!(harness.state().value, 2.5);
}

#[test]
fn typed_garbage_shows_invalid_and_keeps_value() {
    let mut harness =
        ClickProbe::harness_with(1.0_f64, |ui, value| DragField::new(egui::Id::unique(1), value).show(ui, 120.0));
    DragHarness::type_and_enter(&mut harness, "abc");
    assert_eq!(harness.state().value, 1.0);
    let shapes: Vec<Shape> = harness.output().shapes.iter().map(|c| c.shape.clone()).collect();
    assert!(ShapeText::contains(&shapes, "Enter a number"), "{:?}", ShapeText::all(&shapes));
}

#[test]
fn typed_out_of_range_shows_the_range_reason() {
    let mut harness = ClickProbe::harness_with(10.0_f64, |ui, value| {
        DragField::new(egui::Id::unique(1), value).range(0.0..=360.0, Sample::text("Max 360°")).show(ui, 120.0)
    });
    DragHarness::type_and_enter(&mut harness, "412");
    assert_eq!(harness.state().value, 10.0);
    let shapes: Vec<Shape> = harness.output().shapes.iter().map(|c| c.shape.clone()).collect();
    assert!(ShapeText::contains(&shapes, "Max 360°"));
}

#[test]
fn escape_cancels_typing() {
    let mut harness =
        ClickProbe::harness_with(3.0_f64, |ui, value| DragField::new(egui::Id::unique(1), value).show(ui, 120.0));
    ClickProbe::click(&mut harness);
    harness.event(Event::Text("9".into()));
    harness.run();
    harness.key_press(Key::Escape);
    harness.run();
    assert_eq!(harness.state().value, 3.0);
}

#[test]
fn degrees_join_and_words_are_spaced() {
    let rect = Rect::from_min_max(pos2(20.0, 20.0), pos2(140.0, 54.0));
    let gap = |unit: DragUnit| {
        let mut value = 325.0;
        let shapes = PaintCapture::shapes(|painter| {
            DragField::new(egui::Id::NULL, &mut value).decimals(1).unit(unit.clone()).paint(
                painter,
                rect,
                InteractionState::Rest,
                &FieldLook::default(),
            );
        });
        let mut texts: Vec<(String, Rect)> = shapes
            .iter()
            .filter_map(|s| {
                if let Shape::Text(t) = s {
                    Some((t.galley.text().to_owned(), t.galley.rect.translate(t.pos.to_vec2())))
                } else {
                    None
                }
            })
            .collect();
        texts.sort_by(|a, b| a.1.left().total_cmp(&b.1.left()));
        texts[1].1.left() - texts[0].1.right()
    };
    assert!(gap(DragUnit::Degrees).abs() < 0.5);
    assert!((gap(DragUnit::Text(Sample::text("m"))) - 4.0).abs() < 0.5);
}

#[test]
fn joined_unit_has_no_gap() {
    let rect = Rect::from_min_max(pos2(20.0, 20.0), pos2(140.0, 54.0));
    let mut value = 3.0;
    let shapes = PaintCapture::shapes(|painter| {
        DragField::new(egui::Id::NULL, &mut value).unit(DragUnit::Joined(Sample::text("x"))).paint(
            painter,
            rect,
            InteractionState::Rest,
            &FieldLook::default(),
        );
    });
    let mut texts = PaintCapture::text_rects(&shapes);
    texts.sort_by(|a, b| a.1.left().total_cmp(&b.1.left()));

    assert_eq!(texts.iter().map(|(text, _)| text.as_str()).collect::<Vec<_>>(), ["3.00", "x"]);
    assert!((texts[1].1.left() - texts[0].1.right()).abs() < 0.5);
}

#[test]
fn long_value_is_cut_before_the_label() {
    let rect = Rect::from_min_max(pos2(20.0, 20.0), pos2(140.0, 54.0));
    let mut value = 1e12;
    let shapes = PaintCapture::shapes(|painter| {
        DragField::new(egui::Id::NULL, &mut value)
            .label(Sample::text("X"), egui::Color32::RED)
            .unit(DragUnit::Text(Sample::text("m")))
            .paint(painter, rect, InteractionState::Rest, &FieldLook::default());
    });
    let rects: Vec<(String, Rect)> = shapes
        .iter()
        .filter_map(|s| {
            if let Shape::Text(t) = s {
                Some((t.galley.text().to_owned(), t.galley.rect.translate(t.pos.to_vec2())))
            } else {
                None
            }
        })
        .collect();
    let label = rects.iter().find(|(text, _)| text == "X").map(|(_, r)| *r);
    let number = rects.iter().find(|(text, _)| text.starts_with('1')).map(|(_, r)| *r);
    let (Some(label), Some(number)) = (label, number) else { panic!("{rects:?}") };
    assert!(number.left() >= label.right(), "{number:?} overlaps {label:?}");
}

#[test]
fn dragging_look_has_accent_border() {
    let rect = Rect::from_min_max(pos2(20.0, 20.0), pos2(140.0, 54.0));
    let mut value = 1.0;
    let shapes = PaintCapture::shapes(|painter| {
        DragField::new(egui::Id::NULL, &mut value).paint(
            painter,
            rect,
            InteractionState::Pressed,
            &FieldLook::default(),
        );
    });
    let rects = PaintCapture::rects(&shapes);
    assert!(rects.iter().any(|r| r.rect == rect && r.fill == color::BgActive::VALUE));
    assert!(rects.iter().any(|r| r.rect == rect && r.stroke.color == color::Accent::VALUE));
}

#[test]
fn typing_does_not_move_the_next_field() {
    // two fields side by side; the log keeps the second field's rect, the value the first one's
    let mut harness = ClickProbe::harness_with((1.0_f64, 2.0_f64, Rect::NOTHING), |ui, (a, b, first)| {
        ui.horizontal(|ui| {
            *first = DragField::new(egui::Id::unique(1), a).show(ui, 96.0).rect;
            DragField::new(egui::Id::unique(2), b).show(ui, 96.0)
        })
        .inner
    });
    let rest = harness.state().rect;
    let first = harness.state().value.2;
    ClickProbe::click_at(&mut harness, first.center());
    harness.event(Event::Text("7".into()));
    harness.run();
    assert_eq!(harness.state().rect, rest, "the next field moved while the first one was typed into");
}

#[test]
fn two_fields_do_not_share_typing_state() {
    let mut harness = ClickProbe::harness_with((1.0_f64, 2.0_f64), |ui, (a, b)| {
        let first = DragField::new(egui::Id::unique(1), a).show(ui, 120.0);
        DragField::new(egui::Id::unique(2), b).show(ui, 120.0);
        first
    });
    ClickProbe::click(&mut harness);
    harness.event(Event::Text("7".into()));
    harness.run();
    harness.key_press(Key::Enter);
    harness.run();
    assert_eq!(harness.state().value, (7.0, 2.0));
}
