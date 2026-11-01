use amml_i18n::text::UserText;
use amml_ui::draw::TypeSize;
use amml_ui::font::FontSource;
use amml_ui::icon::Icon;
use amml_ui::state::{DisabledReason, InteractionState};
use amml_ui::theme::Theme;
use amml_ui::tokens::{ColorToken, SpacingToken, color, spacing};
use amml_ui::widget::{Avatar, AvatarSize, Chip, ChipKind, ChipTone, IconButton, ListRow, Toggle};
use egui::{Color32, Pos2, Rect, Shape, vec2};
use egui_kittest::Harness;

use crate::support::{PaintCapture, Sample};

// what one list row is built from
#[derive(Clone)]
struct RowSample {
    name: &'static str,
    chips: &'static [&'static str],
    subtitle: Option<&'static str>,
    off: bool,
}

// what the row reported over a run
#[derive(Debug)]
struct RowLog {
    on: bool,
    selects: usize,
    toggles: usize,
    actions: usize,
    with_action: bool,
    action_off: bool,
    rect: Rect,
}

// shows one list row in a `width` wide screen
struct RowProbe;

impl RowProbe {
    fn harness(width: f32, sample: RowSample) -> Harness<'static, RowLog> {
        Self::harness_with(width, sample, false)
    }

    fn harness_with(width: f32, sample: RowSample, with_action: bool) -> Harness<'static, RowLog> {
        let mut harness = Harness::builder()
            .with_size(vec2(width, 120.0))
            .allow_missing_glyphs()
            .with_context_setup(|ctx| Theme::install(ctx, FontSource::BuiltIn))
            .build_ui_state(
                move |ui, log: &mut RowLog| {
                    let avatar = Avatar::new(&UserText::from_model(sample.name.to_owned()), AvatarSize::Regular);
                    let chips = sample
                        .chips
                        .iter()
                        .map(|chip| Chip::new(ChipKind::Display(ChipTone::Neutral), Sample::text(chip)))
                        .collect();
                    let mut row = ListRow::new(avatar, Sample::text(sample.name))
                        .version(Sample::text("v2.11.0"))
                        .chips(chips)
                        .off(sample.off);
                    if let Some(subtitle) = sample.subtitle {
                        row = row.subtitle(Sample::text(subtitle));
                    }
                    if log.with_action {
                        let off = log.action_off;
                        let reason = DisabledReason::new(Sample::text("Not allowed"));
                        row = row.action(IconButton::new(Icon::Delete).enabled(!off, reason));
                    }
                    let response = row.toggle(&mut log.on).show(ui);
                    log.actions += usize::from(response.action);
                    log.selects += usize::from(response.row.clicked());
                    log.toggles += usize::from(response.toggled);
                    log.rect = response.row.rect;
                },
                RowLog {
                    on: true,
                    selects: 0,
                    toggles: 0,
                    actions: 0,
                    with_action,
                    action_off: false,
                    rect: Rect::NOTHING,
                },
            );
        harness.run();
        harness
    }

    fn plain(name: &'static str) -> RowSample {
        RowSample { name, chips: &["Input", "HUD"], subtitle: None, off: false }
    }

    fn shapes(harness: &Harness<'static, RowLog>) -> Vec<Shape> {
        harness.output().shapes.iter().map(|clipped| clipped.shape.clone()).collect()
    }

    // (rect, colour, cut) of every text shape, by text
    fn texts(shapes: &[Shape]) -> Vec<(String, Rect, Color32, bool)> {
        shapes
            .iter()
            .filter_map(|shape| match shape {
                Shape::Text(text) => Some((
                    text.galley.text().to_owned(),
                    text.galley.rect.translate(text.pos.to_vec2()),
                    text.galley.job.sections[0].format.color,
                    text.galley.elided,
                )),
                _ => None,
            })
            .collect()
    }

    fn click(harness: &mut Harness<'static, RowLog>, at: Pos2) {
        harness.hover_at(at);
        harness.run();
        harness.drag_at(at);
        harness.step();
        harness.drop_at(at);
        harness.run();
    }

    // the painted track of the toggle: the only rect the size of `Toggle::TRACK`
    fn track(harness: &Harness<'static, RowLog>) -> Rect {
        let track = PaintCapture::rects(&Self::shapes(harness))
            .into_iter()
            .find(|r| r.rect.size() == Toggle::TRACK)
            .map(|r| r.rect);
        let Some(track) = track else { panic!("no toggle track") };
        track
    }
}

#[test]
fn toggle_click_does_not_select() {
    let mut harness = RowProbe::harness(460.0, RowProbe::plain("Minimap"));
    let track = RowProbe::track(&harness);
    RowProbe::click(&mut harness, track.center());
    let log = harness.state();
    assert!(!log.on, "the toggle did not switch");
    assert_eq!((log.toggles, log.selects), (1, 0), "{log:?}");
}

#[test]
fn row_click_selects() {
    let mut harness = RowProbe::harness(460.0, RowProbe::plain("Minimap"));
    let rect = harness.state().rect;
    RowProbe::click(&mut harness, rect.left_center() + vec2(30.0, 0.0));
    let log = harness.state();
    assert!(log.on, "a row click switched the toggle");
    assert_eq!((log.toggles, log.selects), (0, 1), "{log:?}");
}

#[test]
fn overflow_chips_collapse_to_more() {
    let chips: &[&str] = &["Camera", "Input", "HUD", "Overlay", "World", "Audio"];
    let harness = RowProbe::harness(330.0, RowSample { name: "Freecam", chips, subtitle: None, off: false });
    let track = RowProbe::track(&harness);
    let texts = RowProbe::texts(&RowProbe::shapes(&harness));
    let shown: Vec<_> = texts.iter().filter(|(text, ..)| chips.contains(&text.as_str())).collect();
    let more: Vec<_> = texts.iter().filter(|(text, ..)| text.starts_with('+')).collect();
    assert_eq!(more.len(), 1, "one +N chip: {texts:?}");
    let count: usize = more[0].0[1..].parse().expect("+N has a number");
    assert!(!shown.is_empty() && count > 0, "{texts:?}");
    assert_eq!(shown.len() + count, chips.len(), "{texts:?}");
    for (text, rect, ..) in shown.iter().chain(more.iter()) {
        assert!(rect.right() <= track.left(), "{text:?} runs under the toggle");
    }
}

#[test]
fn long_name_is_cut_version_kept() {
    let name = "World Edit Selection Tools and Brushes Extended For Every Builder";
    let harness = RowProbe::harness(460.0, RowProbe::plain(name));
    let track = RowProbe::track(&harness);
    let texts = RowProbe::texts(&RowProbe::shapes(&harness));
    let Some((_, name_rect, _, cut)) = texts.iter().find(|(text, ..)| text == name) else { panic!("no name") };
    assert!(cut, "the name is not cut");
    let Some((_, version, _, version_cut)) = texts.iter().find(|(text, ..)| text == "v2.11.0") else {
        panic!("no version")
    };
    assert!(!version_cut, "the version is cut");
    assert!(name_rect.right() <= version.left(), "the name runs over the version");
    assert!(version.right() <= track.left(), "the version runs under the toggle");
}

#[test]
fn off_dims_name_letter_and_chips() {
    let harness =
        RowProbe::harness(460.0, RowSample { name: "World Edit", chips: &["World"], subtitle: None, off: true });
    let texts = RowProbe::texts(&RowProbe::shapes(&harness));
    let colour = |wanted: &str| match texts.iter().find(|(text, ..)| text == wanted) {
        Some((_, _, colour, _)) => *colour,
        None => panic!("no {wanted:?}: {texts:?}"),
    };
    assert_eq!(colour("World Edit"), color::TextDim::VALUE, "name");
    assert_eq!(colour("W"), color::TextFaint::VALUE, "avatar letter");
    assert_eq!(colour("World"), color::TextFaint::VALUE, "chip");
    assert_eq!(colour("v2.11.0"), color::TextFaint::VALUE, "version");
}

impl RowProbe {
    fn named(name: &'static str, subtitle: Option<&'static str>) -> RowSample {
        RowSample { name, chips: &[], subtitle, off: false }
    }

    fn text_of(texts: &[(String, Rect, Color32, bool)], wanted: &str) -> (Rect, Color32, bool) {
        match texts.iter().find(|(text, ..)| text == wanted) {
            Some((_, rect, colour, cut)) => (*rect, *colour, *cut),
            None => panic!("no {wanted:?}: {texts:?}"),
        }
    }
}

#[test]
fn row_without_subtitle_is_unchanged() {
    let harness = RowProbe::harness(460.0, RowProbe::named("Main hall", None));
    let rect = harness.state().rect;
    let avatar = Avatar::new(&UserText::from_model("M".to_owned()), AvatarSize::Regular);
    let expected = avatar.side() + 2.0 * spacing::M::VALUE;
    assert_eq!(rect.height(), expected, "the row height");
    let texts = RowProbe::texts(&RowProbe::shapes(&harness));
    let (name, ..) = RowProbe::text_of(&texts, "Main hall");
    assert!((name.center().y - rect.center().y).abs() <= 1.0, "the name is centred: {name:?} in {rect:?}");
}

#[test]
fn subtitle_adds_a_second_line() {
    let subtitle = "32 x 18 x 24 / 4,212 blocks";
    let plain = RowProbe::harness(460.0, RowProbe::named("Main hall", None));
    let harness = RowProbe::harness(460.0, RowProbe::named("Main hall", Some(subtitle)));
    let rect = harness.state().rect;
    assert!(rect.height() >= plain.state().rect.height(), "the row is never shorter");
    let texts = RowProbe::texts(&RowProbe::shapes(&harness));
    let (name, ..) = RowProbe::text_of(&texts, "Main hall");
    let (line, colour, cut) = RowProbe::text_of(&texts, subtitle);
    assert!(line.top() >= name.bottom(), "the subtitle sits under the name: {name:?} {line:?}");
    assert!(name.center().y < rect.center().y, "the name moved up: {name:?} in {rect:?}");
    assert!(line.height() <= TypeSize::Caption.line_height(), "caption size: {line:?}");
    assert_eq!(colour, color::TextFaint::VALUE, "faint colour");
    assert!(!cut, "a short subtitle is not cut");
    assert!(line.left() == name.left(), "left edges line up");
}

#[test]
fn long_subtitle_is_truncated() {
    let subtitle = "128 x 64 x 128 / 1,048,576 blocks / 3 layers / assembled from many separate pieces";
    let harness = RowProbe::harness(460.0, RowProbe::named("Main hall", Some(subtitle)));
    let rect = harness.state().rect;
    let track = RowProbe::track(&harness);
    let texts = RowProbe::texts(&RowProbe::shapes(&harness));
    let (line, _, cut) = RowProbe::text_of(&texts, subtitle);
    assert!(cut, "the subtitle is not cut");
    assert!(line.right() <= track.left(), "the subtitle runs under the toggle: {line:?}");
    assert!(line.right() <= rect.right(), "the subtitle runs past the row");
}

#[test]
fn action_click_is_not_a_row_click() {
    let mut harness = RowProbe::harness_with(460.0, RowProbe::plain("Minimap"), true);
    let rect = harness.state().rect;
    let side = spacing::Row::VALUE;
    let button = Rect::from_min_size(
        egui::pos2(rect.right() - spacing::M::VALUE - side, rect.center().y - side / 2.0),
        vec2(side, side),
    );
    let plain = RowProbe::harness(460.0, RowProbe::plain("Minimap")).state().rect;
    assert_eq!(rect.height(), plain.height(), "the action does not change the row height");
    let bare = RowProbe::harness_with(460.0, RowProbe::named("Main hall", None), true).state().rect;
    assert_eq!(bare.height(), 64.0, "a row with an action stays 64 high");
    RowProbe::click(&mut harness, button.center());
    let log = harness.state();
    assert_eq!((log.actions, log.selects, log.toggles), (1, 0, 0), "{log:?}");
    assert!(log.on, "the toggle did not switch");
}

#[test]
fn action_sits_after_the_toggle() {
    let harness = RowProbe::harness_with(460.0, RowProbe::plain("Minimap"), true);
    let rect = harness.state().rect;
    let track = RowProbe::track(&harness);
    let side = vec2(spacing::Row::VALUE, spacing::Row::VALUE);
    let button = PaintCapture::rects(&RowProbe::shapes(&harness)).into_iter().find(|r| r.rect.size() == side);
    let Some(button) = button else { panic!("no action button") };
    assert_eq!(button.rect.right(), rect.right() - spacing::M::VALUE, "the button sits at the padding");
    assert_eq!(button.rect.left() - track.right(), spacing::M::VALUE, "{track:?} {:?}", button.rect);
}

// the shapes one row paints through `ListRow::paint`
struct FlashPaint;

impl FlashPaint {
    fn shapes(flash: Option<f32>) -> Vec<Shape> {
        PaintCapture::shapes(|painter| {
            let avatar = Avatar::new(&UserText::from_model("Minimap".to_owned()), AvatarSize::Regular);
            let mut row = ListRow::new(avatar, Sample::text("Minimap"));
            if let Some(strength) = flash {
                row = row.flash(strength);
            }
            let rect = Rect::from_min_size(Pos2::ZERO, vec2(460.0, row.height()));
            row.paint(painter, rect, InteractionState::Rest);
        })
    }
}

#[test]
fn flash_zero_paints_like_no_flash() {
    assert_eq!(FlashPaint::shapes(Some(0.0)), FlashPaint::shapes(None));
    assert_eq!(FlashPaint::shapes(Some(-3.0)), FlashPaint::shapes(None), "below 0 clamps to none");
}

#[test]
fn flash_fades_the_fill_and_clamps_at_one() {
    let fills = |flash: Option<f32>| -> Vec<Color32> {
        PaintCapture::rects(&FlashPaint::shapes(flash)).into_iter().map(|rect| rect.fill).collect()
    };
    let full = color::AccentSoftPress::VALUE;
    assert!(fills(Some(1.0)).contains(&full), "full strength fills with the token");
    assert_eq!(fills(Some(7.0)), fills(Some(1.0)), "above 1 clamps to full");
    let half = fills(Some(0.5));
    assert!(half.contains(&full.gamma_multiply(0.5)), "half strength halves the fill: {half:?}");
    let strokes: Vec<Color32> =
        PaintCapture::rects(&FlashPaint::shapes(Some(1.0))).into_iter().map(|rect| rect.stroke.color).collect();
    assert!(strokes.contains(&color::Accent::VALUE), "full strength outlines in accent: {strokes:?}");
}

#[test]
fn disabled_action_click_is_not_a_row_click() {
    let mut harness = RowProbe::harness_with(460.0, RowProbe::plain("Minimap"), true);
    harness.state_mut().action_off = true;
    harness.run();
    let rect = harness.state().rect;
    let side = spacing::Row::VALUE;
    let button = Rect::from_min_size(
        egui::pos2(rect.right() - spacing::M::VALUE - side, rect.center().y - side / 2.0),
        vec2(side, side),
    );
    RowProbe::click(&mut harness, button.center());
    let log = harness.state();
    assert_eq!((log.actions, log.selects, log.toggles), (0, 0, 0), "{log:?}");
}
