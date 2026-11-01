use egui::epaint::Shadow;
use egui::{Color32, CornerRadius, Painter, Rect};

use crate::tokens::{ColorToken, SpacingToken, color, spacing};

/// The keyboard focus ring on every control: a gap in the surface colour, then the ring.
/// Built from `--spacing-focus-gap`, `--spacing-focus-ring` and `--color-focus-ring`.
pub struct FocusRing;

impl FocusRing {
    /// Two sharp shadows: `[0]` is the gap (spread `FocusGap`, in `surface`) and `[1]` is the ring
    /// (spread `FocusGap + FocusRing`, in `--color-focus-ring`). Paint the ring first, then the gap
    /// over it, so the gap shows between the control and the ring.
    pub fn shadows(surface: Color32) -> [Shadow; 2] {
        let gap = <spacing::FocusGap as SpacingToken>::VALUE;
        let ring = <spacing::FocusRing as SpacingToken>::VALUE;
        [
            Shadow { offset: [0, 0], blur: 0, spread: Self::points(gap), color: surface },
            Shadow {
                offset: [0, 0],
                blur: 0,
                spread: Self::points(gap + ring),
                color: <color::FocusRing as ColorToken>::VALUE,
            },
        ]
    }

    /// Paints the ring around `rect` (a control with `corner`) on a `--color-bg-panel` surface, as
    /// every Paper sheet draws it. Paint it before the control.
    pub fn paint(painter: &Painter, rect: Rect, corner: CornerRadius) {
        let [gap, ring] = Self::shadows(<color::BgPanel as ColorToken>::VALUE);
        painter.add(ring.as_shape(rect, corner));
        painter.add(gap.as_shape(rect, corner));
    }

    fn points(value: f32) -> u8 {
        // Clamped to 0..=255 first, so the cast cannot truncate.
        value.round().clamp(0.0, 255.0) as u8
    }
}
