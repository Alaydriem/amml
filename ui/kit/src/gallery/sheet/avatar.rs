use amml_i18n::text::UserText;
use egui::{Align2, Color32, ColorImage, Rect, Sense, TextureHandle, TextureOptions, Ui, pos2, vec2};

use crate::draw::{Draw, TextFit, TextStyle, TypeSize};
use crate::font::FontFace;
use crate::gallery::{GalleryState, SheetGrid};
use crate::tokens::{ColorToken, SpacingToken, color, spacing};
use crate::widget::{Avatar, AvatarImage, AvatarSize};

/// Paper: Mod manager, `Widget / Avatar` (`DGE-0`). The three sizes and the off look, each over
/// its caption, on one bottom line.
pub struct AvatarSheet;

impl AvatarSheet {
    /// Sheet chrome: the gap between the samples.
    const GAP: f32 = 40.0;
    /// Sheet chrome: the side in pixels and the cell size of the sample picture.
    const PICTURE_SIDE: usize = 16;
    const PICTURE_CELL: usize = 4;
    const SAMPLES: [(&'static str, AvatarSize, bool, &'static str); 4] = [
        ("Freecam", AvatarSize::Large, false, "Large · page header"),
        ("Minimap", AvatarSize::Regular, false, "Regular · list row"),
        ("Keystrokes", AvatarSize::Small, false, "Small · players tab"),
        ("World Edit", AvatarSize::Regular, true, "Off"),
    ];

    pub fn show(ui: &mut Ui, _state: &mut GalleryState) {
        let picture = Self::picture(ui);
        SheetGrid::page(ui, "Avatar", |ui| {
            SheetGrid::panel(ui, |ui| {
                ui.horizontal_top(|ui| {
                    ui.spacing_mut().item_spacing = vec2(Self::GAP, 0.0);
                    for (name, size, off, caption) in Self::SAMPLES {
                        Self::sample(ui, name, size, off, caption, None);
                    }
                    Self::sample(ui, "Freecam", AvatarSize::Regular, false, "Picture", Some(picture.clone()));
                    Self::sample(ui, "Freecam", AvatarSize::Regular, true, "Picture off", Some(picture));
                });
            });
        });
    }

    // a two-colour checker, made and loaded once
    fn picture(ui: &Ui) -> AvatarImage {
        let side = Self::PICTURE_SIDE;
        let key = egui::Id::unique("amml-ui-gallery-avatar");
        let cached: Option<TextureHandle> = ui.ctx().data(|data| data.get_temp(key));
        let handle = match cached {
            Some(handle) => handle,
            None => {
                let dark = <color::Accent as ColorToken>::VALUE;
                let mut pixels = Vec::with_capacity(side * side);
                for y in 0..side {
                    for x in 0..side {
                        let odd = (x / Self::PICTURE_CELL + y / Self::PICTURE_CELL) % 2 == 1;
                        pixels.push(if odd { dark } else { Color32::WHITE });
                    }
                }
                let image = ColorImage { size: [side, side], source_size: vec2(side as f32, side as f32), pixels };
                let handle = ui.ctx().load_texture("amml-ui-gallery-avatar", image, TextureOptions::NEAREST);
                ui.ctx().data_mut(|data| data.insert_temp(key, handle.clone()));
                handle
            }
        };
        AvatarImage::new(handle.id(), vec2(side as f32, side as f32))
    }

    // the avatar centred over its caption, with the tallest avatar's space above the caption
    fn sample(ui: &mut Ui, name: &str, size: AvatarSize, off: bool, caption: &str, image: Option<AvatarImage>) {
        let style = TextStyle::new(TypeSize::Caption, FontFace::Regular);
        let caption_size = Draw::measure(ui.ctx(), caption, style);
        let tallest = AvatarSize::Large.side();
        let gap = <spacing::S as SpacingToken>::VALUE;
        let side = size.side();
        let width = caption_size.x.max(side);
        let (rect, _) = ui.allocate_exact_size(vec2(width, tallest + gap + caption_size.y), Sense::hover());
        let avatar =
            Rect::from_min_size(pos2(rect.center().x - side / 2.0, rect.top() + tallest - side), vec2(side, side));
        Avatar::new(&UserText::from_model(name.to_owned()), size).image(image).off(off).paint(ui.painter(), avatar);
        let faint = <color::TextFaint as ColorToken>::VALUE;
        let galley = Draw::galley(ui.painter(), caption, style, faint, TextFit::Line);
        let at = Align2::CENTER_BOTTOM.anchor_size(rect.center_bottom(), galley.size());
        ui.painter().galley(at.min, galley, faint);
    }
}
