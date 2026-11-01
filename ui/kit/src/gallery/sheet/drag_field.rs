use egui::{Color32, Id, Rect, Ui, pos2, vec2};

use crate::draw::TypeSize;
use crate::gallery::{GalleryState, Sample, SheetGrid};
use crate::state::InteractionState;
use crate::tokens::{ColorToken, SpacingToken, color, spacing};
use crate::widget::{DragField, DragUnit, FieldLook};

/// Paper: Widgets, `Widget / Drag field` (`5A1-0`). The axis and track label colours are amml-studio-ui
/// tokens, so this page uses the nearest amml-ui colours: X danger, Y success, Z and t accent,
/// fov warning.
pub struct DragFieldSheet;

impl DragFieldSheet {
    const COLUMN: f32 = 120.0;
    const USE_WIDTH: f32 = 180.0;
    /// Sheet chrome: the TRANSFORM panel's label column and its field width.
    const TRANSFORM_LABEL: f32 = 84.0;
    const TRANSFORM_FIELD: f32 = 96.0;
    /// Sheet chrome: the TRANSFORM panel is `--container-inspector` wide, an amml-studio-ui token.
    const TRANSFORM_WIDTH: f32 = 430.0;

    // the sample tables are tuples of Paper's cell values, read once in this function
    #[allow(clippy::type_complexity)]
    pub fn show(ui: &mut Ui, _state: &mut GalleryState) {
        let height = <spacing::Field as SpacingToken>::VALUE;
        SheetGrid::page(ui, "Drag field", |ui| {
            SheetGrid::panel(ui, |ui| {
                SheetGrid::heads(
                    ui,
                    Self::COLUMN,
                    &["REST", "HOVER", "DRAGGING", "FOCUS", "TYPING", "INVALID", "DISABLED"],
                );
                let reason = <spacing::Xs as SpacingToken>::VALUE + TypeSize::Micro.line_height();
                for row in 0..2 {
                    let label = if row == 0 { "Unit" } else { "Axis label" };
                    SheetGrid::row(ui, label, Self::COLUMN, height + reason, 7, |ui, index, cell| {
                        let (mut value, look) = Self::cell(row, index);
                        let state = [
                            InteractionState::Rest,
                            InteractionState::Hover,
                            InteractionState::Pressed,
                            InteractionState::Focused,
                            InteractionState::Focused,
                            InteractionState::Rest,
                            InteractionState::Disabled,
                        ][index];
                        let rect = Rect::from_min_size(cell.min, vec2(Self::COLUMN, height));
                        let field = DragField::new(Id::NULL, &mut value).decimals(1);
                        let field = if row == 0 {
                            field.unit(DragUnit::Degrees)
                        } else {
                            field.label(Sample::text("X"), Self::red()).unit(DragUnit::Text(Sample::text("m")))
                        };
                        field.paint(ui.painter(), rect, state, &look);
                    });
                }
            });
            SheetGrid::section(ui, "TRANSFORM", |ui| {
                egui::Frame::new().show(ui, |ui| {
                    ui.set_width(Self::TRANSFORM_WIDTH);
                    SheetGrid::list_panel(ui, |ui| {
                        ui.spacing_mut().item_spacing = vec2(0.0, <spacing::S as SpacingToken>::VALUE);
                        let faint = <color::TextFaint as ColorToken>::VALUE;
                        let rows: [(&str, [(&str, Color32, f64); 3], DragUnit); 2] = [
                            (
                                "Position",
                                [("X", Self::red(), 186.0), ("Y", Self::green(), 82.0), ("Z", Self::blue(), -124.0)],
                                DragUnit::None,
                            ),
                            (
                                "Rotation",
                                [("P", faint, 14.0), ("Y", faint, 325.0), ("R", faint, 0.0)],
                                DragUnit::Degrees,
                            ),
                        ];
                        for (label, fields, unit) in rows {
                            let area =
                                vec2(Self::TRANSFORM_FIELD * 3.0 + <spacing::S as SpacingToken>::VALUE * 2.0, height);
                            SheetGrid::labelled(ui, label, Self::TRANSFORM_LABEL, 0.0, area, |ui, rect| {
                                for (index, (name, colour, value)) in fields.into_iter().enumerate() {
                                    let left = rect.left()
                                        + index as f32 * (Self::TRANSFORM_FIELD + <spacing::S as SpacingToken>::VALUE);
                                    let field_rect = Rect::from_min_size(
                                        pos2(left, rect.top()),
                                        vec2(Self::TRANSFORM_FIELD, height),
                                    );
                                    let mut value = value;
                                    DragField::new(Id::NULL, &mut value)
                                        .decimals(1)
                                        .label(Sample::text(name), colour)
                                        .unit(unit.clone())
                                        .paint(ui.painter(), field_rect, InteractionState::Rest, &FieldLook::default());
                                }
                            });
                        }
                    });
                });
            });
            SheetGrid::section(ui, "IN USE", |ui| {
                SheetGrid::panel(ui, |ui| {
                    let blue = Self::blue();
                    let faint = <color::TextFaint as ColorToken>::VALUE;
                    let rows: [(&str, Vec<(Option<(&str, Color32)>, f64, u8, DragUnit)>); 4] = [
                        (
                            "Scene range",
                            vec![
                                (Some(("In", faint)), 28.8, 2, DragUnit::Text(Sample::text("s"))),
                                (Some(("Out", faint)), 41.25, 2, DragUnit::Text(Sample::text("s"))),
                            ],
                        ),
                        ("Time of day", vec![(Some(("t", blue)), 6000.0, 0, DragUnit::Text(Sample::text("12:00")))]),
                        ("FOV", vec![(Some(("fov", Self::fov())), 70.0, 1, DragUnit::Degrees)]),
                        ("Fly speed", vec![(None, 1.0, 2, DragUnit::Text(Sample::text("m/s")))]),
                    ];
                    for (label, fields) in rows {
                        SheetGrid::row(ui, label, Self::USE_WIDTH, height, fields.len(), |ui, index, cell| {
                            let (name, value, decimals, unit) = &fields[index];
                            let mut value = *value;
                            let mut field = DragField::new(Id::NULL, &mut value).decimals(*decimals).unit(unit.clone());
                            if let Some((name, colour)) = name {
                                field = field.label(Sample::text(name), *colour);
                            }
                            field.paint(ui.painter(), cell, InteractionState::Rest, &FieldLook::default());
                        });
                    }
                });
            });
        });
    }

    // (value, look) of a state-grid cell
    fn cell(row: usize, index: usize) -> (f64, FieldLook) {
        let base = if row == 0 { 325.0 } else { 186.0 };
        match index {
            2 => (if row == 0 { 331.5 } else { 193.4 }, FieldLook::default()),
            3 => (base, FieldLook { selection: Some(5..5), invalid: None }),
            4 => (base, FieldLook { selection: Some(if row == 0 { 0..5 } else { 2..5 }), invalid: None }),
            5 => {
                let reason = if row == 0 { "Max 360°" } else { "Outside the world" };
                (if row == 0 { 412.0 } else { 1e9 }, FieldLook { selection: None, invalid: Some(Sample::text(reason)) })
            }
            _ => (base, FieldLook::default()),
        }
    }

    fn red() -> Color32 {
        <color::Danger as ColorToken>::VALUE
    }

    fn green() -> Color32 {
        <color::Success as ColorToken>::VALUE
    }

    fn blue() -> Color32 {
        <color::AccentHover as ColorToken>::VALUE
    }

    fn fov() -> Color32 {
        <color::Warning as ColorToken>::VALUE
    }
}
