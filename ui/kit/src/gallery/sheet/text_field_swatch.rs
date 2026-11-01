use egui::{Id, Ui, vec2};

use crate::gallery::{GalleryState, Sample, SampleColours, SheetGrid};
use crate::state::InteractionState;
use crate::tokens::{SpacingToken, spacing};
use crate::widget::{FieldLook, TextField, TextFieldKind};

/// Paper: Widgets, `Widget / Text field` (`AOO-0`), the state `Leading swatch`.
pub struct TextFieldSwatchSheet;

impl TextFieldSwatchSheet {
    /// Sheet chrome: the field width.
    const WIDTH: f32 = 280.0;

    pub fn show(ui: &mut Ui, _state: &mut GalleryState) {
        let height = <spacing::Button as SpacingToken>::VALUE;
        SheetGrid::page(ui, "Text field", |ui| {
            SheetGrid::panel(ui, |ui| {
                let size = vec2(Self::WIDTH, height);
                SheetGrid::labelled(
                    ui,
                    "Leading swatch",
                    SheetGrid::LABEL_WIDTH,
                    SheetGrid::COLUMN_GAP,
                    size,
                    |ui, rect| {
                        let mut name = "Arrival".to_owned();
                        let field =
                            TextField::new(Id::NULL, &mut name, TextFieldKind::Plain, Sample::text("Scene name"))
                                .leading_swatch(SampleColours::scene()[0]);
                        field.paint(ui.painter(), rect, InteractionState::Rest, &FieldLook::default());
                    },
                );
            });
        });
    }
}
