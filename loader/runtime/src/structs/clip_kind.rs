/// How a mod wants the cursor clipped.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClipKind {
    /// A 2 x 2 box at the centre of the client area.
    Centre,
}
