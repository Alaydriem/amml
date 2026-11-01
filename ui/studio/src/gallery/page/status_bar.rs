use amml_i18n::text::{Text, UserText};
use egui::Ui;

use super::BarRows;
use crate::component::bar::{StatusBar, StatusBarLayout, StatusBarView};
use crate::gallery::GalleryState;
use crate::tokens::selector::ModeColor;

/// Paper: Editor components, `Status bar` (`20D-0`). One row per mode at 1920, and Edit at 1280.
pub struct StatusBarPage;

impl StatusBarPage {
    /// Paper: the status bar rows are 1920 wide; the last is the 1280 window.
    pub const WIDTH: f32 = 1920.0;
    const SMALL: f32 = 1280.0;
    const HINTS: &'static str = "Fly 1.00×   ·   UI 100%  Ctrl+= / Ctrl+- / Ctrl+0   ·   badges on (B)";

    pub fn show(ui: &mut Ui, _state: &mut GalleryState) {
        BarRows::page(ui, |ui| {
            let rows = [
                ("EDIT", ModeColor::Edit, "Edit · free camera", "Rewind cache 1609 checkpoints · no seek yet", false),
                (
                    "PREVIEW · SCENE CAMERA DRIVES",
                    ModeColor::Preview,
                    "Preview · scene camera drives · edits locked",
                    "Rewind cache 1609 · last seek 250 ms from 00:00:25",
                    true,
                ),
                (
                    "PREVIEW ON · PAUSED",
                    ModeColor::Preview,
                    "Preview on · paused · free camera",
                    "Rewind cache 1609 checkpoints · last seek 250 ms from 00:00:25",
                    false,
                ),
                (
                    "READ ONLY",
                    ModeColor::ReadOnly,
                    "Read only: recorded by an older DejaVu",
                    "Rewind cache 1609 checkpoints · no seek yet",
                    false,
                ),
                (
                    "REBUILDING WORLD",
                    ModeColor::Busy,
                    "Rebuilding world",
                    "Rewind cache 1609 checkpoints · last seek 250 ms from 00:00:10",
                    false,
                ),
            ];
            for (caption, mode, words, detail, tinted) in rows {
                let view = Self::view(mode, words, detail, Self::HINTS, tinted);
                BarRows::row(ui, caption, Self::WIDTH, |ui| {
                    StatusBar::new(&view).show(ui);
                });
            }
            let view = Self::view(ModeColor::Edit, "Edit · free camera", "", "UI 100%   ·   badges off (B)", false);
            BarRows::row(ui, "1280", Self::SMALL, |ui| {
                StatusBar::new(&view).layout(StatusBarLayout::Compact).show(ui);
            });
        });
    }

    fn words(text: &str) -> Text {
        Text::user(UserText::from_model(text.to_owned()))
    }

    fn view(mode: ModeColor, words: &str, detail: &str, hints: &str, tinted: bool) -> StatusBarView {
        StatusBarView {
            mode,
            mode_text: Self::words(words),
            detail: Self::words(detail),
            shortcuts: Self::words(hints),
            tinted,
        }
    }
}
