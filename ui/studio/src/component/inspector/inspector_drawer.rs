use amml_i18n::text::Text;
use amml_ui::draw::{Corner, Draw, TextFit, TextStyle, TypeSize};
use amml_ui::font::FontFace;
use amml_ui::icon::Icon;
use amml_ui::tokens::rule::Elevation;
use amml_ui::tokens::{ColorToken, ContainerToken, SpacingToken, color, spacing};
use amml_ui::widget::{IconButton, IconButtonSize};
use egui::{Align2, Area, Id, Key, Order, Rect, Ui, UiBuilder, pos2, vec2};

use super::InspectorTab;
use crate::id::StudioUiId;
use crate::message::StudioUiMessage;
use crate::tokens::container;

/// The inspector at the small breakpoint (`layout-brief.md` item 3): the open tab in a `--container-inspector`
/// drawer over the viewer, against the rail on the right. A close button or Escape closes it.
pub struct InspectorDrawer {
    tab: InspectorTab,
}

impl InspectorDrawer {
    // the header is a tab-strip high; its title and the close button sit `--spacing-l` in
    const HEADER: f32 = 44.0;

    pub fn new(tab: InspectorTab) -> Self {
        Self { tab }
    }

    /// The drawer's rect over `area`: full height, against the rail at the right edge.
    pub fn rect(area: Rect) -> Rect {
        let rail = <container::Rail as ContainerToken>::VALUE;
        let width = <container::Inspector as ContainerToken>::VALUE.min((area.width() - rail).max(0.0));
        let right = area.right() - rail;
        Rect::from_min_max(pos2(right - width, area.top()), pos2(right, area.bottom()))
    }

    /// Shows the drawer over `ui`'s area with `body` under its header. Returns false once it is closed.
    pub fn show(&self, ui: &mut Ui, body: impl FnOnce(&mut Ui)) -> bool {
        let rect = Self::rect(ui.max_rect());
        let mut open = !ui.input(|input| input.key_pressed(Key::Escape));
        Area::new(Id::unique(StudioUiId::InspectorDrawer)).order(Order::Foreground).fixed_pos(rect.min).show(
            ui.ctx(),
            |ui| {
                ui.set_min_size(rect.size());
                let fill = <color::BgPanel as ColorToken>::VALUE;
                let line = <color::Line as ColorToken>::VALUE;
                Draw::surface(ui.painter(), rect, Corner::Small, fill, Some(line), Elevation::Floating);
                let inset = <spacing::L as SpacingToken>::VALUE;
                let header = Rect::from_min_size(rect.min, vec2(rect.width(), Self::HEADER));
                let title = TextStyle::new(TypeSize::Label, FontFace::Semibold);
                let ink = <color::Text as ColorToken>::VALUE;
                let at = pos2(header.left() + inset, header.center().y);
                Draw::text(ui.painter(), at, Align2::LEFT_CENTER, &self.tab.label(), title, ink, TextFit::Line);
                Draw::divider(ui.painter(), header.x_range(), header.bottom() - 1.0, line);
                let close = IconButton::new(Icon::Close).tooltip(Text::message(StudioUiMessage::CloseInspector));
                let side = IconButtonSize::Regular.side();
                let place = Rect::from_center_size(
                    pos2(header.right() - inset - side / 2.0, header.center().y),
                    vec2(side, side),
                );
                if close.show(&mut ui.new_child(UiBuilder::new().max_rect(place))).clicked() {
                    open = false;
                }
                let inner = Rect::from_min_max(
                    pos2(rect.left() + inset, header.bottom() + inset),
                    rect.max - vec2(inset, inset),
                );
                body(&mut ui.new_child(UiBuilder::new().max_rect(inner)));
            },
        );
        open
    }
}
