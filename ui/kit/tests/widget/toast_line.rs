use amml_ui::tokens::{ColorToken, color};
use amml_ui::widget::{ToastLine, ToastTone};
use egui::{Rect, pos2};

use crate::support::{FlowProbe, PaintCapture, Sample, ShapeText};

struct LinePaint;

impl LinePaint {
    fn paint(tone: ToastTone, words: &str) -> (Rect, Vec<egui::Shape>) {
        let mut rect = Rect::NOTHING;
        let line = ToastLine::new(tone, Sample::text(words));
        let shapes = PaintCapture::shapes(|painter| rect = line.paint(painter, pos2(20.0, 20.0)));
        (rect, shapes)
    }
}

#[test]
fn the_only_text_is_the_given_line() {
    let (_, shapes) = LinePaint::paint(ToastTone::Info, "Added Arrival to the render queue");
    assert_eq!(ShapeText::all(&shapes), ["Added Arrival to the render queue"]);
}

#[test]
fn fill_is_the_toast_token_and_the_border_is_the_tone() {
    for tone in [ToastTone::Info, ToastTone::Danger] {
        let (rect, shapes) = LinePaint::paint(tone, "Select two or more keys at different times first");
        let rects = PaintCapture::rects(&shapes);
        assert!(rects.iter().any(|r| r.rect == rect && r.fill == color::Toast::VALUE), "no toast fill for {tone:?}");
        let line = tone.color().gamma_multiply(0.8);
        assert!(rects.iter().any(|r| r.rect == rect && r.stroke.color == line), "no tone border for {tone:?}");
    }
}

#[test]
fn a_three_point_stripe_in_the_tone_colour_runs_down_the_left_edge() {
    for tone in [ToastTone::Info, ToastTone::Danger] {
        let (rect, shapes) = LinePaint::paint(tone, "Added Arrival to the render queue");
        let stripe = PaintCapture::rects(&shapes).into_iter().find(|r| r.fill == tone.color());
        let Some(stripe) = stripe else { panic!("no stripe for {tone:?}") };
        assert_eq!(stripe.rect.width(), 3.0);
        // inside the border at the left, and over the border at the top and bottom
        assert_eq!(stripe.rect.left(), rect.left() + 1.0);
        assert_eq!(stripe.rect.height(), rect.height());
    }
}

#[test]
fn the_toast_hugs_its_text() {
    let (short, _) = LinePaint::paint(ToastTone::Info, "Saved");
    let (long, _) = LinePaint::paint(ToastTone::Info, "Select two or more keys at different times first");
    assert!(long.width() > short.width() + 100.0, "{short:?} {long:?}");
    assert_eq!(long.height(), short.height());
}

#[test]
fn showing_it_takes_its_own_size_in_the_flow() {
    let log = FlowProbe::run(|ui| {
        ToastLine::new(ToastTone::Info, Sample::text("Saved")).show(ui);
    });
    FlowProbe::assert_below(&log);
}
