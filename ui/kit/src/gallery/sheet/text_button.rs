use egui::{Ui, vec2};

use crate::gallery::{GalleryState, Sample, SampleDiamond, SheetGrid};
use crate::icon::Icon;
use crate::state::InteractionState;
use crate::tokens::{ColorToken, SpacingToken, color, spacing};
use crate::widget::{ButtonKind, Leading, TextButton};

/// Paper: Widgets, `Widget / Text button` (`AAT-0`).
pub struct TextButtonSheet;

impl TextButtonSheet {
    const COLUMN: f32 = 180.0;
    const STATES: [InteractionState; 5] = [
        InteractionState::Rest,
        InteractionState::Hover,
        InteractionState::Pressed,
        InteractionState::Focused,
        InteractionState::Disabled,
    ];

    pub fn show(ui: &mut Ui, _state: &mut GalleryState) {
        let rows = [
            ("Primary", ButtonKind::Primary, "Render now", Leading::None),
            ("Secondary", ButtonKind::Secondary, "Add to queue", Leading::Icon(Icon::Add)),
            ("Ghost", ButtonKind::Ghost, "Open queue", Leading::None),
            ("Outline", ButtonKind::Outline, "Apply to keys", Leading::None),
            ("Danger", ButtonKind::Danger, "Delete track", Leading::None),
        ];
        // the studio key diamond stands in the track camera colour; amml-ui has the nearest token
        let diamond = SampleDiamond;
        let leading = [
            (ButtonKind::Secondary, "Preview", Leading::Icon(Icon::Play)),
            (ButtonKind::Secondary, "Add camera key", Leading::Glyph(&diamond, <color::Warning as ColorToken>::VALUE)),
            (ButtonKind::Primary, "Rendering", Leading::Spinner),
        ];
        SheetGrid::page(ui, "Text button", |ui| {
            SheetGrid::panel(ui, |ui| {
                SheetGrid::heads(ui, Self::COLUMN, &["REST", "HOVER", "PRESSED", "FOCUS", "DISABLED"]);
                for (label, kind, words, lead) in rows {
                    SheetGrid::row(ui, label, Self::COLUMN, Self::height(), Self::STATES.len(), |ui, index, cell| {
                        Self::paint(
                            ui,
                            cell,
                            TextButton::new(kind, Sample::text(words)).leading(lead),
                            Self::STATES[index],
                        );
                    });
                }
                SheetGrid::row(
                    ui,
                    "Leading · rest",
                    Self::COLUMN,
                    Self::height(),
                    leading.len(),
                    |ui, index, cell| {
                        let (kind, words, lead) = leading[index];
                        let button = TextButton::new(kind, Sample::text(words)).leading(lead);
                        Self::paint(ui, cell, button, InteractionState::Rest);
                    },
                );
            });
        });
    }

    fn paint(ui: &Ui, cell: egui::Rect, button: TextButton<'_>, state: InteractionState) {
        let width = button.desired_width(ui.ctx());
        button.paint(ui.painter(), SheetGrid::place(cell, vec2(width, Self::height())), state);
    }

    fn height() -> f32 {
        <spacing::Button as SpacingToken>::VALUE
    }
}
