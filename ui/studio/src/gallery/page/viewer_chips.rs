use amml_i18n::text::Text;
use amml_ui::tokens::{ColorToken, SpacingToken, color, spacing};
use egui::{Rect, Sense, Ui, pos2, vec2};

use super::{BarRows, ViewerSamples};
use crate::component::viewer::{ViewerChips, ViewerOverlay, ViewerOwner, ViewerPill, ViewerTone};
use crate::gallery::GalleryState;
use crate::message::StudioUiMessage;
use crate::tokens::selector::SceneColor;

/// Paper: Editor components, `Viewer chips and overlays` (`3TE-0`). The mode chips, the source, lens and owner
/// chips, the readout at three picture widths, and the rebuild and join cards. The toasts are the amml-ui `Toast`.
pub struct ViewerChipsPage;

impl ViewerChipsPage {
    pub const WIDTH: f32 = 1490.0;
    // sheet chrome: the band each section is drawn on, and the padding round it
    const BAND: f32 = 60.0;
    const PADDING: f32 = 16.0;
    const READOUT_BAND: f32 = 58.0;
    // Paper: the readout at a wide (980 and up), a middle and a small picture
    const READOUT_WIDTHS: [f32; 3] = [1064.0, 800.0, 640.0];
    const CARDS: f32 = 160.0;
    /// Paper: the picture each owner tag sample sits on.
    const OWNER_VIEWER: egui::Vec2 = vec2(280.0, 40.0);

    pub fn show(ui: &mut Ui, _state: &mut GalleryState) {
        BarRows::page(ui, |ui| {
            ViewerSamples::row(ui, "A · MODE CHIPS", Self::modes);
            ViewerSamples::row(ui, "B · SCENE, LENS AND OWNER", Self::sources);
            ViewerSamples::row(ui, "C · CAMERA READOUT", Self::readouts);
            ViewerSamples::row(ui, "D · REBUILD AND JOIN", Self::cards);
        });
    }

    fn band(ui: &mut Ui, height: f32) -> Rect {
        let (rect, _) = ui.allocate_exact_size(vec2(Self::WIDTH, height), Sense::hover());
        let band = rect.shrink2(vec2(Self::PADDING * 1.5, 0.0));
        ViewerSamples::picture(ui, band);
        band
    }

    // the pills of `chips` in a row from the band's left, `--spacing-m` apart
    fn row(ui: &Ui, band: Rect, chips: &[ViewerPill<'_>]) {
        let mut x = band.left() + Self::PADDING;
        for chip in chips {
            let width = chip.width(ui.ctx());
            let rect = Rect::from_min_size(
                pos2(x, band.center().y - ViewerPill::HEIGHT / 2.0),
                vec2(width, ViewerPill::HEIGHT),
            );
            chip.paint(ui.painter(), rect);
            x = rect.right() + <spacing::M as SpacingToken>::VALUE;
        }
    }

    // two bands of three chips: Paper wraps the row
    fn modes(ui: &mut Ui) {
        let sets: [(ViewerTone, Vec<Text>); 6] = [
            (ViewerTone::Edit, Self::words(&["Edit", "Free camera"])),
            (ViewerTone::Playing, Self::words(&["Edit", "Playing", "Free camera"])),
            (ViewerTone::Paused, Self::words(&["Preview on", "Paused", "Free camera"])),
            (ViewerTone::Paused, Self::words(&["Preview", "Outside scene", "Free camera"])),
            (ViewerTone::SceneCamera, Self::words(&["Preview", "Playing 1.00×"])),
            (ViewerTone::Following, Self::words(&["Following a player"])),
        ];
        let chips: Vec<ViewerPill<'_>> = sets
            .iter()
            .map(|(tone, parts)| ViewerPill { dot: Some(tone.color()), border: tone.border(), parts })
            .collect();
        for half in chips.chunks(3) {
            let band = Self::band(ui, Self::BAND);
            Self::row(ui, band, half);
        }
    }

    fn sources(ui: &mut Ui) {
        let band = Self::band(ui, Self::BAND);
        let overlay = <color::LineOverlay as ColorToken>::VALUE;
        let scene = vec![
            Text::message(StudioUiMessage::ViewerScene),
            ViewerSamples::words("Portal run"),
            ViewerSamples::words("2 of 5"),
        ];
        let original = vec![Text::message(StudioUiMessage::Original), Text::message(StudioUiMessage::AsRecorded)];
        let lens = Self::words(&["FOV 64° · 29 mm"]);
        let chips = [
            ViewerPill { dot: Some(SceneColor::Pink.color()), border: overlay, parts: &scene },
            ViewerPill { dot: None, border: overlay, parts: &original },
            ViewerPill { dot: None, border: overlay, parts: &lens },
        ];
        Self::row(ui, band, &chips);
        // the owner tags are centred in a stand-in viewer; place two of them on the band's right half
        for (index, owner) in [
            ViewerOwner::SceneCamera(ViewerSamples::words("Portal run")),
            ViewerOwner::Following(ViewerSamples::words("AuroraMiner")),
        ]
        .iter()
        .enumerate()
        {
            let viewer = Rect::from_min_size(
                pos2(band.left() + 760.0 + 300.0 * index as f32, band.top() + 11.0),
                Self::OWNER_VIEWER,
            );
            ViewerChips::owner_tag(ui.painter(), viewer, owner);
        }
    }

    fn readouts(ui: &mut Ui) {
        let readout =
            ViewerSamples::readout("186.0", "82.0", "-124.0", "325.0°", "14.0°", "0.0°", Some("recorded · recorded"));
        for width in Self::READOUT_WIDTHS {
            let (rect, _) = ui.allocate_exact_size(vec2(Self::WIDTH, Self::READOUT_BAND), Sense::hover());
            let viewer = Rect::from_min_size(
                pos2(rect.left() + Self::PADDING * 1.5, rect.top()),
                vec2(width, Self::READOUT_BAND),
            );
            ViewerSamples::picture(ui, viewer);
            ViewerChips::readout(ui.painter(), viewer, &readout);
        }
    }

    fn cards(ui: &mut Ui) {
        let (rect, _) = ui.allocate_exact_size(vec2(Self::WIDTH, Self::CARDS), Sense::hover());
        let rebuild = Rect::from_min_size(rect.min + vec2(Self::PADDING * 1.5, 0.0), vec2(432.0, Self::CARDS));
        ViewerOverlay::Rebuilding { progress: 0.73, detail: ViewerSamples::words("from checkpoint 00:00:10") }
            .paint(ui.painter(), rebuild);
        let join = Rect::from_min_size(pos2(rebuild.right() + Self::PADDING, rect.top()), vec2(512.0, Self::CARDS));
        ViewerOverlay::Joining {
            title: ViewerSamples::words("Joining “Nether trip”"),
            body: ViewerSamples::words("Attempt 1 of 5 · loopback server ready · 1.4 s"),
            hint: ViewerSamples::words("After 5 failed tries the mod asks you to join by hand."),
        }
        .paint(ui.painter(), join);
    }

    fn words(parts: &[&str]) -> Vec<Text> {
        parts.iter().map(|part| ViewerSamples::words(part)).collect()
    }
}
