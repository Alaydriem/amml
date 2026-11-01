use egui::{Id, Rect, Shape, Ui, UiBuilder, pos2, vec2};

use crate::draw::{Corner, TextStyle, TypeSize};
use crate::font::FontFace;
use crate::gallery::{GalleryState, Sample, SheetGrid};
use crate::tokens::{ColorToken, ContainerToken, SpacingToken, color, container, spacing};
use crate::widget::{ButtonKind, ModalFrame, TextButton};

/// Paper: Widgets, `Widget / Modal frame` (`70K-0`). Two cards on a stage: the game picture
/// stand-in under the `--color-bg-overlay` scrim, `--spacing-lane` padding round the cards.
pub struct ModalFrameSheet;

impl ModalFrameSheet {
    const BODY: TextStyle = TextStyle::new(TypeSize::Help, FontFace::Regular);
    const NOTE: TextStyle = TextStyle::new(TypeSize::Note, FontFace::Regular);

    pub fn show(ui: &mut Ui, _state: &mut GalleryState) {
        SheetGrid::page(ui, "Modal frame", |ui| {
            let lane = <spacing::Lane as SpacingToken>::VALUE;
            let gap = <spacing::L as SpacingToken>::VALUE;
            let dialog = <container::Dialog as ContainerToken>::VALUE;
            let top = ui.cursor().min;
            let width = ui.available_width();
            // the stage is painted under the cards once their height is known
            let picture = ui.painter().add(Shape::Noop);
            let scrim = ui.painter().add(Shape::Noop);
            let left = top.x + (width - dialog * 2.0 - gap) / 2.0;
            let mut bottom = top.y;
            let examples = [
                (
                    "Export scene",
                    "Arrival · 00:00:28.80 to 00:00:37.40",
                    "Renders the scene to a video file in the replay folder.",
                    "Output 6.2 s at 60 fps · 1920 × 1080",
                    "Render now",
                ),
                (
                    "Close replay",
                    "Portal run · 3 scenes edited",
                    "Scene edits are saved with the replay. Camera, FOV and Time of day keys stay as they are now. \
                     The next time you open Portal run, the editor opens at the same playhead.",
                    "Last saved 00:02 ago",
                    "Close replay",
                ),
            ];
            for (index, (title, subtitle, body, note, action)) in examples.into_iter().enumerate() {
                let at = pos2(left + (dialog + gap) * index as f32, top.y + lane);
                let place = Rect::from_min_size(at, vec2(dialog, f32::INFINITY));
                let card = ui
                    .scope_builder(UiBuilder::new().max_rect(place).id_salt(index), |ui| {
                        ModalFrame::new(Id::NULL, Sample::text(title)).subtitle(Sample::text(subtitle)).show_card(
                            ui,
                            |ui| {
                                SheetGrid::paragraph(ui, body, Self::BODY, <color::Text as ColorToken>::VALUE);
                                SheetGrid::paragraph(ui, note, Self::NOTE, <color::TextDim as ColorToken>::VALUE);
                            },
                            |ui| {
                                TextButton::new(ButtonKind::Primary, Sample::text(action)).show(ui);
                                TextButton::new(ButtonKind::Secondary, Sample::text("Cancel")).show(ui);
                            },
                        )
                    })
                    .inner
                    .card;
                bottom = bottom.max(card.bottom());
            }
            let stage = Rect::from_min_max(top, pos2(top.x + width, bottom + lane));
            let radius = Corner::Panel.radius(stage);
            ui.painter().set(picture, Shape::rect_filled(stage, radius, <color::BgApp as ColorToken>::VALUE));
            ui.painter().set(scrim, Shape::rect_filled(stage, radius, <color::BgOverlay as ColorToken>::VALUE));
            ui.allocate_rect(stage, egui::Sense::hover());
        });
    }
}
