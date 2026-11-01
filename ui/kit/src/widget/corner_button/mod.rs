mod corner_button_lead;

pub use corner_button_lead::CornerButtonLead;

use amml_i18n::text::Text;
use egui::{Align2, Context, CursorIcon, Painter, Rect, Response, Sense, Ui, pos2, vec2};

use crate::draw::{Corner, Draw, TextFit, TextStyle, TypeSize};
use crate::font::FontFace;
use crate::icon::IconSize;
use crate::state::{InteractionState, StateStyle};
use crate::text::UiText;
use crate::tokens::rule::FocusRing;
use crate::tokens::{ColorToken, color};

/// Paper: Entry Point, `Components · capture rail & corner buttons`, row "Corner buttons". An accent button in the
/// corner of the start and pause screens: the square Mods button, or a mod's entry with its mark and a label.
pub struct CornerButton {
    lead: CornerButtonLead,
    label: Option<Text>,
}

impl CornerButton {
    /// Paper: Corner buttons, the height of both buttons and the side of the square one.
    pub const HEIGHT: f32 = 45.0;
    /// Paper: Corner buttons, the mark tile's side.
    const MARK: f32 = 26.0;
    /// Paper: Corner buttons, the labelled button's left and right padding, and the gap after the mark.
    const PAD_LEFT: f32 = 10.0;
    const PAD_RIGHT: f32 = 18.0;
    const GAP: f32 = 10.0;
    const MARK_TEXT: TextStyle = TextStyle::new(TypeSize::Caption, FontFace::Semibold);
    const LABEL: TextStyle = TextStyle::new(TypeSize::Title, FontFace::Semibold);

    pub fn new(lead: CornerButtonLead) -> Self {
        Self { lead, label: None }
    }

    /// A label after the lead; without one the button is a square.
    pub fn label(mut self, label: Text) -> Self {
        self.label = Some(label);
        self
    }

    /// The width `show` takes: the height for a square button, else the padding, the lead, the gap and the label.
    pub fn width(&self, ctx: &Context) -> f32 {
        match &self.label {
            None => Self::HEIGHT,
            Some(label) => {
                let label = Draw::measure(ctx, &UiText::resolve(ctx, label), Self::LABEL).x;
                (Self::PAD_LEFT + Self::MARK + Self::GAP + label + Self::PAD_RIGHT).ceil()
            }
        }
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        let (rect, response) = ui.allocate_exact_size(vec2(self.width(ui.ctx()), Self::HEIGHT), Sense::click());
        self.paint(ui.painter(), rect, InteractionState::of(&response, true));
        response.on_hover_cursor(CursorIcon::PointingHand)
    }

    pub fn paint(&self, painter: &Painter, rect: Rect, state: InteractionState) {
        if state == InteractionState::Focused {
            FocusRing::paint(painter, rect, Corner::Control.radius(rect));
        }
        let fill = StateStyle::Accent.fill(state.inner());
        let on_accent = <color::OnAccent as ColorToken>::VALUE;
        Draw::fill(painter, rect, Corner::Control, fill);
        let lead = match self.label {
            None => rect,
            Some(_) => Rect::from_center_size(
                pos2(rect.left() + Self::PAD_LEFT + Self::MARK / 2.0, rect.center().y),
                vec2(Self::MARK, Self::MARK),
            ),
        };
        match &self.lead {
            CornerButtonLead::Icon(icon) => icon.paint(painter, lead, IconSize::Regular, on_accent),
            CornerButtonLead::Mark(mark) => {
                let tile = Rect::from_center_size(lead.center(), vec2(Self::MARK, Self::MARK));
                Draw::fill(painter, tile, Corner::Control, on_accent);
                let mark = Text::user(mark.clone());
                Draw::text(painter, tile.center(), Align2::CENTER_CENTER, &mark, Self::MARK_TEXT, fill, TextFit::Line);
            }
        }
        if let Some(label) = &self.label {
            let left = pos2(lead.right() + Self::GAP, rect.center().y);
            let room = (rect.right() - Self::PAD_RIGHT - left.x).max(0.0);
            Draw::text(painter, left, Align2::LEFT_CENTER, label, Self::LABEL, on_accent, TextFit::Truncate(room));
        }
    }
}
