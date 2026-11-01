use amml_studio_ui::component::viewer::{
    CameraReadout, ReadoutLayout, ViewerChips, ViewerChipsView, ViewerOwner, ViewerSource, ViewerTone,
};
use amml_studio_ui::tokens::selector::SceneColor;
use egui::{Rect, pos2, vec2};

use crate::glyph::words;
use crate::support::UiProbe;

fn readout() -> CameraReadout {
    CameraReadout {
        x: words("186.0"),
        y: words("82.0"),
        z: words("-124.0"),
        yaw: words("325.0°"),
        pitch: words("14.0°"),
        roll: words("0.0°"),
        extra: Some(words("12:00 · Clear")),
    }
}

// every chip at once: a scene, a lens, the mode, the scene camera owner, a warning and the readout
fn busy() -> ViewerChipsView {
    ViewerChipsView {
        source: ViewerSource::Scene { name: words("Portal run"), color: SceneColor::Pink, index: 2, count: 5 },
        lens: Some(words("FOV 79° · 22 mm")),
        tone: ViewerTone::SceneCamera,
        mode: vec![words("Preview"), words("Playing 1.00×")],
        owner: Some(ViewerOwner::SceneCamera(words("Portal run"))),
        warning: Some(words("Timelapse clamped to 4.00× in preview")),
        readout: Some(readout()),
    }
}

#[test]
fn chips_stay_inside_the_viewer() {
    let view = busy();
    for size in [vec2(640.0, 360.0), vec2(1600.0, 900.0)] {
        let viewer = Rect::from_min_size(pos2(20.0, 30.0), size);
        let shapes = UiProbe::new(1700.0).shapes(|ui| ViewerChips::new(&view).show(ui, viewer));
        assert!(!shapes.is_empty());
        for shape in &shapes {
            let bounds = shape.visual_bounding_rect();
            assert!(viewer.contains_rect(bounds), "{bounds:?} is outside the {size:?} viewer");
        }
    }
}

#[test]
fn readout_drops_runs_on_small_pictures() {
    assert_eq!(ReadoutLayout::for_width(719.0), ReadoutLayout::Small);
    assert_eq!(ReadoutLayout::for_width(720.0), ReadoutLayout::Middle);
    assert_eq!(ReadoutLayout::for_width(980.0), ReadoutLayout::Wide);
    let view = busy();
    let small = Rect::from_min_size(pos2(0.0, 0.0), vec2(640.0, 360.0));
    let texts = UiProbe::texts(&UiProbe::new(1700.0).shapes(|ui| ViewerChips::new(&view).show(ui, small)));
    assert!(texts.iter().any(|text| text == "186.0"), "{texts:?}");
    assert!(!texts.iter().any(|text| text == "325.0°"), "{texts:?}");
}

#[test]
fn viewer_with_no_chips() {
    let view = ViewerChipsView {
        source: ViewerSource::Original,
        lens: None,
        tone: ViewerTone::Edit,
        mode: Vec::new(),
        owner: None,
        warning: None,
        readout: None,
    };
    let viewer = Rect::from_min_size(pos2(0.0, 0.0), vec2(640.0, 360.0));
    let texts = UiProbe::texts(&UiProbe::new(700.0).shapes(|ui| ViewerChips::new(&view).show(ui, viewer)));
    assert_eq!(texts, ["Original", "As recorded"]);
}
