use amml_ui::tokens::{ColorToken, SpacingToken, TextSizeToken, TrackingToken, color, spacing, text_size, tracking};
use amml_ui::widget::SectionHeader;
use egui::{Rect, Shape, pos2, vec2};

use crate::support::{PaintCapture, Sample};

struct HeaderPaint;

impl HeaderPaint {
    // a header in an inspector column whose content is inset `--spacing-l` from the panel edge
    const RECT: Rect = Rect::from_min_max(pos2(56.0, 40.0), pos2(454.0, 76.0));

    fn shapes(header: &SectionHeader) -> Vec<Shape> {
        PaintCapture::shapes(|painter| header.paint(painter, Self::RECT))
    }
}

#[test]
fn divider_spans_the_parent() {
    let shapes = HeaderPaint::shapes(&SectionHeader::new(Sample::text("RANGE")).divider(true));
    let line = shapes.iter().find_map(|shape| match shape {
        Shape::LineSegment { points, stroke } if stroke.color == color::Line::VALUE => Some(*points),
        _ => None,
    });
    let Some([from, to]) = line else { panic!("no divider") };
    // edge to edge of the panel: the content rect plus the panel's `--spacing-l` inset
    let inset = <spacing::L as SpacingToken>::VALUE;
    assert_eq!((from.x, to.x), (HeaderPaint::RECT.left() - inset, HeaderPaint::RECT.right() + inset));
    assert!((from.y - HeaderPaint::RECT.top()).abs() <= 1.0, "the divider is not at the top: {from:?}");
    let plain = HeaderPaint::shapes(&SectionHeader::new(Sample::text("SCENE")));
    assert!(plain.iter().all(|shape| !matches!(shape, Shape::LineSegment { .. })), "a first section has no divider");
}

#[test]
fn title_is_tracked_caption_faint() {
    let shapes = HeaderPaint::shapes(&SectionHeader::new(Sample::text("CAMERA PATH")));
    let title = shapes.iter().find_map(|shape| match shape {
        Shape::Text(text) if text.galley.text() == "CAMERA PATH" => Some(text.clone()),
        _ => None,
    });
    let Some(title) = title else { panic!("no title") };
    let format = &title.galley.job.sections[0].format;
    let size = <text_size::Caption as TextSizeToken>::VALUE;
    assert_eq!(format.font_id.size, size);
    assert_eq!(format.color, color::TextFaint::VALUE);
    assert_eq!(format.extra_letter_spacing, <tracking::Section as TrackingToken>::VALUE * size);
    // the title sits in the `--spacing-row` band under the `--spacing-s` top space
    let top = HeaderPaint::RECT.top() + <spacing::S as SpacingToken>::VALUE;
    let band =
        Rect::from_min_size(pos2(HeaderPaint::RECT.left(), top), vec2(1.0, <spacing::Row as SpacingToken>::VALUE));
    let centre = title.pos.y + title.galley.size().y / 2.0;
    assert!((centre - band.center().y).abs() <= 1.0, "title centre {centre}, band centre {}", band.center().y);
}
