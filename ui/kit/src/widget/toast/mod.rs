mod toast_response;
mod toast_tone;

pub use toast_response::ToastResponse;
pub use toast_tone::ToastTone;

use std::sync::Arc;

use amml_i18n::text::{Args, Text};
use egui::{Align2, Galley, Rect, Sense, Ui, UiBuilder, pos2, vec2};

use crate::AmmlUiMessage;
use crate::draw::{Corner, Draw, TextFit, TextStyle, TypeSize};
use crate::font::FontFace;
use crate::icon::{Icon, IconSize};
use crate::text::UiText;
use crate::tokens::rule::Elevation;
use crate::tokens::{ColorToken, SpacingToken, color, spacing};
use crate::widget::{IconButton, IconButtonSize, Surface, SurfaceLevel};

/// Paper: Mod manager, `Widget / Toast`. A floating card with a tone tile, a title, an optional
/// message and hint, its age, an optional close button, and a countdown bar along the bottom.
/// It takes the full width the caller gives it.
pub struct Toast {
    tone: ToastTone,
    title: Text,
    message: Option<Text>,
    hint: Option<Text>,
    age_ms: Option<u64>,
    remaining: f32,
    dismissable: bool,
}

impl Toast {
    /// Paper: Toast, the countdown track height.
    const COUNTDOWN: f32 = 2.0;
    /// Paper: Toast, the round tone tile.
    const TILE: f32 = 36.0;
    /// Paper: Toast, the content row's left padding.
    const PAD_LEFT: f32 = 20.0;
    /// Paper: Toast, the gap between the title, message and hint lines.
    const LINE_GAP: f32 = 2.0;
    /// Paper: Toast, the countdown track's inset from the card sides.
    const TRACK_INSET: f32 = 14.0;
    const TITLE: TextStyle = TextStyle::new(TypeSize::Body, FontFace::Semibold);
    const BODY: TextStyle = TextStyle::new(TypeSize::Help, FontFace::Regular);
    const AGE: TextStyle = TextStyle::new(TypeSize::Caption, FontFace::Regular);

    pub fn new(tone: ToastTone, title: Text) -> Self {
        Self { tone, title, message: None, hint: None, age_ms: None, remaining: 1.0, dismissable: false }
    }

    pub fn message(mut self, text: Text) -> Self {
        self.message = Some(text);
        self
    }

    /// A third line in `--color-text-dim`: what the user can do about it.
    pub fn hint(mut self, text: Text) -> Self {
        self.hint = Some(text);
        self
    }

    /// The toast's age, shown at the top right: "now" under one second, then whole seconds.
    pub fn age_ms(mut self, age_ms: u64) -> Self {
        self.age_ms = Some(age_ms);
        self
    }

    /// The part of the time-to-live that is left, 0 to 1. The countdown bar is this wide.
    pub fn remaining(mut self, remaining: f32) -> Self {
        self.remaining = remaining.clamp(0.0, 1.0);
        self
    }

    /// Shows a close button after the age.
    pub fn dismissable(mut self, dismissable: bool) -> Self {
        self.dismissable = dismissable;
        self
    }

    pub fn show(self, ui: &mut Ui) -> ToastResponse {
        let gap = <spacing::L as SpacingToken>::VALUE;
        let pad_top = <spacing::L as SpacingToken>::VALUE;
        let pad_right = <spacing::L as SpacingToken>::VALUE;
        let pad_bottom = <spacing::M as SpacingToken>::VALUE;
        let width = ui.available_width();
        let painter = ui.painter().clone();

        let age = self.age_ms.map(|age| {
            let text = UiText::resolve(ui.ctx(), &Self::age_text(age));
            Draw::galley(&painter, &text, Self::AGE, <color::TextFaint as ColorToken>::VALUE, TextFit::Line)
        });
        let close = <spacing::ControlSmall as SpacingToken>::VALUE;
        let mut right = age.as_ref().map_or(0.0, |age| gap + age.size().x);
        if self.dismissable {
            right += gap + close;
        }
        let column = (width - Self::PAD_LEFT - Self::TILE - gap - right - pad_right).max(0.0);
        let lines = self.lines(ui, column);
        let text_height = lines.iter().map(|(galley, _)| galley.size().y).sum::<f32>()
            + Self::LINE_GAP * lines.len().saturating_sub(1) as f32;
        let content = text_height.max(Self::TILE);
        let bottom = <spacing::Xs as SpacingToken>::VALUE;
        let height = pad_top + content + pad_bottom + Self::COUNTDOWN + bottom;
        let (rect, _) = ui.allocate_exact_size(vec2(width, height), Sense::hover());
        Surface::paint(&painter, rect, SurfaceLevel::Panel, Elevation::Floating);

        let top = rect.top() + pad_top;
        let tile = Rect::from_min_size(pos2(rect.left() + Self::PAD_LEFT, top), vec2(Self::TILE, Self::TILE));
        Draw::fill(&painter, tile, Corner::Full, self.tone.soft());
        self.tone.icon().paint(&painter, tile, IconSize::Regular, self.tone.color());

        let mut y = top;
        let x = tile.right() + gap;
        for (galley, colour) in lines {
            let line = galley.size().y;
            painter.galley(pos2(x, y), galley, colour);
            y += line + Self::LINE_GAP;
        }

        // the age and the close button sit on the title's centre line
        let centre = top + TypeSize::Body.line_height() / 2.0;
        let mut edge = rect.right() - pad_right;
        let mut dismissed = false;
        if self.dismissable {
            let button = Rect::from_center_size(pos2(edge - close / 2.0, centre), vec2(close, close));
            let tooltip = Text::message(AmmlUiMessage::Dismiss);
            let close_button = IconButton::new(Icon::Close).size(IconButtonSize::Small).tooltip(tooltip);
            // a child, not a scope: a scope would move the cursor back up to the button
            dismissed = close_button.show(&mut ui.new_child(UiBuilder::new().max_rect(button))).clicked();
            edge = button.left() - gap;
        }
        if let Some(age) = age {
            let place = Align2::RIGHT_CENTER.anchor_size(pos2(edge, centre), age.size());
            painter.galley(place.min, age, <color::TextFaint as ColorToken>::VALUE);
        }

        let track_top = rect.bottom() - bottom - Self::COUNTDOWN;
        let track = Rect::from_min_max(
            pos2(rect.left() + Self::TRACK_INSET, track_top),
            pos2(rect.right() - Self::TRACK_INSET, track_top + Self::COUNTDOWN),
        );
        Draw::fill(&painter, track, Corner::None, <color::Line as ColorToken>::VALUE);
        if self.remaining > 0.0 {
            let fill = Rect::from_min_size(track.min, vec2(track.width() * self.remaining, Self::COUNTDOWN));
            Draw::fill(&painter, fill, Corner::None, self.tone.color());
        }
        ToastResponse { dismissed }
    }

    // the title, message and hint, wrapped at `width`, each with its colour
    fn lines(&self, ui: &Ui, width: f32) -> Vec<(Arc<Galley>, egui::Color32)> {
        let text = <color::Text as ColorToken>::VALUE;
        let dim = <color::TextDim as ColorToken>::VALUE;
        let parts = [
            (Some(&self.title), Self::TITLE, text),
            (self.message.as_ref(), Self::BODY, text),
            (self.hint.as_ref(), Self::BODY, dim),
        ];
        parts
            .into_iter()
            .filter_map(|(part, style, colour)| {
                let resolved = UiText::resolve(ui.ctx(), part?);
                Some((Draw::galley(ui.painter(), &resolved, style, colour, TextFit::Wrap(width)), colour))
            })
            .collect()
    }

    fn age_text(age_ms: u64) -> Text {
        if age_ms < 1000 {
            return Text::message(AmmlUiMessage::Now);
        }
        let seconds = age_ms / 1000;
        Text::plural(AmmlUiMessage::SecondsAgo, seconds, Args::new().uint("count", seconds))
    }
}
