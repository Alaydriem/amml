use amml_i18n::text::Text;
use amml_ui::AmmlUiMessage;
use amml_ui::state::{DisabledReason, InteractionState};
use amml_ui::text::UiLocale;
use amml_ui::tokens::{ColorToken, OpacityToken, color, opacity};
use amml_ui::widget::{ButtonKind, Leading, TextButton};
use egui::{Color32, Context, Key, Rect, Shape, pos2, vec2};

use crate::support::{ClickProbe, PackFixture, PaintCapture, Sample};

struct ButtonPaint;

impl ButtonPaint {
    const RECT: Rect = Rect::from_min_max(pos2(20.0, 20.0), pos2(180.0, 56.0));

    // the fill of the button body (the rect shape that is exactly the button), or none
    fn fill(kind: ButtonKind, state: InteractionState) -> (Color32, Option<Color32>) {
        let shapes = PaintCapture::shapes(|painter| {
            TextButton::new(kind, Sample::text("Render now")).paint(painter, Self::RECT, state);
        });
        let rects = PaintCapture::rects(&shapes);
        let fill = rects.iter().find(|r| r.rect == Self::RECT && r.fill != Color32::TRANSPARENT).map(|r| r.fill);
        let border = rects.iter().find(|r| r.rect == Self::RECT && r.stroke.width > 0.0).map(|r| r.stroke.color);
        (fill.unwrap_or(Color32::TRANSPARENT), border)
    }
}

#[test]
fn each_kind_paints_its_ladder() {
    let table = [
        (
            ButtonKind::Primary,
            [
                <color::Accent as ColorToken>::VALUE,
                <color::AccentHover as ColorToken>::VALUE,
                <color::AccentPress as ColorToken>::VALUE,
            ],
            Some(<color::LineOverlay as ColorToken>::VALUE),
        ),
        (
            ButtonKind::Secondary,
            [
                <color::BgField as ColorToken>::VALUE,
                <color::BgHover as ColorToken>::VALUE,
                <color::BgActive as ColorToken>::VALUE,
            ],
            None,
        ),
        (
            ButtonKind::Ghost,
            [Color32::TRANSPARENT, <color::BgHover as ColorToken>::VALUE, <color::BgActive as ColorToken>::VALUE],
            None,
        ),
        (
            ButtonKind::Outline,
            [Color32::TRANSPARENT, <color::BgHover as ColorToken>::VALUE, <color::BgActive as ColorToken>::VALUE],
            Some(<color::LineStrong as ColorToken>::VALUE),
        ),
        (
            ButtonKind::Danger,
            [
                <color::Danger as ColorToken>::VALUE,
                <color::DangerHover as ColorToken>::VALUE,
                <color::DangerPress as ColorToken>::VALUE,
            ],
            Some(<color::LineOverlay as ColorToken>::VALUE),
        ),
    ];
    let states = [InteractionState::Rest, InteractionState::Hover, InteractionState::Pressed];
    for (kind, fills, border) in table {
        for (state, expected) in states.into_iter().zip(fills) {
            let (fill, painted_border) = ButtonPaint::fill(kind, state);
            assert_eq!(fill, expected, "{kind:?} {state:?}");
            assert_eq!(painted_border, border, "{kind:?} {state:?}");
        }
    }
}

#[test]
fn focus_paints_the_ring_and_the_rest_fill() {
    let shapes = PaintCapture::shapes(|painter| {
        TextButton::new(ButtonKind::Primary, Sample::text("Render now")).paint(
            painter,
            ButtonPaint::RECT,
            InteractionState::Focused,
        );
    });
    let rects = PaintCapture::rects(&shapes);
    assert!(
        rects
            .iter()
            .any(|r| r.fill == <color::FocusRing as ColorToken>::VALUE && r.rect == ButtonPaint::RECT.expand(4.0))
    );
    assert_eq!(
        ButtonPaint::fill(ButtonKind::Primary, InteractionState::Focused).0,
        <color::Accent as ColorToken>::VALUE
    );
}

#[test]
fn disabled_fill_is_muted() {
    let (fill, _) = ButtonPaint::fill(ButtonKind::Primary, InteractionState::Disabled);
    let muted = (255.0 * <opacity::Muted as OpacityToken>::VALUE).round() as u8;
    assert_eq!(fill.a(), muted);
}

#[test]
fn disabled_button_never_clicks() {
    let mut harness = ClickProbe::harness(|ui| {
        TextButton::new(ButtonKind::Primary, Sample::text("Render now"))
            .enabled(false, DisabledReason::new(Sample::text("Select a scene first")))
            .show(ui)
    });
    ClickProbe::click(&mut harness);
    ClickProbe::tab_and_press(&mut harness, Key::Space);
    ClickProbe::tab_and_press(&mut harness, Key::Enter);
    assert_eq!(harness.state().clicks, 0);
}

#[test]
fn pointer_click_clicks() {
    let mut harness =
        ClickProbe::harness(|ui| TextButton::new(ButtonKind::Secondary, Sample::text("Add to queue")).show(ui));
    ClickProbe::click(&mut harness);
    assert_eq!(harness.state().clicks, 1);
}

#[test]
fn space_activates_a_focused_button() {
    let mut harness =
        ClickProbe::harness(|ui| TextButton::new(ButtonKind::Secondary, Sample::text("Add to queue")).show(ui));
    ClickProbe::tab_and_press(&mut harness, Key::Space);
    assert_eq!(harness.state().clicks, 1);
}

#[test]
fn button_is_button_height() {
    let harness = ClickProbe::harness(|ui| TextButton::new(ButtonKind::Primary, Sample::text("Render now")).show(ui));
    assert_eq!(harness.state().rect.height(), 36.0);
}

#[test]
fn width_follows_the_resolved_text() {
    let english = Context::default();
    let pseudo = Context::default();
    UiLocale::set(&pseudo, PackFixture::localizer(&[("Close", "Çľöšé Çľöšé Çľöšé")]));
    let button = TextButton::new(ButtonKind::Secondary, Text::message(AmmlUiMessage::Close));
    let mut widths = [0.0; 2];
    for (index, ctx) in [&english, &pseudo].into_iter().enumerate() {
        PaintCapture::shapes_in(ctx, 0.0, |painter| widths[index] = button.desired_width(painter.ctx()));
    }
    assert!(widths[1] > widths[0] + 50.0, "{widths:?}");
}

#[test]
fn long_text_is_cut_inside_the_width() {
    let rect = Rect::from_min_size(pos2(20.0, 20.0), vec2(80.0, 36.0));
    let long = Sample::text("Render every scene of this replay now");
    let shapes = PaintCapture::shapes(|painter| {
        TextButton::new(ButtonKind::Primary, long.clone()).paint(painter, rect, InteractionState::Rest);
    });
    for shape in &shapes {
        if let Shape::Text(text) = shape {
            let painted = text.galley.rect.translate(text.pos.to_vec2());
            assert!(rect.expand(0.5).contains_rect(painted), "{painted:?} is outside {rect:?}");
        }
    }
}

#[test]
fn spinner_leading_turns() {
    let arc_start = |time: f64| {
        let ctx = Context::default();
        let shapes = PaintCapture::shapes_in(&ctx, time, |painter| {
            TextButton::new(ButtonKind::Primary, Sample::text("Rendering")).leading(Leading::Spinner).paint(
                painter,
                ButtonPaint::RECT,
                InteractionState::Rest,
            );
        });
        shapes.iter().find_map(|shape| match shape {
            Shape::Path(path) => path.points.first().copied(),
            _ => None,
        })
    };
    let (start, later) = (arc_start(0.0), arc_start(0.3));
    assert!(start.is_some() && later.is_some());
    assert_ne!(start, later);
}
