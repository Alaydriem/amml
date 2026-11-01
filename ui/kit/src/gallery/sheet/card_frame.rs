use egui::{Sense, Ui, UiBuilder, vec2};

use crate::draw::{Corner, Draw, TextStyle, TypeSize};
use crate::font::FontFace;
use crate::gallery::{GalleryState, SheetGrid};
use crate::tokens::rule::Elevation;
use crate::tokens::{ColorToken, SpacingToken, color, spacing};
use crate::widget::{Surface, SurfaceLevel};

/// Paper: Widgets, `Widget / Card frame + surfaces` (`337-0`). The game picture is a flat
/// stand-in block, and the scene colour of the pill dot is an amml-ui stand-in.
pub struct CardFrameSheet;

impl CardFrameSheet {
    /// Sheet chrome: a level's slot and swatch, the elevation examples, and the card frame example.
    const SLOT_WIDTH: f32 = 180.0;
    const SWATCH_HEIGHT: f32 = 96.0;
    const EXAMPLE_WIDTH: f32 = 376.0;
    const PICTURE_HEIGHT: f32 = 240.0;
    const CARD_WIDTH: f32 = 300.0;
    /// Paper: Card frame, the pill's scene dot.
    const DOT: f32 = 10.0;
    const LEVELS: [(SurfaceLevel, &'static str, &'static str); 5] = [
        (SurfaceLevel::App, "App", "--color-bg-app"),
        (SurfaceLevel::Panel, "Panel", "--color-bg-panel"),
        (SurfaceLevel::Raised, "Raised", "--color-bg-raised"),
        (SurfaceLevel::Card, "Card", "--color-card"),
        (SurfaceLevel::Pill, "Pill", "--color-pill"),
    ];

    pub fn show(ui: &mut Ui, _state: &mut GalleryState) {
        SheetGrid::page(ui, "Card frame and surfaces", |ui| {
            SheetGrid::panel(ui, |ui| {
                SheetGrid::head(ui, "SURFACE LEVELS");
                ui.horizontal_top(|ui| {
                    ui.spacing_mut().item_spacing = vec2(<spacing::L as SpacingToken>::VALUE, 0.0);
                    for (level, name, token) in Self::LEVELS {
                        Self::level(ui, level, name, token);
                    }
                });
            });
            SheetGrid::panel(ui, |ui| {
                ui.horizontal_top(|ui| {
                    ui.spacing_mut().item_spacing = vec2(<spacing::L as SpacingToken>::VALUE, 0.0);
                    for head in ["FLOATING", "FLAT"] {
                        ui.allocate_ui(vec2(Self::EXAMPLE_WIDTH, 0.0), |ui| {
                            ui.set_width(Self::EXAMPLE_WIDTH);
                            SheetGrid::head(ui, head);
                        });
                    }
                });
                ui.horizontal_top(|ui| {
                    ui.spacing_mut().item_spacing = vec2(<spacing::L as SpacingToken>::VALUE, 0.0);
                    Self::example(ui, Elevation::Floating, "Easing", "In Out Cubic · Camera key 3 to 4");
                    Self::example(ui, Elevation::Flat, "Arrival", "00:00:28.80 to 00:00:37.40 · 4 keys");
                });
            });
            SheetGrid::panel(ui, |ui| {
                SheetGrid::head(ui, "CARD FRAME");
                let (picture, _) =
                    ui.allocate_exact_size(vec2(ui.available_width(), Self::PICTURE_HEIGHT), Sense::hover());
                SheetGrid::picture(ui, picture);
                let inside = picture.shrink(<spacing::L as SpacingToken>::VALUE);
                ui.scope_builder(UiBuilder::new().max_rect(inside), |ui| {
                    ui.horizontal_top(|ui| {
                        ui.spacing_mut().item_spacing = vec2(<spacing::L as SpacingToken>::VALUE, 0.0);
                        Self::camera_card(ui);
                        Self::scene_pill(ui);
                    });
                });
            });
        });
    }

    // a swatch of `level`, its name and its token
    fn level(ui: &mut Ui, level: SurfaceLevel, name: &str, token: &str) {
        ui.vertical(|ui| {
            ui.set_width(Self::SLOT_WIDTH);
            ui.spacing_mut().item_spacing = vec2(0.0, <spacing::S as SpacingToken>::VALUE);
            let (swatch, _) = ui.allocate_exact_size(vec2(Self::SLOT_WIDTH, Self::SWATCH_HEIGHT), Sense::hover());
            match level {
                SurfaceLevel::Card | SurfaceLevel::Pill => {
                    // see-through levels sit on the picture, `--spacing-l` in
                    SheetGrid::picture(ui, swatch);
                    let inside = swatch.shrink(<spacing::L as SpacingToken>::VALUE);
                    let surface = if level == SurfaceLevel::Pill {
                        let row = <spacing::Row as SpacingToken>::VALUE;
                        egui::Rect::from_center_size(inside.center(), vec2(inside.width(), row))
                    } else {
                        inside
                    };
                    Surface::paint(ui.painter(), surface, level, Elevation::Flat);
                }
                _ => Surface::paint(ui.painter(), swatch, level, Elevation::Flat),
            }
            let text = <color::Text as ColorToken>::VALUE;
            SheetGrid::label(ui, name, TextStyle::new(TypeSize::Note, FontFace::Semibold), text);
            let faint = <color::TextFaint as ColorToken>::VALUE;
            SheetGrid::label(ui, token, TextStyle::new(TypeSize::Micro, FontFace::Regular), faint);
        });
    }

    // a raised surface with a title and a subtitle
    fn example(ui: &mut Ui, elevation: Elevation, title: &str, subtitle: &str) {
        let inner = Self::EXAMPLE_WIDTH - Self::chrome_width();
        Surface::frame(SurfaceLevel::Raised, elevation).show(ui, |ui| {
            ui.vertical(|ui| {
                ui.set_width(inner);
                ui.spacing_mut().item_spacing = vec2(0.0, <spacing::S as SpacingToken>::VALUE);
                let text = <color::Text as ColorToken>::VALUE;
                SheetGrid::label(ui, title, TextStyle::new(TypeSize::Label, FontFace::Semibold), text);
                let dim = <color::TextDim as ColorToken>::VALUE;
                SheetGrid::label(ui, subtitle, TextStyle::new(TypeSize::Note, FontFace::Regular), dim);
            });
        });
    }

    fn camera_card(ui: &mut Ui) {
        Surface::frame(SurfaceLevel::Card, Elevation::Floating).show(ui, |ui| {
            ui.vertical(|ui| {
                ui.set_width(Self::CARD_WIDTH - Self::chrome_width());
                ui.spacing_mut().item_spacing = vec2(0.0, <spacing::S as SpacingToken>::VALUE);
                SheetGrid::label(ui, "CAMERA", SheetGrid::head_style(), <color::TextFaint as ColorToken>::VALUE);
                let style = TextStyle::new(TypeSize::Label, FontFace::Semibold);
                SheetGrid::label(ui, "X 186.0 Y 82.0 Z -124.0", style, <color::Text as ColorToken>::VALUE);
            });
        });
    }

    fn scene_pill(ui: &mut Ui) {
        Surface::frame(SurfaceLevel::Pill, Elevation::Flat).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing = vec2(<spacing::S as SpacingToken>::VALUE, 0.0);
                let line = TypeSize::Caption.line_height();
                let (slot, _) = ui.allocate_exact_size(vec2(Self::DOT, line), Sense::hover());
                let dot = egui::Rect::from_center_size(slot.center(), vec2(Self::DOT, Self::DOT));
                // stand-in for the studio-owned `--color-scene-1`
                Draw::fill(ui.painter(), dot, Corner::Full, <color::Success as ColorToken>::VALUE);
                SheetGrid::label(ui, "ARRIVAL · 1 OF 5", SheetGrid::head_style(), <color::Text as ColorToken>::VALUE);
            });
        });
    }

    // a surface frame's padding and border on both sides
    fn chrome_width() -> f32 {
        (<spacing::L as SpacingToken>::VALUE + <spacing::Border as SpacingToken>::VALUE) * 2.0
    }
}
