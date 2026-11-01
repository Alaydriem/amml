use amml_studio_ui::component::inspector::{KeyHeader, KeyHeaderTone, KeyHeaderView};
use amml_studio_ui::tokens::selector::TrackColor;
use amml_ui::tokens::{ColorToken, color};
use egui::vec2;

use crate::glyph::words;
use crate::support::UiProbe;

struct HeaderSample;

impl HeaderSample {
    fn view(tone: KeyHeaderTone) -> KeyHeaderView {
        KeyHeaderView {
            title: words("Camera key"),
            detail: words("00:00:34.70 and tick 694"),
            chip: words("#4 of 6"),
            tone,
        }
    }
}

#[test]
fn the_stripe_and_the_dot_take_the_track_colour_of_every_track() {
    for track in TrackColor::ALL {
        let view = HeaderSample::view(KeyHeaderTone::Track(track));
        let shapes = UiProbe::new(398.0).shapes(|ui| KeyHeader::new(&view).show(ui));
        let stripe =
            UiProbe::rects(&shapes).into_iter().find(|r| r.fill == track.color() && r.rect.size() == vec2(3.0, 38.0));
        assert!(stripe.is_some(), "no stripe for {track:?}");
        let dot =
            shapes.iter().any(|s| matches!(s, egui::Shape::Circle(c) if c.fill == track.color() && c.radius == 4.5));
        assert!(dot, "no dot for {track:?}");
    }
}

#[test]
fn several_keys_use_the_accent() {
    let view = HeaderSample::view(KeyHeaderTone::Several);
    let shapes = UiProbe::new(398.0).shapes(|ui| KeyHeader::new(&view).show(ui));
    assert!(UiProbe::rects(&shapes).iter().any(|r| r.fill == color::Accent::VALUE && r.rect.size() == vec2(3.0, 38.0)));
}

#[test]
fn header_is_sixty_six_high_and_fills_the_width() {
    let view = HeaderSample::view(KeyHeaderTone::Track(TrackColor::Camera));
    let (size, _) = UiProbe::new(398.0).frame(Vec::new(), |ui| {
        KeyHeader::new(&view).show(ui);
        ui.min_rect().size()
    });
    assert_eq!(size, vec2(398.0, 66.0));
}

#[test]
fn title_detail_and_chip_come_from_the_view_and_the_chip_is_at_the_right() {
    let view = HeaderSample::view(KeyHeaderTone::Track(TrackColor::Fov));
    let shapes = UiProbe::new(398.0).shapes(|ui| KeyHeader::new(&view).show(ui));
    let mut painted = UiProbe::words(&shapes);
    painted.sort();
    assert_eq!(painted, ["#4 of 6", "00:00:34.70 and tick 694", "Camera key"]);
    let title = UiProbe::text_rect(&shapes, "Camera key");
    let chip = UiProbe::text_rect(&shapes, "#4 of 6");
    let (Some(title), Some(chip)) = (title, chip) else { panic!("text missing") };
    assert!(chip.left() > title.right());
    let pill = UiProbe::rects(&shapes).into_iter().find(|r| r.fill == color::BgField::VALUE);
    let Some(pill) = pill else { panic!("no chip pill") };
    assert!(pill.rect.contains_rect(chip));
    assert!(pill.rect.right() <= 398.0 - 14.0 + 0.5, "{:?}", pill.rect);
}
