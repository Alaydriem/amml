use egui::{Rect, Sense, Ui, pos2, vec2};

use crate::draw::{TextStyle, TypeSize};
use crate::font::FontFace;
use crate::gallery::{GalleryId, GalleryState, Sample, SheetGrid};
use crate::tokens::{ColorToken, SpacingToken, color, spacing};
use crate::widget::{GridCellLook, GridCellRef, GridChipLook, GridPlacer, GridPlacerResponse};

/// Paper: Widgets, `Widget / Grid placer`.
pub struct GridPlacerSheet;

impl GridPlacerSheet {
    const COLUMN: f32 = 144.0;
    const CHIP_COLUMN: f32 = 176.0;
    /// The "IN USE" placer: about 168 wide cells, as on the Stratum grid fallback.
    const IN_USE_WIDTH: f32 = 520.0;
    const NAMES: [&'static str; 3] = ["Harbour west", "Harbour middle", "Harbour east"];
    /// The keyboard example's cells, 168 wide as in the "IN USE" placer.
    const KEYS_CELL: f32 = 168.0;
    const ROUTE: [&'static str; 5] = [
        "Tab, Enter: focus a chip, then pick it up.",
        "Arrow keys: move the focus from cell to cell.",
        "Enter: place the piece in the focused cell.",
        "Esc: let go; the piece stays in the tray.",
        "Delete: on a focused filled cell, unplace it.",
    ];
    const CHIPS: [GridChipLook; 5] =
        [GridChipLook::Rest, GridChipLook::Hover, GridChipLook::Dragging, GridChipLook::Focused, GridChipLook::Held];

    pub fn show(ui: &mut Ui, state: &mut GalleryState) {
        let cell_height = GridPlacer::cell_height();
        SheetGrid::page(ui, "Grid placer", |ui| {
            SheetGrid::panel(ui, |ui| {
                SheetGrid::heads(
                    ui,
                    Self::COLUMN,
                    &["EMPTY", "DROP TARGET", "FILLED", "FILLED · HOVER", "BLOCKED", "FOCUS"],
                );
                let west = || Sample::text("Harbour west");
                let cells: [(u32, GridCellLook, bool); 6] = [
                    (4, GridCellLook::Empty, false),
                    (3, GridCellLook::DropTarget, false),
                    (1, GridCellLook::Filled(west()), false),
                    (2, GridCellLook::FilledHover(Sample::text("Harbour middle")), false),
                    (1, GridCellLook::Blocked(west()), false),
                    (5, GridCellLook::Empty, true),
                ];
                SheetGrid::row(ui, "Cell", Self::COLUMN, cell_height, cells.len(), |ui, index, cell| {
                    if let Some((number, look, focused)) = cells.get(index) {
                        let rect = SheetGrid::place(cell, vec2(Self::COLUMN, cell_height));
                        GridPlacer::paint_cell(ui.painter(), rect, *number, look, *focused);
                    }
                });
            });
            SheetGrid::panel(ui, |ui| {
                SheetGrid::heads(ui, Self::CHIP_COLUMN, &["REST", "HOVER", "DRAGGING", "FOCUS", "HELD · KEYS"]);
                let label = Sample::text("Harbour east");
                let size =
                    vec2(GridPlacer::chip_width(ui.ctx(), &label).min(Self::CHIP_COLUMN), GridPlacer::chip_height());
                SheetGrid::row(ui, "Piece chip", Self::CHIP_COLUMN, size.y, Self::CHIPS.len(), |ui, index, cell| {
                    let look = Self::CHIPS[index];
                    GridPlacer::paint_chip(ui.painter(), SheetGrid::place(cell, size), &label, look);
                });
            });
            SheetGrid::section(ui, "IN USE · DRAG A CHIP ONTO A CELL", |ui| {
                SheetGrid::panel(ui, |ui| {
                    ui.scope(|ui| {
                        ui.set_max_width(Self::IN_USE_WIDTH);
                        let mut placer = GridPlacer::new(egui::Id::unique(GalleryId::GridPlacer), 3, 2);
                        for (key, (name, cell)) in Self::NAMES.iter().zip(state.grid_pieces).enumerate() {
                            placer = placer.piece(key as u64, Sample::text(name), cell);
                        }
                        match placer.show(ui) {
                            GridPlacerResponse::None => {}
                            GridPlacerResponse::Placed { key, cell } => Self::set(state, key, Some(cell)),
                            GridPlacerResponse::Unplaced { key } => Self::set(state, key, None),
                        }
                    });
                });
            });
            SheetGrid::section(ui, "KEYBOARD · NO POINTER NEEDED", |ui| {
                SheetGrid::panel(ui, Self::keyboard);
            });
        });
    }

    // the keyboard route, painted: the held chip in the tray, and the focused cell offering the piece
    fn keyboard(ui: &mut Ui) {
        let gap = <spacing::S as SpacingToken>::VALUE;
        let cell_height = GridPlacer::cell_height();
        let label = Sample::text("Harbour east");
        let chip = vec2(GridPlacer::chip_width(ui.ctx(), &label).min(Self::CHIP_COLUMN), GridPlacer::chip_height());
        let grid = vec2(Self::KEYS_CELL * 3.0 + gap * 2.0, cell_height * 2.0 + gap);
        let size = vec2(Self::CHIP_COLUMN + SheetGrid::COLUMN_GAP + grid.x, grid.y);
        let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
        let painter = ui.painter();
        GridPlacer::paint_chip(painter, Rect::from_min_size(rect.min, chip), &label, GridChipLook::Held);
        let left = rect.left() + Self::CHIP_COLUMN + SheetGrid::COLUMN_GAP;
        let cells = [
            GridCellLook::Filled(Sample::text("Harbour west")),
            GridCellLook::Filled(Sample::text("Harbour middle")),
            GridCellLook::PlaceHere,
            GridCellLook::Empty,
            GridCellLook::Empty,
            GridCellLook::Empty,
        ];
        for (index, look) in cells.iter().enumerate() {
            let (column, row) = ((index % 3) as f32, (index / 3) as f32);
            let min = pos2(left + column * (Self::KEYS_CELL + gap), rect.top() + row * (cell_height + gap));
            let cell = Rect::from_min_size(min, vec2(Self::KEYS_CELL, cell_height));
            let focused = matches!(look, GridCellLook::PlaceHere);
            GridPlacer::paint_cell(painter, cell, (index + 1) as u32, look, focused);
        }
        SheetGrid::head(ui, "KEYBOARD ROUTE");
        let style = TextStyle::new(TypeSize::Note, FontFace::Regular);
        for line in Self::ROUTE {
            SheetGrid::paragraph(ui, line, style, <color::TextDim as ColorToken>::VALUE);
        }
    }

    fn set(state: &mut GalleryState, key: u64, cell: Option<GridCellRef>) {
        if let Some(slot) = usize::try_from(key).ok().and_then(|index| state.grid_pieces.get_mut(index)) {
            *slot = cell;
        }
    }
}
