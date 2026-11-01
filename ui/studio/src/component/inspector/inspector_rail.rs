use amml_ui::tokens::{ColorToken, ContainerToken, color};
use amml_ui::widget::{IconButton, IconButtonSize};
use egui::{Rect, Sense, Stroke, Ui, UiBuilder, pos2, vec2};

use super::InspectorTab;
use crate::tokens::container;

/// The inspector at the small breakpoint (`layout-brief.md` item 3): a `--container-rail` strip with one icon
/// button per tab. A click opens that tab in the `InspectorDrawer`.
pub struct InspectorRail {
    selected: InspectorTab,
}

impl InspectorRail {
    // the buttons start this far down and sit this far apart
    const PADDING: f32 = 8.0;
    const GAP: f32 = 4.0;

    pub fn new(selected: InspectorTab) -> Self {
        Self { selected }
    }

    /// Takes the rail's width and the height left; returns the tab that was clicked.
    pub fn show(&self, ui: &mut Ui) -> Option<InspectorTab> {
        let width = <container::Rail as ContainerToken>::VALUE;
        let (rect, _) = ui.allocate_exact_size(vec2(width, ui.available_height()), Sense::hover());
        let painter = ui.painter();
        painter.rect_filled(rect, 0, <color::BgPanel as ColorToken>::VALUE);
        painter.vline(rect.left() + 0.5, rect.y_range(), Stroke::new(1.0, <color::Line as ColorToken>::VALUE));
        let mut clicked = None;
        for tab in InspectorTab::ALL {
            let button = IconButton::new(tab.icon()).on(tab == self.selected).tooltip(tab.label());
            let place = Self::place(rect, tab);
            if button.show(&mut ui.new_child(UiBuilder::new().max_rect(place))).clicked() {
                clicked = Some(tab);
            }
        }
        clicked
    }

    /// Where `tab`'s button sits on a rail at `rail`.
    pub fn place(rail: Rect, tab: InspectorTab) -> Rect {
        let side = IconButtonSize::Regular.side();
        let top = rail.top() + Self::PADDING + (side + Self::GAP) * tab.index() as f32;
        Rect::from_min_size(pos2(rail.center().x - side / 2.0, top), vec2(side, side))
    }
}
