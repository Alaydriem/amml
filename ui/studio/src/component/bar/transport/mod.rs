mod transport_action;
mod transport_control;
mod transport_layout;
mod transport_view;

pub use transport_action::TransportAction;
pub use transport_control::TransportControl;
pub use transport_layout::TransportLayout;
pub use transport_view::TransportView;

use amml_i18n::text::{Args, Text, UserText};
use amml_ui::draw::{Corner, Draw, TextFit, TextStyle, TypeSize};
use amml_ui::font::FontFace;
use amml_ui::glyph::{Glyph, KeyEased, KeyLook};
use amml_ui::icon::{Icon, IconSize};
use amml_ui::state::{DisabledReason, InteractionState};
use amml_ui::text::UiText;
use amml_ui::tokens::{ColorToken, SpacingToken, color, spacing};
use amml_ui::widget::{ButtonKind, IconButton, IconButtonSize, KeyChord, Leading, Slider, TextButton, Tooltip};
use egui::{Align2, CursorIcon, Rect, Response, Sense, Stroke, StrokeKind, Ui, UiBuilder, pos2, vec2};

use super::BarControl;
use crate::message::StudioUiMessage;
use crate::tokens::color as studio;
use crate::tokens::spacing as studio_spacing;

/// Paper: Editor components, `Transport` (`UW-0`). Seek buttons around the round play button, the timecode and
/// speed, the loop / snap / guides toggles, then the zoom slider, Add camera key and Fit.
pub struct Transport<'a> {
    view: &'a TransportView,
    layout: TransportLayout,
}

impl<'a> Transport<'a> {
    // Paper `UW-0`: padding 10 each side; the seek buttons 1 apart, 4 either side of the 36 px play button
    const PADDING: f32 = 10.0;
    const SEEK_GAP: f32 = 1.0;
    const PLAY_GAP: f32 = 4.0;
    const PLAY: f32 = 36.0;
    // Paper: the paused play icon sits 2 right of centre, for its optical centre
    const PLAY_NUDGE: f32 = 2.0;
    // Paper: the timecode group starts 12 after the seek buttons, its parts 16 apart
    const TIME_PADDING: f32 = 12.0;
    const TIME_GAP: f32 = 16.0;
    // Paper: the speed pill is 56 × 26
    const SPEED: [f32; 2] = [56.0, 26.0];
    // Paper: the toggles start 14 after the speed pill, 2 apart
    const TOGGLE_PADDING: f32 = 14.0;
    const TOGGLE_GAP: f32 = 2.0;
    // Paper: the zoom slider is 110 wide (120 in Compact), its icons 4 away; 10 (4 in Compact) before Add camera key
    const ZOOM: [f32; 2] = [110.0, 120.0];
    const ZOOM_GAP: f32 = 4.0;
    const ZOOM_AFTER: [f32; 2] = [10.0, 4.0];
    // a narrower slider is dropped with its zoom buttons (not in Paper)
    const ZOOM_MIN: f32 = 48.0;
    /// The key that adds a camera key, shown on the Compact control's tooltip.
    const ADD_KEY_KEY: &'static str = "K";
    const TIME: TextStyle = TextStyle::new(TypeSize::Title, FontFace::Regular);
    const NOTE: TextStyle = TextStyle::new(TypeSize::Note, FontFace::Regular);

    pub fn new(view: &'a TransportView) -> Self {
        Self { view, layout: TransportLayout::Full }
    }

    pub fn layout(mut self, layout: TransportLayout) -> Self {
        self.layout = layout;
        self
    }

    pub fn show(&self, ui: &mut Ui) -> Vec<TransportAction> {
        self.show_placed(ui).0
    }

    /// Like `show`, with the rect of each control, in points.
    pub fn show_placed(&self, ui: &mut Ui) -> (Vec<TransportAction>, Vec<(TransportControl, Rect)>) {
        let height = <studio_spacing::Transport as SpacingToken>::VALUE;
        let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), height), Sense::hover());
        self.paint_bar(ui, rect);
        let mut actions = Vec::new();
        let mut places = Vec::new();
        let x = self.seek(ui, rect, &mut actions, &mut places);
        let x = self.time(ui, rect, x + Self::TIME_PADDING, &mut actions, &mut places);
        let x = self.toggles(ui, rect, x + Self::TOGGLE_PADDING, &mut actions, &mut places);
        self.right(ui, rect, x + Self::TOGGLE_PADDING, &mut actions, &mut places);
        // `allocate_rect` moved the cursor to each control; the page goes on under the bar
        ui.advance_cursor_after_rect(rect);
        (actions, places)
    }

    fn paint_bar(&self, ui: &Ui, rect: Rect) {
        let painter = ui.painter();
        painter.rect_filled(rect, 0, <color::BgRaised as ColorToken>::VALUE);
        if self.view.playing {
            painter.rect_filled(rect, 0, <studio::ModePreviewSoft as ColorToken>::VALUE);
        }
        let line = <color::Line as ColorToken>::VALUE;
        Draw::divider(painter, rect.x_range(), rect.top() + 0.5, line);
        Draw::divider(painter, rect.x_range(), rect.bottom() - 0.5, line);
    }

    // the seek buttons and the play button; returns the right edge of the group
    fn seek(
        &self,
        ui: &mut Ui,
        rect: Rect,
        actions: &mut Vec<TransportAction>,
        places: &mut Vec<(TransportControl, Rect)>,
    ) -> f32 {
        let mut x = rect.left() + Self::PADDING;
        let before = [
            (Icon::Previous, StudioUiMessage::ToSceneStart, TransportAction::ToSceneStart),
            (Icon::SkipBack, StudioUiMessage::PreviousKey, TransportAction::PreviousKey),
            (Icon::Rewind, StudioUiMessage::BackFive, TransportAction::BackFive),
        ];
        let after = [
            (Icon::FastForward, StudioUiMessage::ForwardFive, TransportAction::ForwardFive),
            (Icon::SkipForward, StudioUiMessage::NextKey, TransportAction::NextKey),
            (Icon::Next, StudioUiMessage::ToSceneEnd, TransportAction::ToSceneEnd),
        ];
        for (index, (icon, word, action)) in before.into_iter().enumerate() {
            x += if index == 0 { 0.0 } else { Self::SEEK_GAP };
            x = Self::icon(ui, rect, x, (icon, word, false, action), actions, places);
        }
        x += Self::SEEK_GAP + Self::PLAY_GAP;
        let play = Rect::from_min_size(pos2(x, rect.center().y - Self::PLAY / 2.0), vec2(Self::PLAY, Self::PLAY));
        self.play(ui, play, actions, places);
        x = play.right() + Self::PLAY_GAP;
        for (icon, word, action) in after {
            x += Self::SEEK_GAP;
            x = Self::icon(ui, rect, x, (icon, word, false, action), actions, places);
        }
        x
    }

    fn play(
        &self,
        ui: &mut Ui,
        rect: Rect,
        actions: &mut Vec<TransportAction>,
        places: &mut Vec<(TransportControl, Rect)>,
    ) {
        places.push((TransportControl::Play, rect));
        let word = if self.view.playing { StudioUiMessage::Pause } else { StudioUiMessage::Play };
        let response = ui.allocate_rect(rect, Sense::click()).on_hover_cursor(CursorIcon::PointingHand);
        let response = Tooltip::new(Text::message(word)).attach(response);
        if response.clicked() {
            actions.push(TransportAction::TogglePlay);
        }
        let state = InteractionState::of(&response, true);
        let fill = match (self.view.playing, state) {
            (true, _) => <studio::TrackCamera as ColorToken>::VALUE,
            (false, InteractionState::Hover) => <color::AccentHover as ColorToken>::VALUE,
            (false, InteractionState::Pressed) => <color::AccentPress as ColorToken>::VALUE,
            (false, _) => <color::Accent as ColorToken>::VALUE,
        };
        let painter = ui.painter();
        Draw::fill(painter, rect, Corner::Full, fill);
        let (icon, nudge) = if self.view.playing { (Icon::Pause, 0.0) } else { (Icon::Play, Self::PLAY_NUDGE) };
        icon.paint(
            painter,
            rect.translate(vec2(nudge, 0.0)),
            IconSize::Regular,
            <color::OnAccent as ColorToken>::VALUE,
        );
    }

    // the timecode, the length and the speed pill from `x`; returns the right edge
    fn time(
        &self,
        ui: &mut Ui,
        rect: Rect,
        x: f32,
        actions: &mut Vec<TransportAction>,
        places: &mut Vec<(TransportControl, Rect)>,
    ) -> f32 {
        let painter = ui.painter();
        let ink = <color::Text as ColorToken>::VALUE;
        let time = Draw::text(
            painter,
            pos2(x, rect.center().y),
            Align2::LEFT_CENTER,
            &self.view.timecode,
            Self::TIME,
            ink,
            TextFit::Line,
        );
        places.push((TransportControl::Timecode, time));
        let mut right = time.right();
        if let Some(duration) = &self.view.duration {
            let length =
                Text::format(StudioUiMessage::Duration, Args::new().user("duration", Self::plain(ui, duration)));
            let at = pos2(right + <spacing::S as SpacingToken>::VALUE, rect.center().y);
            let dim = <color::TextDim as ColorToken>::VALUE;
            right = Draw::text(painter, at, Align2::LEFT_CENTER, &length, Self::NOTE, dim, TextFit::Line).right();
        }
        let pill = Rect::from_min_size(
            pos2(right + Self::TIME_GAP, rect.center().y - Self::SPEED[1] / 2.0),
            vec2(Self::SPEED[0], Self::SPEED[1]),
        );
        places.push((TransportControl::Speed, pill));
        let response = ui.allocate_rect(pill, Sense::click()).on_hover_cursor(CursorIcon::PointingHand);
        let response = Tooltip::new(Text::message(StudioUiMessage::PlaybackSpeed)).attach(response);
        if response.clicked() {
            actions.push(TransportAction::CycleSpeed);
        }
        let painter = ui.painter();
        let fill = if response.hovered() {
            <color::BgHover as ColorToken>::VALUE
        } else {
            <color::BgField as ColorToken>::VALUE
        };
        Draw::fill(painter, pill, Corner::Full, fill);
        let border = if self.view.playing {
            <color::Accent as ColorToken>::VALUE
        } else {
            <color::LineStrong as ColorToken>::VALUE
        };
        painter.rect_stroke(pill, Corner::Full.radius(pill), Stroke::new(1.0, border), StrokeKind::Inside);
        Draw::text(painter, pill.center(), Align2::CENTER_CENTER, &self.view.speed, Self::NOTE, ink, TextFit::Line);
        pill.right()
    }

    // loop, snap and the composition guides, from `x`; returns the right edge
    fn toggles(
        &self,
        ui: &mut Ui,
        rect: Rect,
        x: f32,
        actions: &mut Vec<TransportAction>,
        places: &mut Vec<(TransportControl, Rect)>,
    ) -> f32 {
        let toggles = [
            (Icon::Loop, StudioUiMessage::Loop, self.view.looping, TransportAction::ToggleLoop),
            (Icon::Snap, StudioUiMessage::Snap, self.view.snap, TransportAction::ToggleSnap),
            (Icon::Grid, StudioUiMessage::Guides, self.view.guides, TransportAction::ToggleGuides),
        ];
        let mut x = x;
        for (index, toggle) in toggles.into_iter().enumerate() {
            x += if index == 0 { 0.0 } else { Self::TOGGLE_GAP };
            x = Self::icon(ui, rect, x, toggle, actions, places);
        }
        x
    }

    // Fit, Add camera key and the zoom slider, from the right edge leftwards, none left of `floor`. In a narrow
    // window the slider shrinks, then the zoom buttons and slider drop, before they reach the toggles.
    fn right(
        &self,
        ui: &mut Ui,
        rect: Rect,
        floor: f32,
        actions: &mut Vec<TransportAction>,
        places: &mut Vec<(TransportControl, Rect)>,
    ) {
        let compact = self.layout == TransportLayout::Compact;
        let mut right = rect.right() - Self::PADDING;
        let side = IconButtonSize::Regular.side();
        if compact {
            right = Self::icon(
                ui,
                rect,
                right - side,
                (Icon::Fit, StudioUiMessage::Fit, false, TransportAction::Fit),
                actions,
                places,
            ) - side;
        } else {
            let fit = TextButton::new(ButtonKind::Ghost, Text::message(StudioUiMessage::Fit));
            let width = fit.desired_width(ui.ctx());
            let place = Self::control(rect, right - width, width);
            places.push((TransportControl::Fit, place));
            if Self::place(ui, place, |ui| BarControl::text(ui, &fit, true)).clicked() {
                actions.push(TransportAction::Fit);
            }
            right = place.left();
        }
        right = self.add_key(ui, rect, right, actions, places) - Self::ZOOM_AFTER[usize::from(compact)];
        // the slider's room once both zoom buttons are placed
        let room = right - side - Self::ZOOM_GAP * 2.0 - side - floor;
        if room < Self::ZOOM_MIN {
            return;
        }
        right = Self::icon(
            ui,
            rect,
            right - side,
            (Icon::ZoomIn, StudioUiMessage::ZoomIn, false, TransportAction::ZoomIn),
            actions,
            places,
        ) - side;
        let width = Self::ZOOM[usize::from(compact)].min(room);
        let slider = Rect::from_min_size(pos2(right - Self::ZOOM_GAP - width, rect.top()), vec2(width, rect.height()));
        places.push((TransportControl::ZoomSlider, slider));
        let mut zoom = f64::from(self.view.zoom);
        let changed = Self::place(ui, slider, |ui| {
            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                Slider::new(&mut zoom, 0.0..=1.0).show(ui, width)
            })
            .inner
        })
        .changed();
        if changed {
            actions.push(TransportAction::SetZoom(zoom as f32));
        }
        let x = slider.left() - Self::ZOOM_GAP - side;
        Self::icon(
            ui,
            rect,
            x,
            (Icon::ZoomOut, StudioUiMessage::ZoomOut, false, TransportAction::ZoomOut),
            actions,
            places,
        );
    }

    // Add camera key, ending at `right`; returns its left edge. Off while playing.
    fn add_key(
        &self,
        ui: &mut Ui,
        rect: Rect,
        right: f32,
        actions: &mut Vec<TransportAction>,
        places: &mut Vec<(TransportControl, Rect)>,
    ) -> f32 {
        let enabled = !self.view.playing;
        let key = KeyEased { look: KeyLook::Rest };
        let camera = <studio::TrackCamera as ColorToken>::VALUE;
        let response = if self.layout == TransportLayout::Compact {
            let side = IconButtonSize::Regular.side();
            let place = Rect::from_min_size(pos2(right - side, rect.center().y - side / 2.0), vec2(side, side));
            let sense = if enabled { Sense::click() } else { Sense::hover() };
            let response = ui.allocate_rect(place, sense);
            let state = InteractionState::of(&response, enabled);
            let painter = state.painter(ui.painter());
            let fill = if state == InteractionState::Hover {
                <color::BgActive as ColorToken>::VALUE
            } else {
                <color::BgHover as ColorToken>::VALUE
            };
            Draw::fill(&painter, place, Corner::Control, fill);
            key.paint(&painter, place, camera);
            let chord = KeyChord::new(vec![Text::user(UserText::from_model(Self::ADD_KEY_KEY.to_owned()))]);
            Tooltip::new(Text::message(StudioUiMessage::AddCameraKey)).shortcut(chord).attach(response)
        } else {
            let button = TextButton::new(ButtonKind::Secondary, Text::message(StudioUiMessage::AddCameraKey))
                .leading(Leading::Glyph(&key, camera));
            let width = button.desired_width(ui.ctx());
            let place = Self::control(rect, right - width, width);
            Self::place(ui, place, |ui| BarControl::text(ui, &button, enabled))
        };
        let response = if enabled {
            response
        } else {
            DisabledReason::message(StudioUiMessage::PauseToAddKey).tooltip().attach(response)
        };
        places.push((TransportControl::AddCameraKey, response.rect));
        if response.clicked() && enabled {
            actions.push(TransportAction::AddCameraKey);
        }
        response.rect.left()
    }

    // an icon button at `x`; returns its right edge
    fn icon(
        ui: &mut Ui,
        rect: Rect,
        x: f32,
        (icon, word, on, action): (Icon, StudioUiMessage, bool, TransportAction),
        actions: &mut Vec<TransportAction>,
        places: &mut Vec<(TransportControl, Rect)>,
    ) -> f32 {
        let side = IconButtonSize::Regular.side();
        let place = Rect::from_min_size(pos2(x, rect.center().y - side / 2.0), vec2(side, side));
        if let Some(control) = Self::control_of(action) {
            places.push((control, place));
        }
        let button = IconButton::new(icon).on(on).tooltip(Text::message(word));
        if Self::place(ui, place, |ui| button.show(ui)).clicked() {
            actions.push(action);
        }
        place.right()
    }

    // the control an icon button's action belongs to
    fn control_of(action: TransportAction) -> Option<TransportControl> {
        Some(match action {
            TransportAction::ToSceneStart => TransportControl::ToSceneStart,
            TransportAction::PreviousKey => TransportControl::PreviousKey,
            TransportAction::BackFive => TransportControl::BackFive,
            TransportAction::ForwardFive => TransportControl::ForwardFive,
            TransportAction::NextKey => TransportControl::NextKey,
            TransportAction::ToSceneEnd => TransportControl::ToSceneEnd,
            TransportAction::ToggleLoop => TransportControl::Loop,
            TransportAction::ToggleSnap => TransportControl::Snap,
            TransportAction::ToggleGuides => TransportControl::Guides,
            TransportAction::ZoomOut => TransportControl::ZoomOut,
            TransportAction::ZoomIn => TransportControl::ZoomIn,
            TransportAction::Fit => TransportControl::Fit,
            _ => return None,
        })
    }

    // a bar-height control from `left`, `width` wide
    fn control(rect: Rect, left: f32, width: f32) -> Rect {
        Rect::from_min_size(pos2(left, rect.center().y - BarControl::HEIGHT / 2.0), vec2(width, BarControl::HEIGHT))
    }

    fn place(ui: &mut Ui, place: Rect, show: impl FnOnce(&mut Ui) -> Response) -> Response {
        ui.scope_builder(UiBuilder::new().max_rect(place), show).inner
    }

    // the text of `text` in the context's locale, as user text for a placeholder
    fn plain(ui: &Ui, text: &Text) -> UserText {
        UserText::from_model(UiText::resolve(ui.ctx(), text))
    }
}
