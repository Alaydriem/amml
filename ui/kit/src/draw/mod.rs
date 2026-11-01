mod corner;
mod letter_spacing;
mod text_fit;
mod text_style;
mod type_size;

pub use corner::Corner;
pub use letter_spacing::LetterSpacing;
pub use text_fit::TextFit;
pub use text_style::TextStyle;
pub use type_size::TypeSize;

use std::sync::Arc;

use amml_i18n::text::Text;
use egui::text::{LayoutJob, TextFormat};
use egui::{Align, Align2, Color32, Galley, Margin, Painter, Pos2, Rangef, Rect, Stroke, StrokeKind, pos2};

use crate::text::UiText;
use crate::tokens::rule::Elevation;
use crate::tokens::{SpacingToken, spacing};

/// The painting helpers that every widget shares. Each one takes token selectors, or colours read
/// from tokens, never raw sizes.
pub struct Draw;

impl Draw {
    pub fn fill(painter: &Painter, rect: Rect, corner: Corner, color: Color32) {
        painter.rect_filled(rect, corner.radius(rect), color);
    }

    /// A `--spacing-border` outline inside `rect`.
    pub fn stroke(painter: &Painter, rect: Rect, corner: Corner, color: Color32) {
        let stroke = Stroke::new(<spacing::Border as SpacingToken>::VALUE, color);
        painter.rect_stroke(rect, corner.radius(rect), stroke, StrokeKind::Inside);
    }

    /// A `--spacing-divider` horizontal line, sharp on the pixel grid.
    pub fn divider(painter: &Painter, x: Rangef, y: f32, color: Color32) {
        let width = <spacing::Divider as SpacingToken>::VALUE;
        let y = y.floor() + width / 2.0;
        painter.line_segment([pos2(x.min, y), pos2(x.max, y)], Stroke::new(width, color));
    }

    /// A surface: its shadow (when it floats), its fill, then its border.
    pub fn surface(
        painter: &Painter,
        rect: Rect,
        corner: Corner,
        fill: Color32,
        border: Option<Color32>,
        elevation: Elevation,
    ) {
        let radius = corner.radius(rect);
        if elevation != Elevation::Flat {
            painter.add(elevation.shadow().as_shape(rect, radius));
        }
        painter.rect_filled(rect, radius, fill);
        if let Some(border) = border {
            Self::stroke(painter, rect, corner, border);
        }
    }

    /// Paints `text`, resolved in the context's locale, anchored at `pos`. Returns its rect.
    pub fn text(
        painter: &Painter,
        pos: Pos2,
        anchor: Align2,
        text: &Text,
        style: TextStyle,
        color: Color32,
        fit: TextFit,
    ) -> Rect {
        let resolved = UiText::resolve(painter.ctx(), text);
        let galley = Self::galley(painter, &resolved, style, color, fit);
        let rect = anchor.anchor_size(pos, galley.size());
        painter.galley(rect.min, galley, color);
        rect
    }

    /// A frame margin from token values, `x` left and right and `y` top and bottom.
    pub fn margin(x: f32, y: f32) -> Margin {
        // Clamped to the i8 range first, so the casts cannot truncate.
        let x = x.round().clamp(0.0, 127.0) as i8;
        let y = y.round().clamp(0.0, 127.0) as i8;
        Margin { left: x, right: x, top: y, bottom: y }
    }

    /// A `--spacing-border` stroke in `color`.
    pub fn border(color: Color32) -> Stroke {
        Stroke::new(<spacing::Border as SpacingToken>::VALUE, color)
    }

    /// The size `text` takes on one line in `style`, from the context's fonts.
    pub fn measure(ctx: &egui::Context, text: &str, style: TextStyle) -> egui::Vec2 {
        ctx.fonts_mut(|fonts| fonts.layout_job(Self::job(text, style, Color32::WHITE, TextFit::Line)).size())
    }

    /// Lays out `text` in `style`, fitted as `fit` says.
    pub fn galley(painter: &Painter, text: &str, style: TextStyle, color: Color32, fit: TextFit) -> Arc<Galley> {
        painter.layout_job(Self::job(text, style, color, fit))
    }

    /// Lays out `text` in `style`, wrapped at `width`, each row centred on x = 0. Paint it at the
    /// top centre of its place.
    pub fn galley_centred(painter: &Painter, text: &str, style: TextStyle, color: Color32, width: f32) -> Arc<Galley> {
        let mut job = Self::job(text, style, color, TextFit::Wrap(width));
        job.halign = Align::Center;
        painter.layout_job(job)
    }

    fn job(text: &str, style: TextStyle, color: Color32, fit: TextFit) -> LayoutJob {
        let size = style.size.points();
        let format = TextFormat {
            font_id: style.font_id(),
            color,
            extra_letter_spacing: style.spacing.points(size),
            line_height: Some(style.size.line_height()),
            ..Default::default()
        };
        let mut job = LayoutJob::single_section(text.to_owned(), format);
        job.wrap = fit.wrapping();
        job
    }
}
