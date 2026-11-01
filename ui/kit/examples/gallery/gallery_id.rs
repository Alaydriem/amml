use std::hash::{Hash, Hasher};

use amml_ui::id::IdScope;

// the gallery window's own egui ids
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GalleryId {
    Pages,
}

impl Hash for GalleryId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        IdScope::hash_type::<Self, H>(state);
        std::mem::discriminant(self).hash(state);
    }
}
