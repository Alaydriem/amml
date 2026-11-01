use egui::{Rect, Ui, vec2};

use crate::gallery::{GalleryState, Sample, SheetGrid};
use crate::state::InteractionState;
use crate::tokens::{ContainerToken, SpacingToken, container, spacing};
use crate::widget::{Menu, MenuItem, MenuLeading};

/// Paper: Widgets, `Widget / Menu item` (`5TY-0`), the row `RADIO`.
pub struct MenuItemRadioSheet;

impl MenuItemRadioSheet {
    pub fn show(ui: &mut Ui, _state: &mut GalleryState) {
        let width = <container::Menu as ContainerToken>::VALUE;
        let padding = <spacing::S as SpacingToken>::VALUE;
        let row = <spacing::Row as SpacingToken>::VALUE;
        let options = [
            ("Draft", false, InteractionState::Rest),
            ("Ready", true, InteractionState::Hover),
            ("Rendered", false, InteractionState::Rest),
        ];
        let height = row * options.len() as f32 + 2.0 * padding;
        SheetGrid::page(ui, "Menu item", |ui| {
            SheetGrid::panel(ui, |ui| {
                SheetGrid::head_row(ui, "RADIO", width, height, 1, |ui, _, cell| {
                    let menu = Rect::from_min_size(cell.left_top(), vec2(width, height));
                    Menu::paint_frame(ui.painter(), menu);
                    for (index, (name, chosen, state)) in options.into_iter().enumerate() {
                        let top = menu.top() + padding + row * index as f32;
                        let rect = Rect::from_min_size(
                            egui::pos2(menu.left() + padding, top),
                            vec2(width - 2.0 * padding, row),
                        );
                        MenuItem::new(Sample::text(name)).leading(MenuLeading::Radio(chosen)).paint(
                            ui.painter(),
                            rect,
                            state,
                        );
                    }
                });
            });
        });
    }
}
