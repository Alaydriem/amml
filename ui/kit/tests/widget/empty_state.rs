use amml_ui::font::FontSource;
use amml_ui::icon::Icon;
use amml_ui::theme::Theme;
use amml_ui::tokens::{ColorToken, SpacingToken, color, spacing};
use amml_ui::widget::{ButtonKind, EmptyState, TextButton};
use egui::{Rect, Shape, vec2};
use egui_kittest::Harness;

use crate::support::{FlowProbe, PaintCapture, Sample};

// shows one empty state in a `width` x 320 screen and keeps the rect it was given
struct EmptyProbe;

impl EmptyProbe {
    fn harness(width: f32, message: &'static str) -> Harness<'static, Rect> {
        let mut harness = Harness::builder()
            .with_size(vec2(width, 320.0))
            .allow_missing_glyphs()
            .with_context_setup(|ctx| Theme::install(ctx, FontSource::BuiltIn))
            .build_ui_state(
                move |ui, area: &mut Rect| {
                    *area = ui.available_rect_before_wrap();
                    EmptyState::new(Icon::Folder, Sample::text("No replays yet"), Sample::text(message))
                        .action(TextButton::new(ButtonKind::Secondary, Sample::text("Open replays folder")))
                        .show(ui);
                },
                Rect::NOTHING,
            );
        harness.run();
        harness
    }

    fn shapes(harness: &Harness<'static, Rect>) -> Vec<Shape> {
        harness.output().shapes.iter().map(|clipped| clipped.shape.clone()).collect()
    }

    fn text(shapes: &[Shape], text: &str) -> Option<(Rect, usize)> {
        shapes.iter().find_map(|shape| match shape {
            Shape::Text(shape) if shape.galley.text() == text => {
                Some((shape.galley.rect.translate(shape.pos.to_vec2()), shape.galley.rows.len()))
            }
            _ => None,
        })
    }
}

#[test]
fn stack_is_centred() {
    let harness = EmptyProbe::harness(640.0, "Recorded sessions show up here.");
    let area = *harness.state();
    let shapes = EmptyProbe::shapes(&harness);
    let tile = PaintCapture::rects(&shapes).into_iter().find(|r| r.fill == color::BgRaised::VALUE);
    let Some(tile) = tile else { panic!("no icon tile") };
    assert_eq!(tile.rect.size(), vec2(1.0, 1.0) * <spacing::Tile as SpacingToken>::VALUE);
    let button = PaintCapture::rects(&shapes).into_iter().find(|r| r.fill == color::BgField::VALUE);
    let Some(button) = button else { panic!("no secondary action") };
    let Some((title, _)) = EmptyProbe::text(&shapes, "No replays yet") else { panic!("no title") };
    for (part, rect) in [("tile", tile.rect), ("title", title), ("action", button.rect)] {
        assert!((rect.center().x - area.center().x).abs() <= 1.0, "{part} is not centred: {rect:?} in {area:?}");
    }
    // the stack, tile to action, is centred on the height too
    let stack = tile.rect.top()..button.rect.bottom();
    let middle = (stack.start + stack.end) / 2.0;
    assert!((middle - area.center().y).abs() <= 1.0, "stack middle {middle}, area middle {}", area.center().y);
}

#[test]
fn long_message_wraps_inside_width() {
    let message = "Add a camera key to start a scene. Keys mark where the camera is at a time on the timeline.";
    let harness = EmptyProbe::harness(430.0, message);
    let area = *harness.state();
    let Some((rect, rows)) = EmptyProbe::text(&EmptyProbe::shapes(&harness), message) else { panic!("no message") };
    assert!(rows > 1, "the message did not wrap");
    assert!(rect.left() >= area.left() && rect.right() <= area.right(), "{rect:?} is outside {area:?}");
    assert!((rect.center().x - area.center().x).abs() <= 1.0, "the wrapped message is not centred");
}

#[test]
fn card_has_no_icon_tile_and_keeps_its_text_to_the_left() {
    let message = "Click a key on the timeline. Shift-click or drag a box for more. Right-click a lane to add a key. \
        Click a track name for the track inspector.";
    let mut harness = Harness::builder()
        .with_size(vec2(430.0, 320.0))
        .allow_missing_glyphs()
        .with_context_setup(|ctx| Theme::install(ctx, FontSource::BuiltIn))
        .build_ui_state(
            move |ui, area: &mut Rect| {
                *area = ui.available_rect_before_wrap();
                EmptyState::card(Sample::text("No key selected"), Sample::text(message)).show(ui);
            },
            Rect::NOTHING,
        );
    harness.run();
    let area = *harness.state();
    let shapes = EmptyProbe::shapes(&harness);
    let rects = PaintCapture::rects(&shapes);
    let tile = <spacing::Tile as SpacingToken>::VALUE;
    assert!(!rects.iter().any(|r| r.rect.size() == vec2(tile, tile)), "an icon tile was painted");
    let Some(card) = rects.iter().find(|r| r.fill == color::BgRaised::VALUE) else { panic!("no card") };
    assert!(rects.iter().any(|r| r.rect == card.rect && r.stroke.color == color::Line::VALUE), "no card line");
    assert_eq!(card.rect.width(), area.width());
    let Some((title, _)) = EmptyProbe::text(&shapes, "No key selected") else { panic!("no title") };
    let Some((body, rows)) = EmptyProbe::text(&shapes, message) else { panic!("no message") };
    let pad = <spacing::L as SpacingToken>::VALUE + 1.0;
    assert!((title.left() - (card.rect.left() + pad)).abs() <= 1.0, "title {title:?} in card {:?}", card.rect);
    assert!((body.left() - title.left()).abs() <= 1.0, "the message is not left aligned with the title");
    assert!(rows > 1, "the message did not wrap");
    assert!(card.rect.contains_rect(body), "{body:?} leaves the card {:?}", card.rect);
}

#[test]
fn card_does_not_pull_the_next_widget_up() {
    let log = FlowProbe::run(|ui| {
        EmptyState::card(Sample::text("No key selected"), Sample::text("Click a key on the timeline.")).show(ui);
    });
    FlowProbe::assert_below(&log);
}

#[test]
fn action_does_not_pull_the_next_widget_up() {
    let log = FlowProbe::run(|ui| {
        EmptyState::new(Icon::Folder, Sample::text("No replays yet"), Sample::text("Recorded sessions show up here."))
            .action(TextButton::new(ButtonKind::Secondary, Sample::text("Open replays folder")))
            .show(ui);
    });
    FlowProbe::assert_below(&log);
}
