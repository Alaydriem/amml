use amml_studio_ui::component::bar::{
    SceneSwitcherFooter, SceneSwitcherList, SceneSwitcherListAction, SceneSwitcherListView, SceneSwitcherOriginal,
    SceneSwitcherRow,
};
use amml_studio_ui::tokens::container;
use amml_studio_ui::tokens::selector::SceneColor;
use amml_ui::icon::Icon;
use amml_ui::tokens::{ColorToken, ContainerToken, color};
use egui::Shape;

use crate::glyph::words;
use crate::support::UiProbe;

struct ListSample;

impl ListSample {
    fn row(name: &str, range: &str, cameras: &str, status: &str, colour: SceneColor, active: bool) -> SceneSwitcherRow {
        SceneSwitcherRow {
            name: words(name),
            range: words(range),
            cameras: words(cameras),
            status: words(status),
            color: colour,
            active,
        }
    }

    fn view() -> SceneSwitcherListView {
        SceneSwitcherListView {
            header: words("Scenes in this replay"),
            hint: words("right-click a scene for more"),
            original: SceneSwitcherOriginal {
                name: words("Original (as recorded)"),
                detail: words("no overrides, FOV 72"),
                active: false,
            },
            scenes: vec![
                Self::row("Arrival", "00:28.80 to 00:37.40", "6 cam", "Ready", SceneColor::Lime, true),
                Self::row("Portal run", "00:37.80 to 00:52.00", "4 cam", "Rendered", SceneColor::Pink, false),
                Self::row("Fortress", "00:12.40 to 00:28.40", "3 cam", "Draft", SceneColor::Orchid, false),
            ],
            footer: vec![
                SceneSwitcherFooter {
                    icon: Icon::Add,
                    label: words("New scene at playhead"),
                    shortcut: Some(words("Ctrl+Shift+N")),
                },
                SceneSwitcherFooter { icon: Icon::Copy, label: words("Duplicate active scene"), shortcut: None },
            ],
        }
    }

    fn shapes(view: &SceneSwitcherListView) -> Vec<Shape> {
        UiProbe::sized(600.0, 600.0).shapes(|ui| SceneSwitcherList::new(view).show(ui))
    }

    fn click(view: &SceneSwitcherListView, text: &str) -> Option<SceneSwitcherListAction> {
        let mut probe = UiProbe::sized(600.0, 600.0);
        let shapes = probe.shapes(|ui| SceneSwitcherList::new(view).show(ui));
        let Some(rect) = UiProbe::text_rect(&shapes, text) else { panic!("{text} is not painted") };
        probe.click(rect.center(), |ui| SceneSwitcherList::new(view).show(ui))
    }
}

#[test]
fn the_list_is_as_wide_as_the_scene_menu() {
    let view = ListSample::view();
    let (size, _) = UiProbe::sized(600.0, 600.0).frame(Vec::new(), |ui| {
        SceneSwitcherList::new(&view).show(ui);
        ui.min_rect().size()
    });
    assert_eq!(size.x, <container::SceneMenu as ContainerToken>::VALUE);
}

#[test]
fn every_word_comes_from_the_view() {
    let shapes = ListSample::shapes(&ListSample::view());
    let mut painted = UiProbe::words(&shapes);
    painted.sort();
    let mut expected: Vec<String> = [
        "Scenes in this replay",
        "right-click a scene for more",
        "Original (as recorded)",
        "no overrides, FOV 72",
        "Arrival",
        "00:28.80 to 00:37.40",
        "6 cam",
        "Ready",
        "Portal run",
        "00:37.80 to 00:52.00",
        "4 cam",
        "Rendered",
        "Fortress",
        "00:12.40 to 00:28.40",
        "3 cam",
        "Draft",
        "New scene at playhead",
        "Ctrl+Shift+N",
        "Duplicate active scene",
    ]
    .map(String::from)
    .to_vec();
    expected.sort();
    assert_eq!(painted, expected);
}

#[test]
fn the_active_row_is_soft_accent_and_a_pill_follows_the_scene_colour() {
    let shapes = ListSample::shapes(&ListSample::view());
    let rects = UiProbe::rects(&shapes);
    assert_eq!(rects.iter().filter(|r| r.fill == color::AccentSoft::VALUE && r.rect.height() == 38.0).count(), 1);
    let lime = SceneColor::Lime.color();
    assert!(rects.iter().any(|r| r.rect.height() == 20.0 && r.fill == lime.gamma_multiply(0.2)), "no pill fill");
    assert!(
        rects.iter().any(|r| r.rect.height() == 20.0 && r.stroke.color == lime.gamma_multiply(0.55)),
        "no pill line"
    );
}

#[test]
fn rows_report_their_selection() {
    let view = ListSample::view();
    assert_eq!(ListSample::click(&view, "Original (as recorded)"), Some(SceneSwitcherListAction::SelectOriginal));
    assert_eq!(ListSample::click(&view, "Portal run"), Some(SceneSwitcherListAction::SelectScene(1)));
    assert_eq!(ListSample::click(&view, "Fortress"), Some(SceneSwitcherListAction::SelectScene(2)));
}

#[test]
fn footer_rows_report_their_index() {
    let view = ListSample::view();
    assert_eq!(ListSample::click(&view, "Duplicate active scene"), Some(SceneSwitcherListAction::Footer(1)));
    assert_eq!(ListSample::click(&view, "New scene at playhead"), Some(SceneSwitcherListAction::Footer(0)));
}

#[test]
fn a_right_click_on_a_scene_asks_for_its_menu() {
    let view = ListSample::view();
    let mut probe = UiProbe::sized(600.0, 600.0);
    let shapes = probe.shapes(|ui| SceneSwitcherList::new(&view).show(ui));
    let Some(rect) = UiProbe::text_rect(&shapes, "Portal run") else { panic!("no row") };
    let action = probe.secondary_click(rect.center(), |ui| SceneSwitcherList::new(&view).show(ui));
    assert!(matches!(action, Some(SceneSwitcherListAction::OpenMenu(1, _))), "{action:?}");
}

#[test]
fn the_footer_has_a_line_above_it() {
    let shapes = ListSample::shapes(&ListSample::view());
    let Some(footer) = UiProbe::text_rect(&shapes, "New scene at playhead") else { panic!("no footer") };
    let line = shapes.iter().any(|s| match s {
        Shape::LineSegment { stroke, points } => {
            stroke.color == color::Line::VALUE && points[0].y < footer.top() && points[0].y > footer.top() - 20.0
        }
        Shape::Rect(r) => r.fill == color::Line::VALUE && r.rect.height() <= 1.0 && r.rect.top() < footer.top(),
        _ => false,
    });
    assert!(line, "no line above the footer");
}

#[test]
fn show_placed_gives_the_rect_of_every_row_in_order() {
    let view = ListSample::view();
    let mut probe = UiProbe::sized(600.0, 600.0);
    let show = |ui: &mut egui::Ui| {
        let (_, places) = SceneSwitcherList::new(&view).show_placed(ui);
        (ui.min_rect(), places)
    };
    probe.frame(Vec::new(), show);
    let (list, places) = probe.frame(Vec::new(), show).0;
    let actions: Vec<SceneSwitcherListAction> = places.iter().map(|(action, _)| *action).collect();
    assert_eq!(
        actions,
        vec![
            SceneSwitcherListAction::SelectOriginal,
            SceneSwitcherListAction::SelectScene(0),
            SceneSwitcherListAction::SelectScene(1),
            SceneSwitcherListAction::SelectScene(2),
            SceneSwitcherListAction::Footer(0),
            SceneSwitcherListAction::Footer(1),
        ]
    );
    for (action, rect) in &places {
        assert!(list.contains_rect(*rect), "{action:?} {rect:?} outside {list:?}");
        assert!(rect.width() > 0.0 && rect.height() > 0.0);
    }
    assert!(places.windows(2).all(|pair| pair[0].1.bottom() <= pair[1].1.top()));
}
