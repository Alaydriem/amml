use egui::scroll_area::ScrollAreaOutput;
use egui::style::ScrollStyle;
use egui::{Color32, CornerRadius, ScrollArea, Ui};

use crate::tokens::{ColorToken, SpacingToken, color, spacing};

/// Paper: Mod manager, the mod list's scroll bar; Editor, `Editor / Splitter + scroll`, the lane
/// area's. A thin solid bar: a `--color-line-strong` thumb on a `--color-line` track, or on no track.
/// egui draws it; this sets the style and the colours egui reads.
pub struct ScrollBar;

impl ScrollBar {
    /// A round end on a bar this thin: egui holds a corner to half the bar's width.
    const ROUND: CornerRadius = CornerRadius::same(u8::MAX);

    /// The bar's size and margins. `Theme::install` puts it in the style.
    pub fn style() -> ScrollStyle {
        let width = <spacing::Scrollbar as SpacingToken>::VALUE;
        ScrollStyle {
            floating: false,
            bar_width: width,
            floating_width: width,
            bar_inner_margin: <spacing::Xs as SpacingToken>::VALUE,
            bar_outer_margin: 0.0,
            foreground_color: false,
            ..ScrollStyle::solid()
        }
    }

    /// Shows `area` with the bar's colours, and the track when `track` is true (the mod list shows
    /// it; the lane area does not). egui reads the bar's colours from the widget visuals, so they
    /// are set for the area only, and `content` gets the visuals it had.
    pub fn show<R>(
        ui: &mut Ui,
        area: ScrollArea,
        track: bool,
        content: impl FnOnce(&mut Ui) -> R,
    ) -> ScrollAreaOutput<R> {
        ui.scope(|ui| {
            let content_visuals = ui.visuals().clone();
            let visuals = ui.visuals_mut();
            visuals.extreme_bg_color = if track { <color::Line as ColorToken>::VALUE } else { Color32::TRANSPARENT };
            let widgets = &mut visuals.widgets;
            for state in [&mut widgets.inactive, &mut widgets.hovered, &mut widgets.active] {
                state.bg_fill = <color::LineStrong as ColorToken>::VALUE;
                state.corner_radius = Self::ROUND;
            }
            area.show(ui, |ui| {
                *ui.visuals_mut() = content_visuals;
                content(ui)
            })
        })
        .inner
    }
}
