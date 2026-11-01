use amml_i18n::text::{Text, UserText};
use amml_ui::draw::{Corner, Draw};
use amml_ui::gallery::SheetGrid;
use amml_ui::tokens::{ColorToken, color};
use egui::{Pos2, Rect, Ui, UiBuilder, pos2};

use crate::component::viewer::{CameraReadout, PathMap, PathMapCamera, PathMapPoint, PathMapView};
use crate::tokens::selector::SceneColor;

/// The sample data the viewer pages share: Paper's Arrival path, players and readout (`2LV-0`, `46E-0`).
pub struct ViewerSamples;

impl ViewerSamples {
    /// Paper: the path keys, camera and players, in the map's own coordinates.
    const PATH: [Pos2; 6] = [
        pos2(104.0, 80.0),
        pos2(122.0, 96.0),
        pos2(138.0, 106.0),
        pos2(150.0, 120.0),
        pos2(162.0, 136.0),
        pos2(168.0, 150.0),
    ];
    const CAMERA: Pos2 = pos2(86.0, 38.0);
    const HEADING: f32 = 2.33;
    /// Sheet chrome: the caption's inset and the space around it, as on the bar pages.
    const CAPTION_INSET: f32 = 24.0;
    const CAPTION_TOP: f32 = 20.0;
    const CAPTION_BOTTOM: f32 = 8.0;
    const PLAYERS: [Pos2; 5] =
        [pos2(114.0, 62.0), pos2(117.0, 106.0), pos2(134.0, 114.0), pos2(176.0, 66.0), pos2(191.0, 137.0)];

    pub fn words(text: &str) -> Text {
        Text::user(UserText::from_model(text.to_owned()))
    }

    /// The stand-in for the game picture: the studio bundles no images.
    pub fn picture(ui: &Ui, rect: Rect) {
        Draw::fill(ui.painter(), rect, Corner::Small, <color::BgRaised as ColorToken>::VALUE);
    }

    /// The path map of Arrival; `selected` marks a key, `path` false leaves the path out (the Original replay).
    pub fn path_map(label: &str, path: bool, selected: Option<usize>, hidden: bool, scene_drives: bool) -> PathMapView {
        let keys = if path {
            Self::PATH
                .iter()
                .enumerate()
                .map(|(index, at)| PathMapPoint { at: *at, selected: Some(index) == selected })
                .collect()
        } else {
            Vec::new()
        };
        PathMapView {
            label: Self::words(label),
            scene: SceneColor::Lime,
            path: keys,
            hidden,
            camera: PathMapCamera { at: Self::CAMERA, heading: Self::HEADING },
            scene_drives,
            players: Self::PLAYERS.to_vec(),
        }
    }

    pub fn readout(
        x: &str,
        y: &str,
        z: &str,
        yaw: &str,
        pitch: &str,
        roll: &str,
        extra: Option<&str>,
    ) -> CameraReadout {
        CameraReadout {
            x: Self::words(x),
            y: Self::words(y),
            z: Self::words(z),
            yaw: Self::words(yaw),
            pitch: Self::words(pitch),
            roll: Self::words(roll),
            extra: extra.map(Self::words),
        }
    }

    /// A small-caps caption, then `body` in the page's column. The viewer pages draw full-width stages, so the body
    /// allocates straight into the page.
    pub fn row(ui: &mut Ui, caption: &str, body: impl FnOnce(&mut Ui)) {
        ui.add_space(Self::CAPTION_TOP);
        ui.horizontal(|ui| {
            ui.add_space(Self::CAPTION_INSET);
            SheetGrid::head(ui, caption);
        });
        ui.add_space(Self::CAPTION_BOTTOM);
        body(ui);
    }

    /// Shows the path map with its top left at `at`.
    pub fn show_map(ui: &mut Ui, at: Pos2, view: &PathMapView) {
        let place = Rect::from_min_size(at, PathMap::SIZE);
        ui.scope_builder(UiBuilder::new().max_rect(place), |ui| PathMap::new(view).show(ui));
    }
}
