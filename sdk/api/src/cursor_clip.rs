use amml_abi::AmmlCursorClip;

/// How `Window::clip_cursor` clips the system cursor.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CursorClip {
    /// A 2 x 2 box at the centre of the client area.
    Centre,
}

impl From<CursorClip> for AmmlCursorClip {
    fn from(clip: CursorClip) -> Self {
        match clip {
            CursorClip::Centre => AmmlCursorClip::CENTRE,
        }
    }
}
