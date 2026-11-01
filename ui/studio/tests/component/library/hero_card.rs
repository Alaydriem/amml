use amml_studio_ui::component::library::{HeroCard, HeroCardAction, HeroCardView, HeroPicture, HeroProblem};
use amml_studio_ui::tokens::color as studio_color;
use amml_ui::tokens::{ColorToken, color};
use egui::{Rect, Shape, vec2};

use crate::glyph::words;
use crate::support::UiProbe;

struct HeroSample;

impl HeroSample {
    const SIZE: egui::Vec2 = vec2(900.0, 506.0);

    fn view(broken: bool) -> HeroCardView {
        HeroCardView {
            title: words("Nether trip"),
            meta: vec![words("02:14:08"), words("6 players"), words("5 scenes"), words("Today, 14:02")],
            open: words("Open in editor"),
            problem: broken.then(|| HeroProblem {
                title: words("Game version not supported"),
                reason: words("This recording was made with a game protocol this version cannot play."),
            }),
        }
    }

    fn shapes(view: &HeroCardView) -> Vec<Shape> {
        UiProbe::sized(1000.0, 700.0).shapes(|ui| HeroCard::new(view).show(ui, Self::SIZE))
    }

    fn open_button(shapes: &[Shape]) -> Rect {
        let Some(label) = UiProbe::text_rect(shapes, "Open in editor") else { panic!("no button label") };
        UiProbe::rects(shapes)
            .into_iter()
            .map(|r| r.rect)
            .find(|rect| rect.contains_rect(label) && rect.height() == 42.0)
            .unwrap_or_else(|| panic!("no 42 high button around {label:?}"))
    }
}

#[test]
fn the_only_text_is_the_view_text() {
    let shapes = HeroSample::shapes(&HeroSample::view(false));
    assert_eq!(
        UiProbe::words(&shapes),
        ["Nether trip", "02:14:08", "6 players", "5 scenes", "Today, 14:02", "Open in editor"]
    );
}

#[test]
fn the_card_takes_the_size_it_is_given() {
    let view = HeroSample::view(false);
    let (size, _) = UiProbe::sized(1000.0, 700.0).frame(Vec::new(), |ui| {
        HeroCard::new(&view).show(ui, HeroSample::SIZE);
        ui.min_rect().size()
    });
    assert_eq!(size, HeroSample::SIZE);
}

#[test]
fn the_title_is_on_picture_white_and_the_bottom_fades_to_the_picture_fade() {
    let shapes = HeroSample::shapes(&HeroSample::view(false));
    let title = shapes.iter().find_map(|s| match s {
        Shape::Text(t) if t.galley.text() == "Nether trip" => Some(t.galley.job.sections[0].format.color),
        _ => None,
    });
    assert_eq!(title, Some(color::OnPicture::VALUE));
    let fade = studio_color::PictureFade::VALUE;
    let faded = shapes.iter().any(|s| match s {
        Shape::Mesh(mesh) => {
            mesh.vertices.iter().any(|v| v.color == fade) && mesh.vertices.iter().any(|v| v.color.a() == 0)
        }
        _ => false,
    });
    assert!(faded, "no gradient from transparent to the picture fade");
}

#[test]
fn the_open_button_is_accent_and_reports_open() {
    let view = HeroSample::view(false);
    let shapes = HeroSample::shapes(&view);
    let button = HeroSample::open_button(&shapes);
    assert!(UiProbe::rects(&shapes).iter().any(|r| r.rect == button && r.fill == color::Accent::VALUE));
    let mut probe = UiProbe::sized(1000.0, 700.0);
    let action = probe.click(button.center(), |ui| HeroCard::new(&view).show(ui, HeroSample::SIZE));
    assert_eq!(action, Some(HeroCardAction::Open));
}

#[test]
fn a_broken_card_has_a_muted_button_that_does_nothing() {
    let view = HeroSample::view(true);
    let shapes = HeroSample::shapes(&view);
    let button = HeroSample::open_button(&shapes);
    let fill = UiProbe::rects(&shapes).into_iter().find(|r| r.rect == button).map(|r| r.fill);
    let muted = color::BgActive::VALUE.gamma_multiply(0.55);
    assert_eq!(fill, Some(muted));
    let mut probe = UiProbe::sized(1000.0, 700.0);
    assert_eq!(probe.click(button.center(), |ui| HeroCard::new(&view).show(ui, HeroSample::SIZE)), None);
}

#[test]
fn a_picture_is_cropped_to_fill_the_card_without_stretching() {
    let wide = HeroPicture { texture: egui::TextureId::Managed(1), size: vec2(200.0, 100.0) };
    let uv = wide.cover(vec2(100.0, 100.0));
    assert_eq!((uv.min, uv.max), (egui::pos2(0.25, 0.0), egui::pos2(0.75, 1.0)));
    let tall = HeroPicture { texture: egui::TextureId::Managed(1), size: vec2(100.0, 200.0) };
    let uv = tall.cover(vec2(100.0, 100.0));
    assert_eq!((uv.min, uv.max), (egui::pos2(0.0, 0.25), egui::pos2(1.0, 0.75)));
    let exact = wide.cover(vec2(400.0, 200.0));
    assert_eq!((exact.min, exact.max), (egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)));
}

#[test]
fn the_button_sits_at_the_bottom_right_inside_the_card() {
    let view = HeroSample::view(false);
    let mut probe = UiProbe::sized(1000.0, 700.0);
    let mut show = |ui: &mut egui::Ui| {
        HeroCard::new(&view).show(ui, HeroSample::SIZE);
        ui.min_rect()
    };
    probe.frame(Vec::new(), &mut show);
    let (card, shapes) = probe.frame(Vec::new(), &mut show);
    let button = HeroSample::open_button(&shapes);
    assert!((card.right() - button.right() - 24.0).abs() <= 1.0, "{button:?}");
    assert!((card.bottom() - button.bottom() - 24.0).abs() <= 1.0, "{button:?}");
}

#[test]
fn a_card_without_a_picture_paints_the_placeholder_over_the_whole_card() {
    let shapes = HeroSample::shapes(&HeroSample::view(false));
    let placeholder = UiProbe::rects(&shapes)
        .into_iter()
        .find(|r| r.rect.size() == HeroSample::SIZE && r.fill_texture_id() != egui::TextureId::default());
    assert!(placeholder.is_some(), "no placeholder picture");
}

#[test]
fn a_broken_card_writes_the_problem_over_the_picture() {
    let shapes = HeroSample::shapes(&HeroSample::view(true));
    let words = UiProbe::words(&shapes);
    assert!(words.contains(&"Game version not supported".to_string()), "{words:?}");
    assert!(words.iter().any(|w| w.starts_with("This recording was made")), "{words:?}");
    let title = UiProbe::text_rect(&shapes, "Game version not supported").expect("problem title");
    let name = UiProbe::text_rect(&shapes, "Nether trip").expect("name");
    assert!(title.bottom() <= name.top(), "problem {title:?} must sit above the name {name:?}");
    let ready = UiProbe::words(&HeroSample::shapes(&HeroSample::view(false)));
    assert!(!ready.contains(&"Game version not supported".to_string()));
}
