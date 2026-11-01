mod hero_card_action;
mod hero_card_view;
mod hero_picture;
mod hero_problem;

pub use hero_card_action::HeroCardAction;
pub use hero_card_view::HeroCardView;
pub use hero_picture::HeroPicture;
pub use hero_problem::HeroProblem;

use amml_ui::draw::{Corner, Draw, TextFit, TextStyle, TypeSize};
use amml_ui::font::FontFace;
use amml_ui::icon::{Icon, IconSize};
use amml_ui::state::{InteractionState, StateStyle};
use amml_ui::text::UiText;
use amml_ui::tokens::rule::FocusRing;
use amml_ui::tokens::{ColorToken, color};
use amml_ui::widget::MetaSeparator;
use egui::epaint::{Mesh, RectShape};
use egui::{Color32, Context, CornerRadius, CursorIcon, Painter, Rect, Sense, Ui, Vec2, pos2, vec2};

use crate::component::library::ReplayPlaceholder;
use crate::id::StudioUiId;
use crate::tokens::color as studio_color;

/// Paper: Editor components, `Editor / Hero card` (`G64-0`). The Library's big card: the replay's picture, a shade
/// rising from the bottom, the title and facts at the bottom left and the open button at the bottom right. A
/// broken replay shows a faint picture and a muted button that does nothing.
pub struct HeroCard<'a> {
    view: &'a HeroCardView,
    picture: Option<HeroPicture>,
}

impl<'a> HeroCard<'a> {
    /// Paper: Hero card sheet, the height of the shade.
    const SHADE: f32 = 170.0;
    /// Paper: Hero card sheet, where along the shade it has reached its full colour.
    const SHADE_FULL: f32 = 0.94;
    /// Paper: Hero card sheet, the room left of the title, and right of and under the parts.
    const INSET_LEFT: f32 = 26.0;
    const INSET_RIGHT: f32 = 24.0;
    const INSET_BOTTOM: f32 = 24.0;
    /// Paper: Hero card sheet, the gap between the title and the facts, and the facts' indent.
    const TITLE_GAP: f32 = 10.0;
    const META_INDENT: f32 = 2.0;
    /// Paper: Hero card sheet, the gap between a fact and the dot after it (the dot's own width is on top).
    const SEPARATOR_GAP: f32 = 27.0;
    /// Paper: Hero card sheet, the button size and the gap between its icon and its words.
    const BUTTON: Vec2 = vec2(200.0, 42.0);
    const BUTTON_GAP: f32 = 10.0;
    /// Paper: Hero card sheet, how much of the picture and of the button a broken card keeps.
    const BROKEN_PICTURE: f32 = 0.35;
    const BROKEN_BUTTON: f32 = 0.55;
    /// Paper: Editor components, `Editor / Hero card` (`G64-0`), BROKEN card, the `Problem (title + reason)` group
    /// (`IEH-0`): the gap between the problem's title and its reason, and the width of the group as a share of the
    /// card (630 of 900).
    const PROBLEM_GAP: f32 = 6.0;
    const PROBLEM_WIDTH: f32 = 0.7;
    const PROBLEM_TITLE: TextStyle = TextStyle::new(TypeSize::Heading, FontFace::Semibold);
    const PROBLEM_REASON: TextStyle = TextStyle::new(TypeSize::Body, FontFace::Regular);
    const TITLE: TextStyle = TextStyle::new(TypeSize::Display, FontFace::Semibold);
    const META: TextStyle = TextStyle::new(TypeSize::Note, FontFace::Regular);
    const LABEL: TextStyle = TextStyle::new(TypeSize::Label, FontFace::Semibold);

    pub fn new(view: &'a HeroCardView) -> Self {
        Self { view, picture: None }
    }

    /// The replay's picture. Without one the card shows the app background.
    pub fn picture(mut self, picture: HeroPicture) -> Self {
        self.picture = Some(picture);
        self
    }

    /// Shows the card `size` big; returns the action of the button when it was clicked.
    pub fn show(&self, ui: &mut Ui, size: Vec2) -> Option<HeroCardAction> {
        let (rect, frame) = ui.allocate_exact_size(size, Sense::hover());
        let painter = ui.painter().clone();
        self.paint_picture(ui.ctx(), &painter, rect);
        Self::paint_shade(&painter, rect);
        let words_top = self.paint_words(ui, rect);
        if let Some(problem) = &self.view.problem {
            Self::paint_problem(ui, rect, words_top, problem);
        }
        let button = Rect::from_min_size(
            pos2(
                rect.right() - Self::INSET_RIGHT - Self::BUTTON.x,
                rect.bottom() - Self::INSET_BOTTOM - Self::BUTTON.y,
            ),
            Self::BUTTON,
        );
        let enabled = self.view.problem.is_none();
        let sense = if enabled { Sense::click() } else { Sense::hover() };
        let response = ui.interact(button, frame.id.with(StudioUiId::HeroCard), sense);
        self.paint_button(ui, button, InteractionState::of(&response, enabled));
        if enabled {
            response.on_hover_cursor(CursorIcon::PointingHand).clicked().then_some(HeroCardAction::Open)
        } else {
            None
        }
    }

    fn paint_picture(&self, ctx: &Context, painter: &Painter, rect: Rect) {
        let radius = Corner::Panel.radius(rect);
        let opacity = if self.view.problem.is_some() { Self::BROKEN_PICTURE } else { 1.0 };
        let tint = Color32::WHITE.gamma_multiply(opacity);
        let picture = self.picture.unwrap_or_else(|| ReplayPlaceholder::picture(ctx));
        painter.add(RectShape::filled(rect, radius, tint).with_texture(picture.texture, picture.cover(rect.size())));
    }

    // transparent at the top of the shade, the full shade from `SHADE_FULL` down; the last corner radius is
    // a rounded strip so the shade keeps the card's corners
    fn paint_shade(painter: &Painter, rect: Rect) {
        let fade = <studio_color::PictureFade as ColorToken>::VALUE;
        let corner = Corner::Panel.radius(rect);
        let radius = f32::from(corner.sw);
        let top = rect.bottom() - Self::SHADE;
        let rows =
            [(top, Color32::TRANSPARENT), (top + Self::SHADE * Self::SHADE_FULL, fade), (rect.bottom() - radius, fade)];
        let mut mesh = Mesh::default();
        for (y, colour) in rows {
            mesh.colored_vertex(pos2(rect.left(), y), colour);
            mesh.colored_vertex(pos2(rect.right(), y), colour);
        }
        for row in 0..rows.len() as u32 - 1 {
            let at = row * 2;
            mesh.add_triangle(at, at + 1, at + 2);
            mesh.add_triangle(at + 1, at + 3, at + 2);
        }
        painter.add(mesh);
        let strip = Rect::from_min_max(pos2(rect.left(), rect.bottom() - radius), rect.max);
        painter.rect_filled(strip, CornerRadius { nw: 0, ne: 0, sw: corner.sw, se: corner.se }, fade);
    }

    // the title and the facts, bottom aligned with the button; returns the top of the title
    fn paint_words(&self, ui: &Ui, rect: Rect) -> f32 {
        let painter = ui.painter();
        let left = rect.left() + Self::INSET_LEFT;
        let title = UiText::resolve(ui.ctx(), &self.view.title);
        let ink = <color::OnPicture as ColorToken>::VALUE;
        let title = Draw::galley(painter, &title, Self::TITLE, ink, TextFit::Line);
        let facts_top = rect.bottom() - Self::INSET_BOTTOM - TypeSize::Note.line_height();
        let title_top = facts_top - Self::TITLE_GAP - title.size().y;
        painter.galley(pos2(left, title_top), title, ink);
        let words_top = title_top;
        let dim = <color::TextDim as ColorToken>::VALUE;
        let mut x = left + Self::META_INDENT;
        let last = self.view.meta.len().saturating_sub(1);
        for (index, fact) in self.view.meta.iter().enumerate() {
            let text = UiText::resolve(ui.ctx(), fact);
            let galley = Draw::galley(painter, &text, Self::META, dim, TextFit::Line);
            let width = galley.size().x;
            painter.galley(pos2(x, facts_top), galley, dim);
            x += width;
            if index != last {
                x += Self::SEPARATOR_GAP;
                let middle = facts_top + TypeSize::Note.line_height() / 2.0;
                MetaSeparator::paint(painter, pos2(x + MetaSeparator::SIDE / 2.0, middle));
                x += MetaSeparator::SIDE + Self::SEPARATOR_GAP;
            }
        }
        words_top
    }

    // the problem's title and reason, centred in the picture above the replay's own title
    fn paint_problem(ui: &Ui, rect: Rect, words_top: f32, problem: &HeroProblem) {
        let painter = ui.painter();
        let ink = <color::OnPicture as ColorToken>::VALUE;
        let dim = <color::TextDim as ColorToken>::VALUE;
        let width = rect.width() * Self::PROBLEM_WIDTH;
        let title = UiText::resolve(ui.ctx(), &problem.title);
        let title = Draw::galley_centred(painter, &title, Self::PROBLEM_TITLE, ink, width);
        let reason = UiText::resolve(ui.ctx(), &problem.reason);
        let reason = Draw::galley_centred(painter, &reason, Self::PROBLEM_REASON, dim, width);
        let height = title.size().y + Self::PROBLEM_GAP + reason.size().y;
        let top = rect.top() + (words_top - rect.top() - height) / 2.0;
        let centre = rect.center().x;
        let reason_top = top + title.size().y + Self::PROBLEM_GAP;
        painter.galley(pos2(centre, top), title, ink);
        painter.galley(pos2(centre, reason_top), reason, dim);
    }

    // the button; a broken card keeps `BROKEN_BUTTON` of its fill and words
    fn paint_button(&self, ui: &Ui, rect: Rect, state: InteractionState) {
        let painter = ui.painter();
        let broken = self.view.problem.is_some();
        let keep = |colour: Color32| if broken { colour.gamma_multiply(Self::BROKEN_BUTTON) } else { colour };
        if state == InteractionState::Focused {
            FocusRing::paint(painter, rect, Corner::Control.radius(rect));
        }
        let fill = if broken { <color::BgActive as ColorToken>::VALUE } else { StateStyle::Accent.fill(state.inner()) };
        Draw::fill(painter, rect, Corner::Control, keep(fill));
        let ink = keep(<color::OnAccent as ColorToken>::VALUE);
        let label = UiText::resolve(ui.ctx(), &self.view.open);
        let galley = Draw::galley(painter, &label, Self::LABEL, ink, TextFit::Truncate(rect.width()));
        let icon = IconSize::Regular.points();
        let group = icon + Self::BUTTON_GAP + galley.size().x;
        let left = rect.center().x - group / 2.0;
        let slot = Rect::from_center_size(pos2(left + icon / 2.0, rect.center().y), vec2(icon, icon));
        Icon::Play.paint(painter, slot, IconSize::Regular, ink);
        let at = pos2(left + icon + Self::BUTTON_GAP, rect.center().y - galley.size().y / 2.0);
        painter.galley(at, galley, ink);
    }
}
