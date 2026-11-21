use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard};

use amml_data::{AdapterCatalog, NeedKind};

/// The areas this loader offers, by id and the highest version offered. A mod `uses` row for an area
/// is satisfied when the area is in the book.
pub struct AreaBook {
    offered: Mutex<HashMap<String, u32>>,
}

impl Default for AreaBook {
    fn default() -> Self {
        Self::new()
    }
}

impl AreaBook {
    const LOADER_SHAPE: &'static str = "loader";

    /// A book that offers only the areas the loader provides itself (shape `loader` in the catalog).
    /// Game areas are offered by their adapters.
    pub fn new() -> Self {
        let book = Self { offered: Mutex::new(HashMap::new()) };
        for need in AdapterCatalog::needs() {
            if need.kind == NeedKind::Area && need.shapes.iter().any(|shape| shape.name == Self::LOADER_SHAPE) {
                book.offer(need.id, need.version);
            }
        }
        book
    }

    /// Marks `id` available up to `version`.
    pub fn offer(&self, id: &str, version: u32) {
        let mut offered = self.lock();
        let slot = offered.entry(id.to_owned()).or_insert(0);
        *slot = (*slot).max(version);
    }

    /// Takes `id` back after it was offered, when a later check finds the loader cannot serve it
    /// (an input feed on the wrong thread; the caller logs why). From then on the area reads as not
    /// available.
    pub fn withdraw(&self, id: &str) {
        if self.lock().remove(id).is_some() {
            curia::warn!("area withdrawn", { area: id });
        }
    }

    pub fn is_available(&self, id: &str, version: u32) -> bool {
        self.lock().get(id).is_some_and(|offered| *offered >= version)
    }

    // The map holds plain data, so a lock poisoned by a panic is recovered.
    fn lock(&self) -> MutexGuard<'_, HashMap<String, u32>> {
        self.offered.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}
