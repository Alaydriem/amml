use egui::epaint::RectShape;
use egui::{Color32, CornerRadius, Painter, Pos2, Rect, Stroke, StrokeKind, Vec2};

use super::KeyLook;
use crate::tokens::{ColorToken, color};

/// The parts every key glyph shares: the halo behind a selected key and the outline of the mark.
pub(crate) struct KeyMark;

impl KeyMark {
    /// Paper: the halo circle behind a selected key, radius 11.
    pub(crate) const HALO_RADIUS: f32 = 11.0;
    // Paper: the halo is the track colour at alpha 0x38 (about 22 %)
    const HALO_ALPHA: u8 = 0x38;
    // Paper: a key at rest has a 1 px `--color-bg-app` outline, a selected key a 1.8 px white one
    const REST_OUTLINE: f32 = 1.0;
    const SELECTED_OUTLINE: f32 = 1.8;

    /// Paints the halo of a selected key; a key at rest has none.
    pub(crate) fn halo(painter: &Painter, centre: Pos2, color: Color32, look: KeyLook) {
        if look == KeyLook::Selected {
            painter.circle_filled(centre, Self::HALO_RADIUS, color.gamma_multiply_u8(Self::HALO_ALPHA));
        }
    }

    /// The outline of the mark in `look`.
    pub(crate) fn outline(look: KeyLook) -> Stroke {
        match look {
            KeyLook::Rest => Stroke::new(Self::REST_OUTLINE, <color::BgApp as ColorToken>::VALUE),
            KeyLook::Selected => Stroke::new(Self::SELECTED_OUTLINE, <color::OnAccent as ColorToken>::VALUE),
        }
    }

    /// A square turned 45 degrees with its corners `half` from `centre`. A turned rect needs no point list, so a
    /// diamond allocates nothing.
    pub(crate) fn diamond(painter: &Painter, centre: Pos2, half: f32, fill: Color32, stroke: Stroke) {
        let rect = Rect::from_center_size(centre, Vec2::splat(half * std::f32::consts::SQRT_2));
        let shape = RectShape::new(rect, CornerRadius::ZERO, fill, stroke, StrokeKind::Middle)
            .with_round_to_pixels(false)
            .with_angle(std::f32::consts::FRAC_PI_4);
        painter.add(shape);
    }
}
