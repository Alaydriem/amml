use amml_studio_ui::component::dialog::{
    ExportChoice, ExportDialog, ExportDialogAction, ExportDialogView, ExportFrameRate, ExportResolution,
};

use super::export;
use crate::glyph::words;
use crate::support::UiProbe;

// the words the card paints
fn texts(view: &ExportDialogView) -> Vec<String> {
    UiProbe::texts(&UiProbe::sized(1000.0, 900.0).shapes(|ui| ExportDialog::new(view).show_card(ui)))
}

#[test]
fn the_dialog_has_no_motion_blur_row_and_keeps_the_queue_actions() {
    let texts = texts(&export());
    for gone in ["MOTION BLUR", "ProRes 422", "PNG seq."] {
        assert!(!texts.iter().any(|text| text == gone), "{gone} still shows: {texts:?}");
    }
    for shown in ["FORMAT", "H.264", "H.265", "SIDECAR FILE", "Render", "Add to queue", "Open queue"] {
        assert!(texts.iter().any(|text| text == shown), "{shown} is missing: {texts:?}");
    }
}

#[test]
fn the_sidecar_row_has_its_label_and_switch_only() {
    let shapes = UiProbe::sized(1000.0, 900.0).shapes(|ui| ExportDialog::new(&export()).show_card(ui));
    let label = UiProbe::text_rect(&shapes, "SIDECAR FILE").unwrap_or_else(|| panic!("no sidecar row"));
    let on_the_row: Vec<String> = shapes
        .iter()
        .filter_map(|shape| match shape {
            egui::Shape::Text(text) => {
                let rect = text.galley.rect.translate(text.pos.to_vec2());
                (rect.center().y - label.center().y).abs().lt(&8.0).then(|| text.galley.text().to_owned())
            }
            _ => None,
        })
        .collect();
    assert_eq!(on_the_row, vec!["SIDECAR FILE".to_owned()], "the sidecar row paints a note");
}

#[test]
fn a_blocked_render_shows_why_and_does_nothing() {
    let mut view = export();
    view.render_blocked = Some(words("No ffmpeg is set."));
    let shapes = UiProbe::sized(1000.0, 900.0).shapes(|ui| ExportDialog::new(&view).show_card(ui));
    assert!(UiProbe::texts(&shapes).iter().any(|text| text == "No ffmpeg is set."));

    let at = UiProbe::text_rect(&shapes, "Render").expect("the Render button").center();
    let mut probe = UiProbe::sized(1000.0, 900.0);
    let action = probe.click(at, |ui| ExportDialog::new(&view).show_card(ui));
    assert_eq!(action, None);

    let open = export();
    let mut probe = UiProbe::sized(1000.0, 900.0);
    let action = probe.click(at, |ui| ExportDialog::new(&open).show_card(ui));
    assert_eq!(action, Some(ExportDialogAction::RenderNow));
}

#[test]
fn an_unavailable_choice_cannot_be_picked() {
    let mut view = export();
    view.unavailable = vec![(ExportChoice::Resolution(ExportResolution::Qhd), words("Not here"))];
    let shapes = UiProbe::sized(1000.0, 900.0).shapes(|ui| ExportDialog::new(&view).show_card(ui));
    let qhd = UiProbe::text_rect(&shapes, "1440p").expect("the 1440p segment").center();
    let full_hd = UiProbe::text_rect(&shapes, "1080p").expect("the 1080p segment").center();

    let mut probe = UiProbe::sized(1000.0, 900.0);
    assert_eq!(probe.click(qhd, |ui| ExportDialog::new(&view).show_card(ui)), None);
    let mut probe = UiProbe::sized(1000.0, 900.0);
    assert_eq!(
        probe.click(full_hd, |ui| ExportDialog::new(&view).show_card(ui)),
        Some(ExportDialogAction::SetResolution(ExportResolution::FullHd))
    );
    assert_eq!(ExportFrameRate::ALL.len(), 3);
}

#[test]
fn the_queue_buttons_send_their_actions() {
    let view = export();
    let shapes = UiProbe::sized(1000.0, 900.0).shapes(|ui| ExportDialog::new(&view).show_card(ui));
    for (word, action) in
        [("Add to queue", ExportDialogAction::AddToQueue), ("Open queue", ExportDialogAction::OpenQueue)]
    {
        let at = UiProbe::text_rect(&shapes, word).unwrap_or_else(|| panic!("no {word}")).center();
        let mut probe = UiProbe::sized(1000.0, 900.0);
        assert_eq!(probe.click(at, |ui| ExportDialog::new(&view).show_card(ui)), Some(action), "{word}");
    }
}

#[test]
fn a_blocked_scene_cannot_be_added_to_the_queue() {
    let mut view = export();
    view.render_blocked = Some(words("No ffmpeg is set."));
    let shapes = UiProbe::sized(1000.0, 900.0).shapes(|ui| ExportDialog::new(&view).show_card(ui));
    let at = UiProbe::text_rect(&shapes, "Add to queue").expect("the Add to queue button").center();
    let mut probe = UiProbe::sized(1000.0, 900.0);
    assert_eq!(probe.click(at, |ui| ExportDialog::new(&view).show_card(ui)), None);
}
