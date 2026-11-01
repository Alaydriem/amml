use amml_ui::font::FontSource;
use amml_ui::icon::{Icon, IconShape};
use amml_ui::theme::Theme;
use amml_ui::tokens::rule::SoftFill;
use amml_ui::tokens::{ColorToken, color};
use amml_ui::widget::{Banner, BannerTone, ButtonKind, KeyChord, TextButton};
use egui::{Color32, Rect, Shape, vec2};
use egui_kittest::Harness;

use crate::support::{FlowProbe, PaintCapture, Sample};

// shows one banner in a 1000 x 120 screen
struct BannerProbe;

impl BannerProbe {
    fn shapes(banner: impl Fn() -> Banner<'static> + 'static) -> Vec<Shape> {
        let mut harness: Harness<'static, ()> = Harness::builder()
            .with_size(vec2(1000.0, 120.0))
            .allow_missing_glyphs()
            .with_context_setup(|ctx| Theme::install(ctx, FontSource::BuiltIn))
            .build_ui_state(
                move |ui, _: &mut ()| {
                    banner().show(ui);
                },
                (),
            );
        harness.run();
        harness.output().shapes.iter().map(|clipped| clipped.shape.clone()).collect()
    }

    fn glyph(icon: Icon) -> String {
        let IconShape::Font(code) = icon.shape() else { panic!("{icon:?} is not a font glyph") };
        code.to_string()
    }

    // (rect, colour) of the first text shape equal to `text`
    fn text(shapes: &[Shape], text: &str) -> Option<(Rect, Color32)> {
        shapes.iter().find_map(|shape| match shape {
            Shape::Text(shape) if shape.galley.text() == text => {
                Some((shape.galley.rect.translate(shape.pos.to_vec2()), shape.galley.job.sections[0].format.color))
            }
            _ => None,
        })
    }
}

#[test]
fn tones_match_the_sheet() {
    // (tone, fill, border, icon, icon and title colour); Paper: Banner sheet
    let tones = [
        (BannerTone::Info, color::BgApp::VALUE, color::Line::VALUE, Icon::Info, color::TextDim::VALUE),
        (BannerTone::Warning, color::WarningSoft::VALUE, color::Warning::VALUE, Icon::Warning, color::Warning::VALUE),
        (BannerTone::Danger, color::DangerSoft::VALUE, color::Danger::VALUE, Icon::Blocked, color::Danger::VALUE),
        (
            BannerTone::Success,
            SoftFill::of(color::Success::VALUE),
            color::Success::VALUE,
            Icon::Check,
            color::Success::VALUE,
        ),
    ];
    for (tone, fill, border, icon, colour) in tones {
        let shapes = BannerProbe::shapes(move || Banner::new(tone, Sample::text("No settings")));
        let rects = PaintCapture::rects(&shapes);
        let Some(body) = rects.iter().find(|r| r.fill == fill) else { panic!("{tone:?}: no fill") };
        assert!(
            rects.iter().any(|r| r.stroke.color == border && r.rect == body.rect),
            "{tone:?}: no border in the tone colour"
        );
        let Some((_, icon_colour)) = BannerProbe::text(&shapes, &BannerProbe::glyph(icon)) else {
            panic!("{tone:?}: no {icon:?} icon")
        };
        assert_eq!(icon_colour, colour, "{tone:?}: icon colour");
        let title = if tone == BannerTone::Info { color::Text::VALUE } else { colour };
        assert_eq!(BannerProbe::text(&shapes, "No settings").map(|(_, c)| c), Some(title), "{tone:?}: title colour");
    }
}

#[test]
fn chord_line_has_caps_and_tail() {
    let shapes = BannerProbe::shapes(|| {
        Banner::new(BannerTone::Warning, Sample::text("1 key conflict.")).chord(
            KeyChord::new(vec![Sample::text("Ctrl"), Sample::text("B")]),
            Sample::text("is also bound by Light Level."),
        )
    });
    let part = |text: &str| match BannerProbe::text(&shapes, text) {
        Some((rect, colour)) => (rect, colour),
        None => panic!("no {text:?}"),
    };
    let (lead, _) = part("1 key conflict.");
    let (ctrl, _) = part("Ctrl");
    let (add, _) = part(&BannerProbe::glyph(Icon::Add));
    let (b, _) = part("B");
    let (tail, tail_colour) = part("is also bound by Light Level.");
    assert_eq!(tail_colour, color::Text::VALUE, "the tail is not in the text colour");
    let line = [lead, ctrl, add, b, tail];
    for pair in line.windows(2) {
        assert!(pair[0].right() <= pair[1].left(), "the line is out of order: {line:?}");
    }
    for rect in line {
        assert!((rect.center().y - lead.center().y).abs() <= 1.0, "the line is not on one row: {line:?}");
    }
    // the caps are drawn boxes, not "+" in a string
    let caps = PaintCapture::rects(&shapes).into_iter().filter(|r| r.fill == color::BgField::VALUE).count();
    assert_eq!(caps, 2, "two key caps");
}

#[test]
fn a_long_chord_line_is_cut_inside_the_banner() {
    let tail = "is also bound by a mod with a very long name that runs well past the right edge of this banner, \
                and then some more words, so no screen this narrow can hold it.";
    let shapes = BannerProbe::shapes(move || {
        Banner::new(BannerTone::Warning, Sample::text("1 key conflict."))
            .chord(KeyChord::new(vec![Sample::text("Ctrl"), Sample::text("B")]), Sample::text(tail))
    });
    let banner = PaintCapture::rects(&shapes)
        .into_iter()
        .find(|r| r.fill == color::WarningSoft::VALUE)
        .map(|r| r.rect)
        .unwrap_or_else(|| panic!("no banner fill"));

    let right = PaintCapture::text_rects(&shapes).into_iter().map(|(_, rect)| rect.right()).fold(0.0, f32::max);
    assert!(right <= banner.right(), "text ends at {right}, past the banner's edge {}", banner.right());
}

#[test]
fn action_does_not_pull_the_next_widget_up() {
    let log = FlowProbe::run(|ui| {
        Banner::new(BannerTone::Danger, Sample::text("Light Level did not load"))
            .body(Sample::text("It needs LeviLamina 1.5.0 or later."))
            .action(TextButton::new(ButtonKind::Secondary, Sample::text("Show log")))
            .show(ui);
    });
    FlowProbe::assert_below(&log);
}
