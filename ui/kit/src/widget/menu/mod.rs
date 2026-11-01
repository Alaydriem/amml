mod menu_item;
mod menu_leading;
mod menu_tone;
mod menu_trailing;
mod submenu;

pub use menu_item::MenuItem;
pub use menu_leading::MenuLeading;
pub use menu_tone::MenuTone;
pub use menu_trailing::MenuTrailing;
pub use submenu::Submenu;

use amml_i18n::text::Text;
use egui::{
    Align, Frame, Id, Layout, Painter, Popup, PopupAnchor, PopupCloseBehavior, PopupKind, Pos2, Rect, RectAlign,
    Response, Sense, Ui, Vec2, pos2, vec2,
};

use crate::draw::{Corner, Draw, TextFit, TextStyle, TypeSize};
use crate::font::FontFace;
use crate::text::UiText;
use crate::tokens::rule::Elevation;
use crate::tokens::{ColorToken, ContainerToken, SpacingToken, color, container, spacing};

/// Paper: Widgets, `Widget / Menu item`, "OPEN MENU". A context menu: a floating raised panel
/// `--container-menu` wide, with an optional title and its items.
pub struct Menu;

impl Menu {
    const TITLE: TextStyle = TextStyle::new(TypeSize::Help, FontFace::Semibold);

    /// Opens on a secondary click of `response`; `add` adds the items.
    pub fn show(response: &Response, title: Option<Text>, add: impl FnOnce(&mut Ui)) {
        // leaf items close the menu themselves; a click on a disabled item or a title leaves it open
        let popup = Popup::context_menu(response).close_behavior(PopupCloseBehavior::CloseOnClickOutside);
        popup.frame(Self::frame()).width(Self::inner_width()).show(|ui| {
            ui.set_width(Self::inner_width());
            ui.spacing_mut().item_spacing = Vec2::ZERO;
            if let Some(title) = &title {
                let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), Self::row()), Sense::hover());
                Self::paint_title(ui.painter(), rect, title);
                Self::separator(ui);
            }
            add(ui);
        });
    }

    /// Paper: Widgets, `Widget / Menu item`, "Open menu". A menu that a click on `button` toggles, below the button's
    /// left edge. A click outside closes it. Returns whether it is open after this frame, so the caller can draw
    /// `button` pressed.
    pub fn show_from_button(button: &Response, add: impl FnOnce(&mut Ui)) -> bool {
        Popup::from_toggle_button_response(button)
            .close_behavior(PopupCloseBehavior::CloseOnClickOutside)
            .frame(Self::frame())
            .gap(<spacing::Xs as SpacingToken>::VALUE)
            .width(Self::inner_width())
            .align(RectAlign::BOTTOM_START)
            .show(|ui| {
                ui.set_width(Self::inner_width());
                add(ui);
            });
        Popup::is_id_open(&button.ctx, Popup::default_response_id(button))
    }

    /// Paper: Widgets, `Widget / Menu item`, "Open menu". A menu with its top left corner at `at`, in the popup `popup`.
    /// The caller opens it with `Popup::open_id` and closes it with `Popup::close_id`. A click outside closes it.
    /// Returns whether it is open.
    pub fn show_at(ui: &Ui, popup: Id, at: Pos2, add: impl FnOnce(&mut Ui)) -> bool {
        Popup::new(popup, ui.ctx().clone(), PopupAnchor::Position(at), ui.layer_id())
            .kind(PopupKind::Menu)
            .layout(Layout::top_down_justified(Align::Min))
            .close_behavior(PopupCloseBehavior::CloseOnClickOutside)
            .gap(0.0)
            .frame(Self::frame())
            .width(Self::inner_width())
            .open_memory(None)
            .show(add)
            .is_some()
    }

    /// A separator row: `--spacing-s` high with a 1 px `--color-line` across.
    pub fn separator(ui: &mut Ui) {
        let (rect, _) =
            ui.allocate_exact_size(vec2(ui.available_width(), <spacing::S as SpacingToken>::VALUE), Sense::hover());
        Self::paint_separator(ui.painter(), rect);
    }

    /// The menu panel: `--color-bg-raised`, 1 px `--color-line-strong`, `--radius-panel`, Floating.
    pub fn frame() -> Frame {
        let padding = <spacing::S as SpacingToken>::VALUE;
        Frame::new()
            .fill(<color::BgRaised as ColorToken>::VALUE)
            .stroke(Draw::border(<color::LineStrong as ColorToken>::VALUE))
            .corner_radius(Corner::Panel.radius(Rect::NOTHING))
            .inner_margin(Draw::margin(padding, padding))
            .shadow(Elevation::Floating.shadow())
    }

    /// The width inside the menu's padding.
    pub fn inner_width() -> f32 {
        <container::Menu as ContainerToken>::VALUE - <spacing::S as SpacingToken>::VALUE * 2.0
    }

    /// Paints the panel into `rect` (the whole menu), for the gallery.
    pub fn paint_frame(painter: &Painter, rect: Rect) {
        let raised = <color::BgRaised as ColorToken>::VALUE;
        let strong = <color::LineStrong as ColorToken>::VALUE;
        Draw::surface(painter, rect, Corner::Panel, raised, Some(strong), Elevation::Floating);
    }

    pub fn paint_title(painter: &Painter, rect: Rect, title: &Text) {
        let text = UiText::resolve(painter.ctx(), title);
        let colour = <color::Text as ColorToken>::VALUE;
        let galley = Draw::galley(painter, &text, Self::TITLE, colour, TextFit::Truncate(rect.width()));
        let padding = <spacing::S as SpacingToken>::VALUE;
        painter.galley(pos2(rect.left() + padding, rect.center().y - galley.size().y / 2.0), galley, colour);
    }

    pub fn paint_separator(painter: &Painter, rect: Rect) {
        Draw::divider(painter, rect.x_range(), rect.center().y, <color::Line as ColorToken>::VALUE);
    }

    fn row() -> f32 {
        <spacing::Row as SpacingToken>::VALUE
    }
}
