use amml_studio_ui::component::dialog::{RenameAction, RenameDialog};
use egui::{Event, Key};

use crate::glyph::words;
use crate::support::UiProbe;

#[test]
fn rename_enter_confirms_the_text() {
    let mut value = String::new();
    let title = words("Rename scene");
    let mut probe = UiProbe::sized(1000.0, 700.0);
    let mut show = |ui: &mut egui::Ui| RenameDialog::new(&title, &mut value).show(ui.ctx());
    probe.frame(Vec::new(), &mut show);
    probe.frame(vec![Event::Text("Intro".to_owned())], &mut show);
    let (action, _) = probe.frame(vec![UiProbe::key(Key::Enter)], &mut show);
    assert_eq!(action, Some(RenameAction::Confirm("Intro".to_owned())));
}

#[test]
fn rename_with_empty_text_cannot_confirm() {
    let mut value = String::new();
    let title = words("Rename scene");
    let mut probe = UiProbe::sized(1000.0, 700.0);
    let shapes = probe.shapes(|ui| RenameDialog::new(&title, &mut value).show(ui.ctx()));
    let button = UiProbe::text_rect(&shapes, "Rename").expect("the Rename button is painted");
    let clicked = probe.click(button.center(), |ui| RenameDialog::new(&title, &mut value).show(ui.ctx()));
    assert_eq!(clicked, None);
    let (pressed, _) =
        probe.frame(vec![UiProbe::key(Key::Enter)], |ui| RenameDialog::new(&title, &mut value).show(ui.ctx()));
    assert_eq!(pressed, None);
}

#[test]
fn rename_with_a_problem_cannot_confirm_and_shows_the_problem() {
    let mut value = "p1".to_owned();
    let title = words("Rename replay");
    let problem = || Some(words("A replay with this name is already there"));
    let mut probe = UiProbe::sized(1000.0, 700.0);
    let shapes = probe.shapes(|ui| RenameDialog::new(&title, &mut value).problem(problem()).show(ui.ctx()));
    assert!(UiProbe::text_rect(&shapes, "A replay with this name is already there").is_some());
    let button = UiProbe::text_rect(&shapes, "Rename").expect("the Rename button is painted");
    let clicked =
        probe.click(button.center(), |ui| RenameDialog::new(&title, &mut value).problem(problem()).show(ui.ctx()));
    assert_eq!(clicked, None);
    let (pressed, _) = probe.frame(vec![UiProbe::key(Key::Enter)], |ui| {
        RenameDialog::new(&title, &mut value).problem(problem()).show(ui.ctx())
    });
    assert_eq!(pressed, None);
}

#[test]
fn rename_shows_the_hint_it_is_given() {
    let mut value = "p1".to_owned();
    let title = words("Rename replay");
    let mut probe = UiProbe::sized(1000.0, 700.0);
    let shapes = probe.shapes(|ui| RenameDialog::new(&title, &mut value).hint(words("My own hint")).show(ui.ctx()));
    assert!(UiProbe::text_rect(&shapes, "My own hint").is_some());
}
