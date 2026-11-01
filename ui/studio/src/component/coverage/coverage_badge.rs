use amml_i18n::text::Text;
use amml_ui::draw::{Corner, Draw, TextFit, TextStyle, TypeSize};
use amml_ui::font::FontFace;
use amml_ui::text::UiText;
use amml_ui::tokens::{ColorToken, SpacingToken, color, spacing};
use egui::{Painter, Rect, Response, Sense, Stroke, StrokeKind, Ui, Vec2, vec2};

use crate::tokens::selector::StatusColor;

/// Paper: Widgets, `Widget / Status badge + Placed badge` (`42V-0`), section "BADGE". A coverage status on a
/// `--color-pill` pill with a 1 px border and text in the status colour.
pub struct CoverageBadge {
    status: StatusColor,
    label: Text,
}

impl CoverageBadge {
    /// Paper: the badge is `--spacing-l` high.
    pub const HEIGHT: f32 = <spacing::L as SpacingToken>::VALUE;
    const STYLE: TextStyle = TextStyle::new(TypeSize::Micro, FontFace::Semibold);

    /// The label is a `Text`, so a raw string does not compile:
    ///
    /// ```compile_fail
    /// use amml_studio_ui::component::coverage::CoverageBadge;
    /// use amml_studio_ui::tokens::selector::StatusColor;
    ///
    /// CoverageBadge::new(StatusColor::Works, "Done");
    /// ```
    pub fn new(status: StatusColor, label: Text) -> Self {
        Self { status, label }
    }

    /// The badge's size: its label plus `--spacing-s` each side, `HEIGHT` high.
    pub fn size(&self, ctx: &egui::Context) -> Vec2 {
        let text = Draw::measure(ctx, &UiText::resolve(ctx, &self.label), Self::STYLE);
        vec2(text.x + Self::padding() * 2.0, Self::HEIGHT)
    }

    pub fn show(&self, ui: &mut Ui) -> Response {
        let (rect, response) = ui.allocate_exact_size(self.size(ui.ctx()), Sense::hover());
        self.paint(ui.painter(), rect);
        response
    }

    /// Paints the badge filling `rect`; a label longer than `rect` is cut.
    pub fn paint(&self, painter: &Painter, rect: Rect) {
        let hue = self.status.color();
        Draw::fill(painter, rect, Corner::Full, <color::Pill as ColorToken>::VALUE);
        painter.rect_stroke(rect, Corner::Full.radius(rect), Stroke::new(1.0, hue), StrokeKind::Inside);
        let fit = TextFit::Truncate((rect.width() - Self::padding() * 2.0).max(0.0));
        let galley = Draw::galley(painter, &UiText::resolve(painter.ctx(), &self.label), Self::STYLE, hue, fit);
        let text = Rect::from_center_size(rect.center(), galley.size());
        painter.galley(text.min, galley, hue);
    }

    fn padding() -> f32 {
        <spacing::S as SpacingToken>::VALUE
    }
}
