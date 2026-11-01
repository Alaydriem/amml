/// What a drag on the lanes is, decided on its first frame and kept until it ends: the host moves the key under the
/// press at once, so the key soon leaves the press point and a fresh look would see empty lane.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum LaneDrag {
    Move {
        lane: usize,
        key: usize,
    },
    Box,
    /// A press on a locked lane's key: it does nothing.
    Locked,
}
