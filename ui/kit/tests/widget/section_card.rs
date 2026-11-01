use amml_i18n::text::{Args, Text, UserText};
use amml_ui::AmmlUiMessage;
use amml_ui::draw::TypeSize;
use amml_ui::font::FontSource;
use amml_ui::theme::Theme;
use amml_ui::tokens::{ColorToken, SpacingToken, color, spacing};
use amml_ui::widget::{SectionCard, SettingControl, SettingRow};
use egui::{Color32, Rect, Shape, Ui, vec2};
use egui_kittest::Harness;

use crate::support::{PaintCapture, Sample};

// shows one section card in a `width` wide screen; keeps what the card took
struct CardProbe;

impl CardProbe {
    fn run(width: f32, mut show: impl FnMut(&mut Ui) -> Rect + 'static) -> (Rect, Vec<Shape>) {
        let mut harness = Harness::builder()
            .with_size(vec2(width, 700.0))
            .allow_missing_glyphs()
            .with_context_setup(|ctx| Theme::install(ctx, FontSource::BuiltIn))
            .build_ui_state(move |ui, card: &mut Rect| *card = show(ui), Rect::NOTHING);
        harness.run();
        let shapes = harness.output().shapes.iter().map(|clipped| clipped.shape.clone()).collect();
        (*harness.state(), shapes)
    }

    // a card of toggle rows: (label, help, changed)
    fn toggles(width: f32, rows: &'static [(&'static str, &'static str, bool)]) -> (Rect, Vec<Shape>) {
        Self::run(width, move |ui| {
            let mut values = vec![true; rows.len()];
            SectionCard::new(Sample::text("MAP"))
                .show(ui, |card| {
                    for ((label, help, changed), on) in rows.iter().zip(values.iter_mut()) {
                        card.row(
                            SettingRow::new(Sample::text(label), SettingControl::Toggle(on))
                                .help(Sample::text(help))
                                .changed(*changed),
                        );
                    }
                })
                .rect
        })
    }

    // (rect, colour) of the first text shape equal to `text`
    fn text(shapes: &[Shape], text: &str) -> Option<(Rect, Color32, usize)> {
        shapes.iter().find_map(|shape| match shape {
            Shape::Text(shape) if shape.galley.text() == text => Some((
                shape.galley.rect.translate(shape.pos.to_vec2()),
                shape.galley.job.sections[0].format.color,
                shape.galley.rows.len(),
            )),
            _ => None,
        })
    }
}

#[test]
fn rows_are_split_by_lines_except_the_last() {
    let rows =
        &[("Show the corner map", "Hide it", false), ("Chunk grid", "Lines", false), ("Shape", "Outline", false)];
    let (card, shapes) = CardProbe::toggles(900.0, rows);
    let lines: Vec<f32> = shapes
        .iter()
        .filter_map(|shape| match shape {
            Shape::LineSegment { points, stroke } if stroke.color == color::Line::VALUE => Some(points[0].y),
            _ => None,
        })
        .collect();
    assert_eq!(lines.len(), 2, "{lines:?}");
    let label = |text: &str| match CardProbe::text(&shapes, text) {
        Some((rect, ..)) => rect,
        None => panic!("no {text:?}"),
    };
    let (first, second, last) = (label("Show the corner map"), label("Chunk grid"), label("Shape"));
    assert!(first.bottom() < lines[0] && lines[0] < second.top(), "line 1 is not between rows 1 and 2");
    assert!(second.bottom() < lines[1] && lines[1] < last.top(), "line 2 is not between rows 2 and 3");
    assert!(lines.iter().all(|y| card.contains(egui::pos2(card.center().x, *y))), "a line is outside the card");
}

#[test]
fn changed_marker_and_note() {
    let (card, shapes) = CardProbe::run(900.0, |ui| {
        let mut zoom = true;
        let mut grid = true;
        let note = Text::format(
            AmmlUiMessage::ChangedInGame,
            Args::new().user("value", UserText::from_model("2.50x".to_owned())),
        );
        SectionCard::new(Sample::text("MAP"))
            .show(ui, |card| {
                card.row(
                    SettingRow::new(Sample::text("Zoom"), SettingControl::Toggle(&mut zoom))
                        .help(Sample::text("Blocks per map pixel"))
                        .changed(true)
                        .note(note),
                );
                card.row(SettingRow::new(Sample::text("Chunk grid"), SettingControl::Toggle(&mut grid)));
            })
            .rect
    });
    let side = <spacing::Indicator as SpacingToken>::VALUE;
    // the toggle tracks are accent too, but `Toggle::TRACK` wide
    let markers: Vec<_> = PaintCapture::rects(&shapes)
        .into_iter()
        .filter(|r| r.fill == color::Accent::VALUE && r.rect.width() == side)
        .collect();
    assert_eq!(markers.len(), 1, "one changed marker: {markers:?}");
    let marker = markers[0].rect;
    let border = <spacing::Border as SpacingToken>::VALUE;
    assert!((marker.left() - (card.left() + border)).abs() <= 0.5, "the marker is not at the card's inner edge");
    let Some((zoom, ..)) = CardProbe::text(&shapes, "Zoom") else { panic!("no label") };
    assert!(marker.top() < zoom.center().y && zoom.center().y < marker.bottom(), "the marker is not on the row");
    let Some((_, colour, _)) = CardProbe::text(&shapes, "Changed in game to 2.50x") else { panic!("no note") };
    assert_eq!(colour, color::Warning::VALUE);
}

#[test]
fn help_wraps_and_row_grows() {
    const LONG: &str = "Chunks scanned around the player each tick, counted out from the chunk the player stands in, in every direction";
    let (short_card, _) = CardProbe::toggles(500.0, &[("Scan radius", "Chunks", false)]);
    let (long_card, shapes) = CardProbe::toggles(500.0, &[("Scan radius", LONG, false)]);
    let Some((help, _, rows)) = CardProbe::text(&shapes, LONG) else { panic!("no help") };
    assert!(rows > 1, "the help does not wrap");
    assert!(help.right() <= long_card.right(), "the help runs out of the card");
    let extra = long_card.height() - short_card.height();
    let line = TypeSize::Help.line_height();
    assert!(extra >= (rows - 1) as f32 * line - 0.5, "the row did not grow with the help: {extra}");
}
