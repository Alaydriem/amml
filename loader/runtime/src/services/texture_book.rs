use std::collections::BTreeMap;
use std::sync::{Mutex, MutexGuard};

use crate::structs::texture_book_state::TextureBookState;
use crate::structs::texture_entry::TextureEntry;
use crate::structs::texture_kind::TextureKind;
use crate::{TextureJob, TextureKey, TextureRefusal, TextureRelease, TextureSourceKind, TextureTarget, TextureWork};

/// The textures of the loader: an id with a generation and an owner mod for each, at most one game `TexturePtr`
/// held for each, and the game calls that are due.
///
/// The book makes no game call. Every call that changes it only queues: `take_work` gives the releases and jobs of
/// one UI draw point, the texture pump runs them and reports the answers with `bound`, `uploaded` and
/// `mark_ready`. So a call is allowed from any thread.
///
/// Ids come from one counter that never goes back, so an id is never given twice. `world_left` frees every texture
/// and starts a new generation: an id of an older generation gives `Stale`. A held `TexturePtr` is given back
/// when its texture is unloaded, bound again (`rebind_all`), or its mod is disabled (`disable_owner`).
/// A texture that is held but not ready is asked for again at most once a second.
pub struct TextureBook {
    state: Mutex<TextureBookState>,
}

impl Default for TextureBook {
    fn default() -> Self {
        Self::new()
    }
}

impl TextureBook {
    /// The most uploads one `take_work` gives.
    pub const MAX_UPLOADS: usize = 4;
    /// The time, in milliseconds, between two asks of the game for a texture that is not ready.
    pub const REASK_MS: u64 = 1000;
    /// The name of an uploaded texture is this prefix and a number.
    pub const NAME_PREFIX: &'static str = "amml/tex_";

    pub fn new() -> Self {
        Self {
            state: Mutex::new(TextureBookState {
                next_id: 1,
                generation: 0,
                next_name: 0,
                free_names: Vec::new(),
                entries: BTreeMap::new(),
                releases: Vec::new(),
            }),
        }
    }

    // The state holds plain data, so a lock poisoned by a panic is recovered.
    fn lock(&self) -> MutexGuard<'_, TextureBookState> {
        self.state.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Asks for the texture `path` of `source` for the mod `owner`. A mod file path is relative to `mod_dir` and
    /// may end in `.png`; a path that is absolute or leaves the folder gives `OutsideModFolder`.
    pub fn load(
        &self,
        owner: usize,
        source: TextureSourceKind,
        path: &str,
        mod_dir: &str,
    ) -> Result<TextureKey, TextureRefusal> {
        let target = match source {
            TextureSourceKind::Vanilla => TextureTarget::Vanilla(Self::strip_extension(path.trim()).to_string()),
            TextureSourceKind::ModFile => TextureTarget::Raw(Self::mod_file_path(mod_dir, path)?),
        };
        if matches!(&target, TextureTarget::Vanilla(text) if text.is_empty()) {
            return Err(TextureRefusal::EmptyPath);
        }
        let mut state = self.lock();
        Ok(Self::insert(&mut state, owner, TextureKind::File { target }))
    }

    /// Uploads `width` x `height` RGBA pixels as a new texture of `owner`. The pixels are copied.
    pub fn upload(&self, owner: usize, width: u32, height: u32, pixels: &[u8]) -> Result<TextureKey, TextureRefusal> {
        let wanted = u64::from(width) * u64::from(height) * 4;
        if width == 0 || height == 0 || u64::try_from(pixels.len()) != Ok(wanted) {
            return Err(TextureRefusal::BadImage);
        }
        let mut state = self.lock();
        let name = match state.free_names.pop() {
            Some(name) => name,
            None => {
                let name = format!("{}{}", Self::NAME_PREFIX, state.next_name);
                state.next_name += 1;
                name
            }
        };
        let kind = TextureKind::Upload { name, width, height, pixels: pixels.to_vec(), uploaded: false };
        Ok(Self::insert(&mut state, owner, kind))
    }

    /// Uploads `width` x `height` RGBA pixels under `name` in the texture group, as a new texture of `owner`: the
    /// game then finds the pixels where it asks for the resource `name` (a name without extension, like
    /// `font/glyph_E1`). The pixels are copied. A name of an earlier upload is replaced only after that texture was
    /// unloaded, because the release of the old name runs before the upload.
    pub fn upload_named(
        &self,
        owner: usize,
        name: &str,
        width: u32,
        height: u32,
        pixels: &[u8],
    ) -> Result<TextureKey, TextureRefusal> {
        let name = Self::strip_extension(name.trim());
        if name.is_empty() {
            return Err(TextureRefusal::EmptyPath);
        }
        let wanted = u64::from(width) * u64::from(height) * 4;
        if width == 0 || height == 0 || u64::try_from(pixels.len()) != Ok(wanted) {
            return Err(TextureRefusal::BadImage);
        }
        let mut state = self.lock();
        let kind =
            TextureKind::Upload { name: name.to_string(), width, height, pixels: pixels.to_vec(), uploaded: false };
        Ok(Self::insert(&mut state, owner, kind))
    }

    /// Replaces the pixels of the upload `key` of `owner`; the texture is uploaded again under its own name at the next
    /// `take_work`, in place: nothing is unloaded, and the only release is the old reference that the new bind gives back.
    /// `BadImage` for pixels that do not fit the size and for a texture that is not an upload.
    pub fn replace_pixels(
        &self,
        owner: usize,
        key: TextureKey,
        width: u32,
        height: u32,
        pixels: &[u8],
    ) -> Result<(), TextureRefusal> {
        let mut state = self.lock();
        Self::check(&state, key)?;
        let entry = state.entries.get_mut(&key.id).ok_or(TextureRefusal::Stale)?;
        if entry.owner != owner {
            return Err(TextureRefusal::NotOwner);
        }
        let wanted = u64::from(width) * u64::from(height) * 4;
        let fits = width != 0 && height != 0 && u64::try_from(pixels.len()) == Ok(wanted);
        let TextureKind::Upload { width: old_width, height: old_height, pixels: old_pixels, uploaded, .. } =
            &mut entry.kind
        else {
            return Err(TextureRefusal::BadImage);
        };
        if !fits {
            return Err(TextureRefusal::BadImage);
        }
        *old_width = width;
        *old_height = height;
        *old_pixels = pixels.to_vec();
        *uploaded = false;
        entry.due = true;
        Ok(())
    }

    /// Frees `key` of `owner`: its held reference and, for an upload, its name in the texture group are given back
    /// at the next `take_work`.
    pub fn unload(&self, owner: usize, key: TextureKey) -> Result<(), TextureRefusal> {
        let mut state = self.lock();
        Self::check(&state, key)?;
        let entry = state.entries.get(&key.id).ok_or(TextureRefusal::Stale)?;
        if entry.owner != owner {
            return Err(TextureRefusal::NotOwner);
        }
        Self::remove(&mut state, key.id);
        Ok(())
    }

    /// Whether `key` is a texture of the current generation that was not freed.
    pub fn is_current(&self, key: TextureKey) -> bool {
        Self::check(&self.lock(), key).is_ok()
    }

    /// Whether the game gave the data of `key`. False for an unknown or stale key.
    pub fn ready(&self, key: TextureKey) -> bool {
        let state = self.lock();
        Self::check(&state, key).is_ok() && state.entries.get(&key.id).is_some_and(|entry| entry.ready)
    }

    /// The address of the `TexturePtr` of `key` to draw with: `Ok(None)` while the texture is not ready.
    pub fn drawable(&self, key: TextureKey) -> Result<Option<usize>, TextureRefusal> {
        let state = self.lock();
        Self::check(&state, key)?;
        let entry = state.entries.get(&key.id).ok_or(TextureRefusal::Stale)?;
        Ok((entry.ready && entry.held != 0).then_some(entry.held))
    }

    /// Runs `read` on the address of the `TexturePtr` of `key` while the book holds the lock, so no release can free it
    /// meanwhile: `Ok(None)` (and `read` does not run) while the texture is not ready. For a reader that copies the
    /// pointer and takes its own uses.
    pub fn with_drawable<R>(
        &self,
        key: TextureKey,
        read: impl FnOnce(usize) -> R,
    ) -> Result<Option<R>, TextureRefusal> {
        let state = self.lock();
        Self::check(&state, key)?;
        let entry = state.entries.get(&key.id).ok_or(TextureRefusal::Stale)?;
        Ok((entry.ready && entry.held != 0).then(|| read(entry.held)))
    }

    /// The releases and jobs for one UI draw point, at time `now_ms`. Gives at most `MAX_UPLOADS` uploads; a
    /// texture whose turn did not come stays due. A held texture that is not ready is asked for again when
    /// `REASK_MS` passed since the last ask.
    pub fn take_work(&self, now_ms: u64) -> TextureWork {
        self.take_work_with(now_ms, true)
    }

    /// Like `take_work`. With `releases` false the releases stay in the book for a later call and the work holds only
    /// jobs.
    pub fn take_work_with(&self, now_ms: u64, releases: bool) -> TextureWork {
        let mut state = self.lock();
        let generation = state.generation;
        let taken = if releases { std::mem::take(&mut state.releases) } else { Vec::new() };
        let mut work = TextureWork { releases: taken, jobs: Vec::new() };
        let mut uploads = 0;
        for (id, entry) in &mut state.entries {
            let reask = !entry.due
                && !entry.ready
                && entry.asked_at.is_some_and(|at| now_ms.saturating_sub(at) >= Self::REASK_MS);
            if !entry.due && !reask {
                continue;
            }
            let key = TextureKey { id: *id, generation };
            let job = match &mut entry.kind {
                TextureKind::File { target } => TextureJob::Bind { key, target: target.clone() },
                TextureKind::Upload { name, width, height, pixels, uploaded } => {
                    if *uploaded {
                        TextureJob::Bind { key, target: TextureTarget::Group(name.clone()) }
                    } else if uploads < Self::MAX_UPLOADS {
                        uploads += 1;
                        TextureJob::Upload {
                            key,
                            name: name.clone(),
                            width: *width,
                            height: *height,
                            pixels: pixels.clone(),
                        }
                    } else {
                        continue;
                    }
                }
            };
            entry.due = false;
            entry.asked_at = Some(now_ms);
            work.jobs.push(job);
        }
        work
    }

    /// The game answered a job: `held` is the address of the `TexturePtr` it gave, 0 when it gave none. A texture
    /// already held gives its old reference back. Returns false when `key` was freed meanwhile; the caller then
    /// releases `held` itself.
    pub fn bound(&self, key: TextureKey, held: usize) -> bool {
        let mut state = self.lock();
        if Self::check(&state, key).is_err() {
            return false;
        }
        let Some(entry) = state.entries.get_mut(&key.id) else { return false };
        let old = std::mem::replace(&mut entry.held, held);
        entry.ready = false;
        if old != 0 && old != held {
            state.releases.push(TextureRelease { held: old, unload_name: None });
        }
        true
    }

    /// The upload job of `key` reached the game.
    pub fn uploaded(&self, key: TextureKey) {
        let mut state = self.lock();
        if Self::check(&state, key).is_err() {
            return;
        }
        if let Some(TextureKind::Upload { uploaded, .. }) = state.entries.get_mut(&key.id).map(|entry| &mut entry.kind)
        {
            *uploaded = true;
        }
    }

    /// The textures that are held but not ready, with their `TexturePtr` addresses: the pump reads the ready word
    /// of each and reports it with `mark_ready`.
    pub fn waiting(&self) -> Vec<(TextureKey, usize)> {
        let state = self.lock();
        state
            .entries
            .iter()
            .filter(|(_, entry)| entry.held != 0 && !entry.ready)
            .map(|(id, entry)| (TextureKey { id: *id, generation: state.generation }, entry.held))
            .collect()
    }

    /// The game gave the data of `key`.
    pub fn mark_ready(&self, key: TextureKey) {
        let mut state = self.lock();
        if Self::check(&state, key).is_err() {
            return;
        }
        if let Some(entry) = state.entries.get_mut(&key.id) {
            entry.ready = true;
        }
    }

    /// The game dropped its textures (a world join, a dimension change, a resource pack change): every texture gives
    /// its held reference back and is bound again at the next `take_work`; an upload is uploaded again.
    pub fn rebind_all(&self) {
        let mut state = self.lock();
        let mut given = Vec::new();
        for entry in state.entries.values_mut() {
            if entry.held != 0 {
                given.push(entry.held);
            }
            entry.held = 0;
            entry.ready = false;
            entry.due = true;
            entry.asked_at = None;
            if let TextureKind::Upload { uploaded, .. } = &mut entry.kind {
                *uploaded = false;
            }
        }
        state.releases.extend(given.into_iter().map(|held| TextureRelease { held, unload_name: None }));
    }

    /// Frees every texture and starts a new generation. Gives the number of textures freed.
    pub fn world_left(&self) -> usize {
        let mut state = self.lock();
        let ids: Vec<u64> = state.entries.keys().copied().collect();
        for id in &ids {
            Self::remove(&mut state, *id);
        }
        state.generation = state.generation.wrapping_add(1);
        ids.len()
    }

    /// Frees every texture of `owner`. Gives the number of textures freed.
    pub fn disable_owner(&self, owner: usize) -> usize {
        let mut state = self.lock();
        let ids: Vec<u64> = state.entries.iter().filter(|(_, entry)| entry.owner == owner).map(|(id, _)| *id).collect();
        for id in &ids {
            Self::remove(&mut state, *id);
        }
        ids.len()
    }

    /// Whether the pump has work now: a release, or a call that is due (a new request, an upload that waits for its
    /// turn).
    pub fn has_due_work(&self) -> bool {
        let state = self.lock();
        !state.releases.is_empty() || state.entries.values().any(|entry| entry.due)
    }

    /// Whether a texture waits for the game: it was asked for and is not ready. The pump polls it at most once in
    /// `REASK_MS`.
    pub fn has_waiting(&self) -> bool {
        self.lock().entries.values().any(|entry| !entry.due && !entry.ready)
    }

    /// The number of textures in the book.
    pub fn len(&self) -> usize {
        self.lock().entries.len()
    }

    /// Whether the book holds no texture.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// The current generation.
    pub fn generation(&self) -> u32 {
        self.lock().generation
    }

    fn insert(state: &mut TextureBookState, owner: usize, kind: TextureKind) -> TextureKey {
        let id = state.next_id;
        state.next_id += 1;
        state.entries.insert(id, TextureEntry { owner, kind, held: 0, ready: false, due: true, asked_at: None });
        TextureKey { id, generation: state.generation }
    }

    fn check(state: &TextureBookState, key: TextureKey) -> Result<(), TextureRefusal> {
        if key.generation == state.generation && state.entries.contains_key(&key.id) {
            Ok(())
        } else {
            Err(TextureRefusal::Stale)
        }
    }

    // Takes `id` out of the book: its held reference and, for an uploaded texture, its name go to the releases.
    fn remove(state: &mut TextureBookState, id: u64) {
        let Some(entry) = state.entries.remove(&id) else { return };
        let mut unload_name = None;
        if let TextureKind::Upload { name, uploaded, .. } = entry.kind {
            if uploaded {
                unload_name = Some(name.clone());
            }
            if name.starts_with(Self::NAME_PREFIX) {
                state.free_names.push(name);
            }
        }
        if entry.held != 0 || unload_name.is_some() {
            state.releases.push(TextureRelease { held: entry.held, unload_name });
        }
    }

    // The path without a trailing `.png`.
    fn strip_extension(path: &str) -> &str {
        path.strip_suffix(".png").unwrap_or(path)
    }

    // The absolute path of the mod file `path` under `mod_dir`, with forward slashes and without `.png`.
    fn mod_file_path(mod_dir: &str, path: &str) -> Result<String, TextureRefusal> {
        let path = Self::strip_extension(path.trim()).replace('\\', "/");
        if path.is_empty() {
            return Err(TextureRefusal::EmptyPath);
        }
        let leaves = path.starts_with('/') || path.contains(':') || path.split('/').any(|part| part == "..");
        if leaves || mod_dir.trim().is_empty() {
            return Err(TextureRefusal::OutsideModFolder);
        }
        let base = mod_dir.replace('\\', "/");
        Ok(format!("{}/{}", base.trim_end_matches('/'), path))
    }
}
