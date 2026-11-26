use crate::LoaderState;

/// The functions, slots and offsets the texture pump uses.
///
/// `group_base` is the offset of the `TextureGroupBase` in a `TextureGroup`, which `getTexture` runs on. The
/// image buffer words are offsets in `cg::ImageBuffer`: the blob deleter, the data pointer and the byte size.
#[derive(Clone, Copy, Debug)]
pub struct TextureCalls {
    pub resource_location_ctor: usize,
    pub image_buffer_ctor: usize,
    pub image_buffer_words: usize,
    pub buffer_deleter: usize,
    pub buffer_data: usize,
    pub buffer_size: usize,
    pub upload: usize,
    pub group_get: usize,
    pub unload: usize,
    pub group_base: usize,
    pub get_texture_group_slot: usize,
    pub context_get_texture_slot: usize,
}

impl TextureCalls {
    /// The most words of an `ImageBuffer` the pump builds on its stack.
    pub const BUFFER_CAPACITY_WORDS: usize = 16;
    /// The size of a `ResourceLocation`, which the mirror of the pump has.
    pub const LOCATION_SIZE: usize = 0x38;
    /// The offset of the size word of a `BedrockTextureData`: width and height as two 32-bit values. The game
    /// counts a texture as ready when it is not zero. Taken from the old hotbar-manager `texture_ready`; not
    /// proven from the executable.
    pub const READY_WORD: usize = 0x10;
    const WORD: usize = size_of::<usize>();

    /// The calls from the resolved entries; `None` when one is missing, when `ResourceLocation` is not 38h
    /// bytes, or when the `ImageBuffer` does not fit the stack buffer.
    pub fn new(state: &LoaderState) -> Option<Self> {
        let value = |name: &str| state.resolve(name).ok().and_then(|value| usize::try_from(value).ok());
        let function = |name: &str| value(name).filter(|address| *address != 0);
        if value("ResourceLocation.size")? != Self::LOCATION_SIZE {
            return None;
        }
        let buffer_bytes = value("ImageBuffer.size")?;
        let words = buffer_bytes.checked_div(Self::WORD).filter(|_| buffer_bytes.is_multiple_of(Self::WORD))?;
        if words == 0 || words > Self::BUFFER_CAPACITY_WORDS {
            return None;
        }
        let member =
            |name: &str| value(name).filter(|offset| offset.is_multiple_of(Self::WORD) && offset / Self::WORD < words);
        Some(Self {
            resource_location_ctor: function("ResourceLocation.ctor")?,
            image_buffer_ctor: function("ImageBuffer.ctorFromDescription")?,
            image_buffer_words: words,
            buffer_deleter: member("ImageBuffer.mDeleter")?,
            buffer_data: member("ImageBuffer.mData")?,
            buffer_size: member("ImageBuffer.mSize")?,
            upload: function("TextureGroup.uploadTexture")?,
            group_get: function("TextureGroup.getTexture")?,
            unload: function("TextureGroup.unloadTexture")?,
            group_base: value("TextureGroup.textureGroupBase")?,
            get_texture_group_slot: value("ClientInstance.getTextureGroup")?,
            context_get_texture_slot: value("MinecraftUIRenderContext.getTexture")?,
        })
    }
}
