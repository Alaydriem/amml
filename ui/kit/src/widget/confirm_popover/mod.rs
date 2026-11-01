mod confirm_popover_choice;

pub use confirm_popover_choice::ConfirmPopoverChoice;

use std::sync::Arc;

use amml_i18n::text::Text;
use egui::{
    Area, Context, Galley, Id, Key, LayerId, Modifiers, Order, Painter, Rect, Sense, Ui, UiBuilder, Vec2, pos2, vec2,
};

use crate::AmmlUiMessage;
use crate::draw::{Corner, Draw, TextFit, TextStyle, TypeSize};
use crate::font::FontFace;
use crate::icon::Icon;
use crate::id::AmmlUiId;
use crate::state::InteractionState;
use crate::text::UiText;
use crate::tokens::rule::Elevation;
use crate::tokens::{ColorToken, SpacingToken, color, spacing};
use crate::widget::{ButtonKind, Leading, TextButton};

/// Paper: Widgets, `Widget / Popover · Confirm` (copied from the Entry Point discard popover). A small question
/// beside the control that asked it: a title, an optional body, then the cancel button and the danger button.
/// Enter confirms (when no text field has the keyboard) and Escape cancels; both from the second frame it shows.
pub struct ConfirmPopover {
    id: Id,
    title: Text,
    confirm: Text,
    body: Option<Text>,
    cancel: Text,
}

impl ConfirmPopover {
    /// Paper: the popover is 340 wide.
    pub const WIDTH: f32 = 340.0;
    // Paper: 18 on top, 20 at the sides and bottom, parts 14 apart, the buttons 4 lower; the danger button 120 wide
    const TOP: f32 = 18.0;
    const SIDE: f32 = 20.0;
    const BOTTOM: f32 = 20.0;
    const GAP: f32 = 14.0;
    const ACTIONS_TOP: f32 = 4.0;
    const CONFIRM_WIDTH: f32 = 120.0;
    // the popover keeps this far from the control it sits beside
    const ANCHOR_GAP: f32 = 12.0;
    const TITLE: TextStyle = TextStyle::new(TypeSize::Title, FontFace::Semibold);
    const BODY: TextStyle = TextStyle::new(TypeSize::Help, FontFace::Regular);

    pub fn new(title: Text, confirm: Text) -> Self {
        Self {
            id: Id::unique(AmmlUiId::ConfirmPopover),
            title,
            confirm,
            body: None,
            cancel: Text::message(AmmlUiMessage::Cancel),
        }
    }

    pub fn body(mut self, body: Text) -> Self {
        self.body = Some(body);
        self
    }

    /// The words on the cancel button, as in "Keep recording"; "Cancel" when not set.
    pub fn cancel(mut self, cancel: Text) -> Self {
        self.cancel = cancel;
        self
    }

    /// Another id, for a second popover on one screen.
    pub fn id(mut self, id: Id) -> Self {
        self.id = id;
        self
    }

    /// Shows the popover left of `anchor` (right of it when there is no room), its top level with the anchor's
    /// and kept on the screen. Returns the answer in the frame it was given.
    pub fn show(&self, ui: &mut Ui, anchor: Rect) -> Option<ConfirmPopoverChoice> {
        let ctx = ui.ctx().clone();
        let at = self.rect(&ctx, anchor).min;
        // a key held as the popover opens is not an answer to it
        let open = ctx.memory(|memory| memory.areas().visible_last_frame(&LayerId::new(Order::Foreground, self.id)));
        let shown = Area::new(self.id).order(Order::Foreground).fixed_pos(at).show(&ctx, |ui| self.show_card(ui));
        shown.inner.or_else(|| open.then(|| Self::keys(&ctx)).flatten())
    }

    /// The rect `show` puts the card at for `anchor`, for a caller that routes clicks to it.
    pub fn rect(&self, ctx: &Context, anchor: Rect) -> Rect {
        let size = self.size(ctx);
        let screen = ctx.content_rect();
        let left = anchor.left() - Self::ANCHOR_GAP - Self::WIDTH;
        let x = if left >= screen.left() { left } else { anchor.right() + Self::ANCHOR_GAP };
        let y = anchor.top().min(screen.bottom() - size.y).max(screen.top());
        Rect::from_min_size(pos2(x, y), size)
    }

    // Escape cancels; Enter confirms unless another widget has the keyboard (a text field takes its own Enter).
    // The key is taken, so a dialog under the popover does not also close.
    fn keys(ctx: &Context) -> Option<ConfirmPopoverChoice> {
        let free = ctx.memory(|memory| memory.focused().is_none());
        ctx.input_mut(|input| {
            if input.consume_key(Modifiers::NONE, Key::Escape) {
                Some(ConfirmPopoverChoice::Cancel)
            } else if free && input.consume_key(Modifiers::NONE, Key::Enter) {
                Some(ConfirmPopoverChoice::Confirm)
            } else {
                None
            }
        })
    }

    /// Shows the card at `ui`'s cursor, with no keys, for the gallery.
    pub fn show_card(&self, ui: &mut Ui) -> Option<ConfirmPopoverChoice> {
        let (rect, _) = ui.allocate_exact_size(self.size(ui.ctx()), Sense::hover());
        let (cancel, confirm) = Self::buttons(Self::actions(rect));
        let mut child = ui.new_child(UiBuilder::new().max_rect(rect));
        let cancel_response = child.allocate_rect(cancel, Sense::click());
        let confirm_response = child.allocate_rect(confirm, Sense::click());
        let states = (InteractionState::of(&cancel_response, true), InteractionState::of(&confirm_response, true));
        self.paint_states(ui.painter(), rect, states);
        if confirm_response.clicked() {
            Some(ConfirmPopoverChoice::Confirm)
        } else {
            cancel_response.clicked().then_some(ConfirmPopoverChoice::Cancel)
        }
    }

    /// Paints the popover in `rect` with its danger button in `confirm`, for the gallery.
    pub fn paint(&self, painter: &Painter, rect: Rect, confirm: InteractionState) {
        self.paint_states(painter, rect, (InteractionState::Rest, confirm));
    }

    /// The popover's size for its words.
    pub fn size(&self, ctx: &Context) -> Vec2 {
        let painter = Painter::new(ctx.clone(), egui::LayerId::background(), Rect::EVERYTHING);
        let (title, body) = self.galleys(&painter);
        let body_height = body.map_or(0.0, |body| body.size().y + Self::GAP);
        let height = Self::TOP
            + title.size().y
            + Self::GAP
            + body_height
            + Self::ACTIONS_TOP
            + <spacing::Button as SpacingToken>::VALUE
            + Self::BOTTOM;
        vec2(Self::WIDTH, height)
    }

    fn paint_states(&self, painter: &Painter, rect: Rect, (cancel, confirm): (InteractionState, InteractionState)) {
        let raised = <color::BgRaised as ColorToken>::VALUE;
        let strong = <color::LineStrong as ColorToken>::VALUE;
        Draw::surface(painter, rect, Corner::Panel, raised, Some(strong), Elevation::Floating);
        let (title, body) = self.galleys(painter);
        let mut y = rect.top() + Self::TOP;
        let title_height = title.size().y;
        painter.galley(pos2(rect.left() + Self::SIDE, y), title, <color::Text as ColorToken>::VALUE);
        y += title_height + Self::GAP;
        if let Some(body) = body {
            painter.galley(pos2(rect.left() + Self::SIDE, y), body, <color::TextDim as ColorToken>::VALUE);
        }
        let (cancel_rect, confirm_rect) = Self::buttons(Self::actions(rect));
        TextButton::new(ButtonKind::Secondary, self.cancel.clone()).paint(painter, cancel_rect, cancel);
        TextButton::new(ButtonKind::Danger, self.confirm.clone()).leading(Leading::Icon(Icon::Delete)).paint(
            painter,
            confirm_rect,
            confirm,
        );
    }

    fn galleys(&self, painter: &Painter) -> (Arc<Galley>, Option<Arc<Galley>>) {
        let width = Self::WIDTH - Self::SIDE * 2.0;
        let title = UiText::resolve(painter.ctx(), &self.title);
        let title =
            Draw::galley(painter, &title, Self::TITLE, <color::Text as ColorToken>::VALUE, TextFit::Wrap(width));
        let body = self.body.as_ref().map(|body| {
            let body = UiText::resolve(painter.ctx(), body);
            Draw::galley(painter, &body, Self::BODY, <color::TextDim as ColorToken>::VALUE, TextFit::Wrap(width))
        });
        (title, body)
    }

    // the button row along the bottom
    fn actions(rect: Rect) -> Rect {
        let height = <spacing::Button as SpacingToken>::VALUE;
        Rect::from_min_max(
            pos2(rect.left() + Self::SIDE, rect.bottom() - Self::BOTTOM - height),
            pos2(rect.right() - Self::SIDE, rect.bottom() - Self::BOTTOM),
        )
    }

    // (cancel, confirm): the cancel button takes the room the 120 wide danger button leaves
    fn buttons(actions: Rect) -> (Rect, Rect) {
        let gap = <spacing::S as SpacingToken>::VALUE;
        let confirm = Rect::from_min_max(pos2(actions.right() - Self::CONFIRM_WIDTH, actions.top()), actions.max);
        let cancel = Rect::from_min_max(actions.min, pos2(confirm.left() - gap, actions.bottom()));
        (cancel, confirm)
    }
}
