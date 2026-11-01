use egui::{Id, Ui, vec2};

use crate::draw::TypeSize;
use crate::gallery::{GalleryState, Sample, SheetGrid};
use crate::state::InteractionState;
use crate::tokens::{SpacingToken, spacing};
use crate::widget::{FieldLook, TextField, TextFieldKind};

/// Paper: Widgets, `Widget / Text field` (`AOO-0`). Rows are states, columns are the two kinds.
pub struct TextFieldSheet;

impl TextFieldSheet {
    const COLUMN: f32 = 280.0;

    // the sample tables are tuples of Paper's cell values, read once in this function
    #[allow(clippy::type_complexity)]
    pub fn show(ui: &mut Ui, _state: &mut GalleryState) {
        let height = <spacing::Field as SpacingToken>::VALUE;
        // (row, state, plain value, search value, typed selection, invalid reason)
        let rows: [(&str, InteractionState, &str, Option<&str>, Option<(usize, usize)>, Option<&str>); 7] = [
            ("REST", InteractionState::Rest, "Sunset pan", Some("nether"), None, None),
            ("HOVER", InteractionState::Hover, "Sunset pan", Some("nether"), None, None),
            ("FOCUS", InteractionState::Focused, "Sunset pan", Some("nether"), Some((10, 6)), None),
            ("TYPING", InteractionState::Focused, "Sunset pan", Some("nether fort"), Some((7, 7)), None),
            ("PLACEHOLDER", InteractionState::Rest, "", Some(""), None, None),
            (
                "INVALID",
                InteractionState::Rest,
                "Sunset pan",
                None,
                None,
                Some("A scene with this name already exists"),
            ),
            ("DISABLED", InteractionState::Disabled, "Sunset pan", Some(""), None, None),
        ];
        SheetGrid::page(ui, "Text field", |ui| {
            SheetGrid::panel(ui, |ui| {
                SheetGrid::heads(ui, Self::COLUMN, &["FIELD", "SEARCH"]);
                for (label, state, plain, search, typed, invalid) in rows {
                    // an invalid field's reason line sits `--spacing-s` under it
                    let reason = <spacing::S as SpacingToken>::VALUE + TypeSize::Micro.line_height();
                    let row_height = height + if invalid.is_some() { reason } else { 0.0 };
                    SheetGrid::row(ui, label, Self::COLUMN, row_height, 2, |ui, index, cell| {
                        let (value, kind, placeholder, caret_at) = match index {
                            0 => (plain, TextFieldKind::Plain, "Scene name", typed.map(|(start, _)| start)),
                            _ => match search {
                                Some(value) => {
                                    (value, TextFieldKind::Search, "Search replays", typed.map(|(_, start)| start))
                                }
                                None => return,
                            },
                        };
                        let mut value = value.to_owned();
                        let end = value.chars().count();
                        let look = FieldLook {
                            selection: caret_at.map(|start| start.min(end)..end),
                            invalid: invalid.map(Sample::text),
                        };
                        let rect = egui::Rect::from_min_size(cell.min, vec2(Self::COLUMN, height));
                        TextField::new(Id::NULL, &mut value, kind, Sample::text(placeholder)).paint(
                            ui.painter(),
                            rect,
                            state,
                            &look,
                        );
                    });
                }
            });
        });
    }
}
