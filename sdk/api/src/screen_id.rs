/// The id of a screen: the loader id for one screen controller of the game. It is the same in `ScreenRender`,
/// `ScreenClosing` and `ScreenChanged`, it is never zero for a screen and never given twice: a screen the game
/// frees keeps its id retired, and a new screen at the same address gets another id.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ScreenId(pub u64);

impl ScreenId {
    /// No screen: `ScreenChanged::new` when no view was drawn.
    pub const NONE: Self = Self(0);
}
