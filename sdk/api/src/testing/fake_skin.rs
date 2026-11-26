use crate::AnimatedKind;

/// Width, height and pixel bytes of an image of a `FakeSkin`.
pub type FakeImage = (u32, u32, Vec<u8>);

/// The skin a `FakeGame` gives in a player list entry or a skin change. Every field is what the read of the same name
/// gives; an image whose pixel bytes are not `width * height * 4` gives no view, as in the loader.
#[derive(Clone, Debug, Default)]
pub struct FakeSkin {
    /// Width, height and pixel bytes of the main image.
    pub image: Option<FakeImage>,
    /// The kind and the width, height and pixel bytes of each animated image.
    pub animated: Vec<(AnimatedKind, FakeImage)>,
    pub persona: bool,
    pub premium: bool,
    pub geometry_name: Option<String>,
    pub geometry_json: Option<String>,
    pub resource_patch: Option<String>,
}
