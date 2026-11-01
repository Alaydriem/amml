use amml_studio_ui::component::inspector::{InspectorDrawer, InspectorRail, InspectorTab, InspectorTabs};
use amml_studio_ui::tokens::container;
use amml_ui::tokens::ContainerToken;
use egui::{Rect, pos2, vec2};

use crate::support::UiProbe;

#[test]
fn tab_click_selects_the_tab() {
    let mut probe = UiProbe::new(430.0);
    let shapes = probe.shapes(|ui| InspectorTabs::new(InspectorTab::Key).show(ui));
    let scene = UiProbe::text_rect(&shapes, "Scene").expect("the Scene tab is painted");
    let picked = probe.click(scene.center(), |ui| InspectorTabs::new(InspectorTab::Key).show(ui));
    assert_eq!(picked, Some(InspectorTab::Scene));
}

#[test]
fn inspector_has_three_tabs() {
    assert_eq!(InspectorTab::ALL, [InspectorTab::Key, InspectorTab::Scene, InspectorTab::Players]);
    let mut probe = UiProbe::new(430.0);
    let shapes = probe.shapes(|ui| InspectorTabs::new(InspectorTab::Key).show(ui));
    for name in ["Key", "Scene", "Players"] {
        assert!(UiProbe::text_rect(&shapes, name).is_some(), "the {name} tab is painted");
    }
    assert!(UiProbe::text_rect(&shapes, "Track").is_none(), "Track is a state of the Key tab, not a tab");
}

#[test]
fn rail_opens_the_drawer() {
    let screen = Rect::from_min_size(pos2(0.0, 0.0), vec2(1280.0, 720.0));
    let mut probe = UiProbe::sized(1280.0, 720.0);
    let mut rail = Rect::NOTHING;
    probe.frame(Vec::new(), |ui| {
        let action = InspectorRail::new(InspectorTab::Key).show(ui);
        rail = ui.min_rect();
        action
    });
    let place = InspectorRail::place(rail, InspectorTab::Scene);
    let picked = probe.click(place.center(), |ui| InspectorRail::new(InspectorTab::Key).show(ui));
    assert_eq!(picked, Some(InspectorTab::Scene));

    let drawer = InspectorDrawer::rect(screen);
    assert_eq!(drawer.width(), <container::Inspector as ContainerToken>::VALUE);
    assert!(screen.contains_rect(drawer), "{drawer:?}");
    let (open, _) = probe.frame(Vec::new(), |ui| InspectorDrawer::new(InspectorTab::Scene).show(ui, |_| {}));
    assert!(open, "the drawer stays open until it is closed");
}
