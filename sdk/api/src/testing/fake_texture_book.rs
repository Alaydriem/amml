use std::sync::{Arc, Mutex, MutexGuard};

use amml_abi::AmmlStatus;

use crate::TextureId;
use crate::testing::FakeTextureBookData;

// The book of the newest fake. A thread that holds no fake reaches it here: the loader checks an `unload` at once on any
// thread and defers only the free, and so does the fake.
static ACTIVE: Mutex<Option<Arc<Mutex<FakeTextureBookData>>>> = Mutex::new(None);

/// The texture side of a `FakeGame`: the textures of the current generation with their ready flag, and what the mod
/// asked for.
pub(crate) struct FakeTextureBook {
    data: Arc<Mutex<FakeTextureBookData>>,
}

impl Default for FakeTextureBook {
    fn default() -> Self {
        let data = Arc::new(Mutex::new(FakeTextureBookData::default()));
        *ACTIVE.lock().unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(data.clone());
        Self { data }
    }
}

impl FakeTextureBook {
    fn lock(&self) -> MutexGuard<'_, FakeTextureBookData> {
        self.data.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// A file texture asked for; `line` is what `loads` reports.
    pub(crate) fn load(&self, line: String) -> TextureId {
        let mut data = self.lock();
        data.loads.push(line);
        data.add()
    }

    pub(crate) fn upload(&self, width: u32, height: u32) -> TextureId {
        let mut data = self.lock();
        let id = data.add();
        data.uploads.push((id, width, height));
        id
    }

    pub(crate) fn is_current(&self, texture: TextureId) -> bool {
        self.lock().is_current(texture)
    }

    pub(crate) fn is_ready(&self, texture: TextureId) -> bool {
        let data = self.lock();
        data.is_current(texture) && data.ready.get(&texture.id).copied().unwrap_or(false)
    }

    pub(crate) fn set_ready(&self, texture: TextureId, ready: bool) {
        let mut data = self.lock();
        if data.is_current(texture) {
            data.ready.insert(texture.id, ready);
        }
    }

    pub(crate) fn unload(&self, texture: TextureId) -> Result<(), AmmlStatus> {
        let mut data = self.lock();
        data.take(texture)?;
        data.unloads.push(texture);
        Ok(())
    }

    /// Every texture is freed and the generation goes up.
    pub(crate) fn leave_world(&self) {
        let mut data = self.lock();
        data.ready.clear();
        data.generation += 1;
    }

    pub(crate) fn loads(&self) -> Vec<String> {
        self.lock().loads.clone()
    }

    pub(crate) fn uploads(&self) -> Vec<(TextureId, u32, u32)> {
        self.lock().uploads.clone()
    }

    pub(crate) fn unloads(&self) -> Vec<TextureId> {
        self.lock().unloads.clone()
    }

    /// The unload of `texture` on a thread that holds no fake: checked at once (`STALE` for a texture that is not
    /// current, also one already queued), freed at `pump`.
    pub(crate) fn unload_off_thread(texture: TextureId) -> Result<(), AmmlStatus> {
        let active = ACTIVE.lock().unwrap_or_else(|poisoned| poisoned.into_inner()).clone();
        let Some(active) = active else { return Err(AmmlStatus::FAILED) };
        let mut data = active.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        data.take(texture)?;
        data.queued.push(texture);
        Ok(())
    }

    /// Applies the queued unloads; gives the number of textures freed.
    pub(crate) fn pump(&self) -> usize {
        let mut data = self.lock();
        let queued = std::mem::take(&mut data.queued);
        let count = queued.len();
        data.unloads.extend(queued);
        count
    }
}
