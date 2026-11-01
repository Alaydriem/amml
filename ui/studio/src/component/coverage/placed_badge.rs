use amml_i18n::text::Text;
use amml_ui::tokens::{SpacingToken, spacing};
use egui::{Painter, Rect, pos2};

use super::{BadgePlacement, CoverageBadge};
use crate::tokens::selector::StatusColor;

/// Paper: Widgets, `Widget / Status badge + Placed badge` (`42V-0`), section "RIGHT / ABOVE / BELOW". A coverage
/// badge placed against the control it describes, `--spacing-xs` from it.
pub struct PlacedBadge {
    badge: CoverageBadge,
    placement: BadgePlacement,
}

impl PlacedBadge {
    /// Paper: the gap between the control and the badge, `--spacing-xs`.
    pub const GAP: f32 = <spacing::Xs as SpacingToken>::VALUE;

    pub fn new(status: StatusColor, label: Text, placement: BadgePlacement) -> Self {
        Self { badge: CoverageBadge::new(status, label), placement }
    }

    /// Paints the badge against `control` and returns the badge's rect.
    pub fn paint_on(&self, painter: &Painter, control: Rect) -> Rect {
        let size = self.badge.size(painter.ctx());
        let min = match self.placement {
            BadgePlacement::Right => pos2(control.right() + Self::GAP, control.center().y - size.y / 2.0),
            BadgePlacement::Above => pos2(control.left(), control.top() - Self::GAP - size.y),
            BadgePlacement::Below => pos2(control.left(), control.bottom() + Self::GAP),
        };
        let rect = Rect::from_min_size(min, size);
        self.badge.paint(painter, rect);
        rect
    }
}
