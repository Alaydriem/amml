use egui::{Rect, Sense, Ui, vec2};

use super::{BarRows, ViewerSamples};
use crate::component::viewer::{
    CompositionGuides, ViewerChips, ViewerChipsView, ViewerOverlay, ViewerOwner, ViewerSource, ViewerTone,
};
use crate::gallery::GalleryState;
use crate::glyph::NameTag;
use crate::tokens::selector::SceneColor;

/// Paper: Editor components, `Viewer, edit free camera` (`2LV-0`), `Viewer, preview with scene camera` (`30B-0`) and
/// `Viewer, original, typing` (`3I7-0`). The world gizmo is the host's (Plan 08).
pub struct ViewerPage;

impl ViewerPage {
    /// Paper: the stage is 1490 × 626, the picture 1064 × 598 in its centre.
    pub const WIDTH: f32 = 1490.0;
    const STAGE: f32 = 626.0;
    const PICTURE: [f32; 2] = [1064.0, 598.0];
    // Paper: the path map sits at (800, 414) in the picture
    const MAP: [f32; 2] = [800.0, 414.0];
    // Paper: the name labels' places in `2LV-0` and `3I7-0`
    const TAGS: [(&'static str, f32, f32); 5] = [
        ("nether_nomad", 216.0, 296.0),
        ("PixelPilgrim", 408.0, 256.0),
        ("AuroraMiner", 500.0, 258.0),
        ("basalt_bea", 553.0, 272.0),
        ("Kestrel", 396.0, 368.0),
    ];
    // a name tag is placed by its top left; this is the room it is centred in
    const TAG_ROOM: [f32; 2] = [96.0, 20.0];

    pub fn show(ui: &mut Ui, _state: &mut GalleryState) {
        BarRows::page(ui, |ui| {
            ViewerSamples::row(ui, "EDIT · FREE CAMERA", |ui| {
                let picture = Self::stage(ui, true);
                Self::tags(ui, picture, &Self::TAGS);
                let view = Self::chips(
                    ViewerSource::Scene {
                        name: ViewerSamples::words("Arrival"),
                        color: SceneColor::Lime,
                        index: 1,
                        count: 5,
                    },
                    "FOV 79° · 22 mm",
                    ViewerTone::Edit,
                    &["Edit", "Free camera"],
                    None,
                    None,
                    ["186.0", "82.0", "-124.0", "325.0°", "14.0°", "0.0°"],
                );
                ViewerChips::new(&view).show(ui, picture);
                let map = ViewerSamples::path_map("Path · Arrival", true, None, false, false);
                ViewerSamples::show_map(ui, picture.min + vec2(Self::MAP[0], Self::MAP[1]), &map);
            });
            ViewerSamples::row(ui, "PREVIEW · SCENE CAMERA", |ui| {
                let picture = Self::stage(ui, true);
                Self::tags(ui, picture, &[("Kestrel", 716.0, 330.0)]);
                let view = Self::chips(
                    ViewerSource::Scene {
                        name: ViewerSamples::words("Arrival"),
                        color: SceneColor::Lime,
                        index: 1,
                        count: 5,
                    },
                    "FOV 83° · 20 mm",
                    ViewerTone::SceneCamera,
                    &["Preview", "Playing 1.00×"],
                    Some(ViewerOwner::SceneCamera(ViewerSamples::words("Arrival"))),
                    Some("Timelapse clamped to 4.00× in preview"),
                    ["207.2", "72.1", "-94.0", "124.7°", "-8.4°", "-0.2°"],
                );
                ViewerChips::new(&view).show(ui, picture);
                let map = ViewerSamples::path_map("Path · Arrival", true, None, false, true);
                ViewerSamples::show_map(ui, picture.min + vec2(Self::MAP[0], Self::MAP[1]), &map);
            });
            ViewerSamples::row(ui, "ORIGINAL · TYPING", |ui| {
                let picture = Self::stage(ui, false);
                ViewerOverlay::Typing.paint(ui.painter(), picture);
                Self::tags(ui, picture, &Self::TAGS);
                let view = Self::chips(
                    ViewerSource::Original,
                    "FOV 72° · 25 mm",
                    ViewerTone::Edit,
                    &["Edit", "Free camera"],
                    None,
                    None,
                    ["186.0", "82.0", "-124.0", "325.0°", "14.0°", "0.0°"],
                );
                ViewerChips::new(&view).show(ui, picture);
                let map = ViewerSamples::path_map("Path · Original (no camera)", false, None, false, false);
                ViewerSamples::show_map(ui, picture.min + vec2(Self::MAP[0], Self::MAP[1]), &map);
            });
        });
    }

    // the stage, and the picture in its centre (with the composition guides when `picture`); returns the picture
    fn stage(ui: &mut Ui, picture: bool) -> Rect {
        let (stage, _) = ui.allocate_exact_size(vec2(Self::WIDTH, Self::STAGE), Sense::hover());
        let frame = Rect::from_center_size(stage.center(), vec2(Self::PICTURE[0], Self::PICTURE[1]));
        if picture {
            ViewerSamples::picture(ui, frame);
            CompositionGuides::paint(ui.painter(), frame);
        }
        frame
    }

    fn tags(ui: &Ui, picture: Rect, tags: &[(&str, f32, f32)]) {
        for (name, x, y) in tags {
            let room = Rect::from_min_size(picture.min + vec2(*x, *y), vec2(Self::TAG_ROOM[0], Self::TAG_ROOM[1]));
            NameTag { name: ViewerSamples::words(name) }.paint(ui.painter(), room);
        }
    }

    fn chips(
        source: ViewerSource,
        lens: &str,
        tone: ViewerTone,
        mode: &[&str],
        owner: Option<ViewerOwner>,
        warning: Option<&str>,
        [x, y, z, yaw, pitch, roll]: [&str; 6],
    ) -> ViewerChipsView {
        ViewerChipsView {
            source,
            lens: Some(ViewerSamples::words(lens)),
            tone,
            mode: mode.iter().map(|word| ViewerSamples::words(word)).collect(),
            owner,
            warning: warning.map(ViewerSamples::words),
            readout: Some(ViewerSamples::readout(x, y, z, yaw, pitch, roll, Some("12:00 · Clear"))),
        }
    }
}
