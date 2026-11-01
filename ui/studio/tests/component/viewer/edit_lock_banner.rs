use amml_studio_ui::component::viewer::EditLockBanner;
use amml_ui::tokens::{ColorToken, color};
use egui::{Shape, vec2};

use crate::glyph::words;
use crate::support::UiProbe;

struct BannerSample;

impl BannerSample {
    fn banner() -> EditLockBanner {
        EditLockBanner::new(words("Locked: Scene camera drives, pause to edit"))
    }
}

#[test]
fn the_banner_is_as_wide_as_asked_and_thirty_two_high() {
    let (size, _) = UiProbe::new(500.0).frame(Vec::new(), |ui| {
        BannerSample::banner().show(ui, 402.0);
        ui.min_rect().size()
    });
    assert_eq!(size, vec2(402.0, 32.0));
}

#[test]
fn the_reason_comes_from_the_caller_and_is_in_the_scene_camera_colour() {
    let shapes = UiProbe::new(500.0).shapes(|ui| BannerSample::banner().show(ui, 402.0));
    let reason = "Locked: Scene camera drives, pause to edit";
    assert_eq!(UiProbe::words(&shapes), [reason]);
    let ink = shapes.iter().find_map(|s| match s {
        Shape::Text(t) if t.galley.text() == reason => Some(t.galley.job.sections[0].format.color),
        _ => None,
    });
    assert_eq!(ink, Some(amml_studio_ui::tokens::color::SceneCamera::VALUE));
}

#[test]
fn the_pill_has_a_faded_app_fill_and_a_scene_camera_border() {
    let shapes = UiProbe::new(500.0).shapes(|ui| BannerSample::banner().show(ui, 402.0));
    let rects = UiProbe::rects(&shapes);
    let faded = color::BgApp::VALUE.gamma_multiply(0.92);
    assert!(rects.iter().any(|r| r.rect.size() == vec2(402.0, 32.0) && r.fill == faded), "no faded fill");
    let border = amml_studio_ui::tokens::color::SceneCamera::VALUE;
    assert!(rects.iter().any(|r| r.rect.size() == vec2(402.0, 32.0) && r.stroke.color == border), "no border");
}

#[test]
fn the_icon_and_the_words_are_centred_as_one_group() {
    let shapes = UiProbe::new(500.0).shapes(|ui| BannerSample::banner().show(ui, 402.0));
    let texts = UiProbe::texts(&shapes);
    assert_eq!(texts.len(), 2, "an icon and the reason: {texts:?}");
    let all: Vec<egui::Rect> = shapes
        .iter()
        .filter_map(|s| match s {
            Shape::Text(t) => Some(t.galley.rect.translate(t.pos.to_vec2())),
            _ => None,
        })
        .collect();
    let group = all.iter().fold(egui::Rect::NOTHING, |all, rect| all.union(*rect));
    let pill = UiProbe::rects(&shapes).into_iter().find(|r| r.rect.size() == vec2(402.0, 32.0)).map(|r| r.rect);
    let Some(pill) = pill else { panic!("no pill") };
    // the icon's glyph box is a little wider than its slot
    assert!((group.center().x - pill.center().x).abs() <= 3.0, "{group:?} in {pill:?}");
}
