use amml_ui::font::FontFace;
use amml_ui::icon::Icon;
use amml_ui::state::InteractionState;
use amml_ui::tokens::{ColorToken, color};
use amml_ui::widget::{IconButton, IconButtonSize};
use egui::{Color32, Rect, Shape, pos2, vec2};

use crate::support::{ClickProbe, PaintCapture, Sample, ShapeText};

struct IconPaint;

impl IconPaint {
    const RECT: Rect = Rect::from_min_max(pos2(20.0, 20.0), pos2(48.0, 48.0));

    // (body fill, icon colour)
    fn look(on: bool, state: InteractionState) -> (Color32, Option<Color32>) {
        let shapes =
            PaintCapture::shapes(|painter| IconButton::new(Icon::Loop).on(on).paint(painter, Self::RECT, state));
        let fill = PaintCapture::rects(&shapes)
            .iter()
            .find(|r| r.rect == Self::RECT && r.fill != Color32::TRANSPARENT)
            .map_or(Color32::TRANSPARENT, |r| r.fill);
        let icon = shapes.iter().find_map(|shape| match shape {
            Shape::Text(text) => text.galley.job.sections.first().map(|section| section.format.color),
            _ => None,
        });
        (fill, icon)
    }
}

#[test]
fn plain_and_on_ladders() {
    let dim = <color::TextDim as ColorToken>::VALUE;
    let text = <color::Text as ColorToken>::VALUE;
    let accent_hover = <color::AccentHover as ColorToken>::VALUE;
    let plain = [
        (InteractionState::Rest, Color32::TRANSPARENT, dim),
        (InteractionState::Hover, <color::BgHover as ColorToken>::VALUE, text),
        (InteractionState::Pressed, <color::BgActive as ColorToken>::VALUE, text),
        (InteractionState::Focused, Color32::TRANSPARENT, text),
    ];
    for (state, fill, icon) in plain {
        assert_eq!(IconPaint::look(false, state), (fill, Some(icon)), "plain {state:?}");
    }
    let on = [
        (InteractionState::Rest, <color::AccentSoft as ColorToken>::VALUE),
        (InteractionState::Hover, <color::AccentSoftHover as ColorToken>::VALUE),
        (InteractionState::Pressed, <color::AccentSoftPress as ColorToken>::VALUE),
        (InteractionState::Focused, <color::AccentSoft as ColorToken>::VALUE),
    ];
    for (state, fill) in on {
        assert_eq!(IconPaint::look(true, state), (fill, Some(accent_hover)), "on {state:?}");
    }
}

#[test]
fn small_is_20_with_a_12_icon() {
    let harness = ClickProbe::harness(|ui| IconButton::new(Icon::Close).size(IconButtonSize::Small).show(ui));
    assert_eq!(harness.state().rect.size(), vec2(20.0, 20.0));
    let rect = Rect::from_min_size(pos2(20.0, 20.0), vec2(20.0, 20.0));
    let shapes = PaintCapture::shapes(|painter| {
        IconButton::new(Icon::Close).size(IconButtonSize::Small).paint(painter, rect, InteractionState::Hover);
    });
    let texts = PaintCapture::texts(&shapes);
    assert_eq!(texts.first().map(|(_, size, family)| (*size, family.clone())), Some((12.0, FontFace::Icons.family())));
    let corner = PaintCapture::rects(&shapes).iter().find(|r| r.rect == rect).map(|r| r.corner_radius);
    assert_eq!(corner, Some(egui::CornerRadius::same(4)));
}

#[test]
fn regular_is_row_square() {
    let harness = ClickProbe::harness(|ui| IconButton::new(Icon::Loop).show(ui));
    assert_eq!(harness.state().rect.size(), vec2(28.0, 28.0));
}

#[test]
fn tooltip_text_shows_on_hover() {
    let mut harness =
        ClickProbe::harness(|ui| IconButton::new(Icon::Loop).tooltip(Sample::text("Loop playback")).show(ui));
    harness.ctx.all_styles_mut(|style| style.interaction.tooltip_delay = 0.0);
    let centre = harness.state().rect.center();
    harness.hover_at(centre);
    harness.run();
    let shapes: Vec<Shape> = harness.output().shapes.iter().map(|clipped| clipped.shape.clone()).collect();
    assert!(ShapeText::contains(&shapes, "Loop playback"));
}

#[test]
fn click_clicks() {
    let mut harness = ClickProbe::harness(|ui| IconButton::new(Icon::Loop).show(ui));
    ClickProbe::click(&mut harness);
    assert_eq!(harness.state().clicks, 1);
}
