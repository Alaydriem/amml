use amml_studio_ui::component::library::{HeroPicture, ReplayRow, ReplayRowAction, ReplayRowNote, ReplayRowView};
use amml_studio_ui::tokens::color as studio_color;
use amml_ui::tokens::{ColorToken, color};
use egui::{Color32, Pos2, Shape, TextureId, vec2};

use crate::glyph::words;
use crate::support::UiProbe;

struct RowSample;

impl RowSample {
    const WIDTH: f32 = 539.0;
    const TEXTURE: TextureId = TextureId::Managed(1);

    fn view(selected: bool, broken: bool) -> ReplayRowView {
        ReplayRowView {
            name: words("Nether trip"),
            length: words("02:14:08"),
            facts: vec![words("Today, 14:02"), words("6 players"), words("5 scenes"), words("3 bookmarks")],
            note: if broken {
                ReplayRowNote::Broken(words("missing capture"))
            } else {
                ReplayRowNote::Source(vec![words("BVC"), words("recent")])
            },
            thumbnail: Some(HeroPicture { texture: Self::TEXTURE, size: vec2(1600.0, 900.0) }),
            selected,
        }
    }

    fn shapes(view: &ReplayRowView, hovered: bool) -> Vec<Shape> {
        UiProbe::new(Self::WIDTH).shapes(|ui| ReplayRow::new(view).hovered(hovered).show(ui))
    }

    fn row_rects(shapes: &[Shape]) -> Vec<egui::epaint::RectShape> {
        UiProbe::rects(shapes).into_iter().filter(|r| r.rect.height() == 74.0).collect()
    }

    fn picture(shapes: &[Shape]) -> egui::epaint::RectShape {
        match UiProbe::rects(shapes).into_iter().find(|r| r.fill_texture_id() == Self::TEXTURE) {
            Some(rect) => rect,
            None => panic!("no thumbnail"),
        }
    }

    fn click_on(view: &ReplayRowView, text: &str) -> Option<ReplayRowAction> {
        let mut probe = UiProbe::new(Self::WIDTH);
        let shapes = probe.shapes(|ui| ReplayRow::new(view).show(ui));
        let Some(rect) = UiProbe::text_rect(&shapes, text) else { panic!("{text} is not painted") };
        probe.click(rect.center(), |ui| ReplayRow::new(view).show(ui))
    }
}

#[test]
fn row_is_seventy_four_high_and_fills_the_width() {
    let view = RowSample::view(false, false);
    let (size, _) = UiProbe::new(RowSample::WIDTH).frame(Vec::new(), |ui| {
        ReplayRow::new(&view).show(ui);
        ui.min_rect().size()
    });
    assert_eq!(size, vec2(RowSample::WIDTH, 74.0));
}

#[test]
fn the_only_text_is_the_view_text() {
    let shapes = RowSample::shapes(&RowSample::view(false, false), false);
    let mut painted = UiProbe::words(&shapes);
    painted.sort();
    let mut expected: Vec<String> =
        ["Nether trip", "02:14:08", "Today, 14:02", "6 players", "5 scenes", "3 bookmarks", "BVC", "recent"]
            .map(String::from)
            .to_vec();
    expected.sort();
    assert_eq!(painted, expected);
}

#[test]
fn replay_row_states_follow_paper() {
    // rest: no fill, no line
    let rest = RowSample::row_rects(&RowSample::shapes(&RowSample::view(false, false), false));
    assert!(rest.iter().all(|r| r.fill == Color32::TRANSPARENT && r.stroke.width == 0.0), "{rest:?}");

    // hover: the faded hover fill
    let hover = RowSample::row_rects(&RowSample::shapes(&RowSample::view(false, false), true));
    let fill = color::BgHover::VALUE.gamma_multiply(0.7);
    assert!(hover.iter().any(|r| r.fill == fill), "no hover fill");

    // selected: the soft accent fill and the accent line at the muted opacity
    let selected = RowSample::row_rects(&RowSample::shapes(&RowSample::view(true, false), false));
    assert!(selected.iter().any(|r| r.fill == color::AccentSoft::VALUE), "no soft accent fill");
    let line = color::Accent::VALUE.gamma_multiply(0.45);
    assert!(selected.iter().any(|r| r.stroke.color == line && r.stroke.width == 1.0), "no accent line");
}

#[test]
fn the_thumbnail_is_89_by_50_with_the_length_on_a_strip() {
    let shapes = RowSample::shapes(&RowSample::view(false, false), false);
    assert_eq!(RowSample::picture(&shapes).rect.size(), vec2(89.0, 50.0));
    let strip =
        UiProbe::rects(&shapes).into_iter().find(|r| r.fill == studio_color::PictureStrip::VALUE).map(|r| r.rect);
    let Some(strip) = strip else { panic!("no strip") };
    assert_eq!(strip.height(), 18.0);
    let Some(length) = UiProbe::text_rect(&shapes, "02:14:08") else { panic!("no length") };
    assert!(strip.contains_rect(length));
    assert!((strip.right() - length.right() - 6.0).abs() <= 1.0, "{strip:?} {length:?}");
}

#[test]
fn a_broken_row_fades_the_thumbnail_and_says_why_in_the_danger_colour() {
    let ok = RowSample::picture(&RowSample::shapes(&RowSample::view(false, false), false));
    assert_eq!(ok.fill, Color32::WHITE);
    let shapes = RowSample::shapes(&RowSample::view(false, true), false);
    assert_eq!(RowSample::picture(&shapes).fill, Color32::WHITE.gamma_multiply(0.35));
    let ink = shapes.iter().find_map(|s| match s {
        Shape::Text(t) if t.galley.text() == "missing capture" => Some(t.galley.job.sections[0].format.color),
        _ => None,
    });
    assert_eq!(ink, Some(color::Danger::VALUE));
    assert!(!UiProbe::words(&shapes).contains(&"BVC".to_string()));
}

#[test]
fn a_row_without_a_picture_shows_the_placeholder() {
    let mut view = RowSample::view(false, false);
    view.thumbnail = None;
    let shapes = RowSample::shapes(&view, false);
    // a rect with no texture reports the default id
    let thumbnail = UiProbe::rects(&shapes)
        .into_iter()
        .find(|r| r.rect.size() == vec2(89.0, 50.0) && r.fill_texture_id() != TextureId::default());
    assert!(thumbnail.is_some(), "no placeholder thumbnail");
}

#[test]
fn a_click_selects_a_double_click_opens_and_a_right_click_asks_for_the_menu() {
    let view = RowSample::view(false, false);
    assert_eq!(RowSample::click_on(&view, "Nether trip"), Some(ReplayRowAction::Select));

    let mut probe = UiProbe::new(RowSample::WIDTH);
    let shapes = probe.shapes(|ui| ReplayRow::new(&view).show(ui));
    let Some(name) = UiProbe::text_rect(&shapes, "Nether trip") else { panic!("no name") };
    let at: Pos2 = name.center();
    probe.click(at, |ui| ReplayRow::new(&view).show(ui));
    assert_eq!(probe.click(at, |ui| ReplayRow::new(&view).show(ui)), Some(ReplayRowAction::Open));

    let mut probe = UiProbe::new(RowSample::WIDTH);
    probe.shapes(|ui| ReplayRow::new(&view).show(ui));
    let action = probe.secondary_click(at, |ui| ReplayRow::new(&view).show(ui));
    assert_eq!(action, Some(ReplayRowAction::Menu(at)));
}

#[test]
fn the_facts_have_a_dot_between_them_and_no_dot_character() {
    let shapes = RowSample::shapes(&RowSample::view(false, false), false);
    assert!(UiProbe::words(&shapes).iter().all(|word| !word.contains('\u{00B7}')));
    let dots = UiProbe::rects(&shapes).into_iter().filter(|r| r.rect.size() == vec2(3.0, 3.0)).count();
    // three gaps between four facts, one between the two source words
    assert_eq!(dots, 4);
}
