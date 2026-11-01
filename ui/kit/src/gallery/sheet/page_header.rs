use amml_i18n::text::{Text, UserText};
use egui::{Sense, Ui, vec2};

use crate::draw::{Draw, TextFit, TextStyle, TypeSize};
use crate::font::FontFace;
use crate::gallery::{GalleryState, Sample, SheetGrid};
use crate::icon::Icon;
use crate::tokens::{ColorToken, SpacingToken, color, spacing};
use crate::widget::{Chip, ChipKind, ChipTone, IconButton, PageHeader, PageHeaderLead, Toggle};

/// Paper: Mod manager, `Widget / Page header` (`DEP-0`). The panel form, the two page forms, and a
/// long name cut in a 640 wide header.
pub struct PageHeaderSheet;

impl PageHeaderSheet {
    /// Sheet chrome: the long-name sample's width.
    const LONG_WIDTH: f32 = 640.0;

    pub fn show(ui: &mut Ui, state: &mut GalleryState) {
        SheetGrid::page(ui, "Page header", |ui| {
            SheetGrid::section(ui, "PANEL", |ui| {
                let meta = Self::meta(&["4 mods installed", "4 enabled", "LeviLamina 1.4.1"]);
                PageHeader::new(PageHeaderLead::Tile(Icon::Apps, ChipTone::Accent), Sample::text("Mod Manager"))
                    .eyebrow(Sample::text("ALAYDRIEM"))
                    .meta(meta)
                    .trailing(|ui| {
                        ui.spacing_mut().item_spacing = vec2(<spacing::M as SpacingToken>::VALUE, 0.0);
                        IconButton::new(Icon::Close).show(ui);
                        Chip::new(ChipKind::Display(ChipTone::Outline), Sample::text("Saving\u{2026}")).show(ui);
                    })
                    .show(ui);
            });
            SheetGrid::section(ui, "PAGE · WITH AVATAR, CONFLICT CHIP AND TOGGLE", |ui| {
                let lead = PageHeaderLead::Avatar(UserText::from_model("Freecam".to_owned()), None);
                PageHeader::new(lead, Sample::text("Freecam"))
                    .meta(Self::meta(&["freecam", "v0.1.0", "by Alaydriem"]))
                    .trailing(|ui| {
                        ui.spacing_mut().item_spacing = vec2(<spacing::M as SpacingToken>::VALUE, 0.0);
                        Toggle::new(&mut state.mod_enabled).show(ui);
                        Self::label(ui, "Enabled");
                        ui.add_space(<spacing::S as SpacingToken>::VALUE);
                        Chip::new(ChipKind::Display(ChipTone::Warning), Sample::text("1 conflict"))
                            .icon(Icon::Warning)
                            .show(ui);
                    })
                    .show(ui);
            });
            SheetGrid::section(ui, "PAGE · WITH ICON TILE", |ui| {
                let subtitle = "Found from the capability and keybind declarations of 4 installed mods";
                PageHeader::new(PageHeaderLead::Tile(Icon::Warning, ChipTone::Warning), Sample::text("Conflicts"))
                    .subtitle(Sample::text(subtitle))
                    .show(ui);
                ui.allocate_ui(vec2(Self::LONG_WIDTH, 0.0), |ui| {
                    let name = "World Edit Selection Tools and Brushes Extended";
                    let lead = PageHeaderLead::Avatar(UserText::from_model(name.to_owned()), None);
                    PageHeader::new(lead, Sample::text(name))
                        .meta(Self::meta(&["worldedit-selection-tools", "v2.11.0", "by a very long author name"]))
                        .show(ui);
                });
            });
        });
    }

    fn meta(items: &[&str]) -> Vec<Text> {
        items.iter().map(|item| Sample::text(item)).collect()
    }

    // the toggle's label, `--text-label` in `--color-text`
    fn label(ui: &mut Ui, text: &str) {
        let style = TextStyle::new(TypeSize::Label, FontFace::Regular);
        let colour = <color::Text as ColorToken>::VALUE;
        let galley = Draw::galley(ui.painter(), text, style, colour, TextFit::Line);
        let (rect, _) = ui.allocate_exact_size(galley.size(), Sense::hover());
        ui.painter().galley(rect.min, galley, colour);
    }
}
