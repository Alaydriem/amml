use amml_studio_ui::component::inspector::{PlayerRow, PlayerRowAction, PlayerRowView};
use amml_ui::tokens::{ColorToken, color};
use egui::{Rect, Shape, vec2};

use crate::glyph::words;
use crate::support::UiProbe;

struct RowSample;

impl RowSample {
    const WIDTH: f32 = 414.0;

    fn view(following: bool) -> PlayerRowView {
        PlayerRowView {
            initial: words("K"),
            name: words("Kestrel"),
            distance: words("32 m from the camera"),
            follow: words(if following { "Following" } else { "Follow" }),
            go_to: words("Go to"),
            following,
        }
    }

    fn shapes(view: &PlayerRowView) -> Vec<Shape> {
        UiProbe::new(Self::WIDTH).shapes(|ui| PlayerRow::new(view).show(ui))
    }

    fn click(view: &PlayerRowView, text: &str) -> Option<PlayerRowAction> {
        let mut probe = UiProbe::new(Self::WIDTH);
        let shapes = probe.shapes(|ui| PlayerRow::new(view).show(ui));
        let Some(rect) = UiProbe::text_rect(&shapes, text) else { panic!("{text} is not painted") };
        probe.click(rect.center(), |ui| PlayerRow::new(view).show(ui))
    }
}

#[test]
fn row_is_fifty_four_high_and_fills_the_width() {
    let view = RowSample::view(false);
    let (size, _) = UiProbe::new(RowSample::WIDTH).frame(Vec::new(), |ui| {
        PlayerRow::new(&view).show(ui);
        ui.min_rect().size()
    });
    assert_eq!(size, vec2(RowSample::WIDTH, 54.0));
}

#[test]
fn the_only_text_is_the_view_text() {
    let shapes = RowSample::shapes(&RowSample::view(false));
    let mut painted = UiProbe::words(&shapes);
    painted.sort();
    assert_eq!(painted, ["32 m from the camera", "Follow", "Go to", "K", "Kestrel"]);
}

#[test]
fn the_buttons_report_follow_and_go_to() {
    let view = RowSample::view(false);
    assert_eq!(RowSample::click(&view, "Follow"), Some(PlayerRowAction::ToggleFollow));
    assert_eq!(RowSample::click(&view, "Go to"), Some(PlayerRowAction::GoTo));
    let following = RowSample::view(true);
    assert_eq!(RowSample::click(&following, "Following"), Some(PlayerRowAction::ToggleFollow));
}

#[test]
fn a_followed_player_has_the_accent_look() {
    let shapes = RowSample::shapes(&RowSample::view(true));
    let rects = UiProbe::rects(&shapes);
    assert!(rects.iter().any(|r| r.fill == color::AccentSoft::VALUE && r.rect.height() == 54.0), "no soft row");
    let line = color::Accent::VALUE.gamma_multiply(0.4);
    assert!(rects.iter().any(|r| r.stroke.color == line && r.rect.height() == 54.0), "no accent line");
    let button = rects.iter().find(|r| r.fill == color::Accent::VALUE && r.rect.height() == 26.0);
    let Some(button) = button else { panic!("no accent button") };
    let Some(label) = UiProbe::text_rect(&shapes, "Following") else { panic!("no label") };
    assert!(button.rect.contains_rect(label));
}

#[test]
fn the_hover_look_can_be_forced_without_the_pointer() {
    let view = RowSample::view(false);
    let shapes = UiProbe::new(RowSample::WIDTH).shapes(|ui| PlayerRow::new(&view).hovered(true).show(ui));
    let fill = color::BgHover::VALUE.gamma_multiply(0.7);
    assert!(UiProbe::rects(&shapes).iter().any(|r| r.rect.height() == 54.0 && r.fill == fill));
}

#[test]
fn a_row_at_rest_has_no_fill_and_a_hovered_row_has_the_faded_hover_fill() {
    let view = RowSample::view(false);
    let rest = RowSample::shapes(&view);
    assert!(!UiProbe::rects(&rest).iter().any(|r| r.rect.height() == 54.0 && r.fill != egui::Color32::TRANSPARENT));
    let mut probe = UiProbe::new(RowSample::WIDTH);
    probe.shapes(|ui| PlayerRow::new(&view).show(ui));
    let at = Rect::from_min_size(egui::pos2(0.0, 0.0), vec2(RowSample::WIDTH, 54.0)).left_center() + vec2(60.0, 0.0);
    let (_, hovered) = probe.frame(vec![egui::Event::PointerMoved(at)], |ui| PlayerRow::new(&view).show(ui));
    let fill = color::BgHover::VALUE.gamma_multiply(0.7);
    assert!(UiProbe::rects(&hovered).iter().any(|r| r.rect.height() == 54.0 && r.fill == fill), "no hover fill");
}
