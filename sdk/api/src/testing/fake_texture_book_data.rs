use std::collections::HashMap;

use amml_abi::AmmlStatus;

use crate::TextureId;

#[derive(Default)]
pub(crate) struct FakeTextureBookData {
    pub(crate) generation: u32,
    pub(crate) next: u64,
    pub(crate) ready: HashMap<u64, bool>,
    pub(crate) loads: Vec<String>,
    pub(crate) uploads: Vec<(TextureId, u32, u32)>,
    pub(crate) unloads: Vec<TextureId>,
    // Unloads made on a thread with no fake: checked at once, applied by `pump`.
    pub(crate) queued: Vec<TextureId>,
}

impl FakeTextureBookData {
    pub(crate) fn add(&mut self) -> TextureId {
        self.next += 1;
        self.ready.insert(self.next, false);
        TextureId { id: self.next, generation: self.generation }
    }

    pub(crate) fn is_current(&self, texture: TextureId) -> bool {
        texture.generation == self.generation && self.ready.contains_key(&texture.id)
    }

    // Takes the texture out of the book at once, as the loader's book does; gives `STALE` for one that is not current.
    pub(crate) fn take(&mut self, texture: TextureId) -> Result<(), AmmlStatus> {
        if !self.is_current(texture) {
            return Err(AmmlStatus::STALE);
        }
        self.ready.remove(&texture.id);
        Ok(())
    }
}
