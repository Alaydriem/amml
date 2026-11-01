use amml_i18n::text::Text;
use egui::{Align2, Context, CursorIcon, Painter, Rect, Response, Sense, Ui, pos2, vec2};

use super::{ButtonKind, Leading};
use crate::draw::{Corner, Draw, TextFit, TextStyle, TypeSize};
use crate::font::FontFace;
use crate::state::{DisabledReason, InteractionState};
use crate::text::UiText;
use crate::tokens::rule::FocusRing;
use crate::tokens::{SpacingToken, spacing};

/// Paper: Widgets, `Widget / Text button`. A labelled button of one `ButtonKind`, with an optional
/// leading icon, glyph or spinner. Height `--spacing-button`; it fits its content unless given a width.
pub struct TextButton<'a> {
    kind: ButtonKind,
    text: Text,
    leading: Leading<'a>,
    disabled: Option<DisabledReason>,
    width: Option<f32>,
}

impl<'a> TextButton<'a> {
    const STYLE: TextStyle = TextStyle::new(TypeSize::Help, FontFace::Semibold);

    /// The label is a `Text`, so a raw string does not compile:
    ///
    /// ```compile_fail
    /// use amml_ui::widget::{ButtonKind, TextButton};
    ///
    /// TextButton::new(ButtonKind::Primary, "Save");
    /// ```
    pub fn new(kind: ButtonKind, text: Text) -> Self {
        Self { kind, text, leading: Leading::None, disabled: None, width: None }
    }

    pub fn leading(mut self, leading: Leading<'a>) -> Self {
        self.leading = leading;
        self
    }

    /// When `enabled` is false the button is muted, takes no clicks, and shows `reason` as its tooltip.
    pub fn enabled(mut self, enabled: bool, reason: DisabledReason) -> Self {
        self.disabled = if enabled { None } else { Some(reason) };
        self
    }

    /// A fixed width; the label is cut with an ellipsis when it does not fit.
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// The width that fits the label in the context's locale, the leading part and the padding.
    pub fn desired_width(&self, ctx: &Context) -> f32 {
        let text = UiText::resolve(ctx, &self.text);
        let label = ctx.fonts_mut(|fonts| {
            let format = egui::TextFormat { font_id: Self::STYLE.font_id(), ..Default::default() };
            fonts.layout_job(egui::text::LayoutJob::single_section(text, format)).size().x
        });
        (label + self.lead_width() + Self::padding() * 2.0).ceil()
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        let width = self.width.unwrap_or_else(|| self.desired_width(ui.ctx()));
        let enabled = self.disabled.is_none();
        let sense = if enabled { Sense::click() } else { Sense::hover() };
        let (rect, response) = ui.allocate_exact_size(vec2(width, Self::height()), sense);
        let state = InteractionState::of(&response, enabled);
        self.paint(ui.painter(), rect, state);
        match &self.disabled {
            None => response.on_hover_cursor(CursorIcon::PointingHand),
            Some(reason) => reason.tooltip().attach(response),
        }
    }

    pub fn paint(&self, painter: &Painter, rect: Rect, state: InteractionState) {
        let painter = state.painter(painter);
        if state == InteractionState::Focused {
            FocusRing::paint(&painter, rect, Corner::Control.radius(rect));
        }
        Draw::fill(&painter, rect, Corner::Control, self.kind.style().fill(state));
        if let Some(border) = self.kind.border() {
            Draw::stroke(&painter, rect, Corner::Control, border);
        }
        self.paint_content(&painter, rect);
    }

    // the leading part and the label, centred as one group and cut to the padded width
    fn paint_content(&self, painter: &Painter, rect: Rect) {
        let color = self.kind.text();
        let room = (rect.width() - Self::padding() * 2.0 - self.lead_width()).max(0.0);
        let text = UiText::resolve(painter.ctx(), &self.text);
        let galley = Draw::galley(painter, &text, Self::STYLE, color, TextFit::Truncate(room));
        let group = self.lead_width() + galley.size().x;
        let left = rect.center().x - group / 2.0;
        let lead = self.leading.size();
        if lead.x > 0.0 {
            let lead_rect = Rect::from_center_size(pos2(left + lead.x / 2.0, rect.center().y), lead);
            self.leading.paint(painter, lead_rect, color);
        }
        let at = Align2::LEFT_CENTER.anchor_size(pos2(left + self.lead_width(), rect.center().y), galley.size());
        painter.galley(at.min, galley, color);
    }

    fn lead_width(&self) -> f32 {
        let lead = self.leading.size().x;
        if lead > 0.0 { lead + <spacing::S as SpacingToken>::VALUE } else { 0.0 }
    }

    fn padding() -> f32 {
        <spacing::ButtonX as SpacingToken>::VALUE
    }

    fn height() -> f32 {
        <spacing::Button as SpacingToken>::VALUE
    }
}
