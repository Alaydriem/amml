use amml_ui::font::FontSource;
use amml_ui::theme::Theme;
use amml_ui::tokens::{ContainerToken, SpacingToken, container, spacing};
use amml_ui::widget::ModalFrame;
use egui::{Id, Key, Pos2, Rect, Sense, pos2, vec2};
use egui_kittest::Harness;

use crate::support::{Sample, ShapeText};

#[derive(Debug, Hash)]
enum ModalTestId {
    Export,
}

// what the modal reported over a harness run
struct ModalLog {
    closed: usize,
    card: Rect,
}

// shows one modal per pass over a 1000 x 700 screen, its body `lines` blocks 20 high
struct ModalProbe;

impl ModalProbe {
    fn harness(lines: usize) -> Harness<'static, ModalLog> {
        Self::harness_wide(lines, None)
    }

    // as `harness`, with the card `width` wide when given
    fn harness_wide(lines: usize, width: Option<f32>) -> Harness<'static, ModalLog> {
        let mut harness = Harness::builder()
            .with_size(vec2(1000.0, 700.0))
            .allow_missing_glyphs()
            .with_context_setup(|ctx| Theme::install(ctx, FontSource::BuiltIn))
            .build_ui_state(
                move |ui, log: &mut ModalLog| {
                    let ctx = ui.ctx().clone();
                    let mut frame = ModalFrame::new(Id::unique(ModalTestId::Export), Sample::text("Export scene"))
                        .subtitle(Sample::text("Arrival"));
                    if let Some(width) = width {
                        frame = frame.width(width);
                    }
                    let response = frame.show(
                        &ctx,
                        |ui| {
                            for _ in 0..lines {
                                ui.allocate_exact_size(vec2(100.0, 20.0), Sense::hover());
                            }
                        },
                        |_| {},
                    );
                    log.closed += usize::from(response.closed);
                    log.card = response.card;
                },
                ModalLog { closed: 0, card: Rect::NOTHING },
            );
        harness.run();
        harness
    }

    fn click_at(harness: &mut Harness<'static, ModalLog>, at: Pos2) {
        harness.hover_at(at);
        harness.run();
        harness.drag_at(at);
        harness.step();
        harness.drop_at(at);
        harness.run();
    }

    // the centre of the close button: `--spacing-row` square in the header's top-right corner
    fn close_button(card: Rect) -> Pos2 {
        let inset = <spacing::L as SpacingToken>::VALUE + <spacing::Row as SpacingToken>::VALUE / 2.0;
        pos2(card.right() - inset, card.top() + inset)
    }
}

#[test]
fn escape_closes() {
    let mut harness = ModalProbe::harness(1);
    assert_eq!(harness.state().closed, 0);
    harness.key_press(Key::Escape);
    harness.run();
    assert!(harness.state().closed > 0);
}

#[test]
fn scrim_click_closes_and_card_click_does_not() {
    let mut harness = ModalProbe::harness(1);
    let card = harness.state().card;
    assert_eq!(card.width(), <container::Dialog as ContainerToken>::VALUE);
    ModalProbe::click_at(&mut harness, pos2(card.center().x, card.bottom() - 4.0));
    assert_eq!(harness.state().closed, 0, "a click in the card closed it");
    ModalProbe::click_at(&mut harness, pos2(10.0, 10.0));
    assert!(harness.state().closed > 0, "a scrim click did not close it");
}

#[test]
fn height_grows_with_body() {
    let short = ModalProbe::harness(1).state().card;
    let tall = ModalProbe::harness(5).state().card;
    // four more blocks, each 20 high plus the `--spacing-m` body gap
    let gap = <spacing::M as SpacingToken>::VALUE;
    assert!((tall.height() - short.height() - 4.0 * (20.0 + gap)).abs() <= 1.0, "{short:?} {tall:?}");
    // centred on the screen
    assert!((tall.center().x - 500.0).abs() <= 1.0 && (tall.center().y - 350.0).abs() <= 1.0, "{tall:?}");
}

#[test]
fn tall_body_keeps_a_lane_from_the_screen_edge() {
    let card = ModalProbe::harness(60).state().card;
    let lane = <spacing::Lane as SpacingToken>::VALUE;
    assert!(card.top() >= lane - 1.0 && card.bottom() <= 700.0 - lane + 1.0, "{card:?}");
}

#[test]
fn wider_card_keeps_a_lane_from_the_screen_edge() {
    assert_eq!(ModalProbe::harness_wide(1, Some(700.0)).state().card.width(), 700.0);
    let lane = <spacing::Lane as SpacingToken>::VALUE;
    assert_eq!(ModalProbe::harness_wide(1, Some(2000.0)).state().card.width(), 1000.0 - lane * 2.0);
}

#[test]
fn close_button_has_close_tooltip() {
    let mut harness = ModalProbe::harness(1);
    let button = ModalProbe::close_button(harness.state().card);
    harness.hover_at(button);
    // the tooltip shows after egui's hover delay
    for _ in 0..30 {
        harness.step();
    }
    let shapes: Vec<_> = harness.output().shapes.iter().map(|clipped| clipped.shape.clone()).collect();
    assert!(ShapeText::all(&shapes).iter().any(|text| text == "Close"), "{:?}", ShapeText::all(&shapes));
    ModalProbe::click_at(&mut harness, button);
    assert!(harness.state().closed > 0, "the close button did not close it");
}
