use amml_ui::draw::{Corner, Draw};
use amml_ui::tokens::{ColorToken, color};
use egui::{Painter, Rect};

use super::{LaneKey, TimeAxis};

/// Paper: `Box select, multi-select, lock` (`7J7-0`). The rect dragged over the lanes, and the keys it takes in.
pub struct BoxSelect {
    rect: Rect,
}

impl BoxSelect {
    // Paper: the box is `--color-accent` at 12 % with a `--color-accent-hover` border
    const FILL_ALPHA: u8 = 0x1F;

    pub fn new(rect: Rect) -> Self {
        Self { rect }
    }

    /// The `(lane index, key index)` of every key whose centre is in the box, lane by lane and in key order.
    pub fn pick(&self, axis: &TimeAxis, lanes: &[(Rect, &[LaneKey])]) -> Vec<(usize, usize)> {
        let mut picked = Vec::new();
        for (lane, (row, keys)) in lanes.iter().enumerate() {
            let y = row.center().y;
            if y < self.rect.top() || y > self.rect.bottom() {
                continue;
            }
            for (index, key) in keys.iter().enumerate() {
                let x = axis.x_of(key.seconds);
                if x >= self.rect.left() && x <= self.rect.right() {
                    picked.push((lane, index));
                }
            }
        }
        picked
    }

    pub fn paint(&self, painter: &Painter) {
        let fill = <color::Accent as ColorToken>::VALUE.gamma_multiply_u8(Self::FILL_ALPHA);
        Draw::fill(painter, self.rect, Corner::Small, fill);
        Draw::stroke(painter, self.rect, Corner::Small, <color::AccentHover as ColorToken>::VALUE);
    }
}
