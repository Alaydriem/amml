use amml_i18n::text::Text;
use egui::{Rect, Response, Sense, Ui, UiBuilder, pos2, vec2};

use crate::draw::{Corner, Draw, TextFit, TextStyle, TypeSize};
use crate::font::FontFace;
use crate::icon::{Icon, IconSize};
use crate::text::UiText;
use crate::tokens::rule::Elevation;
use crate::tokens::{ColorToken, SpacingToken, color, spacing};
use crate::widget::TextButton;

/// Paper: Widgets, `Widget / Empty state`. An icon tile, a title, a message and an optional
/// action, stacked and centred in the space the caller gives it. The text wraps. The card look
/// (`EmptyState::card`, Paper state `Card, no icon`) is a raised card with the title and the message
/// at the left, with no icon tile and no action.
pub struct EmptyState<'a> {
    icon: Option<Icon>,
    title: Text,
    message: Text,
    action: Option<TextButton<'a>>,
}

impl<'a> EmptyState<'a> {
    const TITLE: TextStyle = TextStyle::new(TypeSize::Label, FontFace::Semibold);
    const MESSAGE: TextStyle = TextStyle::new(TypeSize::Note, FontFace::Regular);

    pub fn new(icon: Icon, title: Text, message: Text) -> Self {
        Self { icon: Some(icon), title, message, action: None }
    }

    /// The card look: a raised card, the title and the message left aligned, no icon and no action.
    pub fn card(title: Text, message: Text) -> Self {
        Self { icon: None, title, message, action: None }
    }

    /// The button under the message; Paper draws a Secondary button.
    pub fn action(mut self, button: TextButton<'a>) -> Self {
        self.action = Some(button);
        self
    }

    /// Fills the space left in `ui` and centres the stack in it. Returns the action's response.
    pub fn show(self, ui: &mut Ui) -> Option<Response> {
        let Some(icon) = self.icon else {
            self.show_card(ui);
            return None;
        };
        let area = ui.available_rect_before_wrap();
        ui.allocate_rect(area, Sense::hover());
        let gap_m = <spacing::M as SpacingToken>::VALUE;
        let gap_s = <spacing::S as SpacingToken>::VALUE;
        let tile = <spacing::Tile as SpacingToken>::VALUE;
        let painter = ui.painter().clone();
        let title_text = UiText::resolve(ui.ctx(), &self.title);
        let message_text = UiText::resolve(ui.ctx(), &self.message);
        let text = <color::Text as ColorToken>::VALUE;
        let dim = <color::TextDim as ColorToken>::VALUE;
        let title = Draw::galley_centred(&painter, &title_text, Self::TITLE, text, area.width());
        let message = Draw::galley_centred(&painter, &message_text, Self::MESSAGE, dim, area.width());
        // the action sits `--spacing-s` further down than the stack gap
        let action = self.action.map(|button| (button.desired_width(ui.ctx()), button));
        let action_height = <spacing::Button as SpacingToken>::VALUE;
        let text_height = title.size().y + gap_s + message.size().y;
        let mut height = tile + gap_m + text_height;
        if action.is_some() {
            height += gap_m + gap_s + action_height;
        }

        let centre = area.center().x;
        let mut y = area.center().y - height / 2.0;
        let tile_rect = Rect::from_center_size(pos2(centre, y + tile / 2.0), vec2(tile, tile));
        Draw::fill(&painter, tile_rect, Corner::Control, <color::BgRaised as ColorToken>::VALUE);
        icon.paint(&painter, tile_rect, IconSize::Regular, dim);
        y += tile + gap_m;
        let title_height = title.size().y;
        painter.galley(pos2(centre, y), title, text);
        y += title_height + gap_s;
        let message_height = message.size().y;
        painter.galley(pos2(centre, y), message, dim);
        y += message_height;

        let (width, button) = action?;
        y += gap_m + gap_s;
        let rect = Rect::from_min_size(pos2(centre - width / 2.0, y), vec2(width, action_height));
        // a child, not a scope: a scope would move the cursor back up to the button
        let mut child = ui.new_child(UiBuilder::new().max_rect(rect));
        Some(button.width(width).show(&mut child))
    }

    // the card look: the card takes the width left, and its height from the wrapped text
    fn show_card(self, ui: &mut Ui) {
        let pad = <spacing::L as SpacingToken>::VALUE + <spacing::Border as SpacingToken>::VALUE;
        let gap = <spacing::S as SpacingToken>::VALUE;
        let width = ui.available_width();
        let room = (width - 2.0 * pad).max(0.0);
        let painter = ui.painter().clone();
        let title_text = UiText::resolve(ui.ctx(), &self.title);
        let message_text = UiText::resolve(ui.ctx(), &self.message);
        let text = <color::Text as ColorToken>::VALUE;
        let dim = <color::TextDim as ColorToken>::VALUE;
        let title = Draw::galley(&painter, &title_text, Self::TITLE, text, TextFit::Wrap(room));
        let message = Draw::galley(&painter, &message_text, Self::MESSAGE, dim, TextFit::Wrap(room));
        let height = 2.0 * pad + title.size().y + gap + message.size().y;
        let (rect, _) = ui.allocate_exact_size(vec2(width, height), Sense::hover());
        let fill = <color::BgRaised as ColorToken>::VALUE;
        let line = Some(<color::Line as ColorToken>::VALUE);
        Draw::surface(&painter, rect, Corner::Panel, fill, line, Elevation::Flat);
        let title_height = title.size().y;
        painter.galley(pos2(rect.left() + pad, rect.top() + pad), title, text);
        painter.galley(pos2(rect.left() + pad, rect.top() + pad + title_height + gap), message, dim);
    }
}
