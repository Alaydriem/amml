use amml_studio_ui::component::library::{SceneCard, SceneCardView};
use amml_studio_ui::tokens::selector::SceneColor;
use amml_ui::tokens::{ColorToken, color};
use egui::vec2;

use crate::glyph::words;
use crate::support::UiProbe;

struct CardSample;

impl CardSample {
    fn view(color: SceneColor) -> SceneCardView {
        SceneCardView { name: words("Arrival"), range: words("00:28.80 to 00:37.40"), color }
    }
}

#[test]
fn the_stripe_has_the_scene_colour_for_every_colour() {
    for scene in SceneColor::ALL {
        let view = CardSample::view(scene);
        let shapes = UiProbe::new(316.0).shapes(|ui| SceneCard::new(&view).show(ui));
        let stripe = UiProbe::rects(&shapes).into_iter().find(|r| r.fill == scene.color());
        let Some(stripe) = stripe else { panic!("no stripe for {scene:?}") };
        assert_eq!(stripe.rect.size(), vec2(3.0, 44.0), "{scene:?}");
    }
}

#[test]
fn card_takes_the_width_it_is_given_and_is_seventy_six_high() {
    let view = CardSample::view(SceneColor::Lime);
    let (size, _) = UiProbe::new(316.0).frame(Vec::new(), |ui| {
        SceneCard::new(&view).show(ui);
        ui.min_rect().size()
    });
    assert_eq!(size, vec2(316.0, 76.0));
}

#[test]
fn the_card_is_raised_with_a_line() {
    let view = CardSample::view(SceneColor::Pink);
    let shapes = UiProbe::new(316.0).shapes(|ui| SceneCard::new(&view).show(ui));
    let rects = UiProbe::rects(&shapes);
    assert!(rects.iter().any(|r| r.fill == color::BgRaised::VALUE));
    assert!(rects.iter().any(|r| r.stroke.color == color::Line::VALUE));
}

#[test]
fn the_name_and_the_range_are_the_only_text_and_stack_under_each_other() {
    let view = CardSample::view(SceneColor::Orchid);
    let shapes = UiProbe::new(316.0).shapes(|ui| SceneCard::new(&view).show(ui));
    assert_eq!(UiProbe::words(&shapes), ["Arrival", "00:28.80 to 00:37.40"]);
    let name = UiProbe::text_rect(&shapes, "Arrival");
    let range = UiProbe::text_rect(&shapes, "00:28.80 to 00:37.40");
    let (Some(name), Some(range)) = (name, range) else { panic!("text missing") };
    assert!(name.bottom() <= range.top());
    assert_eq!(name.left(), range.left());
}

#[test]
fn a_click_is_reported() {
    let view = CardSample::view(SceneColor::Lime);
    let mut probe = UiProbe::new(316.0);
    let at = vec2(100.0, 38.0).to_pos2();
    assert!(probe.click(at, |ui| SceneCard::new(&view).show(ui).clicked()));
}
