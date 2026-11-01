use amml_i18n::text::{Args, Text, UserText};
use egui::{Align2, Frame, Margin, Rect, Sense, Ui, Vec2, vec2};

use crate::AmmlUiMessage;
use crate::draw::{Corner, Draw, TextFit, TextStyle, TypeSize};
use crate::font::FontFace;
use crate::gallery::{GalleryState, Sample, SheetGrid};
use crate::tokens::{ColorToken, color};
use crate::widget::{
    Chip, ChipKind, ChipTone, KeyChord, TableAlign, TableColumn, TableHeader, TableRow, TableRowKind, TableRowSize,
};

/// Paper: Mod manager, `Widget / Table header + Table row` (`DK4-0`). A keybind table: the header,
/// a mod's group row, and one row of each kind; then a compact table with right-aligned numeric columns.
pub struct TableSheet;

impl TableSheet {
    /// Sheet chrome: the container's width and padding.
    const WIDTH: f32 = 980.0;
    const PAD_TOP: i8 = 16;
    const PAD: i8 = 20;
    /// Paper: Table, the columns as shares of the row's inner width (358, 244, 170 and the rest of 904).
    const COLUMNS: [TableColumn; 4] = [
        TableColumn { width_fraction: 0.396 },
        TableColumn { width_fraction: 0.270 },
        TableColumn { width_fraction: 0.188 },
        TableColumn { width_fraction: 0.146 },
    ];
    /// Paper: Table, the compact table's columns: the item, then the count (110) and the stacks (200)
    /// of 904.
    const NUMERIC_COLUMNS: [TableColumn; 3] = [
        TableColumn { width_fraction: 0.6571 },
        TableColumn { width_fraction: 0.1217 },
        TableColumn { width_fraction: 0.2212 },
    ];
    const NUMERIC_ALIGNS: [TableAlign; 3] = [TableAlign::Start, TableAlign::End, TableAlign::End];
    /// Paper: Table, the gap between the two tables.
    const GAP: f32 = 40.0;
    /// Paper: Table, the group row's height, inset and top padding.
    const GROUP_HEIGHT: f32 = 36.0;
    const GROUP_INSET: f32 = 18.0;
    const GROUP_TOP: f32 = 12.0;

    pub fn show(ui: &mut Ui, _state: &mut GalleryState) {
        SheetGrid::page(ui, "Table header + Table row", |ui| {
            Self::panel(ui, |ui| {
                let heads = ["ACTION", "KEY CHORD", "CAPABILITY", "SCOPE"];
                TableHeader::new(heads.iter().zip(Self::COLUMNS).map(|(head, c)| (Sample::text(head), c)).collect())
                    .show(ui);
                Self::group(ui, "FREECAM");
                Self::row(ui, TableRowKind::Plain, ["Toggle freecam", "Ctrl+P", "Camera", "Global"]);
                Self::row(ui, TableRowKind::Plain, ["Toggle input capture", "Ctrl+E", "Input", "Global"]);
                Self::row(ui, TableRowKind::Capturing, ["Cycle mode", "", "Camera", "Global"]);
                let other = Args::new().user("name", UserText::from_model("Light Level".to_owned()));
                let conflict = TableRowKind::Conflict(Text::format(AmmlUiMessage::AlsoBoundBy, other));
                Self::row(ui, conflict, ["Toggle path markers", "Ctrl+B", "WorldDraw", "Global"]);
                let disabled = TableRowKind::Disabled(Sample::text("Turn on Freecam to change this key"));
                Self::row(ui, disabled, ["Fly forward", "W", "Input", "While held"]);
            });
            ui.add_space(Self::GAP);
            Self::panel(ui, |ui| {
                let heads = ["ITEM", "COUNT", "STACKS"];
                let columns = heads.iter().zip(Self::NUMERIC_COLUMNS).map(|(head, c)| (Sample::text(head), c));
                TableHeader::new(columns.collect()).aligns(&Self::NUMERIC_ALIGNS).show(ui);
                Self::numeric_row(ui, ["Stone Bricks", "3,412", "53 stacks + 20"]);
                Self::numeric_row(ui, ["Spruce Planks", "2,208", "34 stacks + 32"]);
                Self::numeric_row(ui, ["Lantern", "42", "42"]);
            });
        });
    }

    // the sheet's panel around one table
    fn panel(ui: &mut Ui, body: impl FnOnce(&mut Ui)) {
        Frame::new()
            .fill(<color::BgPanel as ColorToken>::VALUE)
            .stroke(Draw::border(<color::Line as ColorToken>::VALUE))
            .corner_radius(Corner::Panel.radius(Rect::NOTHING))
            .inner_margin(Margin { left: Self::PAD, right: Self::PAD, top: Self::PAD_TOP, bottom: Self::PAD })
            .show(ui, |ui| {
                ui.set_width(Self::WIDTH - 2.0 * f32::from(Self::PAD));
                ui.spacing_mut().item_spacing = Vec2::ZERO;
                body(ui);
            });
    }

    // a compact row: the item, then the count and the stacks, right-aligned
    fn numeric_row(ui: &mut Ui, cells: [&str; 3]) {
        let row = TableRow::new(&Self::NUMERIC_COLUMNS, TableRowKind::Plain)
            .size(TableRowSize::Compact)
            .aligns(&Self::NUMERIC_ALIGNS);
        row.show(ui, |index, ui| {
            let (style, colour) = match index {
                2 => (TextStyle::new(TypeSize::Note, FontFace::Regular), <color::TextFaint as ColorToken>::VALUE),
                _ => (TextStyle::new(TypeSize::Label, FontFace::Regular), <color::Text as ColorToken>::VALUE),
            };
            SheetGrid::label(ui, cells[index], style, colour);
        });
    }

    // a mod's group row: its name in small caps
    fn group(ui: &mut Ui, name: &str) {
        let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), Self::GROUP_HEIGHT), Sense::hover());
        let style = TextStyle::new(TypeSize::Caption, FontFace::Semibold).tracked();
        let dim = <color::TextDim as ColorToken>::VALUE;
        let at = egui::pos2(rect.left() + Self::GROUP_INSET, rect.top() + Self::GROUP_TOP);
        Draw::text(ui.painter(), at, Align2::LEFT_TOP, &Sample::text(name), style, dim, TextFit::Line);
    }

    // one row: the action, the chord ("Ctrl+P", keys split at "+"), the capability chip and the scope
    fn row(ui: &mut Ui, kind: TableRowKind, cells: [&str; 4]) {
        let conflict = matches!(kind, TableRowKind::Conflict(_));
        TableRow::new(&Self::COLUMNS, kind).show(ui, |index, ui| match index {
            0 => {
                let style = TextStyle::new(TypeSize::Body, FontFace::Regular);
                SheetGrid::label(ui, cells[0], style, <color::Text as ColorToken>::VALUE);
            }
            1 => {
                let keys = cells[1].split('+').map(Sample::text).collect();
                KeyChord::new(keys).conflict(conflict).show(ui);
            }
            2 => {
                Chip::new(ChipKind::Display(ChipTone::Outline), Sample::text(cells[2])).show(ui);
            }
            _ => {
                let style = TextStyle::new(TypeSize::Help, FontFace::Regular);
                SheetGrid::label(ui, cells[3], style, <color::TextDim as ColorToken>::VALUE);
            }
        });
    }
}
