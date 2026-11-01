use amml_studio_ui::component::library::{LibraryAppBar, LibraryAppBarAction, LibraryAppBarTab, LibraryAppBarView};
use amml_ui::tokens::{ColorToken, color};
use egui::{Rect, vec2};

use crate::glyph::words;
use crate::support::UiProbe;

struct BarSample;

impl BarSample {
    const WIDTH: f32 = 1000.0;

    fn view(tab: LibraryAppBarTab, back: bool) -> LibraryAppBarView {
        LibraryAppBarView {
            monogram: words("DV"),
            name: words("DejaVu"),
            library: words("Library"),
            settings: words("Settings"),
            refresh: words("Refresh"),
            open_file: words("Open file"),
            back: back.then(|| words("Back to replay")),
            tab,
        }
    }

    fn click(view: &LibraryAppBarView, text: &str) -> Option<LibraryAppBarAction> {
        let mut probe = UiProbe::new(Self::WIDTH);
        let shapes = probe.shapes(|ui| LibraryAppBar::new(view).show(ui));
        let Some(rect) = UiProbe::text_rect(&shapes, text) else { panic!("{text} is not painted") };
        probe.click(rect.center(), |ui| LibraryAppBar::new(view).show(ui))
    }
}

#[test]
fn the_only_text_is_the_view_text() {
    let view = BarSample::view(LibraryAppBarTab::Library, true);
    let shapes = UiProbe::new(BarSample::WIDTH).shapes(|ui| LibraryAppBar::new(&view).show(ui));
    let mut painted = UiProbe::words(&shapes);
    painted.sort();
    let mut expected: Vec<String> =
        ["DV", "DejaVu", "Library", "Settings", "Refresh", "Open file", "Back to replay"].map(String::from).to_vec();
    expected.sort();
    assert_eq!(painted, expected);
}

#[test]
fn no_back_button_without_a_back_text() {
    let view = BarSample::view(LibraryAppBarTab::Library, false);
    let shapes = UiProbe::new(BarSample::WIDTH).shapes(|ui| LibraryAppBar::new(&view).show(ui));
    assert_eq!(UiProbe::words(&shapes).len(), 6);
    assert_eq!(BarSample::click(&view, "Refresh"), Some(LibraryAppBarAction::Refresh));
}

#[test]
fn bar_is_fifty_six_high_and_as_wide_as_the_ui() {
    let view = BarSample::view(LibraryAppBarTab::Library, true);
    let (size, _) = UiProbe::new(BarSample::WIDTH).frame(Vec::new(), |ui| {
        LibraryAppBar::new(&view).show(ui);
        ui.min_rect().size()
    });
    assert_eq!(size, vec2(BarSample::WIDTH, 56.0));
}

#[test]
fn a_click_on_the_other_tab_selects_it_and_on_the_same_tab_does_nothing() {
    let view = BarSample::view(LibraryAppBarTab::Library, true);
    assert_eq!(BarSample::click(&view, "Settings"), Some(LibraryAppBarAction::SelectTab(LibraryAppBarTab::Settings)));
    assert_eq!(BarSample::click(&view, "Library"), None);
    let view = BarSample::view(LibraryAppBarTab::Settings, true);
    assert_eq!(BarSample::click(&view, "Library"), Some(LibraryAppBarAction::SelectTab(LibraryAppBarTab::Library)));
}

#[test]
fn each_button_reports_its_action() {
    let view = BarSample::view(LibraryAppBarTab::Library, true);
    assert_eq!(BarSample::click(&view, "Refresh"), Some(LibraryAppBarAction::Refresh));
    assert_eq!(BarSample::click(&view, "Open file"), Some(LibraryAppBarAction::OpenFile));
    assert_eq!(BarSample::click(&view, "Back to replay"), Some(LibraryAppBarAction::Back));
}

#[test]
fn the_active_tab_has_a_two_point_accent_line_under_its_word() {
    for (tab, word) in [(LibraryAppBarTab::Library, "Library"), (LibraryAppBarTab::Settings, "Settings")] {
        let view = BarSample::view(tab, true);
        let shapes = UiProbe::new(BarSample::WIDTH).shapes(|ui| LibraryAppBar::new(&view).show(ui));
        let Some(text) = UiProbe::text_rect(&shapes, word) else { panic!("{word} is not painted") };
        let lines: Vec<Rect> = UiProbe::rects(&shapes)
            .into_iter()
            .filter(|r| r.fill == color::Accent::VALUE && r.rect.height() == 2.0)
            .map(|r| r.rect)
            .collect();
        assert_eq!(lines.len(), 1, "{lines:?}");
        assert!((lines[0].left() - text.left()).abs() <= 1.0 && (lines[0].right() - text.right()).abs() <= 1.0);
        assert!(lines[0].top() > text.bottom(), "the line {:?} is not under the word {text:?}", lines[0]);
    }
}

#[test]
fn the_monogram_sits_on_an_accent_tile() {
    let view = BarSample::view(LibraryAppBarTab::Library, true);
    let shapes = UiProbe::new(BarSample::WIDTH).shapes(|ui| LibraryAppBar::new(&view).show(ui));
    let tile = UiProbe::rects(&shapes)
        .into_iter()
        .find(|r| r.fill == color::Accent::VALUE && r.rect.size() == vec2(26.0, 26.0));
    let Some(tile) = tile else { panic!("no monogram tile") };
    let Some(mark) = UiProbe::text_rect(&shapes, "DV") else { panic!("no monogram") };
    assert!(tile.rect.contains_rect(mark));
}
