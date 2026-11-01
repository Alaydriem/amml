use egui::{Frame, Rect, Sense, Ui, vec2};

use crate::draw::{Corner, Draw};
use crate::gallery::{GalleryState, Sample, SheetGrid};
use crate::icon::Icon;
use crate::state::InteractionState;
use crate::tokens::{ColorToken, color};
use crate::widget::NavEntry;

/// Paper: Mod manager, `Widget / Nav entry` (`DGF-0`). The entry states, each under its head, in a
/// 460 wide host panel.
pub struct NavEntrySheet;

impl NavEntrySheet {
    /// Sheet chrome: the host panel's width and padding, and the gap under each entry.
    const HOST_WIDTH: f32 = 460.0;
    const HOST_PADDING: f32 = 16.0;
    const ROW_GAP: f32 = 8.0;

    /// The tally panel: each number plain, then on the open page.
    const TALLIES: [(&'static str, u32, bool); 6] = [
        ("TALLY · PLAIN · 0", 0, false),
        ("TALLY · PLAIN · 4", 4, false),
        ("TALLY · PLAIN · 12", 12, false),
        ("TALLY · SELECTED · 0", 0, true),
        ("TALLY · SELECTED · 4", 4, true),
        ("TALLY · SELECTED · 12", 12, true),
    ];

    pub fn show(ui: &mut Ui, _state: &mut GalleryState) {
        SheetGrid::page(ui, "Nav entry", |ui| {
            Self::host(ui, |ui| {
                let keybinds = || NavEntry::new(Icon::Keyboard, Sample::text("Keybinds"));
                Self::sample(ui, "REST", InteractionState::Rest, keybinds());
                let settings = NavEntry::new(Icon::Settings, Sample::text("Manager settings"));
                Self::sample(ui, "HOVER", InteractionState::Hover, settings);
                let conflicts = NavEntry::new(Icon::Warning, Sample::text("Conflicts")).count(1).selected(true);
                Self::sample(ui, "SELECTED · WITH COUNT", InteractionState::Rest, conflicts);
                Self::sample(ui, "FOCUS", InteractionState::Focused, keybinds());
                Self::sample(ui, "DISABLED", InteractionState::Disabled, keybinds());
            });
            Self::host(ui, |ui| {
                for (head, number, selected) in Self::TALLIES {
                    let entry = if selected {
                        NavEntry::new(Icon::Warning, Sample::text("Conflicts")).selected(true)
                    } else {
                        NavEntry::new(Icon::Keyboard, Sample::text("Keybinds"))
                    };
                    Self::sample(ui, head, InteractionState::Rest, entry.tally(number));
                }
            });
        });
    }

    // the host panel the entries sit in
    fn host(ui: &mut Ui, body: impl FnOnce(&mut Ui)) {
        Frame::new()
            .fill(<color::BgPanel as ColorToken>::VALUE)
            .stroke(Draw::border(<color::Line as ColorToken>::VALUE))
            .corner_radius(Corner::Panel.radius(Rect::NOTHING))
            .inner_margin(Draw::margin(Self::HOST_PADDING, Self::HOST_PADDING))
            .show(ui, |ui| {
                ui.set_width(Self::HOST_WIDTH - 2.0 * Self::HOST_PADDING);
                ui.spacing_mut().item_spacing = vec2(0.0, SheetGrid::HEAD_GAP);
                body(ui);
            });
    }

    // a head, then the entry painted in `state`
    fn sample(ui: &mut Ui, head: &str, state: InteractionState, entry: NavEntry) {
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing = vec2(0.0, SheetGrid::HEAD_GAP);
            SheetGrid::head(ui, head);
            let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), NavEntry::ROW), Sense::hover());
            entry.paint(ui.painter(), rect, state);
            ui.add_space(Self::ROW_GAP - SheetGrid::HEAD_GAP);
        });
    }
}
