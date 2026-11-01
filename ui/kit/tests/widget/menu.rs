use amml_ui::icon::Icon;
use amml_ui::state::{DisabledReason, InteractionState};
use amml_ui::tokens::{ColorToken, OpacityToken, SpacingToken, color, opacity, spacing};
use amml_ui::widget::{Menu, MenuItem, MenuLeading, MenuTone, MenuTrailing, Submenu};
use egui::epaint::RectShape;
use egui::{Color32, Id, Popup, Rect, Sense, Shape, pos2, vec2};
use egui_kittest::Harness;

use crate::support::{ClickLog, ClickProbe, PaintCapture, Sample, ShapeText, WarnLog};

struct ItemPaint;

impl ItemPaint {
    const RECT: Rect = Rect::from_min_max(pos2(20.0, 20.0), pos2(270.0, 48.0));

    fn shapes(item: &MenuItem, state: InteractionState) -> Vec<Shape> {
        PaintCapture::shapes(|painter| item.paint(painter, Self::RECT, state))
    }

    fn colour_of(shapes: &[Shape], text: &str) -> Option<Color32> {
        shapes.iter().find_map(|s| match s {
            Shape::Text(t) if t.galley.text() == text => Some(t.galley.job.sections[0].format.color),
            _ => None,
        })
    }
}

#[test]
fn item_ladder_and_danger_colours() {
    let item = MenuItem::new(Sample::text("Rename scene…")).leading(MenuLeading::Icon(Icon::Rename));
    for (state, fill) in
        [(InteractionState::Hover, color::BgHover::VALUE), (InteractionState::Pressed, color::BgActive::VALUE)]
    {
        let shapes = ItemPaint::shapes(&item, state);
        assert!(PaintCapture::rects(&shapes).iter().any(|r| r.rect == ItemPaint::RECT && r.fill == fill), "{state:?}");
    }
    let danger =
        MenuItem::new(Sample::text("Delete scene…")).leading(MenuLeading::Icon(Icon::Delete)).tone(MenuTone::Danger);
    let shapes = ItemPaint::shapes(&danger, InteractionState::Hover);
    assert_eq!(ItemPaint::colour_of(&shapes, "Delete scene…"), Some(color::Danger::VALUE));
    assert_eq!(ItemPaint::colour_of(&shapes, "\u{E74D}"), Some(color::Danger::VALUE));
    assert!(PaintCapture::rects(&shapes).iter().any(|r| r.fill == color::BgHover::VALUE));
}

#[test]
fn disabled_reason_stays_full_opacity() {
    let item = MenuItem::new(Sample::text("Copy key"))
        .leading(MenuLeading::Icon(Icon::Copy))
        .enabled(false, DisabledReason::new(Sample::text("Select a key first")));
    let rect = Rect::from_min_size(pos2(20.0, 20.0), vec2(250.0, MenuItem::REASON_HEIGHT));
    let shapes = PaintCapture::shapes(|painter| item.paint(painter, rect, InteractionState::Disabled));
    // a painter's opacity reaches text through the galley's vertex colours, not its format colour
    let alpha = |text: &str| {
        shapes.iter().find_map(|s| match s {
            Shape::Text(t) if t.galley.text() == text => {
                let vertex = t.galley.rows.iter().find_map(|r| r.row.visuals.mesh.vertices.first())?;
                Some((vertex.color.a(), t.galley.job.sections[0].format.color.a()))
            }
            _ => None,
        })
    };
    let Some((label, label_source)) = alpha("Copy key") else { panic!("no Copy key label") };
    let muted = (f32::from(label_source) * <opacity::Muted as OpacityToken>::VALUE).round();
    assert!((f32::from(label) - muted).abs() <= 1.0, "label alpha {label}, want {muted}");
    let Some((reason, reason_source)) = alpha("Select a key first") else { panic!("no reason line") };
    assert_eq!(reason, reason_source, "the reason line was muted");
    assert_eq!(ItemPaint::colour_of(&shapes, "Select a key first"), Some(color::TextFaint::VALUE));
}

#[test]
fn shortcut_submenu_and_check_trail() {
    let shortcut = MenuItem::new(Sample::text("Duplicate")).trailing(MenuTrailing::Shortcut(Sample::text("Ctrl+D")));
    let shapes = ItemPaint::shapes(&shortcut, InteractionState::Rest);
    assert_eq!(ItemPaint::colour_of(&shapes, "Ctrl+D"), Some(color::TextFaint::VALUE));
    let submenu = MenuItem::new(Sample::text("Colour")).trailing(MenuTrailing::Submenu);
    assert!(ShapeText::contains(&ItemPaint::shapes(&submenu, InteractionState::Rest), "\u{E76C}"));
    let checked = MenuItem::new(Sample::text("Snap to keys")).leading(MenuLeading::Check);
    let shapes = ItemPaint::shapes(&checked, InteractionState::Rest);
    assert_eq!(ItemPaint::colour_of(&shapes, "\u{E73E}"), Some(color::AccentHover::VALUE));
}

#[test]
fn swatch_rows_paint_a_swatch_dot() {
    let lime = Color32::from_rgb(0xB5, 0xE8, 0x53);
    let item = MenuItem::new(Sample::text("Lime")).leading(MenuLeading::Swatch(lime)).trailing(MenuTrailing::Check);
    let shapes = ItemPaint::shapes(&item, InteractionState::Rest);
    let dot = shapes.iter().find_map(|s| if let Shape::Circle(c) = s { Some((c.radius, c.fill)) } else { None });
    assert_eq!(dot, Some((5.0, lime)));
}

#[test]
fn radio_draws_a_ring_and_a_dot_only_when_selected() {
    let circles = |selected: bool| {
        let item = MenuItem::new(Sample::text("Ready")).leading(MenuLeading::Radio(selected));
        let shapes = ItemPaint::shapes(&item, InteractionState::Rest);
        shapes
            .iter()
            .filter_map(|s| if let Shape::Circle(c) = s { Some((c.radius, c.fill, c.stroke)) } else { None })
            .collect::<Vec<_>>()
    };
    let off = circles(false);
    assert_eq!(off.len(), 1, "{off:?}");
    assert_eq!(off[0].0, 6.0 - 0.75);
    assert_eq!(off[0].2.color, color::TextDim::VALUE);
    assert_eq!(off[0].2.width, 1.5);
    let on = circles(true);
    assert_eq!(on.len(), 2, "{on:?}");
    assert!(on.iter().all(|(_, _, stroke)| stroke.width == 0.0 || stroke.color == color::Accent::VALUE));
    assert!(on.iter().any(|(radius, fill, _)| *radius == 3.0 && *fill == color::Accent::VALUE), "{on:?}");
}

#[test]
fn radio_keeps_the_label_where_the_other_leading_parts_put_it() {
    let label_left = |leading: MenuLeading| {
        let item = MenuItem::new(Sample::text("Ready")).leading(leading);
        let shapes = ItemPaint::shapes(&item, InteractionState::Rest);
        PaintCapture::text_rects(&shapes).into_iter().find(|(text, _)| text == "Ready").map(|(_, rect)| rect.left())
    };
    assert_eq!(label_left(MenuLeading::Radio(true)), label_left(MenuLeading::Check));
    assert_eq!(label_left(MenuLeading::Radio(false)), label_left(MenuLeading::None));
}

#[test]
fn leaf_click_closes_menu_and_disabled_item_does_not_click() {
    let mut harness = ClickProbe::harness_with((0_usize, 0_usize), |ui, (renames, copies)| {
        let target = ui.allocate_response(vec2(200.0, 40.0), Sense::click());
        Menu::show(&target, Some(Sample::text("Arrival")), |ui| {
            if MenuItem::new(Sample::text("Rename scene…")).show(ui).clicked() {
                *renames += 1;
            }
            let reason = DisabledReason::new(Sample::text("Select a key first"));
            if MenuItem::new(Sample::text("Copy key")).enabled(false, reason).show(ui).clicked() {
                *copies += 1;
            }
        });
        target
    });
    let target = harness.state().rect;
    ClickProbe::secondary_click_at(&mut harness, target.center());
    assert!(ClickProbe::texts(&harness).contains(&"Arrival".to_owned()), "the menu did not open");
    let copy = ClickProbe::text_rect(&harness, "Copy key").map(|r| r.center());
    let Some(copy) = copy else { panic!("no Copy key row") };
    ClickProbe::click_at(&mut harness, copy);
    assert_eq!(harness.state().value.1, 0, "a disabled item clicked");
    let rename = ClickProbe::text_rect(&harness, "Rename scene…").map(|r| r.center());
    let Some(rename) = rename else { panic!("the menu closed on a disabled item") };
    ClickProbe::click_at(&mut harness, rename);
    assert_eq!(harness.state().value.0, 1);
    assert!(!ClickProbe::texts(&harness).contains(&"Arrival".to_owned()), "the menu did not close");
}

#[test]
fn an_item_clicked_outside_a_menu_makes_egui_warn_of_nothing() {
    let log = WarnLog::install();
    let mut harness = ClickProbe::harness_with(0_usize, |ui, clicks| {
        let response = MenuItem::new(Sample::text("Rename scene…")).show(ui);
        if response.clicked() {
            *clicks += 1;
        }
        response
    });
    let item = harness.state().rect;

    ClickProbe::click_at(&mut harness, item.center());

    assert_eq!(harness.state().value, 1, "the item did not click");
    assert!(log.lines().is_empty(), "egui warned: {:?}", log.lines());
}

#[test]
fn submenu_opens_to_the_right() {
    let mut harness = ClickProbe::harness_with((), |ui, _| {
        let target = ui.allocate_response(vec2(200.0, 40.0), Sense::click());
        Menu::show(&target, None, |ui| {
            Submenu::show(ui, Sample::text("Colour"), |ui| {
                MenuItem::new(Sample::text("Lime")).show(ui);
            });
        });
        target
    });
    let target = harness.state().rect;
    ClickProbe::secondary_click_at(&mut harness, target.center());
    let Some(colour) = ClickProbe::text_rect(&harness, "Colour") else { panic!("no Colour row") };
    harness.hover_at(colour.center());
    harness.run();
    let Some(lime) = ClickProbe::text_rect(&harness, "Lime") else { panic!("the submenu did not open") };
    assert!(lime.left() > colour.right(), "{lime:?} is not right of {colour:?}");
}

struct MenuPanel;

impl MenuPanel {
    // the painted menu panel, found by its fill
    fn shape(harness: &Harness<'static, ClickLog<bool>>) -> Option<RectShape> {
        let shapes: Vec<Shape> = harness.output().shapes.iter().map(|clipped| clipped.shape.clone()).collect();
        PaintCapture::rects(&shapes).into_iter().find(|r| r.fill == color::BgRaised::VALUE)
    }

    fn from_button() -> Harness<'static, ClickLog<bool>> {
        ClickProbe::harness_with(false, |ui, open| {
            let button = ui.allocate_response(vec2(120.0, 30.0), Sense::click());
            *open = Menu::show_from_button(&button, |ui| {
                MenuItem::new(Sample::text("Rename scene…")).show(ui);
            });
            button
        })
    }
}

#[test]
fn show_from_button_toggles_on_a_click_and_closes_on_a_click_outside() {
    let mut harness = MenuPanel::from_button();
    assert!(!harness.state().value);
    assert!(MenuPanel::shape(&harness).is_none());

    ClickProbe::click(&mut harness);
    assert!(harness.state().value, "the click did not open the menu");
    assert!(ClickProbe::texts(&harness).contains(&"Rename scene…".to_owned()));

    ClickProbe::click(&mut harness);
    assert!(!harness.state().value, "a second click did not close the menu");
    assert!(!ClickProbe::texts(&harness).contains(&"Rename scene…".to_owned()));

    ClickProbe::click(&mut harness);
    assert!(harness.state().value);
    ClickProbe::click_at(&mut harness, pos2(700.0, 500.0));
    assert!(!harness.state().value, "a click outside did not close the menu");
}

#[test]
fn show_from_button_draws_below_the_button_left_edge_at_the_menu_width() {
    let mut harness = MenuPanel::from_button();
    let button = harness.state().rect;
    ClickProbe::click(&mut harness);
    let Some(shape) = MenuPanel::shape(&harness) else { panic!("the menu panel was not drawn") };
    let panel = shape.rect;
    assert_eq!(panel.left(), button.left());
    assert!(panel.top() >= button.bottom(), "{panel:?} overlaps {button:?}");
    let padding = <spacing::S as SpacingToken>::VALUE;
    assert_eq!(panel.width(), Menu::inner_width() + (padding + shape.stroke.width) * 2.0);
}

#[test]
fn show_at_draws_at_the_point_and_a_click_outside_closes_it() {
    let popup = Id::unique("show-at-menu-test");
    let at = pos2(300.0, 200.0);
    let mut harness = ClickProbe::harness_with(false, move |ui, open| {
        let anchor = ui.allocate_response(vec2(10.0, 10.0), Sense::hover());
        *open = Menu::show_at(ui, popup, at, |ui| {
            MenuItem::new(Sample::text("Rename scene…")).show(ui);
        });
        anchor
    });
    assert!(!harness.state().value, "the menu was open before it was asked to open");

    Popup::open_id(&harness.ctx, popup);
    harness.run();
    assert!(harness.state().value, "the menu did not open");
    let Some(shape) = MenuPanel::shape(&harness) else { panic!("the menu panel was not drawn") };
    let panel = shape.rect;
    assert_eq!(panel.min, at);

    ClickProbe::click_at(&mut harness, pos2(700.0, 500.0));
    assert!(!harness.state().value, "a click outside did not close the menu");
}
