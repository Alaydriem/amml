use amml_ui::draw::TypeSize;
use amml_ui::icon::{Icon, IconShape};
use amml_ui::state::InteractionState;
use amml_ui::tokens::{ColorToken, SpacingToken, color, spacing};
use amml_ui::widget::NavEntry;
use egui::{Color32, Rect, Shape, pos2};

use crate::support::{PaintCapture, Sample, ShapeText};

struct NavPaint;

impl NavPaint {
    const RECT: Rect = Rect::from_min_max(pos2(20.0, 20.0), pos2(446.0, 68.0));

    fn shapes(entry: &NavEntry, state: InteractionState) -> Vec<Shape> {
        PaintCapture::shapes(|painter| entry.paint(painter, Self::RECT, state))
    }

    // the colour of the text shape that draws `text`
    fn text_colour(shapes: &[Shape], text: &str) -> Option<Color32> {
        shapes.iter().find_map(|shape| match shape {
            Shape::Text(drawn) if drawn.galley.text() == text => Some(drawn.galley.job.sections[0].format.color),
            _ => None,
        })
    }

    // the colour of the text shape that draws `icon`
    fn icon_colour(shapes: &[Shape], icon: Icon) -> Option<Color32> {
        let IconShape::Font(code) = icon.shape() else { panic!("{icon:?} is not a font glyph") };
        shapes.iter().find_map(|shape| match shape {
            Shape::Text(text) if text.galley.text() == code.to_string() => {
                Some(text.galley.job.sections[0].format.color)
            }
            _ => None,
        })
    }
}

#[test]
fn selected_has_bar_and_active_fill() {
    let entry = NavEntry::new(Icon::Keyboard, Sample::text("Keybinds")).selected(true);
    let rects = PaintCapture::rects(&NavPaint::shapes(&entry, InteractionState::Rest));
    assert!(
        rects.iter().any(|r| r.rect == NavPaint::RECT && r.fill == color::BgActive::VALUE),
        "no active fill on the row"
    );
    let bar = rects.iter().find(|r| r.fill == color::Accent::VALUE);
    let Some(bar) = bar else { panic!("no selected bar") };
    assert_eq!(bar.rect.width(), <spacing::Indicator as SpacingToken>::VALUE);
    assert_eq!(bar.rect.left(), NavPaint::RECT.left());
    assert!((bar.rect.center().y - NavPaint::RECT.center().y).abs() <= 0.5, "the bar is not centred");

    let plain = NavEntry::new(Icon::Keyboard, Sample::text("Keybinds"));
    let rects = PaintCapture::rects(&NavPaint::shapes(&plain, InteractionState::Rest));
    assert!(rects.iter().all(|r| r.fill != color::Accent::VALUE), "a row that is not selected has a bar");
}

#[test]
fn count_shows_only_above_zero() {
    let none = NavEntry::new(Icon::Warning, Sample::text("Conflicts")).count(0);
    let shapes = NavPaint::shapes(&none, InteractionState::Rest);
    assert!(!ShapeText::all(&shapes).contains(&"0".to_owned()), "a zero count is shown");
    assert_eq!(NavPaint::icon_colour(&shapes, Icon::Warning), Some(color::TextDim::VALUE));

    let one = NavEntry::new(Icon::Warning, Sample::text("Conflicts")).count(1);
    let shapes = NavPaint::shapes(&one, InteractionState::Rest);
    assert!(ShapeText::all(&shapes).contains(&"1".to_owned()), "the count is not shown");
    assert_eq!(NavPaint::icon_colour(&shapes, Icon::Warning), Some(color::Warning::VALUE));
    let chip = PaintCapture::rects(&shapes).into_iter().find(|r| r.fill == color::WarningSoft::VALUE);
    let Some(chip) = chip else { panic!("no warning count chip") };
    assert!(chip.rect.right() <= NavPaint::RECT.right(), "the chip is outside the row");
    assert!(chip.rect.left() > NavPaint::RECT.center().x, "the chip is not at the right");
}

#[test]
fn hover_brightens_icon() {
    let entry = NavEntry::new(Icon::Settings, Sample::text("Manager settings"));
    let rest = NavPaint::icon_colour(&NavPaint::shapes(&entry, InteractionState::Rest), Icon::Settings);
    let hover = NavPaint::icon_colour(&NavPaint::shapes(&entry, InteractionState::Hover), Icon::Settings);
    assert_eq!(rest, Some(color::TextDim::VALUE));
    assert_eq!(hover, Some(color::Text::VALUE));
    let rects = PaintCapture::rects(&NavPaint::shapes(&entry, InteractionState::Hover));
    assert!(rects.iter().any(|r| r.rect == NavPaint::RECT && r.fill == color::BgHover::VALUE), "no hover fill");
}

#[test]
fn tally_draws_plain_number_and_keeps_icon() {
    let entry = NavEntry::new(Icon::Keyboard, Sample::text("Keybinds")).tally(4);
    let shapes = NavPaint::shapes(&entry, InteractionState::Rest);
    let Some((_, number)) = PaintCapture::text_rects(&shapes).into_iter().find(|(text, _)| text == "4") else {
        panic!("the tally is not shown")
    };
    let texts = PaintCapture::texts(&shapes);
    let Some((_, size, _)) = texts.iter().find(|(text, _, _)| text == "4") else { panic!("the tally is not shown") };
    assert_eq!(*size, TypeSize::Note.points());
    assert_eq!(NavPaint::text_colour(&shapes, "4"), Some(color::TextFaint::VALUE));
    assert!(
        (number.right() - (NavPaint::RECT.right() - <spacing::L as SpacingToken>::VALUE)).abs() <= 0.5,
        "the tally is not right aligned: {number:?}"
    );
    assert!((number.center().y - NavPaint::RECT.center().y).abs() <= 1.0, "the tally is not centred");
    let rects = PaintCapture::rects(&shapes);
    assert!(rects.iter().all(|r| r.fill != color::WarningSoft::VALUE), "the tally has a warning chip");
    assert_eq!(NavPaint::icon_colour(&shapes, Icon::Keyboard), Some(color::TextDim::VALUE));
    assert_eq!(NavPaint::icon_colour(&shapes, Icon::Warning), None);
}

#[test]
fn tally_draws_zero() {
    let entry = NavEntry::new(Icon::Folder, Sample::text("Unsorted")).tally(0);
    let shapes = NavPaint::shapes(&entry, InteractionState::Rest);
    assert!(ShapeText::all(&shapes).contains(&"0".to_owned()), "a zero tally is not shown");
    assert_eq!(NavPaint::text_colour(&shapes, "0"), Some(color::TextFaint::VALUE));
}

#[test]
fn tally_keeps_label_clear_of_the_number() {
    let entry =
        NavEntry::new(Icon::Folder, Sample::text("A very long project name that cannot fit in one row")).tally(12);
    let shapes = NavPaint::shapes(&entry, InteractionState::Rest);
    let rects = PaintCapture::text_rects(&shapes);
    let Some((_, number)) = rects.iter().find(|(text, _)| text == "12") else { panic!("no tally") };
    let Some((_, label)) = rects.iter().find(|(text, _)| text.starts_with("A very")) else { panic!("no label") };
    assert!(label.right() <= number.left(), "the label runs under the tally: {label:?} {number:?}");
}

#[test]
fn count_still_draws_warning() {
    let entry = NavEntry::new(Icon::Keyboard, Sample::text("Conflicts")).count(3);
    let shapes = NavPaint::shapes(&entry, InteractionState::Rest);
    assert!(ShapeText::all(&shapes).contains(&"3".to_owned()), "the count is not shown");
    assert_eq!(NavPaint::icon_colour(&shapes, Icon::Warning), Some(color::Warning::VALUE));
    assert!(PaintCapture::rects(&shapes).iter().any(|r| r.fill == color::WarningSoft::VALUE), "no warning chip");
    assert_eq!(NavPaint::icon_colour(&shapes, Icon::Keyboard), None);
}

#[test]
fn count_above_zero_takes_the_slot_from_the_tally() {
    let entry = NavEntry::new(Icon::Keyboard, Sample::text("Conflicts")).count(3).tally(9);
    let shapes = NavPaint::shapes(&entry, InteractionState::Rest);
    let texts = ShapeText::all(&shapes);
    assert!(texts.contains(&"3".to_owned()), "the count is not shown");
    assert!(!texts.contains(&"9".to_owned()), "the tally is shown beside the count");
    assert_eq!(NavPaint::icon_colour(&shapes, Icon::Warning), Some(color::Warning::VALUE));
    assert!(PaintCapture::rects(&shapes).iter().any(|r| r.fill == color::WarningSoft::VALUE), "no warning chip");

    let zero = NavEntry::new(Icon::Keyboard, Sample::text("Conflicts")).count(0).tally(9);
    let shapes = NavPaint::shapes(&zero, InteractionState::Rest);
    assert!(ShapeText::all(&shapes).contains(&"9".to_owned()), "the tally is hidden by a zero count");
    assert_eq!(NavPaint::icon_colour(&shapes, Icon::Keyboard), Some(color::TextDim::VALUE));
}
