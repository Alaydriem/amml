use amml_ui::gallery::{Sample, SheetGrid};
use egui::{Ui, vec2};

use crate::component::library::{HeroCard, HeroCardView, HeroProblem};
use crate::gallery::GalleryState;

/// Paper: Editor components, `Editor / Hero card` (`G64-0`). A card that opens and a broken one. The studio bundles
/// no pictures, so the card shows the app background in the picture's place.
pub struct HeroCardPage;

impl HeroCardPage {
    /// Sheet chrome: the card size.
    const SIZE: egui::Vec2 = vec2(900.0, 506.0);

    pub fn show(ui: &mut Ui, _state: &mut GalleryState) {
        SheetGrid::page(ui, "Hero card", |ui| {
            let problem = || HeroProblem {
                title: Sample::text("Game version not supported"),
                reason: Sample::text("This recording was made with a game protocol this version cannot play."),
            };
            let cards = [
                ("READY", None, "Nether trip", ["02:14:08", "6 players", "5 scenes", "Today, 14:02"]),
                ("BROKEN", Some(problem()), "Old realm backup", ["02:14:08", "6 players", "5 scenes", "Sep 30"]),
            ];
            for (caption, problem, title, meta) in cards {
                let view = HeroCardView {
                    title: Sample::text(title),
                    meta: meta.iter().map(|fact| Sample::text(fact)).collect(),
                    open: Sample::text("Open in editor"),
                    problem,
                };
                SheetGrid::section(ui, caption, |ui| {
                    HeroCard::new(&view).show(ui, Self::SIZE);
                });
            }
        });
    }
}
