use amml_i18n::text::Text;
use amml_ui::draw::{Corner, Draw, TextFit, TextStyle, TypeSize};
use amml_ui::font::FontFace;
use amml_ui::icon::{Icon, IconSize};
use amml_ui::text::UiText;
use amml_ui::tokens::rule::Elevation;
use amml_ui::tokens::{ColorToken, color};
use egui::{Rect, Response, Sense, Ui, pos2, vec2};

use crate::tokens::color as studio_color;

/// Paper: Editor components, `Editor / Edit lock banner` (`G7V-0`). The pill at the bottom of the viewer that says
/// why edits are locked while the scene camera drives: a lock and the reason, in the scene camera colour on a faded
/// app background with a border of the same colour.
pub struct EditLockBanner {
    reason: Text,
}

impl EditLockBanner {
    /// Paper: Edit lock banner sheet, the pill height.
    const HEIGHT: f32 = 32.0;
    /// Paper: Edit lock banner sheet, how much of the app background shows through.
    const FADE: f32 = 0.92;
    /// Paper: Edit lock banner sheet, the gap between the lock and the reason.
    const GAP: f32 = 10.0;
    const REASON: TextStyle = TextStyle::new(TypeSize::Note, FontFace::Semibold);

    pub fn new(reason: Text) -> Self {
        Self { reason }
    }

    /// Shows the pill `width` wide.
    pub fn show(&self, ui: &mut Ui, width: f32) -> Response {
        let (rect, response) = ui.allocate_exact_size(vec2(width, Self::HEIGHT), Sense::hover());
        self.paint(ui, rect);
        response
    }

    fn paint(&self, ui: &Ui, rect: Rect) {
        let painter = ui.painter();
        let ink = <studio_color::SceneCamera as ColorToken>::VALUE;
        let fill = <color::BgApp as ColorToken>::VALUE.gamma_multiply(Self::FADE);
        Draw::surface(painter, rect, Corner::Control, fill, Some(ink), Elevation::Flat);
        let reason = UiText::resolve(ui.ctx(), &self.reason);
        let galley = Draw::galley(painter, &reason, Self::REASON, ink, TextFit::Truncate(rect.width()));
        let icon = IconSize::Small.points();
        let group = icon + Self::GAP + galley.size().x;
        let left = rect.center().x - group / 2.0;
        let slot = Rect::from_center_size(pos2(left + icon / 2.0, rect.center().y), vec2(icon, icon));
        Icon::Lock.paint(painter, slot, IconSize::Small, ink);
        painter.galley(pos2(left + icon + Self::GAP, rect.center().y - galley.size().y / 2.0), galley, ink);
    }
}
