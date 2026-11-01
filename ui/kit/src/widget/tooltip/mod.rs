use amml_i18n::text::Text;
use egui::{Frame, Label, Pos2, Rect, Response, Ui, pos2, vec2};

use crate::draw::{Corner, Draw, TextFit, TextStyle, TypeSize};
use crate::font::FontFace;
use crate::text::UiText;
use crate::tokens::rule::Elevation;
use crate::tokens::{ColorToken, SpacingToken, color, spacing};
use crate::widget::{KeyCapSize, KeyChord};

/// Paper: Widgets, `Widget / Tooltip`. A floating raised bubble under its control, with an optional
/// shortcut. A disabled control shows its reason the same way.
pub struct Tooltip {
    text: Text,
    shortcut: Option<KeyChord>,
}

impl Tooltip {
    const STYLE: TextStyle = TextStyle::new(TypeSize::Note, FontFace::Regular);

    pub fn new(text: Text) -> Self {
        Self { text, shortcut: None }
    }

    /// The control's shortcut, as small key caps after the text.
    pub fn shortcut(mut self, chord: KeyChord) -> Self {
        self.shortcut = Some(chord.size(KeyCapSize::Small));
        self
    }

    /// Shows the tooltip while `response` is hovered or focused, enabled or disabled.
    pub fn attach(self, response: Response) -> Response {
        for mut tooltip in [egui::Tooltip::for_enabled(&response), egui::Tooltip::for_disabled(&response)] {
            tooltip.popup = tooltip.popup.frame(Self::frame()).gap(Self::gap());
            tooltip.show(|ui| self.contents(ui));
        }
        response
    }

    /// Paints the bubble with its top-left corner at `top_left`, for the gallery. Returns its rect.
    pub fn paint(&self, painter: &egui::Painter, top_left: Pos2) -> Rect {
        let text = UiText::resolve(painter.ctx(), &self.text);
        let galley = Draw::galley(painter, &text, Self::STYLE, Self::text_color(), TextFit::Line);
        let chord = self.shortcut.as_ref().map(|chord| chord.desired_size(painter.ctx()));
        let content_width = galley.size().x + chord.map_or(0.0, |size| Self::gap() + size.x);
        let content_height = galley.size().y.max(chord.map_or(0.0, |size| size.y));
        let padding = Self::padding();
        let rect = Rect::from_min_size(top_left, vec2(content_width, content_height) + padding * 2.0);
        Draw::surface(
            painter,
            rect,
            Corner::Control,
            <color::BgRaised as ColorToken>::VALUE,
            Some(<color::LineStrong as ColorToken>::VALUE),
            Elevation::Floating,
        );
        let centre_y = rect.center().y;
        let text_left = rect.left() + padding.x;
        let text_size = galley.size();
        painter.galley(pos2(text_left, centre_y - text_size.y / 2.0), galley, Self::text_color());
        if let Some(chord) = &self.shortcut {
            chord.paint(painter, pos2(text_left + text_size.x + Self::gap(), centre_y));
        }
        rect
    }

    fn contents(&self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = vec2(Self::gap(), 0.0);
            let text = UiText::resolve(ui.ctx(), &self.text);
            let galley = Draw::galley(ui.painter(), &text, Self::STYLE, Self::text_color(), TextFit::Line);
            ui.add(Label::new(galley));
            if let Some(chord) = &self.shortcut {
                let (rect, _) = ui.allocate_exact_size(chord.desired_size(ui.ctx()), egui::Sense::hover());
                chord.paint(ui.painter(), rect.left_center());
            }
        });
    }

    fn frame() -> Frame {
        let padding = Self::padding();
        let radius = Corner::Control.radius(Rect::NOTHING);
        Frame::new()
            .inner_margin(Draw::margin(padding.x, padding.y))
            .corner_radius(radius)
            .fill(<color::BgRaised as ColorToken>::VALUE)
            .stroke(Draw::border(<color::LineStrong as ColorToken>::VALUE))
            .shadow(Elevation::Floating.shadow())
    }

    fn padding() -> egui::Vec2 {
        vec2(<spacing::M as SpacingToken>::VALUE, <spacing::S as SpacingToken>::VALUE)
    }

    fn gap() -> f32 {
        <spacing::S as SpacingToken>::VALUE
    }

    fn text_color() -> egui::Color32 {
        <color::Text as ColorToken>::VALUE
    }
}
