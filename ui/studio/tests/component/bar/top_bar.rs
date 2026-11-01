use amml_studio_ui::component::bar::{StudioMode, TopBar, TopBarAction, TopBarControl, TopBarView};
use amml_studio_ui::tokens::selector::SceneColor;
use amml_studio_ui::tokens::spacing;
use amml_ui::icon::Icon;
use amml_ui::tokens::SpacingToken;

use super::arrival;
use crate::glyph::words;
use crate::support::UiProbe;

fn view(mode: StudioMode, can_undo: bool) -> TopBarView {
    TopBarView {
        title: words("Nether trip"),
        mode,
        can_undo,
        can_redo: false,
        history_open: false,
        scene: arrival(),
        export_blocked: None,
    }
}

#[test]
fn top_bar_fills_its_token_height() {
    let mut probe = UiProbe::new(1920.0);
    let view = view(StudioMode::Edit, true);
    let (height, _) = probe.frame(Vec::new(), |ui| {
        TopBar::new(&view).show(ui);
        ui.min_rect().height()
    });
    assert_eq!(height, <spacing::TopBar as SpacingToken>::VALUE);
}

#[test]
fn no_mode_switch_is_drawn() {
    // Task 14d: the mode follows playback; Play is Preview, Pause is Edit
    for mode in [StudioMode::Edit, StudioMode::Preview] {
        let mut probe = UiProbe::new(1920.0);
        let view = view(mode, true);
        let shapes = probe.shapes(|ui| TopBar::new(&view).show(ui));
        let texts = UiProbe::texts(&shapes);
        assert!(!texts.iter().any(|text| text == "Edit" || text == "Preview"), "{mode:?}: {texts:?}");
    }
}

#[test]
fn long_scene_name_stays_left_of_the_actions() {
    let mut probe = UiProbe::new(1280.0);
    let mut view = view(StudioMode::Preview, true);
    view.scene.scene = Some((words("Portal run through the nether fortress and back"), SceneColor::Lime));
    let (_, places) = placed_view(&view, 1280.0);
    let undo = places.iter().find(|(c, _)| *c == TopBarControl::Undo).expect("undo").1;
    let shapes = probe.shapes(|ui| TopBar::new(&view).show(ui));
    let place = UiProbe::text_rect(&shapes, "1 of 5").map_or(f32::NEG_INFINITY, |rect| rect.right());
    let name = shapes
        .iter()
        .filter_map(|shape| match shape {
            egui::Shape::Text(text) if text.galley.text().starts_with("Portal") => {
                Some(text.galley.rect.translate(text.pos.to_vec2()).right())
            }
            _ => None,
        })
        .fold(f32::NEG_INFINITY, f32::max);
    assert!(name.max(place) < undo.left(), "the chip text ends at {} past Undo at {}", name.max(place), undo.left());
}

#[test]
fn undo_is_disabled_without_history() {
    let undo = Icon::Undo.shape();
    let glyph = match undo {
        amml_ui::icon::IconShape::Font(code) => code.to_string(),
        amml_ui::icon::IconShape::Drawn(_) => panic!("undo is a font icon"),
    };
    for (can_undo, expected) in [(true, vec![TopBarAction::Undo]), (false, Vec::new())] {
        let mut probe = UiProbe::new(1920.0);
        let view = view(StudioMode::Edit, can_undo);
        let shapes = probe.shapes(|ui| TopBar::new(&view).show(ui));
        let at = UiProbe::text_rect(&shapes, &glyph).unwrap_or_else(|| panic!("no undo icon")).center();
        let actions = probe.click(at, |ui| TopBar::new(&view).show(ui));
        assert_eq!(actions, expected, "can_undo {can_undo}");
    }
}

#[test]
fn coverage_queue_and_export_send_their_actions() {
    for (label, expected) in [
        ("Coverage", TopBarAction::OpenCoverage),
        ("Queue", TopBarAction::OpenQueue),
        ("Export", TopBarAction::OpenExport),
    ] {
        let mut probe = UiProbe::new(1920.0);
        let view = view(StudioMode::Edit, true);
        let shapes = probe.shapes(|ui| TopBar::new(&view).show(ui));
        let at = UiProbe::text_rect(&shapes, label).unwrap_or_else(|| panic!("no {label}")).center();
        assert_eq!(probe.click(at, |ui| TopBar::new(&view).show(ui)), vec![expected]);
    }
}

#[test]
fn a_blocked_export_takes_no_click() {
    let mut probe = UiProbe::new(1920.0);
    let view = TopBarView { export_blocked: Some(words("Pick a scene to export")), ..view(StudioMode::Edit, true) };
    let shapes = probe.shapes(|ui| TopBar::new(&view).show(ui));
    let at = UiProbe::text_rect(&shapes, "Export").unwrap_or_else(|| panic!("no Export")).center();
    assert_eq!(probe.click(at, |ui| TopBar::new(&view).show(ui)), Vec::new());
}

#[test]
fn a_blocked_export_shows_its_reason_on_hover() {
    let reason = "The Original is the replay as recorded. Pick a scene to export.";
    let mut probe = UiProbe::new(1920.0);
    let view = TopBarView { export_blocked: Some(words(reason)), ..view(StudioMode::Edit, true) };
    let shapes = probe.shapes(|ui| TopBar::new(&view).show(ui));
    assert!(!UiProbe::texts(&shapes).iter().any(|text| text == reason), "the reason shows before a hover");
    let at = UiProbe::text_rect(&shapes, "Export").unwrap_or_else(|| panic!("no Export")).center();
    let hovered = probe.hover(at, |ui| TopBar::new(&view).show(ui));
    assert!(UiProbe::texts(&hovered).iter().any(|text| text == reason), "no tooltip: {:?}", UiProbe::texts(&hovered));
}

#[test]
fn the_scene_chip_opens_the_scene_switcher() {
    let mut probe = UiProbe::new(1920.0);
    let view = view(StudioMode::Edit, true);
    let shapes = probe.shapes(|ui| TopBar::new(&view).show(ui));
    let at = UiProbe::text_rect(&shapes, "Arrival").unwrap_or_else(|| panic!("no chip")).center();
    assert_eq!(probe.click(at, |ui| TopBar::new(&view).show(ui)), vec![TopBarAction::OpenSceneSwitcher]);
}

#[test]
fn a_long_title_keeps_the_controls_in_the_bar() {
    let mut probe = UiProbe::new(1280.0);
    let mut view = view(StudioMode::Edit, true);
    view.title = words(&"A very long replay title ".repeat(12));
    let shapes = probe.shapes(|ui| TopBar::new(&view).show(ui));
    let export = UiProbe::text_rect(&shapes, "Export").unwrap_or_else(|| panic!("no Export"));
    assert!(export.right() <= 1280.0, "{export:?}");
    let (_, places) = placed_view(&view, 1280.0);
    let undo = places.iter().find(|(c, _)| *c == TopBarControl::Undo).expect("undo").1;
    for (text, rect) in shapes.iter().filter_map(|shape| match shape {
        egui::Shape::Text(t) if t.galley.text().starts_with("A very") => {
            Some((t.galley.text().to_owned(), t.galley.rect.translate(t.pos.to_vec2())))
        }
        _ => None,
    }) {
        assert!(rect.right() < undo.left(), "{text} runs into the actions");
    }
}

fn placed(mode: StudioMode) -> (egui::Rect, Vec<(TopBarControl, egui::Rect)>) {
    placed_view(&TopBarView { can_redo: true, ..view(mode, true) }, 1920.0)
}

fn placed_view(view: &TopBarView, width: f32) -> (egui::Rect, Vec<(TopBarControl, egui::Rect)>) {
    let mut probe = UiProbe::new(width);
    let show = |ui: &mut egui::Ui| {
        let (_, places) = TopBar::new(view).show_placed(ui);
        (ui.min_rect(), places)
    };
    probe.frame(Vec::new(), show);
    probe.frame(Vec::new(), show).0
}

#[test]
fn top_bar_places_all_9_controls_inside_the_bar() {
    let (bar, places) = placed(StudioMode::Edit);
    assert_eq!(places.len(), 9, "{places:?}");
    for (control, rect) in &places {
        assert!(bar.contains_rect(*rect), "{control:?} {rect:?} outside {bar:?}");
    }
}

#[test]
fn a_click_at_the_centre_of_each_placed_control_gives_its_action() {
    let cases = [
        (TopBarControl::SceneChip, StudioMode::Edit, TopBarAction::OpenSceneSwitcher),
        (TopBarControl::Undo, StudioMode::Edit, TopBarAction::Undo),
        (TopBarControl::Redo, StudioMode::Edit, TopBarAction::Redo),
        (TopBarControl::UndoHistory, StudioMode::Edit, TopBarAction::OpenUndoHistory),
        (TopBarControl::Settings, StudioMode::Edit, TopBarAction::OpenSettings),
        (TopBarControl::Close, StudioMode::Edit, TopBarAction::CloseReplay),
        (TopBarControl::Coverage, StudioMode::Edit, TopBarAction::OpenCoverage),
        (TopBarControl::Queue, StudioMode::Edit, TopBarAction::OpenQueue),
        (TopBarControl::Export, StudioMode::Edit, TopBarAction::OpenExport),
    ];
    for (control, mode, expected) in cases {
        let (_, places) = placed(mode);
        let rect = places.iter().find(|(c, _)| *c == control).unwrap_or_else(|| panic!("no {control:?}")).1;
        let mut probe = UiProbe::new(1920.0);
        let view = TopBarView { can_redo: true, ..view(mode, true) };
        let show = |ui: &mut egui::Ui| TopBar::new(&view).show(ui);
        probe.shapes(show);
        assert_eq!(probe.click(rect.center(), show), vec![expected], "{control:?}");
    }
}
