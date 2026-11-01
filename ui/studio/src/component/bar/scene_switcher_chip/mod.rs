mod scene_switcher_chip_view;

pub use scene_switcher_chip_view::SceneSwitcherChipView;

use amml_i18n::text::{Args, Text};
use amml_ui::draw::{Corner, Draw, TextFit, TextStyle, TypeSize};
use amml_ui::font::FontFace;
use amml_ui::icon::{Icon, IconSize};
use amml_ui::state::InteractionState;
use amml_ui::text::UiText;
use amml_ui::tokens::{ColorToken, SpacingToken, color, spacing};
use egui::{Align2, Color32, CursorIcon, Rect, Response, Sense, Stroke, StrokeKind, Ui, pos2, vec2};

use super::BarControl;
use crate::message::StudioUiMessage;

/// Paper: Editor components, `Scene switcher chip (closed)` (`84X-0`). The active scene's dot, name and place,
/// with a chevron; a click opens the scene switcher. The Original replay has a neutral dot and no place.
pub struct SceneSwitcherChip<'a> {
    view: &'a SceneSwitcherChipView,
    max_width: f32,
}

impl<'a> SceneSwitcherChip<'a> {
    // Paper: padding 11 left and 12 right, parts 8 apart, a 9 px dot, the chevron 8 after the text
    const PADDING_LEFT: f32 = 11.0;
    const PADDING_RIGHT: f32 = 12.0;
    const DOT: f32 = 9.0;
    const CHEVRON_GAP: f32 = 8.0;
    // the border is the dot colour at 60 % (Paper `#8B7CFF99`-style borders)
    const BORDER_ALPHA: u8 = 0x99;
    const NAME: TextStyle = TextStyle::new(TypeSize::Help, FontFace::Semibold);
    const PLACE: TextStyle = TextStyle::new(TypeSize::Caption, FontFace::Regular);

    pub fn new(view: &'a SceneSwitcherChipView) -> Self {
        Self { view, max_width: f32::INFINITY }
    }

    /// The widest the chip may be. Past it the place drops first, then the name is cut.
    pub fn max_width(mut self, max_width: f32) -> Self {
        self.max_width = max_width;
        self
    }

    /// The chip's width in the context's locale.
    pub fn width_in(&self, ctx: &egui::Context) -> f32 {
        let name = UiText::resolve(ctx, &self.name());
        let place = self.place().map(|place| UiText::resolve(ctx, &place));
        self.width(ctx, &name, place.as_deref())
    }

    pub fn show(&self, ui: &mut Ui) -> Response {
        let ctx = ui.ctx().clone();
        let name = UiText::resolve(&ctx, &self.name());
        let place = self.place().map(|place| UiText::resolve(&ctx, &place));
        let size = vec2(self.width(&ctx, &name, place.as_deref()), BarControl::HEIGHT);
        let (rect, response) = ui.allocate_exact_size(size, Sense::click());
        let response = response.on_hover_cursor(CursorIcon::PointingHand);
        let state = InteractionState::of(&response, true);
        self.paint(ui, rect, &name, place.as_deref(), state);
        response
    }

    fn paint(&self, ui: &Ui, rect: Rect, name: &str, place: Option<&str>, state: InteractionState) {
        let painter = ui.painter();
        let dot = self.dot();
        let fill = match (self.view.open, state) {
            (true, _) | (_, InteractionState::Hover | InteractionState::Pressed) => {
                <color::BgHover as ColorToken>::VALUE
            }
            _ => <color::BgField as ColorToken>::VALUE,
        };
        let border = if self.view.open {
            <color::Accent as ColorToken>::VALUE
        } else {
            dot.gamma_multiply_u8(Self::BORDER_ALPHA)
        };
        Draw::fill(painter, rect, Corner::Control, fill);
        painter.rect_stroke(rect, Corner::Control.radius(rect), Stroke::new(1.0, border), StrokeKind::Inside);
        let gap = <spacing::S as SpacingToken>::VALUE;
        let mut x = rect.left() + Self::PADDING_LEFT;
        painter.circle_filled(pos2(x + Self::DOT / 2.0, rect.center().y), Self::DOT / 2.0, dot);
        x += Self::DOT + gap;
        let ink = <color::Text as ColorToken>::VALUE;
        let room = (rect.width() - Self::fixed()).max(0.0);
        let place = place.filter(|place| Self::natural(painter.ctx(), name, Some(place)) <= rect.width() + 0.5);
        let galley = Draw::galley(painter, name, Self::NAME, ink, TextFit::Truncate(room));
        x += galley.size().x;
        painter.galley(
            Align2::LEFT_CENTER.anchor_size(pos2(x - galley.size().x, rect.center().y), galley.size()).min,
            galley,
            ink,
        );
        if let Some(place) = place {
            let dim = <color::TextDim as ColorToken>::VALUE;
            let galley = Draw::galley(painter, place, Self::PLACE, dim, TextFit::Line);
            let at = Align2::LEFT_CENTER.anchor_size(pos2(x + gap, rect.center().y), galley.size());
            x += gap + galley.size().x;
            painter.galley(at.min, galley, dim);
        }
        let chevron = IconSize::Small.points();
        let icon = Rect::from_center_size(
            pos2(x + Self::CHEVRON_GAP + chevron / 2.0, rect.center().y),
            vec2(chevron, chevron),
        );
        Icon::ChevronDown.paint(painter, icon, IconSize::Small, <color::TextDim as ColorToken>::VALUE);
    }

    // the width the text needs, no more than `max_width` (but never less than the dot and chevron)
    fn width(&self, ctx: &egui::Context, name: &str, place: Option<&str>) -> f32 {
        Self::natural(ctx, name, place).min(self.max_width.max(Self::fixed()))
    }

    // the width with the whole name and place
    fn natural(ctx: &egui::Context, name: &str, place: Option<&str>) -> f32 {
        let gap = <spacing::S as SpacingToken>::VALUE;
        let name = Draw::measure(ctx, name, Self::NAME).x;
        let place = place.map_or(0.0, |place| gap + Draw::measure(ctx, place, Self::PLACE).x);
        Self::fixed() + name + place
    }

    // the width of the padding, dot and chevron around the text
    fn fixed() -> f32 {
        let gap = <spacing::S as SpacingToken>::VALUE;
        Self::PADDING_LEFT + Self::DOT + gap + Self::CHEVRON_GAP + IconSize::Small.points() + Self::PADDING_RIGHT
    }

    fn name(&self) -> Text {
        match &self.view.scene {
            Some((name, _)) => name.clone(),
            None => Text::message(StudioUiMessage::Original),
        }
    }

    fn place(&self) -> Option<Text> {
        let (index, count) = self.view.position.filter(|_| self.view.scene.is_some())?;
        let args = Args::new().uint("index", index as u64).uint("count", count as u64);
        Some(Text::format(StudioUiMessage::ScenePosition, args))
    }

    fn dot(&self) -> Color32 {
        match &self.view.scene {
            Some((_, scene)) => scene.color(),
            None => <color::TextDim as ColorToken>::VALUE,
        }
    }
}
