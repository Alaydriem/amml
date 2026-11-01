/// Paper: Assets, `Glyph / key / …`. The top row is a key at rest, the bottom row a selected key in front of its
/// halo. A key has one size in each look.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum KeyLook {
    #[default]
    Rest,
    Selected,
}
