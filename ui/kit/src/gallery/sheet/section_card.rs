use std::ops::RangeInclusive;

use amml_i18n::text::{Args, Text, UserText};
use egui::{Id, Response, Ui, vec2};

use crate::AmmlUiMessage;
use crate::gallery::{GalleryId, GalleryState, Sample, SheetGrid};
use crate::icon::Icon;
use crate::tokens::{SpacingToken, spacing};
use crate::widget::{
    Chip, ChipKind, ChipTone, ColourField, DragField, DragUnit, Dropdown, DropdownOption, SectionCard, SettingControl,
    SettingRow, Slider,
};

/// Paper: Mod manager, `Widget / Section card` (`DK2-0`). Two sections of a mod's settings, with
/// the controls from the Widgets page.
pub struct SectionCardSheet;

impl SectionCardSheet {
    /// Sheet chrome: the card width, and the slider, value box and dropdown widths in it.
    const WIDTH: f32 = 940.0;
    const SLIDER: f32 = 236.0;
    const FIELD: f32 = 104.0;
    const DROPDOWN: f32 = 250.0;

    pub fn show(ui: &mut Ui, state: &mut GalleryState) {
        SheetGrid::page(ui, "Section card", |ui| {
            ui.allocate_ui(vec2(Self::WIDTH, 0.0), |ui| {
                ui.set_width(Self::WIDTH);
                Self::map(ui, state);
            });
            ui.allocate_ui(vec2(Self::WIDTH, 0.0), |ui| {
                ui.set_width(Self::WIDTH);
                Self::overlays(ui, state);
            });
        });
    }

    fn map(ui: &mut Ui, state: &mut GalleryState) {
        let GalleryState { show_map, zoom, scan_radius, shape, .. } = state;
        SectionCard::new(Sample::text("MAP")).show(ui, |card| {
            card.row(
                SettingRow::new(Sample::text("Show the corner map"), SettingControl::Toggle(show_map))
                    .help(Sample::text("Hide it without disabling the mod")),
            );
            let zoom = Self::slider(GalleryId::Zoom, zoom, 0.5..=8.0, DragUnit::Text(Sample::text("x")));
            let in_game = Args::new().user("value", UserText::from_model("2.50x".to_owned()));
            card.row(
                SettingRow::new(Sample::text("Zoom"), zoom)
                    .help(Sample::text("Blocks per map pixel"))
                    .changed(true)
                    .note(Text::format(AmmlUiMessage::ChangedInGame, in_game)),
            );
            let reload = Chip::new(ChipKind::Display(ChipTone::Outline), Sample::text("Applies after world reload"))
                .icon(Icon::Info);
            let radius = Self::slider(GalleryId::ScanRadius, scan_radius, 2.0..=32.0, DragUnit::None);
            card.row(
                SettingRow::new(Sample::text("Scan radius"), radius)
                    .help(Sample::text("Chunks scanned around the player"))
                    .changed(true)
                    .chip(reload),
            );
            let options = ["Circle", "Square"].iter().map(|text| DropdownOption::new(Sample::text(text))).collect();
            let dropdown = SettingControl::Other(Box::new(move |ui: &mut Ui, _| {
                Dropdown::new(Id::unique(GalleryId::Shape), shape, options).show(ui, Self::DROPDOWN)
            }));
            card.row(
                SettingRow::new(Sample::text("Shape"), dropdown)
                    .help(Sample::text("Outline of the corner map"))
                    .changed(true),
            );
        });
    }

    fn overlays(ui: &mut Ui, state: &mut GalleryState) {
        let GalleryState { chunk_grid, grid_colour, .. } = state;
        SectionCard::new(Sample::text("OVERLAYS")).show(ui, |card| {
            card.row(
                SettingRow::new(Sample::text("Chunk grid"), SettingControl::Toggle(chunk_grid))
                    .help(Sample::text("Lines on chunk borders")),
            );
            let colour = SettingControl::Other(Box::new(move |ui: &mut Ui, _| {
                ColourField::new(Id::unique(GalleryId::GridColour), grid_colour).show(ui)
            }));
            card.row(
                SettingRow::new(Sample::text("Grid colour"), colour)
                    .help(Sample::text("Colour and opacity of the grid lines")),
            );
        });
    }

    // a slider with its value box at the right, as one control; laid out right to left
    fn slider(id: GalleryId, value: &mut f64, range: RangeInclusive<f64>, unit: DragUnit) -> SettingControl<'_> {
        let decimals = if matches!(unit, DragUnit::None) { 0 } else { 2 };
        SettingControl::Other(Box::new(move |ui: &mut Ui, _| -> Response {
            ui.spacing_mut().item_spacing = vec2(<spacing::L as SpacingToken>::VALUE, 0.0);
            let field = DragField::new(Id::unique(id), &mut *value).decimals(decimals).unit(unit).show(ui, Self::FIELD);
            let slider = Slider::new(&mut *value, range).show(ui, Self::SLIDER);
            field | slider
        }))
    }
}
