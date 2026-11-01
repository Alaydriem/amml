use amml_ui::font::FontSource;
use amml_ui::state::DisabledReason;
use amml_ui::theme::Theme;
use amml_ui::tokens::{SpacingToken, spacing};
use amml_ui::widget::{GridCellRef, GridPlacer, GridPlacerResponse};
use egui::{Key, Pos2, Rect, Shape, pos2, vec2};
use egui_kittest::Harness;

use crate::support::Sample;

// one piece of the board the test keeps, as a caller would
struct BoardPiece {
    key: u64,
    name: &'static str,
    cell: Option<GridCellRef>,
}

// the caller's model: the grid size, the pieces and every event the placer returned; `other` is a
// second placer shown under the first
struct Board {
    enabled: bool,
    columns: u16,
    rows: u16,
    pieces: Vec<BoardPiece>,
    events: Vec<GridPlacerResponse>,
    other: Option<Box<Board>>,
}

// shows a 3 x 2 placer over a `Board` each pass and applies its events to the board
struct PlacerProbe;

impl PlacerProbe {
    const COLUMNS: u16 = 3;
    const ROWS: u16 = 2;
    // pointer steps between press and release, so egui sees a drag and not a click
    const DRAG_STEPS: usize = 4;

    fn harness(pieces: Vec<BoardPiece>) -> Harness<'static, Board> {
        Self::harness_with(pieces, true)
    }

    fn harness_with(pieces: Vec<BoardPiece>, enabled: bool) -> Harness<'static, Board> {
        Self::harness_for(Self::board(Self::COLUMNS, Self::ROWS, pieces, enabled))
    }

    fn board(columns: u16, rows: u16, pieces: Vec<BoardPiece>, enabled: bool) -> Board {
        Board { enabled, columns, rows, pieces, events: Vec::new(), other: None }
    }

    fn harness_for(board: Board) -> Harness<'static, Board> {
        let mut harness = Harness::builder()
            .with_size(vec2(800.0, 900.0))
            .allow_missing_glyphs()
            .with_context_setup(|ctx| Theme::install(ctx, FontSource::BuiltIn))
            .build_ui_state(
                |ui, board: &mut Board| {
                    Self::show(ui, 17, board);
                    if let Some(other) = board.other.as_deref_mut() {
                        Self::show(ui, 18, other);
                    }
                },
                board,
            );
        harness.run();
        harness
    }

    fn show(ui: &mut egui::Ui, id: u64, board: &mut Board) {
        let mut placer = GridPlacer::new(egui::Id::unique(id), board.columns, board.rows);
        for piece in &board.pieces {
            placer = placer.piece(piece.key, Sample::text(piece.name), piece.cell);
        }
        let placer = placer.enabled(board.enabled, DisabledReason::new(Sample::text("Locked")));
        let response = placer.show(ui);
        Self::apply(board, &response);
        if response != GridPlacerResponse::None {
            board.events.push(response);
        }
    }

    fn apply(board: &mut Board, response: &GridPlacerResponse) {
        match *response {
            GridPlacerResponse::None => {}
            GridPlacerResponse::Placed { key, cell } => {
                if let Some(piece) = board.pieces.iter_mut().find(|piece| piece.key == key) {
                    piece.cell = Some(cell);
                }
            }
            GridPlacerResponse::Unplaced { key } => {
                if let Some(piece) = board.pieces.iter_mut().find(|piece| piece.key == key) {
                    piece.cell = None;
                }
            }
        }
    }

    fn piece(key: u64, name: &'static str, cell: Option<GridCellRef>) -> BoardPiece {
        BoardPiece { key, name, cell }
    }

    fn texts(harness: &Harness<'static, Board>) -> Vec<String> {
        harness
            .output()
            .shapes
            .iter()
            .filter_map(|clipped| match &clipped.shape {
                Shape::Text(text) => Some(text.galley.text().to_owned()),
                _ => None,
            })
            .collect()
    }

    // the rects of every painted text equal to `text`, in paint order
    fn text_rects(harness: &Harness<'static, Board>, text: &str) -> Vec<Rect> {
        harness
            .output()
            .shapes
            .iter()
            .filter_map(|clipped| match &clipped.shape {
                Shape::Text(shape) if shape.galley.text() == text => {
                    Some(shape.galley.rect.translate(shape.pos.to_vec2()))
                }
                _ => None,
            })
            .collect()
    }

    // the rect of the first painted text equal to `text`
    fn text_rect(harness: &Harness<'static, Board>, text: &str) -> Rect {
        let found = harness.output().shapes.iter().find_map(|clipped| match &clipped.shape {
            Shape::Text(shape) if shape.galley.text() == text => Some(shape.galley.rect.translate(shape.pos.to_vec2())),
            _ => None,
        });
        found.unwrap_or_else(|| panic!("{text:?} was not painted; painted: {:?}", Self::texts(harness)))
    }

    // press at `from`, move to `to` over a few passes, and hold there
    fn drag_to(harness: &mut Harness<'static, Board>, from: Pos2, to: Pos2) {
        harness.hover_at(from);
        harness.run();
        harness.drag_at(from);
        harness.step();
        for step in 1..=Self::DRAG_STEPS {
            let at = from + (to - from) * (step as f32 / Self::DRAG_STEPS as f32);
            harness.hover_at(at);
            harness.step();
        }
    }

    fn release_at(harness: &mut Harness<'static, Board>, at: Pos2) {
        harness.drop_at(at);
        harness.step();
        harness.run();
    }

    fn press(harness: &mut Harness<'static, Board>, key: Key) {
        harness.key_press(key);
        harness.run();
    }

    fn cell(column: u16, row: u16) -> GridCellRef {
        GridCellRef { column, row }
    }
}

#[test]
fn drag_chip_to_empty_cell_places_it() {
    let mut harness = PlacerProbe::harness(vec![PlacerProbe::piece(7, "Harbour east", None)]);
    let chip = PlacerProbe::text_rect(&harness, "Harbour east").center();
    let target = PlacerProbe::text_rect(&harness, "Cell 2").center();
    PlacerProbe::drag_to(&mut harness, chip, target);
    assert!(
        PlacerProbe::texts(&harness).contains(&"Drop here".to_owned()),
        "the empty cell under the chip is a drop target"
    );
    PlacerProbe::release_at(&mut harness, target);
    assert_eq!(harness.state().events, vec![GridPlacerResponse::Placed { key: 7, cell: PlacerProbe::cell(1, 0) }]);
    assert!(PlacerProbe::texts(&harness).contains(&"NOT PLACED · 0".to_owned()));
}

#[test]
fn drop_on_taken_cell_is_refused() {
    let mut harness = PlacerProbe::harness(vec![
        PlacerProbe::piece(1, "Harbour west", Some(PlacerProbe::cell(0, 0))),
        PlacerProbe::piece(2, "Harbour east", None),
    ]);
    let chip = PlacerProbe::text_rect(&harness, "Harbour east").center();
    let taken = PlacerProbe::text_rect(&harness, "Harbour west").center();
    PlacerProbe::drag_to(&mut harness, chip, taken);
    assert!(PlacerProbe::texts(&harness).contains(&"Taken".to_owned()), "the filled cell shows the blocked state");
    PlacerProbe::release_at(&mut harness, taken);
    assert!(harness.state().events.is_empty(), "{:?}", harness.state().events);
    assert_eq!(harness.state().pieces[1].cell, None);
    assert!(!PlacerProbe::texts(&harness).contains(&"Taken".to_owned()), "the blocked state ends with the drag");
}

#[test]
fn drag_placed_piece_to_empty_cell_moves_it() {
    let mut harness = PlacerProbe::harness(vec![PlacerProbe::piece(4, "Harbour west", Some(PlacerProbe::cell(0, 0)))]);
    let from = PlacerProbe::text_rect(&harness, "Harbour west").center();
    let target = PlacerProbe::text_rect(&harness, "Cell 6").center();
    PlacerProbe::drag_to(&mut harness, from, target);
    PlacerProbe::release_at(&mut harness, target);
    assert_eq!(harness.state().events, vec![GridPlacerResponse::Placed { key: 4, cell: PlacerProbe::cell(2, 1) }]);
}

#[test]
fn drag_placed_piece_onto_tray_unplaces_it() {
    let mut harness = PlacerProbe::harness(vec![PlacerProbe::piece(6, "Harbour west", Some(PlacerProbe::cell(2, 0)))]);
    let from = PlacerProbe::text_rect(&harness, "Harbour west").center();
    // the tray starts a header band under the heading text's centre, and is one chip high
    let heading = PlacerProbe::text_rect(&harness, "NOT PLACED · 0");
    let band = <spacing::Row as SpacingToken>::VALUE;
    let tray = pos2(heading.left(), heading.center().y + band / 2.0 + GridPlacer::chip_height() / 2.0);
    PlacerProbe::drag_to(&mut harness, from, tray);
    PlacerProbe::release_at(&mut harness, tray);
    assert_eq!(harness.state().events, vec![GridPlacerResponse::Unplaced { key: 6 }]);
    assert!(PlacerProbe::texts(&harness).contains(&"NOT PLACED · 1".to_owned()));
}

#[test]
fn unplace_returns_piece_to_tray() {
    let mut harness = PlacerProbe::harness(vec![PlacerProbe::piece(3, "Harbour west", Some(PlacerProbe::cell(1, 1)))]);
    assert!(PlacerProbe::texts(&harness).contains(&"NOT PLACED · 0".to_owned()));
    let cell = PlacerProbe::text_rect(&harness, "Harbour west").center();
    harness.hover_at(cell);
    harness.run();
    // the unplace button shows on the hovered filled cell
    let close = PlacerProbe::text_rect(&harness, "\u{E711}").center();
    harness.hover_at(close);
    harness.run();
    harness.drag_at(close);
    harness.step();
    harness.drop_at(close);
    harness.run();
    assert_eq!(harness.state().events, vec![GridPlacerResponse::Unplaced { key: 3 }]);
    assert!(PlacerProbe::texts(&harness).contains(&"NOT PLACED · 1".to_owned()));
}

#[test]
fn delete_on_focused_filled_cell_unplaces() {
    for key in [Key::Delete, Key::Backspace] {
        let mut harness =
            PlacerProbe::harness(vec![PlacerProbe::piece(5, "Harbour west", Some(PlacerProbe::cell(0, 0)))]);
        // the tray is empty, so Tab reaches the grid's focus cell first
        PlacerProbe::press(&mut harness, Key::Tab);
        PlacerProbe::press(&mut harness, key);
        assert_eq!(harness.state().events, vec![GridPlacerResponse::Unplaced { key: 5 }], "{key:?}");
    }
}

#[test]
fn keyboard_route_places_a_piece() {
    let mut harness = PlacerProbe::harness(vec![PlacerProbe::piece(9, "Harbour east", None)]);
    // Tab focuses the chip, Enter picks it up and moves the focus to the grid
    PlacerProbe::press(&mut harness, Key::Tab);
    PlacerProbe::press(&mut harness, Key::Enter);
    assert!(PlacerProbe::texts(&harness).contains(&"Place here".to_owned()), "the focus cell offers the piece");
    PlacerProbe::press(&mut harness, Key::ArrowRight);
    PlacerProbe::press(&mut harness, Key::ArrowDown);
    PlacerProbe::press(&mut harness, Key::Enter);
    assert_eq!(harness.state().events, vec![GridPlacerResponse::Placed { key: 9, cell: PlacerProbe::cell(1, 1) }]);
}

#[test]
fn keyboard_focus_stays_inside_the_grid() {
    let mut harness = PlacerProbe::harness(vec![PlacerProbe::piece(9, "Harbour east", None)]);
    PlacerProbe::press(&mut harness, Key::Tab);
    PlacerProbe::press(&mut harness, Key::Enter);
    for key in [Key::ArrowLeft, Key::ArrowUp, Key::ArrowRight, Key::ArrowRight, Key::ArrowRight, Key::ArrowRight] {
        PlacerProbe::press(&mut harness, key);
    }
    PlacerProbe::press(&mut harness, Key::Enter);
    assert_eq!(harness.state().events, vec![GridPlacerResponse::Placed { key: 9, cell: PlacerProbe::cell(2, 0) }]);
}

#[test]
fn escape_lets_go_of_a_held_piece() {
    let mut harness = PlacerProbe::harness(vec![PlacerProbe::piece(9, "Harbour east", None)]);
    PlacerProbe::press(&mut harness, Key::Tab);
    PlacerProbe::press(&mut harness, Key::Enter);
    PlacerProbe::press(&mut harness, Key::Escape);
    assert!(!PlacerProbe::texts(&harness).contains(&"Place here".to_owned()));
    PlacerProbe::press(&mut harness, Key::Enter);
    PlacerProbe::press(&mut harness, Key::Enter);
    // Escape gave the focus back to the chip, so Enter picks it up again and places it in the first cell
    assert_eq!(harness.state().events, vec![GridPlacerResponse::Placed { key: 9, cell: PlacerProbe::cell(0, 0) }]);
}

#[test]
fn disabled_placer_takes_no_drop() {
    let mut harness = PlacerProbe::harness_with(vec![PlacerProbe::piece(7, "Harbour east", None)], false);
    let chip = PlacerProbe::text_rect(&harness, "Harbour east").center();
    let target = PlacerProbe::text_rect(&harness, "Cell 2").center();
    PlacerProbe::drag_to(&mut harness, chip, target);
    PlacerProbe::release_at(&mut harness, target);
    PlacerProbe::press(&mut harness, Key::Tab);
    PlacerProbe::press(&mut harness, Key::Enter);
    assert!(harness.state().events.is_empty(), "{:?}", harness.state().events);
}

#[test]
fn grid_is_capped_at_32() {
    let harness = PlacerProbe::harness_for(PlacerProbe::board(40, 1, Vec::new(), true));
    let texts = PlacerProbe::texts(&harness);
    assert!(texts.contains(&"Cell 32".to_owned()), "the 32nd column is shown");
    assert!(!texts.contains(&"Cell 33".to_owned()), "no column past 32 is shown");
    let placer = GridPlacer::new(egui::Id::NULL, 40, 1000);
    assert_eq!((placer.columns(), placer.rows()), (32, 32));
    let placer = GridPlacer::new(egui::Id::NULL, 0, 0);
    assert_eq!((placer.columns(), placer.rows()), (1, 1));
    let placer = GridPlacer::new(egui::Id::NULL, 32, 5);
    assert_eq!((placer.columns(), placer.rows()), (32, 5));
}

#[test]
fn piece_outside_the_grid_waits_in_the_tray() {
    let harness = PlacerProbe::harness(vec![PlacerProbe::piece(1, "Harbour west", Some(PlacerProbe::cell(5, 0)))]);
    assert!(PlacerProbe::texts(&harness).contains(&"NOT PLACED · 1".to_owned()));
}

#[test]
fn release_of_placed_piece_outside_the_placer_returns_it_to_tray() {
    let mut harness = PlacerProbe::harness(vec![PlacerProbe::piece(8, "Harbour west", Some(PlacerProbe::cell(1, 0)))]);
    let from = PlacerProbe::text_rect(&harness, "Harbour west").center();
    // the 3 x 2 placer ends far above the bottom of the 900 high window
    let outside = pos2(from.x, 850.0);
    PlacerProbe::drag_to(&mut harness, from, outside);
    PlacerProbe::release_at(&mut harness, outside);
    assert_eq!(harness.state().events, vec![GridPlacerResponse::Unplaced { key: 8 }]);
    assert!(PlacerProbe::texts(&harness).contains(&"NOT PLACED · 1".to_owned()));
}

#[test]
fn release_of_tray_piece_outside_the_placer_changes_nothing() {
    let mut harness = PlacerProbe::harness(vec![PlacerProbe::piece(8, "Harbour east", None)]);
    let from = PlacerProbe::text_rect(&harness, "Harbour east").center();
    let outside = pos2(from.x, 850.0);
    PlacerProbe::drag_to(&mut harness, from, outside);
    PlacerProbe::release_at(&mut harness, outside);
    assert!(harness.state().events.is_empty(), "{:?}", harness.state().events);
}

#[test]
fn piece_of_one_placer_is_not_taken_by_another() {
    let mut first = PlacerProbe::board(3, 2, vec![PlacerProbe::piece(1, "Harbour east", None)], true);
    first.other = Some(Box::new(PlacerProbe::board(3, 2, Vec::new(), true)));
    let mut harness = PlacerProbe::harness_for(first);
    let chip = PlacerProbe::text_rect(&harness, "Harbour east").center();
    // the second placer paints its cells after the first, so its "Cell 2" is the last one
    let cells = PlacerProbe::text_rects(&harness, "Cell 2");
    assert_eq!(cells.len(), 2);
    let target = cells[1].center();
    PlacerProbe::drag_to(&mut harness, chip, target);
    assert!(!PlacerProbe::texts(&harness).contains(&"Drop here".to_owned()), "the other placer offers no drop");
    PlacerProbe::release_at(&mut harness, target);
    let other = harness.state().other.as_deref().map(|board| board.events.clone());
    assert_eq!(other, Some(Vec::new()));
    assert!(harness.state().events.is_empty(), "{:?}", harness.state().events);
    assert_eq!(harness.state().pieces[0].cell, None);
}

#[test]
fn unbounded_width_keeps_the_narrowest_cells() {
    let mut harness = Harness::builder()
        .with_size(vec2(800.0, 600.0))
        .allow_missing_glyphs()
        .with_context_setup(|ctx| Theme::install(ctx, FontSource::BuiltIn))
        .build_ui(|ui| {
            // a ui with no right edge, as some layouts and scroll contents give
            let unbounded = Rect::from_min_size(ui.min_rect().min, vec2(f32::INFINITY, 400.0));
            ui.scope_builder(egui::UiBuilder::new().max_rect(unbounded), |ui| {
                GridPlacer::new(egui::Id::unique(19), 3, 1).show(ui);
            });
        });
    harness.run();
    let centre = |text: &str| {
        harness.output().shapes.iter().find_map(|clipped| match &clipped.shape {
            Shape::Text(shape) if shape.galley.text() == text => {
                Some(shape.galley.rect.translate(shape.pos.to_vec2()).center().x)
            }
            _ => None,
        })
    };
    let (Some(first), Some(second)) = (centre("Cell 1"), centre("Cell 2")) else {
        panic!("the cells were not painted");
    };
    let step = GridPlacer::min_cell_width() + <spacing::S as SpacingToken>::VALUE;
    assert!((second - first - step).abs() < 0.5, "cells {first} and {second} are not {step} apart");
}
