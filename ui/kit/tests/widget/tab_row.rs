use amml_ui::font::FontFace;
use amml_ui::icon::Icon;
use amml_ui::state::InteractionState;
use amml_ui::tokens::{ColorToken, color};
use amml_ui::widget::{TabRow, TabRowSize};
use egui::{Color32, Key, Rect, Shape, pos2};

use crate::support::{ClickProbe, PaintCapture, Sample};

#[test]
fn selected_has_underline_inside_padding() {
    let rect = Rect::from_min_max(pos2(20.0, 20.0), pos2(100.0, 56.0));
    let label = Sample::text("Scene");
    let shapes = PaintCapture::shapes(|painter| {
        TabRow::paint_tab(painter, rect, &label, TabRowSize::Inspector, InteractionState::Rest, true);
    });
    let underline = PaintCapture::rects(&shapes).into_iter().find(|r| r.fill == color::Accent::VALUE);
    let underline = underline.map(|r| r.rect);
    // inset by the inspector padding (12) and 2 high at the bottom
    assert_eq!(underline, Some(Rect::from_min_max(pos2(32.0, 54.0), pos2(88.0, 56.0))));
    let text = shapes
        .iter()
        .find_map(|s| if let Shape::Text(t) = s { Some(t.galley.job.sections[0].format.clone()) } else { None });
    let text = text.unwrap_or_default();
    assert_eq!(text.color, color::Text::VALUE);
    assert_eq!(text.font_id.family, FontFace::Semibold.family());
}

#[test]
fn unselected_tab_is_faint_and_steps_its_fill() {
    let rect = Rect::from_min_max(pos2(20.0, 20.0), pos2(100.0, 56.0));
    let label = Sample::text("Scene");
    let shapes = PaintCapture::shapes(|painter| {
        TabRow::paint_tab(painter, rect, &label, TabRowSize::Inspector, InteractionState::Hover, false);
    });
    assert!(PaintCapture::rects(&shapes).iter().any(|r| r.rect == rect && r.fill == color::BgHover::VALUE));
    let text = shapes
        .iter()
        .find_map(|s| if let Shape::Text(t) = s { Some(t.galley.job.sections[0].format.color) } else { None });
    assert_eq!(text, Some(color::TextFaint::VALUE));
}

#[test]
fn compact_is_row_height() {
    let harness = ClickProbe::harness_with(0_usize, |ui, selected| {
        TabRow::new(egui::Id::unique(3), selected, vec![Sample::text("Replays"), Sample::text("Settings")])
            .size(TabRowSize::Compact)
            .show(ui)
    });
    // the strip is the tab height plus its 1 px divider
    assert_eq!(harness.state().rect.height(), 29.0);
}

#[test]
fn click_selects() {
    let mut harness = ClickProbe::harness_with(0_usize, |ui, selected| {
        TabRow::new(
            egui::Id::unique(3),
            selected,
            vec![Sample::text("Key"), Sample::text("Scene"), Sample::text("Players")],
        )
        .show(ui)
    });
    // the third tab sits right of the first two; click near the strip's left plus two tab widths
    let rect = harness.state().rect;
    let at = pos2(rect.left() + 170.0, rect.center().y);
    harness.hover_at(at);
    harness.drag_at(at);
    harness.run();
    harness.drop_at(at);
    harness.run();
    assert_eq!(harness.state().value, 2);
}

#[test]
fn arrow_moves_selection() {
    let mut harness = ClickProbe::harness_with(0_usize, |ui, selected| {
        TabRow::new(egui::Id::unique(3), selected, vec![Sample::text("Key"), Sample::text("Scene")]).show(ui)
    });
    ClickProbe::tab_and_press(&mut harness, Key::ArrowRight);
    assert_eq!(harness.state().value, 1);
}

#[test]
fn icons_lead_their_labels() {
    let harness = ClickProbe::harness_with(0_usize, |ui, selected| {
        TabRow::new(egui::Id::unique(3), selected, vec![Sample::text("General"), Sample::text("Keybinds")])
            .icons(vec![Icon::Settings, Icon::Keyboard])
            .show(ui)
    });
    let shapes: Vec<Shape> = harness.output().shapes.iter().map(|clipped| clipped.shape.clone()).collect();
    let icons: Vec<(Rect, Color32)> = shapes
        .iter()
        .filter_map(|shape| match shape {
            Shape::Text(text) if text.galley.job.sections[0].format.font_id.family == FontFace::Icons.family() => {
                Some((text.galley.rect.translate(text.pos.to_vec2()), text.galley.job.sections[0].format.color))
            }
            _ => None,
        })
        .collect();
    let labels = PaintCapture::text_rects(&shapes);
    let label = |name: &str| labels.iter().find(|(text, _)| text == name).map(|(_, rect)| *rect);
    let (Some(general), Some(keybinds)) = (label("General"), label("Keybinds")) else { panic!("no labels") };

    assert_eq!(icons.len(), 2);
    assert!(icons[0].0.right() <= general.left() && icons[0].0.left() > harness.state().rect.left());
    assert!(icons[1].0.right() <= keybinds.left() && icons[1].0.left() > general.right());
    assert_eq!(icons[0].1, color::AccentHover::VALUE, "the selected tab's icon is in the accent");
    assert_eq!(icons[1].1, color::TextFaint::VALUE);
}

#[test]
fn each_arrow_moves_one_tab() {
    let mut harness = ClickProbe::harness_with(0_usize, |ui, selected| {
        let tabs = ["Key", "Scene", "Players", "Track"].iter().map(|tab| Sample::text(tab)).collect();
        TabRow::new(egui::Id::unique(3), selected, tabs).show(ui)
    });
    ClickProbe::tab_and_press(&mut harness, Key::ArrowRight);
    harness.key_press(Key::ArrowRight);
    harness.run();
    assert_eq!(harness.state().value, 2);
}
