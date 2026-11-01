use amml_i18n::text::{Args, Text, UserText};
use amml_ui::AmmlUiMessage;
use amml_ui::font::FontSource;
use amml_ui::theme::Theme;
use amml_ui::tokens::{ColorToken, SpacingToken, color, spacing};
use amml_ui::widget::{TableAlign, TableColumn, TableHeader, TableRow, TableRowKind, TableRowSize};
use egui::{Color32, Rect, Sense, Shape, vec2};
use egui_kittest::Harness;

use crate::support::{PaintCapture, Sample};

// what the table reported: the cells it asked for, each cell's rect, the row rect and its clicks
#[derive(Debug)]
struct TableLog {
    calls: Vec<usize>,
    cells: Vec<Rect>,
    row: Rect,
    clicks: usize,
}

// a header and one row of `kind`; each cell paints its column's name
struct TableProbe;

impl TableProbe {
    const COLUMNS: [TableColumn; 4] = [
        TableColumn { width_fraction: 0.396 },
        TableColumn { width_fraction: 0.270 },
        TableColumn { width_fraction: 0.188 },
        TableColumn { width_fraction: 0.146 },
    ];
    const NAMES: [&'static str; 4] = ["Fly forward", "W", "Input", "While held"];

    fn harness(kind: impl Fn() -> TableRowKind + 'static) -> Harness<'static, TableLog> {
        let mut harness = Harness::builder()
            .with_size(vec2(980.0, 300.0))
            .allow_missing_glyphs()
            .with_context_setup(|ctx| Theme::install(ctx, FontSource::BuiltIn))
            .build_ui_state(
                move |ui, log: &mut TableLog| {
                    let heads = ["ACTION", "KEY CHORD", "CAPABILITY", "SCOPE"];
                    TableHeader::new(
                        heads.iter().zip(Self::COLUMNS).map(|(head, column)| (Sample::text(head), column)).collect(),
                    )
                    .show(ui);
                    log.calls.clear();
                    log.cells.clear();
                    let response = TableRow::new(&Self::COLUMNS, kind()).show(ui, |index, ui| {
                        log.calls.push(index);
                        log.cells.push(ui.max_rect());
                        ui.label(Self::NAMES[index]);
                    });
                    log.row = response.rect;
                    log.clicks += usize::from(response.clicked());
                    // a later widget, so a row that steals the cursor would show
                    ui.allocate_exact_size(vec2(10.0, 10.0), Sense::hover());
                },
                TableLog { calls: Vec::new(), cells: Vec::new(), row: Rect::NOTHING, clicks: 0 },
            );
        harness.run();
        harness
    }

    fn shapes(harness: &Harness<'static, TableLog>) -> Vec<Shape> {
        harness.output().shapes.iter().map(|clipped| clipped.shape.clone()).collect()
    }

    fn text(shapes: &[Shape], text: &str) -> Option<(Rect, Color32)> {
        shapes.iter().find_map(|shape| match shape {
            Shape::Text(shape) if shape.galley.text() == text => {
                Some((shape.galley.rect.translate(shape.pos.to_vec2()), shape.galley.job.sections[0].format.color))
            }
            _ => None,
        })
    }

    // the alpha a text shape is drawn at: egui fades a text's fallback colour and mesh, not its
    // layout colour
    fn alpha(shapes: &[Shape], text: &str) -> Option<u8> {
        shapes.iter().find_map(|shape| match shape {
            Shape::Text(shape) if shape.galley.text() == text => Some(shape.fallback_color.a()),
            _ => None,
        })
    }

    // a box: a `fill` rect and a `border` outline on the same rect
    fn boxed(shapes: &[Shape], fill: Color32, border: Color32) -> Option<Rect> {
        let rects = PaintCapture::rects(shapes);
        let body = rects.iter().find(|r| r.fill == fill)?;
        rects.iter().any(|r| r.rect == body.rect && r.stroke.color == border).then_some(body.rect)
    }

    fn click(harness: &mut Harness<'static, TableLog>) {
        let at = harness.state().row.center();
        harness.hover_at(at);
        harness.run();
        harness.drag_at(at);
        harness.step();
        harness.drop_at(at);
        harness.run();
    }
}

#[test]
fn columns_follow_fractions() {
    let harness = TableProbe::harness(|| TableRowKind::Plain);
    let shapes = TableProbe::shapes(&harness);
    let log = harness.state();
    let heads: Vec<Rect> = ["ACTION", "KEY CHORD", "CAPABILITY", "SCOPE"]
        .iter()
        .map(|head| match TableProbe::text(&shapes, head) {
            Some((rect, colour)) => {
                assert_eq!(colour, color::TextFaint::VALUE, "{head}");
                rect
            }
            None => panic!("no head {head:?}"),
        })
        .collect();
    let inset = heads[0].left() - log.row.left();
    let width = log.row.width() - 2.0 * inset;
    let mut start = 0.0;
    for (index, column) in TableProbe::COLUMNS.iter().enumerate() {
        let expected = heads[0].left() + start * width;
        assert!(
            (heads[index].left() - expected).abs() <= 1.0,
            "head {index} at {}, wanted {expected}",
            heads[index].left()
        );
        assert!((log.cells[index].left() - expected).abs() <= 1.0, "cell {index} does not line up with its head");
        start += column.width_fraction;
    }
}

#[test]
fn capturing_shows_prompt() {
    let harness = TableProbe::harness(|| TableRowKind::Capturing);
    let shapes = TableProbe::shapes(&harness);
    assert_eq!(harness.state().calls, vec![0, 2, 3], "the chord cell was drawn while capturing");
    let Some((press, colour)) = TableProbe::text(&shapes, "Press a key") else { panic!("no prompt") };
    assert_eq!(colour, color::Accent::VALUE);
    let Some((esc, _)) = TableProbe::text(&shapes, "Esc cancels") else { panic!("no cancel hint") };
    assert!(press.bottom() <= esc.top(), "the cancel hint is not under the prompt");
    let Some((chord, _)) = TableProbe::text(&shapes, "KEY CHORD") else { panic!("no chord head") };
    assert!((press.left() - chord.left()).abs() <= 1.0, "the prompt is not in the chord column");
    let boxed = TableProbe::boxed(&shapes, color::AccentSoft::VALUE, color::Accent::VALUE);
    assert!(boxed.is_some(), "no accent box");
}

#[test]
fn conflict_row_is_boxed_danger_with_reason() {
    let harness = TableProbe::harness(|| {
        let name = UserText::from_model("Light Level".to_owned());
        TableRowKind::Conflict(Text::format(AmmlUiMessage::AlsoBoundBy, Args::new().user("name", name)))
    });
    let shapes = TableProbe::shapes(&harness);
    let row = harness.state().row;
    let boxed = TableProbe::boxed(&shapes, color::DangerSoft::VALUE, color::Danger::VALUE);
    let Some(boxed) = boxed else { panic!("no danger box") };
    let margin = <spacing::Xs as SpacingToken>::VALUE;
    assert_eq!((boxed.top() - row.top(), row.bottom() - boxed.bottom()), (margin, margin));
    let Some((reason, colour)) = TableProbe::text(&shapes, "Also bound by Light Level") else { panic!("no reason") };
    assert_eq!(colour, color::Danger::VALUE);
    let Some((action, _)) = TableProbe::text(&shapes, "Fly forward") else { panic!("no action cell") };
    assert!(action.bottom() <= reason.top(), "the reason is not under the action");
}

#[test]
fn disabled_row_is_muted() {
    let mut harness =
        TableProbe::harness(|| TableRowKind::Disabled(Sample::text("Turn on Freecam to change this key")));
    let shapes = TableProbe::shapes(&harness);
    for text in ["Turn on Freecam to change this key", "Fly forward"] {
        let alpha = TableProbe::alpha(&shapes, text);
        assert!(alpha.is_some_and(|alpha| alpha < u8::MAX), "{text:?} is not muted: {alpha:?}");
    }
    TableProbe::click(&mut harness);
    assert_eq!(harness.state().clicks, 0, "a disabled row took a click");
}

#[test]
fn focused_row_shows_the_focus_ring() {
    let mut harness = TableProbe::harness(|| TableRowKind::Plain);
    harness.key_press(egui::Key::Tab);
    harness.run();
    let rects = PaintCapture::rects(&TableProbe::shapes(&harness));
    assert!(rects.iter().any(|r| r.fill == color::FocusRing::VALUE), "a focused row draws no focus ring");
}

#[test]
fn disabled_row_turns_off_its_cell_widgets() {
    // a button in the action cell of a disabled row; the log counts its clicks
    let mut harness = Harness::builder()
        .with_size(vec2(980.0, 300.0))
        .allow_missing_glyphs()
        .with_context_setup(|ctx| Theme::install(ctx, FontSource::BuiltIn))
        .build_ui_state(
            |ui, log: &mut TableLog| {
                let kind = TableRowKind::Disabled(Sample::text("Turn on Freecam to change this key"));
                TableRow::new(&TableProbe::COLUMNS, kind).show(ui, |index, ui| {
                    if index == 0 {
                        let button = ui.button("Rebind");
                        log.row = button.rect;
                        log.clicks += usize::from(button.clicked());
                    }
                });
            },
            TableLog { calls: Vec::new(), cells: Vec::new(), row: Rect::NOTHING, clicks: 0 },
        );
    harness.run();
    TableProbe::click(&mut harness);
    assert_eq!(harness.state().clicks, 0, "a button in a disabled row took a click");
}

// a header and one row over two columns, with `aligns` and `size`; the log has the row and its cells
struct NumericProbe;

impl NumericProbe {
    const COLUMNS: [TableColumn; 2] = [TableColumn { width_fraction: 0.5 }, TableColumn { width_fraction: 0.5 }];

    fn harness(aligns: Option<[TableAlign; 2]>, size: TableRowSize) -> Harness<'static, TableLog> {
        let mut harness = Harness::builder()
            .with_size(vec2(980.0, 300.0))
            .allow_missing_glyphs()
            .with_context_setup(|ctx| Theme::install(ctx, FontSource::BuiltIn))
            .build_ui_state(
                move |ui, log: &mut TableLog| {
                    let heads = [Sample::text("ITEM"), Sample::text("COUNT")];
                    let mut header = TableHeader::new(heads.into_iter().zip(Self::COLUMNS).collect());
                    let mut row = TableRow::new(&Self::COLUMNS, TableRowKind::Plain).size(size);
                    let aligns: &[TableAlign] = aligns.as_ref().map_or(&[], |aligns| aligns.as_slice());
                    header = header.aligns(aligns);
                    row = row.aligns(aligns);
                    header.show(ui);
                    log.cells.clear();
                    let response = row.show(ui, |index, ui| {
                        log.cells.push(ui.max_rect());
                        ui.label(["Stone Bricks", "3,412"][index]);
                    });
                    log.row = response.rect;
                },
                TableLog { calls: Vec::new(), cells: Vec::new(), row: Rect::NOTHING, clicks: 0 },
            );
        harness.run();
        harness
    }
}

#[test]
fn compact_row_is_38_high() {
    let harness = NumericProbe::harness(None, TableRowSize::Compact);
    assert_eq!(harness.state().row.height(), 38.0);
}

#[test]
fn default_row_is_unchanged() {
    let harness = NumericProbe::harness(None, TableRowSize::default());
    assert_eq!(harness.state().row.height(), 68.0);
    let plain = TableProbe::harness(|| TableRowKind::Plain);
    assert_eq!(plain.state().row.height(), 68.0);
}

#[test]
fn end_aligned_header_is_right_aligned() {
    let harness = NumericProbe::harness(Some([TableAlign::Start, TableAlign::End]), TableRowSize::Compact);
    let shapes = TableProbe::shapes(&harness);
    let log = harness.state();
    let Some((head, _)) = TableProbe::text(&shapes, "COUNT") else { panic!("no head") };
    let column = log.cells[1];
    assert!(
        (head.right() - column.right()).abs() <= 1.0,
        "head ends at {}, column at {}",
        head.right(),
        column.right()
    );
    let Some((cell, _)) = TableProbe::text(&shapes, "3,412") else { panic!("no cell text") };
    assert!(
        (cell.right() - column.right()).abs() <= 1.0,
        "cell ends at {}, column at {}",
        cell.right(),
        column.right()
    );
}

#[test]
fn start_alignment_is_the_default() {
    let harness = NumericProbe::harness(None, TableRowSize::Compact);
    let shapes = TableProbe::shapes(&harness);
    let column = harness.state().cells[1];
    let Some((head, _)) = TableProbe::text(&shapes, "COUNT") else { panic!("no head") };
    assert!((head.left() - column.left()).abs() <= 1.0, "head starts at {}, column at {}", head.left(), column.left());
}
