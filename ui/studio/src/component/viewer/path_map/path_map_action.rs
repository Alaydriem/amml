/// What a click on the path map asks for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathMapAction {
    /// Select the camera key at this index of `PathMapView::path`.
    SelectKey(usize),
}
