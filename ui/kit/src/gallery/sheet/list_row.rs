use amml_i18n::text::{Args, Text, UserText};
use egui::{Frame, Rect, Sense, Ui, vec2};

use crate::AmmlUiMessage;
use crate::draw::{Corner, Draw};
use crate::gallery::{GalleryState, Sample, SheetGrid};
use crate::icon::Icon;
use crate::state::InteractionState;
use crate::tokens::{ColorToken, color};
use crate::widget::{Avatar, AvatarSize, Chip, ChipKind, ChipTone, ListRow};

/// Paper: Mod manager, `Widget / List row` (`DGD-0`). The row states and kinds, each under its
/// head, in a 460 wide host panel.
pub struct ListRowSheet;

impl ListRowSheet {
    /// Sheet chrome: the host panel's width and padding, and the gap under each row.
    const HOST_WIDTH: f32 = 460.0;
    const HOST_PADDING: f32 = 16.0;
    const ROW_GAP: f32 = 8.0;
    const COLUMN_GAP: f32 = 24.0;

    pub fn show(ui: &mut Ui, _state: &mut GalleryState) {
        SheetGrid::page(ui, "List row", |ui| {
            ui.horizontal_top(|ui| {
                ui.spacing_mut().item_spacing.x = Self::COLUMN_GAP;
                Self::host(ui, Self::version_chips);
                Self::host(ui, Self::subtitles);
            });
        });
    }

    // the host panel, filled by `rows`
    fn host(ui: &mut Ui, rows: fn(&mut Ui)) {
        Frame::new()
            .fill(<color::BgPanel as ColorToken>::VALUE)
            .stroke(Draw::border(<color::Line as ColorToken>::VALUE))
            .corner_radius(Corner::Panel.radius(Rect::NOTHING))
            .inner_margin(Draw::margin(Self::HOST_PADDING, Self::HOST_PADDING))
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    ui.set_width(Self::HOST_WIDTH - 2.0 * Self::HOST_PADDING);
                    ui.spacing_mut().item_spacing = vec2(0.0, SheetGrid::HEAD_GAP);
                    rows(ui);
                });
            });
    }

    fn version_chips(ui: &mut Ui) {
        // the toggles are painted, not shown, so these values never change
        let [rest, hover, selected, focus, off] = &mut [true, true, true, true, false];
        let row = Self::row("Minimap", &["Input", "HUD", "Overlay"], rest);
        Self::sample(ui, "REST", InteractionState::Rest, row);
        let hover = Self::row("Light Level", &["World"], hover).badge(1);
        Self::sample(ui, "HOVER", InteractionState::Hover, hover);
        let more = Chip::new(
            ChipKind::Display(ChipTone::Neutral),
            Text::format(AmmlUiMessage::MoreItems, Args::new().uint("count", 3)),
        );
        let mut chips = Self::chips(&["Camera", "Input"]);
        chips.push(more);
        let selected = Self::row("Freecam", &[], selected).chips(chips).selected(true);
        Self::sample(ui, "SELECTED", InteractionState::Rest, selected);
        let claim = Chip::new(ChipKind::Display(ChipTone::Accent), Sample::text("Camera")).icon(Icon::Lock);
        let mut chips = vec![claim];
        chips.extend(Self::chips(&["Input"]));
        Self::sample(ui, "FOCUS", InteractionState::Focused, Self::row("DejaVu", &[], focus).chips(chips));
        let long = Self::row("World Edit Selection Tools and Brushes", &["World"], off)
            .version(Sample::text("v2.11.0"))
            .off(true);
        Self::sample(ui, "MOD OFF · LONG NAME", InteractionState::Rest, long);
    }

    // the name and subtitle rows
    fn subtitles(ui: &mut Ui) {
        // the toggle is painted, not shown, so this value never changes
        let mut off = false;
        let selected = Self::subtitled("Main hall", "32 × 18 × 24 · 4,212 blocks", None).selected(true);
        Self::sample(ui, "SELECTED", InteractionState::Rest, selected);
        let hover = Self::subtitled("East tower", "9 × 41 × 9 · 2,018 blocks", None);
        Self::sample(ui, "HOVER", InteractionState::Hover, hover);
        let plain = Self::subtitled("West wing", "48 × 22 × 30 · 9,604 blocks", None);
        Self::sample(ui, "PLAIN", InteractionState::Rest, plain);
        let lit = Self::subtitled("Lighthouse", "12 × 30 × 9 · 1,204 blocks", Some(&mut off)).off(true);
        Self::sample(ui, "OFF", InteractionState::Rest, lit);
        let long = Self::subtitled(
            "Harbour west with crane and the long quay",
            "128 × 64 × 128 · 1,048,576 blocks · 3 layers · assembled from many pieces",
            None,
        );
        Self::sample(ui, "LONG NAME · ELLIPSIS", InteractionState::Rest, long);
    }

    // a row with its avatar, name and subtitle, and a toggle when `toggle` is given
    fn subtitled<'a>(name: &str, subtitle: &str, toggle: Option<&'a mut bool>) -> ListRow<'a> {
        let avatar = Avatar::new(&UserText::from_model(name.to_owned()), AvatarSize::Regular);
        let row = ListRow::new(avatar, Sample::text(name)).subtitle(Sample::text(subtitle));
        match toggle {
            Some(on) => row.toggle(on),
            None => row,
        }
    }

    // a head, then the row painted in `state`
    fn sample(ui: &mut Ui, head: &str, state: InteractionState, row: ListRow<'_>) {
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing = vec2(0.0, SheetGrid::HEAD_GAP);
            SheetGrid::head(ui, head);
            let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), row.height()), Sense::hover());
            row.paint(ui.painter(), rect, state);
            ui.add_space(Self::ROW_GAP - SheetGrid::HEAD_GAP);
        });
    }

    // a mod row with its avatar, version "v0.1.0", `chips` and its toggle
    fn row<'a>(name: &str, chips: &[&str], on: &'a mut bool) -> ListRow<'a> {
        let avatar = Avatar::new(&UserText::from_model(name.to_owned()), AvatarSize::Regular);
        ListRow::new(avatar, Sample::text(name)).version(Sample::text("v0.1.0")).chips(Self::chips(chips)).toggle(on)
    }

    fn chips(texts: &[&str]) -> Vec<Chip> {
        texts.iter().map(|text| Chip::new(ChipKind::Display(ChipTone::Neutral), Sample::text(text))).collect()
    }
}
