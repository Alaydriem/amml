use egui::{Color32, Rect, Sense, Ui, pos2, vec2};

use crate::gallery::{GalleryState, OpenRow, Sample, SheetGrid};
use crate::icon::Icon;
use crate::state::{DisabledReason, InteractionState};
use crate::tokens::{ColorToken, ContainerToken, SpacingToken, color, container, spacing};
use crate::widget::{Menu, MenuItem, MenuLeading, MenuTone, MenuTrailing};

/// Paper: Widgets, `Widget / Menu item` (`5TY-0`). The scene colours of the colour submenu are
/// amml-studio-ui tokens, so this page shows six amml-ui colours in their place.
pub struct MenuItemSheet;

impl MenuItemSheet {
    const STATES: [InteractionState; 3] = [InteractionState::Rest, InteractionState::Hover, InteractionState::Pressed];

    pub fn show(ui: &mut Ui, _state: &mut GalleryState) {
        let width = <container::Menu as ContainerToken>::VALUE;
        let row = <spacing::Row as SpacingToken>::VALUE;
        SheetGrid::page(ui, "Menu item", |ui| {
            SheetGrid::panel(ui, |ui| {
                let rename = || MenuItem::new(Sample::text("Rename scene…")).leading(MenuLeading::Icon(Icon::Rename));
                for (label, state) in [
                    ("REST", InteractionState::Rest),
                    ("HOVER", InteractionState::Hover),
                    ("PRESSED", InteractionState::Pressed),
                    ("FOCUS", InteractionState::Focused),
                ] {
                    SheetGrid::head_row(ui, label, width, row, 1, |ui, _, cell| {
                        rename().paint(ui.painter(), cell, state)
                    });
                }
                SheetGrid::head_row(ui, "DISABLED", width, MenuItem::REASON_HEIGHT, 1, |ui, _, cell| {
                    Self::copy_key().paint(ui.painter(), cell, InteractionState::Disabled);
                });
            });
            SheetGrid::panel(ui, |ui| {
                SheetGrid::heads(ui, width, &["REST", "HOVER", "PRESSED"]);
                let kinds = [
                    ("CHECKED", MenuItem::new(Sample::text("Snap to keys")).leading(MenuLeading::Check)),
                    (
                        "SHORTCUT",
                        MenuItem::new(Sample::text("Duplicate"))
                            .leading(MenuLeading::Icon(Icon::Copy))
                            .trailing(MenuTrailing::Shortcut(Sample::text("Ctrl+D"))),
                    ),
                    ("SUBMENU", MenuItem::new(Sample::text("Colour")).trailing(MenuTrailing::Submenu)),
                    (
                        "DANGER",
                        MenuItem::new(Sample::text("Delete scene…"))
                            .leading(MenuLeading::Icon(Icon::Delete))
                            .tone(MenuTone::Danger),
                    ),
                ];
                for (label, item) in &kinds {
                    SheetGrid::head_row(ui, label, width, row, Self::STATES.len(), |ui, index, cell| {
                        item.paint(ui.painter(), cell, Self::STATES[index]);
                    });
                }
            });
            SheetGrid::panel(ui, |ui| {
                SheetGrid::section(ui, "OPEN MENU", Self::open_menu);
            });
        });
    }

    fn open_menu(ui: &mut Ui) {
        let width = <container::Menu as ContainerToken>::VALUE;
        let padding = <spacing::S as SpacingToken>::VALUE;
        let row = <spacing::Row as SpacingToken>::VALUE;
        let rows = [
            OpenRow::Separator,
            OpenRow::Item(
                MenuItem::new(Sample::text("Rename scene…")).leading(MenuLeading::Icon(Icon::Rename)),
                InteractionState::Rest,
            ),
            OpenRow::Item(
                MenuItem::new(Sample::text("Duplicate"))
                    .leading(MenuLeading::Icon(Icon::Copy))
                    .trailing(MenuTrailing::Shortcut(Sample::text("Ctrl+D"))),
                InteractionState::Rest,
            ),
            OpenRow::Separator,
            OpenRow::Item(
                MenuItem::new(Sample::text("Snap to keys")).leading(MenuLeading::Check),
                InteractionState::Rest,
            ),
            OpenRow::Item(
                MenuItem::new(Sample::text("Colour")).trailing(MenuTrailing::Submenu),
                InteractionState::Hover,
            ),
            OpenRow::Item(Self::copy_key(), InteractionState::Disabled),
            OpenRow::Separator,
            OpenRow::Item(
                MenuItem::new(Sample::text("Delete scene…"))
                    .leading(MenuLeading::Icon(Icon::Delete))
                    .tone(MenuTone::Danger),
                InteractionState::Rest,
            ),
        ];
        let height_of = |open: &OpenRow| match open {
            OpenRow::Separator => padding,
            OpenRow::Item(_, InteractionState::Disabled) => MenuItem::REASON_HEIGHT,
            OpenRow::Item(..) => row,
        };
        let menu_height = padding * 2.0 + row + rows.iter().map(height_of).sum::<f32>();
        let colours = Self::swatches();
        let submenu_height = padding * 2.0 + row * colours.len() as f32;
        let (area, _) =
            ui.allocate_exact_size(vec2(width * 2.0 + padding, menu_height.max(submenu_height)), Sense::hover());
        let menu = Rect::from_min_size(area.min, vec2(width, menu_height));
        Menu::paint_frame(ui.painter(), menu);
        let inner = menu.shrink(padding);
        let mut y = inner.top();
        Menu::paint_title(
            ui.painter(),
            Rect::from_min_size(pos2(inner.left(), y), vec2(inner.width(), row)),
            &Sample::text("Arrival"),
        );
        y += row;
        let mut colour_row_top = y;
        for open in &rows {
            let height = height_of(open);
            let rect = Rect::from_min_size(pos2(inner.left(), y), vec2(inner.width(), height));
            match open {
                OpenRow::Separator => Menu::paint_separator(ui.painter(), rect),
                OpenRow::Item(item, state) => {
                    if *state == InteractionState::Hover {
                        colour_row_top = y;
                    }
                    item.paint(ui.painter(), rect, *state);
                }
            }
            y += height;
        }
        // the colour submenu: its first row lines up with the Colour row
        let submenu =
            Rect::from_min_size(pos2(menu.right() + padding, colour_row_top - padding), vec2(width, submenu_height));
        Menu::paint_frame(ui.painter(), submenu);
        let inner = submenu.shrink(padding);
        for (index, (name, colour)) in colours.into_iter().enumerate() {
            let rect =
                Rect::from_min_size(pos2(inner.left(), inner.top() + row * index as f32), vec2(inner.width(), row));
            let trailing = if index == 0 { MenuTrailing::Check } else { MenuTrailing::None };
            let state = if index == 2 { InteractionState::Hover } else { InteractionState::Rest };
            MenuItem::new(Sample::text(name)).leading(MenuLeading::Swatch(colour)).trailing(trailing).paint(
                ui.painter(),
                rect,
                state,
            );
        }
    }

    fn copy_key() -> MenuItem {
        MenuItem::new(Sample::text("Copy key"))
            .leading(MenuLeading::Icon(Icon::Copy))
            .enabled(false, DisabledReason::new(Sample::text("Select a key first")))
    }

    // stand-ins for the six scene colours (amml-studio-ui tokens)
    fn swatches() -> [(&'static str, Color32); 6] {
        [
            ("Lime", <color::Success as ColorToken>::VALUE),
            ("Pink", <color::Danger as ColorToken>::VALUE),
            ("Orchid", <color::AccentHover as ColorToken>::VALUE),
            ("Periwinkle", <color::Accent as ColorToken>::VALUE),
            ("Peach", <color::Warning as ColorToken>::VALUE),
            ("Sand", <color::TextDim as ColorToken>::VALUE),
        ]
    }
}
