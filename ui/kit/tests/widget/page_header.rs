use amml_i18n::text::UserText;
use amml_ui::font::FontSource;
use amml_ui::icon::Icon;
use amml_ui::theme::Theme;
use amml_ui::tokens::{ColorToken, color};
use amml_ui::widget::{ChipTone, IconButton, MetaSeparator, PageHeader, PageHeaderLead};
use egui::{Rect, Shape, vec2};
use egui_kittest::Harness;

use crate::support::{FlowProbe, PaintCapture, Sample, ShapeText};

// the header's rect and its trailing button's rect
#[derive(Debug)]
struct HeaderLog {
    header: Rect,
    trailing: Rect,
}

// shows one page-form header, `width` wide, with an avatar, a title, meta and a close button
struct HeaderProbe;

impl HeaderProbe {
    fn harness(width: f32, title: &'static str, meta: &'static [&'static str]) -> Harness<'static, HeaderLog> {
        let mut harness = Harness::builder()
            .with_size(vec2(width, 160.0))
            .allow_missing_glyphs()
            .with_context_setup(|ctx| Theme::install(ctx, FontSource::BuiltIn))
            .build_ui_state(
                move |ui, log: &mut HeaderLog| {
                    let mut trailing = Rect::NOTHING;
                    let lead = PageHeaderLead::Avatar(UserText::from_model(title.to_owned()), None);
                    log.header = PageHeader::new(lead, Sample::text(title))
                        .meta(meta.iter().map(|item| Sample::text(item)).collect())
                        .trailing(|ui| trailing = IconButton::new(Icon::Close).show(ui).rect)
                        .show(ui)
                        .rect;
                    log.trailing = trailing;
                },
                HeaderLog { header: Rect::NOTHING, trailing: Rect::NOTHING },
            );
        harness.run();
        harness
    }

    fn shapes(harness: &Harness<'static, HeaderLog>) -> Vec<Shape> {
        harness.output().shapes.iter().map(|clipped| clipped.shape.clone()).collect()
    }
}

#[test]
fn long_title_is_cut_and_trailing_stays_visible() {
    let title = "World Edit Selection Tools and Brushes Extended For Every Builder On The Server";
    let meta = &["worldedit-selection-tools", "v2.11.0", "by a very long author name that goes on and on"];
    let harness = HeaderProbe::harness(640.0, title, meta);
    let log = harness.state();
    assert!(log.trailing.is_positive(), "the trailing button was not shown");
    assert!(log.header.contains_rect(log.trailing), "the trailing button is outside the header: {log:?}");
    let shapes = HeaderProbe::shapes(&harness);
    let texts: Vec<_> = shapes
        .iter()
        .filter_map(|shape| match shape {
            Shape::Text(text) if text.galley.text() == title || meta.contains(&text.galley.text()) => Some(text),
            _ => None,
        })
        .collect();
    let Some(title_shape) = texts.iter().find(|text| text.galley.text() == title) else { panic!("no title") };
    assert_eq!(title_shape.galley.rows.len(), 1, "the title wraps");
    assert!(title_shape.galley.elided, "the title is not cut");
    for text in texts {
        let rect = text.galley.rect.translate(text.pos.to_vec2());
        assert!(rect.right() <= log.trailing.left(), "{:?} runs under the trailing part", text.galley.text());
        assert_eq!(text.galley.rows.len(), 1, "{:?} wraps", text.galley.text());
    }
}

#[test]
fn meta_items_are_joined_by_drawn_dots() {
    let meta = &["freecam", "v0.1.0", "by Alaydriem"];
    let harness = HeaderProbe::harness(1000.0, "Freecam", meta);
    let shapes = HeaderProbe::shapes(&harness);
    assert!(ShapeText::all(&shapes).iter().all(|text| !text.contains('\u{b7}')), "a text has a '·'");
    let side = MetaSeparator::SIDE;
    let dots: Vec<_> = PaintCapture::rects(&shapes)
        .into_iter()
        .filter(|r| r.fill == color::TextFaint::VALUE && r.rect.size() == vec2(side, side))
        .map(|r| r.rect)
        .collect();
    assert_eq!(dots.len(), 2, "{dots:?}");
    let items: Vec<Rect> = meta
        .iter()
        .map(|item| {
            match harness.output().shapes.iter().find_map(|clipped| match &clipped.shape {
                Shape::Text(text) if text.galley.text() == *item => {
                    Some(text.galley.rect.translate(text.pos.to_vec2()))
                }
                _ => None,
            }) {
                Some(rect) => rect,
                None => panic!("no meta item {item:?}"),
            }
        })
        .collect();
    for (index, dot) in dots.iter().enumerate() {
        assert!(
            items[index].right() < dot.left() && dot.right() < items[index + 1].left(),
            "dot {index} is not between"
        );
        assert!((dot.center().y - items[index].center().y).abs() <= 1.0, "dot {index} is not on the item centre line");
    }
}

#[test]
fn trailing_does_not_pull_the_next_widget_up() {
    let log = FlowProbe::run(|ui| {
        let lead = PageHeaderLead::Avatar(UserText::from_model("Freecam".to_owned()), None);
        PageHeader::new(lead, Sample::text("Freecam"))
            .trailing(|ui| {
                IconButton::new(Icon::Close).show(ui);
            })
            .show(ui);
    });
    FlowProbe::assert_below(&log);
}

#[test]
fn height_is_the_height_it_takes() {
    let harness = HeaderProbe::harness(640.0, "Freecam", &["freecam", "v0.1.0"]);
    let lead = PageHeaderLead::Avatar(UserText::from_model("Freecam".to_owned()), None);
    let header = PageHeader::new(lead, Sample::text("Freecam")).meta(vec![Sample::text("freecam")]);
    let panel = PageHeader::new(PageHeaderLead::Tile(Icon::Apps, ChipTone::Accent), Sample::text("Mod Manager"))
        .eyebrow(Sample::text("ALAYDRIEM"));

    assert_eq!(header.height(), harness.state().header.height());
    assert!(panel.height() < header.height(), "the panel form's smaller tile makes a lower bar");
}
