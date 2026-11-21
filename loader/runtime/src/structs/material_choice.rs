/// The game material a world mesh draw uses.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MaterialChoice {
    /// The level renderer's `wireframe` material, patched to alpha blend without depth write (Vibrant Visuals).
    Wireframe,
    /// The depth-tested name tag material of the loader's own `BaseActorRenderer` (the other graphics modes).
    NameTag,
}
