/// Where `Textures::load` finds a texture. The path has no extension, or `.png`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextureSource<'a> {
    /// A file in the folder of the mod, relative to it. A path that leaves the folder is refused.
    ModFile(&'a str),
    /// A texture of the game or of its resource packs, for example `textures/ui/button_borderless_light`.
    Vanilla(&'a str),
}
