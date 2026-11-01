use amml_ui::tokens::rule::SoftFill;
use amml_ui::tokens::{ColorToken, color};
use amml_ui::widget::{Carousel, CarouselItem, CarouselMark};
use egui::{Color32, Rect, Shape, pos2, vec2};

use crate::support::{PaintCapture, Sample};

struct CarouselPaint;

impl CarouselPaint {
    const TONE: Color32 = Color32::from_rgb(0xE0, 0x6C, 0x75);
    const WIDTH: f32 = 360.0;

    fn items() -> Vec<CarouselItem> {
        ["Move X", "Move Y", "Move Z", "Layer"]
            .into_iter()
            .zip(["X", "Y", "Z", "L"])
            .map(|(name, letter)| CarouselItem {
                mark: CarouselMark::Letter(Sample::text(letter), Self::TONE),
                name: Sample::text(name),
                value: Sample::text("1 block"),
            })
            .collect()
    }

    fn shapes(items: &[CarouselItem], current: usize) -> (Rect, Vec<Shape>) {
        let mut rect = Rect::NOTHING;
        let shapes = PaintCapture::shapes(|painter| {
            rect = Carousel::new(items, current).paint(painter, pos2(20.0, 20.0), Self::WIDTH)
        });
        (rect, shapes)
    }

    // the two 36 x 36 side slots, left then right
    fn slots(shapes: &[Shape]) -> Vec<Rect> {
        let mut slots: Vec<Rect> = PaintCapture::rects(shapes)
            .into_iter()
            .filter(|r| r.rect.size() == vec2(36.0, 36.0))
            .map(|r| r.rect)
            .collect();
        slots.sort_by(|a, b| a.left().total_cmp(&b.left()));
        slots
    }

    // the text painted inside `area`, left to right
    fn text_in(shapes: &[Shape], area: Rect) -> Vec<String> {
        let mut found: Vec<(f32, String)> = PaintCapture::text_rects(shapes)
            .into_iter()
            .filter(|(_, rect)| area.contains_rect(*rect))
            .map(|(text, rect)| (rect.left(), text))
            .collect();
        found.sort_by(|a, b| a.0.total_cmp(&b.0));
        found.into_iter().map(|(_, text)| text).collect()
    }

    // the dots under the row, left to right
    fn dots(shapes: &[Shape]) -> Vec<(Rect, Color32)> {
        let mut dots: Vec<(Rect, Color32)> = PaintCapture::rects(shapes)
            .into_iter()
            .filter(|r| r.rect.height() == 5.0)
            .map(|r| (r.rect, r.fill))
            .collect();
        dots.sort_by(|a, b| a.0.left().total_cmp(&b.0.left()));
        dots
    }
}

#[test]
fn current_item_is_in_the_middle() {
    let items = CarouselPaint::items();
    let (_, shapes) = CarouselPaint::shapes(&items, 1);
    let slots = CarouselPaint::slots(&shapes);
    assert_eq!(slots.len(), 2);
    let active = PaintCapture::rects(&shapes)
        .into_iter()
        .find(|r| r.fill == color::BgActive::VALUE)
        .map(|r| r.rect)
        .unwrap_or(Rect::NOTHING);
    assert!(slots[0].right() < active.left(), "{slots:?} {active:?}");
    assert!(active.right() < slots[1].left(), "{slots:?} {active:?}");
    // the current item shows its mark, name and value; the neighbours show only their mark
    // and the two arrows of the change hint, drawn from the icon font
    let inside = CarouselPaint::text_in(&shapes, active);
    assert_eq!(inside[..3], ["Y", "Move Y", "1 block"]);
    assert_eq!(inside.len(), 5);
    assert_eq!(CarouselPaint::text_in(&shapes, slots[0]), ["X"]);
    assert_eq!(CarouselPaint::text_in(&shapes, slots[1]), ["Z"]);
    let tile_fill = SoftFill::of(CarouselPaint::TONE);
    assert!(PaintCapture::rects(&shapes).iter().any(|r| r.fill == tile_fill && r.rect.size() == vec2(40.0, 40.0)));
}

#[test]
fn neighbours_wrap_at_the_ends() {
    let items = CarouselPaint::items();
    let (_, first) = CarouselPaint::shapes(&items, 0);
    let slots = CarouselPaint::slots(&first);
    assert_eq!(CarouselPaint::text_in(&first, slots[0]), ["L"]);
    assert_eq!(CarouselPaint::text_in(&first, slots[1]), ["Y"]);

    let (_, last) = CarouselPaint::shapes(&items, 3);
    let slots = CarouselPaint::slots(&last);
    assert_eq!(CarouselPaint::text_in(&last, slots[0]), ["Z"]);
    assert_eq!(CarouselPaint::text_in(&last, slots[1]), ["X"]);
}

#[test]
fn dots_count_the_items_and_mark_the_current() {
    let items = CarouselPaint::items();
    for current in 0..items.len() {
        let (rect, shapes) = CarouselPaint::shapes(&items, current);
        let dots = CarouselPaint::dots(&shapes);
        assert_eq!(dots.len(), items.len());
        for (index, (dot, fill)) in dots.iter().enumerate() {
            if index == current {
                assert_eq!((dot.width(), *fill), (14.0, color::Accent::VALUE));
            } else {
                assert_eq!((dot.width(), *fill), (5.0, color::LineStrong::VALUE));
            }
        }
        // the dots are centred under the row
        let middle = (dots[0].0.left() + dots[items.len() - 1].0.right()) / 2.0;
        assert!((middle - rect.center().x).abs() < 0.5, "{middle} {}", rect.center().x);
    }
}
