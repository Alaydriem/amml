use amml_i18n::text::Text;
use egui::{Response, Ui, Vec2};

use super::{Menu, MenuItem, MenuTrailing};

/// A menu row with a CHEVRON that opens a menu to its right, on hover or click (egui's submenu
/// rules, in the Paper menu frame).
pub struct Submenu;

impl Submenu {
    pub fn show(ui: &mut Ui, text: Text, add: impl FnOnce(&mut Ui)) -> Response {
        let response = MenuItem::new(text).trailing(MenuTrailing::Submenu).show(ui);
        egui::containers::menu::SubMenu::new().show(ui, &response, |ui| {
            ui.set_width(Menu::inner_width());
            ui.spacing_mut().item_spacing = Vec2::ZERO;
            add(ui);
        });
        response
    }
}
