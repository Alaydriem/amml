/// The wishes of the mods for the client's chunk (blob) cache: one wish per mod, `true` for the game's own behavior and
/// `false` for no cache. While any mod holds a `false` wish the cache is disabled for everyone; a mod that is disabled
/// loses its wish.
pub struct ChunkCacheSwitch {
    // (mod index, wish), in the order the mods first set one.
    wishes: Vec<(usize, bool)>,
}

impl ChunkCacheSwitch {
    pub fn new() -> Self {
        Self { wishes: Vec::new() }
    }

    /// The mod at `owner` wishes the cache `enabled`; replaces its earlier wish.
    pub fn set(&mut self, owner: usize, enabled: bool) {
        match self.wishes.iter_mut().find(|(mod_index, _)| *mod_index == owner) {
            Some((_, wish)) => *wish = enabled,
            None => self.wishes.push((owner, enabled)),
        }
    }

    /// The mod at `owner` ends its wish; whether it held one.
    pub fn clear(&mut self, owner: usize) -> bool {
        let before = self.wishes.len();
        self.wishes.retain(|(mod_index, _)| *mod_index != owner);
        self.wishes.len() != before
    }

    /// Whether any mod wishes the cache off.
    pub fn disabled(&self) -> bool {
        self.wishes.iter().any(|(_, wish)| !*wish)
    }
}

impl Default for ChunkCacheSwitch {
    fn default() -> Self {
        Self::new()
    }
}
