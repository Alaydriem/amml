mod banner_tone;

pub use banner_tone::BannerTone;

use amml_i18n::text::Text;
use egui::{Rect, Response, Sense, Ui, UiBuilder, pos2, vec2};

use crate::draw::{Corner, Draw, TextFit, TextStyle, TypeSize};
use crate::font::FontFace;
use crate::icon::IconSize;
use crate::text::UiText;
use crate::tokens::{ColorToken, SpacingToken, TextSizeToken, color, spacing, text_size};
use crate::widget::{KeyChord, TextButton};

/// Paper: Mod manager, `Widget / Banner`. A full-width message in a tone: an icon, a title, and
/// a body line or a key chord line, with an optional action at the right. Only the action is
/// interactive.
pub struct Banner<'a> {
    tone: BannerTone,
    title: Text,
    body: Option<Text>,
    chord: Option<(KeyChord, Text)>,
    action: Option<TextButton<'a>>,
}

impl<'a> Banner<'a> {
    /// Paper: Banner, the left and right padding.
    const PAD_X: f32 = 20.0;
    /// Paper: Banner, the gap between the title and the body.
    const LINE_GAP: f32 = 2.0;
    const TITLE: TextStyle = TextStyle::new(TypeSize::Body, FontFace::Semibold);
    const BODY: TextStyle = TextStyle::new(TypeSize::Help, FontFace::Regular);

    pub fn new(tone: BannerTone, title: Text) -> Self {
        Self { tone, title, body: None, chord: None, action: None }
    }

    /// A second line in `--color-text-dim`. It wraps.
    pub fn body(mut self, text: Text) -> Self {
        self.body = Some(text);
        self
    }

    /// Puts the title, `chord` and `tail` on one line: "1 key conflict. Ctrl + B is also bound by
    /// Light Level."
    pub fn chord(mut self, chord: KeyChord, tail: Text) -> Self {
        self.chord = Some((chord, tail));
        self
    }

    /// A button at the right. Paper draws a Secondary button.
    pub fn action(mut self, button: TextButton<'a>) -> Self {
        self.action = Some(button);
        self
    }

    /// Takes the full width. Returns the action's response.
    pub fn show(self, ui: &mut Ui) -> Option<Response> {
        let gap = self.tone.gap();
        let pad_y = self.tone.padding();
        let width = ui.available_width();
        let painter = ui.painter().clone();
        let icon = <text_size::Icon as TextSizeToken>::VALUE;
        let action = self.action.map(|button| (button.desired_width(ui.ctx()), button));
        let action_width = action.as_ref().map_or(0.0, |(width, _)| gap + width);
        let column = (width - 2.0 * Self::PAD_X - icon - gap - action_width).max(0.0);

        let title_colour = self.tone.title_color();
        let title = UiText::resolve(ui.ctx(), &self.title);
        // the chord line is one line; a plain title wraps
        let fit = if self.chord.is_some() { TextFit::Line } else { TextFit::Wrap(column) };
        let title = Draw::galley(&painter, &title, Self::TITLE, title_colour, fit);
        let dim = <color::TextDim as ColorToken>::VALUE;
        let body = self.body.as_ref().map(|body| {
            let body = UiText::resolve(ui.ctx(), body);
            Draw::galley(&painter, &body, Self::BODY, dim, TextFit::Wrap(column))
        });
        let text_height = match (&self.chord, &body) {
            (Some((chord, _)), _) => title.size().y.max(chord.desired_size(ui.ctx()).y),
            (None, Some(body)) => title.size().y + Self::LINE_GAP + body.size().y,
            (None, None) => title.size().y,
        };
        let action_height = <spacing::Button as SpacingToken>::VALUE;
        let content = if action.is_some() { text_height.max(action_height) } else { text_height };
        let (rect, _) = ui.allocate_exact_size(vec2(width, content + 2.0 * pad_y), Sense::hover());
        Draw::fill(&painter, rect, Corner::Panel, self.tone.fill());
        Draw::stroke(&painter, rect, Corner::Panel, self.tone.border());

        let inner = rect.shrink2(vec2(Self::PAD_X, pad_y));
        let slot = Rect::from_min_size(pos2(inner.left(), inner.center().y - icon / 2.0), vec2(icon, icon));
        self.tone.icon().paint(&painter, slot, IconSize::Regular, self.tone.icon_color());

        let x = slot.right() + gap;
        let top = inner.center().y - text_height / 2.0;
        match (self.chord, body) {
            (Some((chord, tail)), _) => {
                let centre = inner.center().y;
                let between = <spacing::S as SpacingToken>::VALUE;
                painter.galley(pos2(x, centre - title.size().y / 2.0), title.clone(), title_colour);
                let chord_rect = chord.paint(&painter, pos2(x + title.size().x + between, centre));
                let text = <color::Text as ColorToken>::VALUE;
                let tail = UiText::resolve(ui.ctx(), &tail);
                // the line is one row: the tail is cut where the text column ends
                let left = chord_rect.right() + between;
                let fit = TextFit::Truncate((x + column - left).max(0.0));
                let tail = Draw::galley(&painter, &tail, Self::BODY, text, fit);
                painter.galley(pos2(left, centre - tail.size().y / 2.0), tail, text);
            }
            (None, body) => {
                let title_height = title.size().y;
                painter.galley(pos2(x, top), title, title_colour);
                if let Some(body) = body {
                    painter.galley(pos2(x, top + title_height + Self::LINE_GAP), body, dim);
                }
            }
        }

        let (button_width, button) = action?;
        let place = Rect::from_min_size(
            pos2(inner.right() - button_width, inner.center().y - action_height / 2.0),
            vec2(button_width, action_height),
        );
        // a child, not a scope: a scope would move the cursor back up to the button
        let mut child = ui.new_child(UiBuilder::new().max_rect(place));
        Some(button.width(button_width).show(&mut child))
    }
}
