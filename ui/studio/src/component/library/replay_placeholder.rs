use egui::{ColorImage, Context, Id, TextureHandle, TextureId, TextureOptions, vec2};

use super::HeroPicture;
use crate::id::StudioUiId;

/// Paper: Entry Point, `Replay placeholder` (`ICW-0`), 1280 by 720. The picture a replay shows in the Library
/// when it has no picture of its own: no text, the DejaVu mark on the dark backdrop. The PNG is decoded once per
/// egui context and the texture is kept in the context's temporary data.
pub struct ReplayPlaceholder;

impl ReplayPlaceholder {
    /// The exported PNG, embedded in the binary.
    pub const PNG: &'static [u8] = include_bytes!("../../../assets/replay_placeholder.png");

    /// The texture of the placeholder; the same id on every call for one context.
    pub fn texture(ctx: &Context) -> TextureId {
        Self::handle(ctx).id()
    }

    /// The placeholder as a picture, so a card paints it with the same crop as a real picture.
    pub fn picture(ctx: &Context) -> HeroPicture {
        let handle = Self::handle(ctx);
        HeroPicture { texture: handle.id(), size: vec2(handle.size()[0] as f32, handle.size()[1] as f32) }
    }

    fn handle(ctx: &Context) -> TextureHandle {
        let key = Id::unique(StudioUiId::ReplayPlaceholder);
        if let Some(handle) = ctx.data(|data| data.get_temp::<TextureHandle>(key)) {
            return handle;
        }
        let handle = ctx.load_texture("replay_placeholder", Self::decode(), TextureOptions::LINEAR);
        ctx.data_mut(|data| data.insert_temp(key, handle.clone()));
        handle
    }

    // the PNG is embedded and checked by a test, so a decode failure is a build defect
    fn decode() -> ColorImage {
        let mut reader = png::Decoder::new(std::io::Cursor::new(Self::PNG))
            .read_info()
            .unwrap_or_else(|error| panic!("the embedded replay placeholder is not a PNG: {error}"));
        let mut pixels = vec![0; reader.output_buffer_size().unwrap_or_default()];
        let info = reader
            .next_frame(&mut pixels)
            .unwrap_or_else(|error| panic!("the embedded replay placeholder does not decode: {error}"));
        let size = [info.width as usize, info.height as usize];
        match info.color_type {
            png::ColorType::Rgba => ColorImage::from_rgba_unmultiplied(size, &pixels[..info.buffer_size()]),
            png::ColorType::Rgb => ColorImage::from_rgb(size, &pixels[..info.buffer_size()]),
            other => panic!("the embedded replay placeholder has the colour type {other:?}"),
        }
    }
}
