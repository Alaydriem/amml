use std::hash::{Hash, Hasher};

use crate::id::IdScope;

/// The egui ids of the gallery's live widgets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GalleryId {
    InspectorTabs,
    LibraryTabs,
    Zoom,
    ScanRadius,
    Shape,
    GridColour,
    GridPlacer,
}

impl Hash for GalleryId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        IdScope::hash_type::<Self, H>(state);
        std::mem::discriminant(self).hash(state);
    }
}
