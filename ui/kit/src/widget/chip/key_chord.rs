use amml_i18n::text::Text;
use egui::{Context, Painter, Pos2, Rect, Response, Sense, Ui, pos2, vec2};

use super::{KeyCap, KeyCapLabel, KeyCapSize};
use crate::icon::{Icon, IconSize};
use crate::tokens::{ColorToken, SpacingToken, color, spacing};

/// Paper: Widgets, `Widget / Chip + Chip style`, "Key chord". Key caps joined by an ADD icon.
pub struct KeyChord {
    keys: Vec<Text>,
    size: KeyCapSize,
    conflict: bool,
}

impl KeyChord {
    pub fn new(keys: Vec<Text>) -> Self {
        Self { keys, size: KeyCapSize::Regular, conflict: false }
    }

    pub fn size(mut self, size: KeyCapSize) -> Self {
        self.size = size;
        self
    }

    /// Every cap in the danger look: the chord is used twice.
    pub fn conflict(mut self, conflict: bool) -> Self {
        self.conflict = conflict;
        self
    }

    pub fn desired_size(&self, ctx: &Context) -> egui::Vec2 {
        let caps: f32 = self.caps().map(|cap| cap.desired_width(ctx)).sum();
        let joins = self.keys.len().saturating_sub(1) as f32 * Self::join();
        vec2(caps + joins, self.size.height())
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        let (rect, response) = ui.allocate_exact_size(self.desired_size(ui.ctx()), Sense::hover());
        self.paint(ui.painter(), rect.left_center());
        response
    }

    /// Paints the chord with its left edge centred on `left_centre`. Returns its rect.
    pub fn paint(&self, painter: &Painter, left_centre: Pos2) -> Rect {
        let height = self.size.height();
        let mut x = left_centre.x;
        for (index, cap) in self.caps().enumerate() {
            if index > 0 {
                let slot = Rect::from_min_size(pos2(x, left_centre.y - height / 2.0), vec2(Self::join(), height));
                Icon::Add.paint(painter, slot, IconSize::Small, <color::TextFaint as ColorToken>::VALUE);
                x += Self::join();
            }
            let width = cap.desired_width(painter.ctx());
            cap.paint(painter, Rect::from_min_size(pos2(x, left_centre.y - height / 2.0), vec2(width, height)));
            x += width;
        }
        Rect::from_min_max(pos2(left_centre.x, left_centre.y - height / 2.0), pos2(x, left_centre.y + height / 2.0))
    }

    fn caps(&self) -> impl Iterator<Item = KeyCap> + '_ {
        self.keys.iter().map(|key| KeyCap {
            label: KeyCapLabel::Text(key.clone()),
            size: self.size,
            conflict: self.conflict,
        })
    }

    // the ADD icon slot between two caps
    fn join() -> f32 {
        <spacing::L as SpacingToken>::VALUE
    }
}
