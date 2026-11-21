use std::collections::HashMap;
use std::sync::Mutex;

/// The names of the biome ids the loader met. The name of an id is read once, the first time a column read meets the
/// id in a world epoch, and kept for that epoch: a biome id can belong to one world (data-driven biomes), so a new
/// world epoch reads the name again the first time it meets the id. `name` answers from the book on any thread and
/// reads no game memory.
pub struct BiomeNameBook {
    // The name of each id (`None` when it was not readable) and the world epoch it was read in.
    inner: Mutex<HashMap<u16, (u64, Option<String>)>>,
}

impl BiomeNameBook {
    pub fn new() -> Self {
        Self { inner: Mutex::new(HashMap::new()) }
    }

    /// Notes that `id` was met in `epoch`. The first time in an epoch, `read_name` gives the bytes of the name; a name
    /// that is not UTF-8, that is empty or that `read_name` does not give stays unknown.
    pub fn meet(&self, id: u16, epoch: u64, read_name: impl FnOnce() -> Option<Vec<u8>>) {
        let mut inner = self.lock();
        if inner.get(&id).is_some_and(|(read_in, _)| *read_in == epoch) {
            return;
        }
        let name = read_name().and_then(|bytes| String::from_utf8(bytes).ok()).filter(|name| !name.is_empty());
        inner.insert(id, (epoch, name));
    }

    /// The name of `id` as read in `epoch`; `None` for an id not met in that epoch yet and for a name that was not
    /// readable.
    pub fn name(&self, id: u16, epoch: u64) -> Option<String> {
        self.lock().get(&id).filter(|(read_in, _)| *read_in == epoch).and_then(|(_, name)| name.clone())
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, HashMap<u16, (u64, Option<String>)>> {
        self.inner.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

impl Default for BiomeNameBook {
    fn default() -> Self {
        Self::new()
    }
}
