/// A game setting that mods change and the loader puts back when the last of them lets go.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RestoreFlag {
    MouseCaptured,
    HideHud,
}
