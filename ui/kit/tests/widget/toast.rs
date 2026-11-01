use amml_ui::font::FontSource;
use amml_ui::icon::{Icon, IconShape};
use amml_ui::theme::Theme;
use amml_ui::tokens::rule::SoftFill;
use amml_ui::tokens::{ColorToken, color};
use amml_ui::widget::{Toast, ToastTone};
use egui::{Rect, Shape, vec2};
use egui_kittest::Harness;

use crate::support::{FlowProbe, PaintCapture, Sample, ShapeText};

// shows one toast in a 600 x 200 screen and keeps whether it was dismissed
struct ToastProbe;

impl ToastProbe {
    // Paper: Toast sheet, the countdown track height
    const TRACK: f32 = 2.0;

    fn harness(toast: impl Fn() -> Toast + 'static) -> Harness<'static, bool> {
        let mut harness = Harness::builder()
            .with_size(vec2(600.0, 200.0))
            .allow_missing_glyphs()
            .with_context_setup(|ctx| Theme::install(ctx, FontSource::BuiltIn))
            .build_ui_state(move |ui, dismissed: &mut bool| *dismissed |= toast().show(ui).dismissed, false);
        harness.run();
        harness
    }

    fn shapes(harness: &Harness<'static, bool>) -> Vec<Shape> {
        harness.output().shapes.iter().map(|clipped| clipped.shape.clone()).collect()
    }

    fn glyph(icon: Icon) -> String {
        let IconShape::Font(code) = icon.shape() else { panic!("{icon:?} is not a font glyph") };
        code.to_string()
    }

    // (text rect, colour) of the first text shape equal to `text`
    fn text(shapes: &[Shape], text: &str) -> Option<(Rect, egui::Color32)> {
        shapes.iter().find_map(|shape| match shape {
            Shape::Text(shape) if shape.galley.text() == text => {
                Some((shape.galley.rect.translate(shape.pos.to_vec2()), shape.galley.job.sections[0].format.color))
            }
            _ => None,
        })
    }

    // (track, fill) bars of the countdown: the `--color-line` track and the bar on top of it
    fn countdown(shapes: &[Shape], tone: ToastTone) -> (Rect, Option<Rect>) {
        let bars: Vec<_> =
            PaintCapture::rects(shapes).into_iter().filter(|r| (r.rect.height() - Self::TRACK).abs() < 0.01).collect();
        let track = bars.iter().find(|r| r.fill == color::Line::VALUE).map(|r| r.rect);
        let Some(track) = track else { panic!("no countdown track") };
        let fill = bars.iter().find(|r| r.fill == tone.color()).map(|r| r.rect);
        (track, fill)
    }
}

#[test]
fn age_reads_now_then_seconds() {
    for (age, expected) in [(999, "now"), (3200, "3 s")] {
        let harness =
            ToastProbe::harness(move || Toast::new(ToastTone::Info, Sample::text("Settings saved.")).age_ms(age));
        let texts = ShapeText::all(&ToastProbe::shapes(&harness));
        assert!(texts.iter().any(|text| text == expected), "{age} ms: {texts:?}");
    }
}

#[test]
fn countdown_width_follows_remaining_and_clamps() {
    let bar = |remaining: f32| {
        let harness = ToastProbe::harness(move || {
            Toast::new(ToastTone::Warning, Sample::text("New key conflict.")).remaining(remaining)
        });
        ToastProbe::countdown(&ToastProbe::shapes(&harness), ToastTone::Warning)
    };
    let (track, fill) = bar(0.5);
    let Some(fill) = fill else { panic!("no countdown fill at a half") };
    assert_eq!(fill.left(), track.left());
    assert!((fill.width() - track.width() / 2.0).abs() <= 0.5, "{fill:?} on {track:?}");
    let (track, fill) = bar(1.5);
    assert_eq!(fill.map(|fill| fill.width()), Some(track.width()), "1.5 is not clamped to the full track");
    let (_, fill) = bar(-1.0);
    assert!(fill.is_none_or(|fill| fill.width() <= 0.0), "-1 is not clamped to an empty bar: {fill:?}");
}

#[test]
fn dismiss_button_only_when_dismissable() {
    let close = ToastProbe::glyph(Icon::Close);
    let plain = ToastProbe::harness(|| Toast::new(ToastTone::Success, Sample::text("Recording saved.")));
    assert!(!ShapeText::contains(&ToastProbe::shapes(&plain), &close), "a toast that is not dismissable has a close");

    let mut harness =
        ToastProbe::harness(|| Toast::new(ToastTone::Success, Sample::text("Recording saved.")).dismissable(true));
    let Some((button, _)) = ToastProbe::text(&ToastProbe::shapes(&harness), &close) else { panic!("no close button") };
    harness.hover_at(button.center());
    harness.run();
    harness.drag_at(button.center());
    harness.step();
    harness.drop_at(button.center());
    harness.run();
    assert!(*harness.state(), "a click on the close button did not dismiss the toast");
}

#[test]
fn tone_sets_tile_icon_and_bar() {
    let tones = [
        (ToastTone::Info, Icon::Info, color::Accent::VALUE, color::AccentSoft::VALUE),
        (ToastTone::Success, Icon::Check, color::Success::VALUE, SoftFill::of(color::Success::VALUE)),
        (ToastTone::Warning, Icon::Warning, color::Warning::VALUE, color::WarningSoft::VALUE),
        (ToastTone::Danger, Icon::Blocked, color::Danger::VALUE, color::DangerSoft::VALUE),
    ];
    for (tone, icon, colour, soft) in tones {
        assert_eq!(tone.color(), colour, "{tone:?}");
        let harness = ToastProbe::harness(move || Toast::new(tone, Sample::text("Camera held by DejaVu.")));
        let shapes = ToastProbe::shapes(&harness);
        let tile = PaintCapture::rects(&shapes).into_iter().find(|r| r.fill == soft);
        let Some(tile) = tile else { panic!("{tone:?}: no tile in the soft colour") };
        assert_eq!(tile.rect.width(), tile.rect.height(), "{tone:?}: the tile is not round");
        let Some((glyph, glyph_colour)) = ToastProbe::text(&shapes, &ToastProbe::glyph(icon)) else {
            panic!("{tone:?}: no {icon:?} icon")
        };
        assert_eq!(glyph_colour, colour, "{tone:?}: icon colour");
        assert!(tile.rect.contains(glyph.center()), "{tone:?}: the icon is not on the tile");
        let (_, fill) = ToastProbe::countdown(&shapes, tone);
        assert!(fill.is_some(), "{tone:?}: the countdown is not in the tone colour");
    }
}

#[test]
fn close_button_does_not_pull_the_next_widget_up() {
    let log = FlowProbe::run(|ui| {
        Toast::new(ToastTone::Info, Sample::text("Settings saved.")).dismissable(true).show(ui);
    });
    FlowProbe::assert_below(&log);
}
