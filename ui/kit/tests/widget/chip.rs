use amml_ui::draw::Draw;
use amml_ui::icon::Icon;
use amml_ui::state::InteractionState;
use amml_ui::tokens::rule::SoftFill;
use amml_ui::tokens::{ColorToken, color};
use amml_ui::widget::{Chip, ChipKind, ChipSize, ChipTone, KeyCap, KeyCapLabel, KeyCapSize, KeyChord};
use egui::{Color32, Pos2, Rect, Shape, pos2, vec2};

use crate::support::{ClickProbe, PaintCapture, Sample};

#[test]
fn a_chip_never_takes_more_than_the_room_given() {
    let harness = ClickProbe::harness(|ui| {
        ui.allocate_ui(egui::vec2(60.0, 40.0), |ui| {
            Chip::new(ChipKind::Display(ChipTone::Neutral), Sample::text("World draw and more words")).show(ui)
        })
        .inner
    });

    assert!(harness.state().rect.width() <= 60.0, "the chip is {} wide", harness.state().rect.width());
}

struct ChipPaint;

impl ChipPaint {
    const RECT: Rect = Rect::from_min_max(pos2(20.0, 20.0), pos2(100.0, 40.0));

    // (fill, border, first text colour, has a check icon)
    fn look(chip: &Chip, state: InteractionState) -> (Color32, Option<Color32>, Option<Color32>, bool) {
        let shapes = PaintCapture::shapes(|painter| chip.paint(painter, Self::RECT, state));
        let rects = PaintCapture::rects(&shapes);
        let fill = rects
            .iter()
            .find(|r| r.rect == Self::RECT && r.fill != Color32::TRANSPARENT)
            .map_or(Color32::TRANSPARENT, |r| r.fill);
        let border = rects.iter().find(|r| r.rect == Self::RECT && r.stroke.width > 0.0).map(|r| r.stroke.color);
        let texts: Vec<_> = shapes
            .iter()
            .filter_map(|s| {
                if let Shape::Text(t) = s {
                    Some((t.galley.text().to_owned(), t.galley.job.sections[0].format.color))
                } else {
                    None
                }
            })
            .collect();
        let check = texts.iter().any(|(text, _)| text == "\u{E73E}");
        let label = texts.iter().find(|(text, _)| text == "Camera").map(|(_, c)| *c);
        (fill, border, label, check)
    }
}

#[test]
fn filter_ladder_and_selected_check() {
    let filter = Chip::new(ChipKind::Filter { selected: false }, Sample::text("Camera"));
    assert_eq!(
        ChipPaint::look(&filter, InteractionState::Rest),
        (color::BgField::VALUE, None, Some(color::TextDim::VALUE), false)
    );
    assert_eq!(
        ChipPaint::look(&filter, InteractionState::Hover),
        (color::BgHover::VALUE, None, Some(color::Text::VALUE), false)
    );
    assert_eq!(ChipPaint::look(&filter, InteractionState::Focused).2, Some(color::Text::VALUE));
    let selected = Chip::new(ChipKind::Filter { selected: true }, Sample::text("Camera"));
    let (fill, _, label, check) = ChipPaint::look(&selected, InteractionState::Pressed);
    assert_eq!((fill, label, check), (color::AccentSoftPress::VALUE, Some(color::AccentHover::VALUE), true));
}

#[test]
fn display_tones_match_the_sheet() {
    let tones = [
        (ChipTone::Neutral, color::BgField::VALUE, color::TextDim::VALUE, None),
        (ChipTone::Accent, color::AccentSoft::VALUE, color::AccentHover::VALUE, None),
        (ChipTone::Warning, color::WarningSoft::VALUE, color::Warning::VALUE, None),
        (ChipTone::Danger, color::DangerSoft::VALUE, color::Danger::VALUE, None),
        (ChipTone::Success, SoftFill::of(color::Success::VALUE), color::Success::VALUE, None),
        (ChipTone::Outline, Color32::TRANSPARENT, color::TextDim::VALUE, Some(color::LineStrong::VALUE)),
    ];
    for (tone, fill, text, border) in tones {
        let chip = Chip::new(ChipKind::Display(tone), Sample::text("Camera"));
        assert_eq!(ChipPaint::look(&chip, InteractionState::Rest), (fill, border, Some(text), false), "{tone:?}");
    }
}

#[test]
fn chip_is_control_small_high() {
    let harness =
        ClickProbe::harness(|ui| Chip::new(ChipKind::Display(ChipTone::Neutral), Sample::text("Camera")).show(ui));
    assert_eq!(harness.state().rect.height(), 20.0);
}

#[test]
fn filter_chip_clicks_and_display_chip_does_not() {
    let mut filter =
        ClickProbe::harness(|ui| Chip::new(ChipKind::Filter { selected: false }, Sample::text("Camera")).show(ui));
    ClickProbe::click(&mut filter);
    assert_eq!(filter.state().clicks, 1);
    let mut display =
        ClickProbe::harness(|ui| Chip::new(ChipKind::Display(ChipTone::Neutral), Sample::text("Camera")).show(ui));
    ClickProbe::click(&mut display);
    assert_eq!(display.state().clicks, 0);
}

#[test]
fn key_chord_joins_caps_with_add_icon() {
    let chord = KeyChord::new(vec![Sample::text("Ctrl"), Sample::text("Shift"), Sample::text("N")]);
    let shapes = PaintCapture::shapes(|painter| {
        chord.paint(painter, Pos2::new(20.0, 40.0));
    });
    let texts: Vec<String> = PaintCapture::texts(&shapes).into_iter().map(|(text, _, _)| text).collect();
    assert_eq!(texts.iter().filter(|text| *text == "\u{E710}").count(), 2);
    assert!(texts.contains(&"Ctrl".to_owned()) && texts.contains(&"N".to_owned()));
    // Regular caps are status-bar high
    let caps: Vec<_> = PaintCapture::rects(&shapes).into_iter().filter(|r| r.fill == color::BgField::VALUE).collect();
    assert_eq!(caps.len(), 3);
    assert!(caps.iter().all(|cap| cap.rect.height() == 26.0 && cap.rect.width() >= 28.0));
}

#[test]
fn conflict_cap_is_danger() {
    let chord = KeyChord::new(vec![Sample::text("Ctrl"), Sample::text("D")]).conflict(true);
    let shapes = PaintCapture::shapes(|painter| {
        chord.paint(painter, Pos2::new(20.0, 40.0));
    });
    let rects = PaintCapture::rects(&shapes);
    assert_eq!(rects.iter().filter(|r| r.fill == color::DangerSoft::VALUE).count(), 2);
    assert_eq!(rects.iter().filter(|r| r.stroke.width > 0.0 && r.stroke.color == color::Danger::VALUE).count(), 2);
}

#[test]
fn small_caps_are_control_small_without_border() {
    let chord = KeyChord::new(vec![Sample::text("Space")]).size(KeyCapSize::Small);
    let shapes = PaintCapture::shapes(|painter| {
        chord.paint(painter, Pos2::new(20.0, 40.0));
    });
    let cap = PaintCapture::rects(&shapes).into_iter().find(|r| r.fill == color::BgField::VALUE);
    assert_eq!(cap.as_ref().map(|c| c.rect.height()), Some(20.0));
    assert_eq!(cap.map(|c| c.stroke.width), Some(0.0));
}

#[test]
fn hud_status_chip_is_taller_and_leads_with_a_dot_before_the_text() {
    let harness = ClickProbe::harness(|ui| {
        let chip = Chip::new(ChipKind::Display(ChipTone::Neutral), Sample::text("Shown"));
        chip.size(ChipSize::Hud).dot(color::Success::VALUE).show(ui)
    });
    assert_eq!(harness.state().rect.height(), 22.0);

    let chip = Chip::new(ChipKind::Display(ChipTone::Neutral), Sample::text("Camera"))
        .size(ChipSize::Hud)
        .dot(color::Success::VALUE)
        .icon(Icon::Lock);
    let rect = Rect::from_min_size(pos2(20.0, 20.0), vec2(100.0, 22.0));
    let shapes = PaintCapture::shapes(|painter| chip.paint(painter, rect, InteractionState::Rest));
    let dot = shapes.iter().find_map(|shape| match shape {
        Shape::Circle(circle) if circle.fill == color::Success::VALUE => Some(*circle),
        _ => None,
    });
    let dot = dot.expect("the dot is painted");
    assert_eq!((dot.radius, dot.center.x), (3.0, 31.0));
    let texts = PaintCapture::text_rects(&shapes);
    assert!(texts.iter().all(|(text, _)| text != "\u{E72E}"), "the dot takes the place of the icon");
    let label = texts.iter().find(|(text, _)| text == "Camera").map(|(_, at)| at.left());
    assert_eq!(label, Some(40.0), "the text starts after the padding, the dot and the HUD gap");
}

#[test]
fn hud_key_cap_is_fixed_size_bordered_and_paints_an_arrow_icon() {
    let cap = KeyCap { label: KeyCapLabel::Icon(Icon::ChevronLeft), size: KeyCapSize::Hud, conflict: false };
    let rect = Rect::from_min_size(pos2(20.0, 20.0), vec2(32.0, 28.0));
    let mut size = vec2(0.0, 0.0);
    let shapes = PaintCapture::shapes(|painter| {
        size = cap.size(painter.ctx());
        cap.paint(painter, rect);
    });
    assert_eq!(size, vec2(32.0, 28.0));
    let texts = PaintCapture::texts(&shapes);
    assert_eq!(texts.len(), 1);
    assert_eq!((texts[0].0.as_str(), texts[0].1), ("\u{E76B}", KeyCapSize::Hud.text().size.points()));
    let rects = PaintCapture::rects(&shapes);
    assert!(rects.iter().any(|r| r.rect == rect && r.fill == color::BgField::VALUE));
    assert!(rects.iter().any(|r| r.stroke.width > 0.0 && r.stroke.color == color::LineStrong::VALUE));
    assert!(
        shapes.iter().all(|shape| !matches!(shape, Shape::Rect(r) if r.fill == color::Shadow::VALUE)),
        "no under-edge"
    );
}

#[test]
fn hud_hint_cap_is_eighteen_high_bordered_with_dim_micro_text() {
    let cap = KeyCap { label: KeyCapLabel::Text(Sample::text("F10")), size: KeyCapSize::HudHint, conflict: true };
    let mut size = vec2(0.0, 0.0);
    let mut label = 0.0;
    let shapes = PaintCapture::shapes(|painter| {
        size = cap.size(painter.ctx());
        label = Draw::measure(painter.ctx(), "F10", KeyCapSize::HudHint.text()).x;
        cap.paint(painter, Rect::from_min_size(pos2(20.0, 20.0), size));
    });
    assert_eq!(size, vec2((label + 10.0).ceil(), 18.0));
    let rects = PaintCapture::rects(&shapes);
    let bordered = rects.iter().any(|r| r.stroke.width > 0.0 && r.stroke.color == color::LineStrong::VALUE);
    assert!(bordered, "a HUD hint cap keeps its look when the key conflicts");
    let colour = shapes.iter().find_map(|shape| match shape {
        Shape::Text(text) => Some(text.galley.job.sections[0].format.color),
        _ => None,
    });
    assert_eq!(colour, Some(color::TextDim::VALUE));
}
