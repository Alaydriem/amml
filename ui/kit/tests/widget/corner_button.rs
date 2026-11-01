use amml_i18n::text::UserText;
use amml_ui::icon::Icon;
use amml_ui::state::InteractionState;
use amml_ui::tokens::{ColorToken, color};
use amml_ui::widget::{CornerButton, CornerButtonLead};
use egui::{Color32, Rect, Shape, pos2, vec2};

use crate::support::{ClickProbe, PaintCapture, Sample};

struct CornerPaint;

impl CornerPaint {
    const RECT: Rect = Rect::from_min_max(pos2(20.0, 20.0), pos2(160.0, 65.0));

    fn mark() -> CornerButtonLead {
        CornerButtonLead::Mark(UserText::from_model("DV".to_owned()))
    }

    fn shapes(state: InteractionState) -> Vec<Shape> {
        let button = CornerButton::new(Self::mark()).label(Sample::text("Dejavu"));
        PaintCapture::shapes(|painter| button.paint(painter, Self::RECT, state))
    }

    // the colour a text shape is drawn in
    fn text_colour(shapes: &[Shape], text: &str) -> Option<Color32> {
        shapes.iter().find_map(|shape| match shape {
            Shape::Text(shape) if shape.galley.text() == text => Some(shape.galley.job.sections[0].format.color),
            _ => None,
        })
    }
}

#[test]
fn icon_form_is_a_square_of_the_corner_height() {
    let harness = ClickProbe::harness(|ui| CornerButton::new(CornerButtonLead::Icon(Icon::Mods)).show(ui));

    assert_eq!(harness.state().rect.size(), vec2(CornerButton::HEIGHT, CornerButton::HEIGHT));
}

#[test]
fn labelled_form_grows_with_its_label() {
    let width = |label: &'static str| {
        let harness =
            ClickProbe::harness(move |ui| CornerButton::new(CornerPaint::mark()).label(Sample::text(label)).show(ui));
        harness.state().rect
    };
    let short = width("Dejavu");
    let long = width("Dejavu replays");

    assert_eq!(short.height(), CornerButton::HEIGHT);
    assert!(short.width() > CornerButton::HEIGHT, "{short:?}");
    assert!(long.width() > short.width(), "{long:?} is not wider than {short:?}");
}

#[test]
fn click_is_reported() {
    let mut harness = ClickProbe::harness(|ui| CornerButton::new(CornerButtonLead::Icon(Icon::Mods)).show(ui));

    ClickProbe::click(&mut harness);

    assert_eq!(harness.state().clicks, 1);
}

#[test]
fn fill_follows_the_accent_ladder() {
    let looks = [
        (InteractionState::Rest, color::Accent::VALUE),
        (InteractionState::Hover, color::AccentHover::VALUE),
        (InteractionState::Pressed, color::AccentPress::VALUE),
        (InteractionState::Focused, color::Accent::VALUE),
    ];
    for (state, fill) in looks {
        let shapes = CornerPaint::shapes(state);
        let rects = PaintCapture::rects(&shapes);
        assert!(rects.iter().any(|r| r.rect == CornerPaint::RECT && r.fill == fill), "{state:?}: body fill");
        // the mark is drawn in the body's fill, on an on-accent tile
        assert_eq!(CornerPaint::text_colour(&shapes, "DV"), Some(fill), "{state:?}: mark colour");
        assert!(rects.iter().any(|r| r.fill == color::OnAccent::VALUE), "{state:?}: no mark tile");
        assert_eq!(CornerPaint::text_colour(&shapes, "Dejavu"), Some(color::OnAccent::VALUE), "{state:?}: label");
    }
}

#[test]
fn focus_shows_the_ring() {
    let rects = PaintCapture::rects(&CornerPaint::shapes(InteractionState::Focused));
    assert!(rects.iter().any(|r| r.fill == color::FocusRing::VALUE), "no focus ring");
    let rest = PaintCapture::rects(&CornerPaint::shapes(InteractionState::Rest));
    assert!(rest.iter().all(|r| r.fill != color::FocusRing::VALUE), "a ring at rest");
}
