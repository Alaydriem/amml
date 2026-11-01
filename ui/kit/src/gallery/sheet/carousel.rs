use egui::{Ui, vec2};

use crate::gallery::{GalleryState, Sample, SheetGrid};
use crate::icon::Icon;
use crate::tokens::{ColorToken, color};
use crate::widget::{Carousel, CarouselItem, CarouselMark};

/// Paper: Mod manager, `Widget / Carousel`. The axis colours stand in for the tone a mod passes for a letter mark.
pub struct CarouselSheet;

impl CarouselSheet {
    /// Sheet chrome: the carousel width.
    const WIDTH: f32 = 360.0;

    pub fn show(ui: &mut Ui, _state: &mut GalleryState) {
        let letters = Self::letter_items();
        let icons = Self::icon_items();
        SheetGrid::page(ui, "Carousel", |ui| {
            SheetGrid::panel(ui, |ui| {
                let variants = [("Letter mark, middle item", &letters, 4), ("Icon mark, first item, wraps", &icons, 0)];
                for (label, items, current) in variants {
                    let carousel = Carousel::new(items, current);
                    let size = vec2(Self::WIDTH, carousel.height());
                    SheetGrid::labelled(ui, label, SheetGrid::LABEL_WIDTH, SheetGrid::COLUMN_GAP, size, |ui, rect| {
                        carousel.paint(ui.painter(), rect.min, Self::WIDTH);
                    });
                }
            });
        });
    }

    fn letter_items() -> Vec<CarouselItem> {
        let tones = [
            ("X", "Move X", <color::AxisX as ColorToken>::VALUE),
            ("Y", "Move Y", <color::AxisY as ColorToken>::VALUE),
            ("Z", "Move Z", <color::AxisZ as ColorToken>::VALUE),
        ];
        (0..8)
            .map(|index| {
                let (letter, name, tone) = tones[index % tones.len()];
                CarouselItem {
                    mark: CarouselMark::Letter(Sample::text(letter), tone),
                    name: Sample::text(name),
                    value: Sample::text("1 block"),
                }
            })
            .collect()
    }

    fn icon_items() -> Vec<CarouselItem> {
        let icons = [Icon::Grid, Icon::Fit, Icon::Rename, Icon::Refresh];
        (0..8)
            .map(|index| CarouselItem {
                mark: CarouselMark::Icon(icons[index % icons.len()]),
                name: Sample::text("Layer"),
                value: Sample::text("Layer 1 of 8"),
            })
            .collect()
    }
}
