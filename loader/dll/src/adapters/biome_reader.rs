use amml_runtime::BiomeNameBook;

use crate::{LoaderState, NameReader};

/// Reads a game `Biome`: its id through the `Biome.mId` entry, its name (the text of the `HashedString` that
/// `Biome.mHash` names) through the `HashedString.layout` mirror. The name of an id is kept per world epoch
/// (`BiomeNameBook`), so a name is a copy that any thread can ask for.
pub struct BiomeReader {
    names: NameReader,
    id_at: usize,
    hash_at: usize,
    book: BiomeNameBook,
}

impl BiomeReader {
    /// A reader for the derived `Biome.mId`, `Biome.mHash` and `HashedString.layout`; `None` when one is missing or the
    /// layout has no mirror.
    pub fn new(state: &LoaderState) -> Option<Self> {
        let value = |name: &str| state.resolve(name).ok();
        Some(Self {
            names: NameReader::new(value("HashedString.layout")?)?,
            id_at: usize::try_from(value("Biome.mId")?).ok()?,
            hash_at: usize::try_from(value("Biome.mHash")?).ok()?,
            book: BiomeNameBook::new(),
        })
    }

    /// The id of the live `biome`; the name of the id is read the first time the id is met in `epoch`. `None` for a
    /// null biome.
    ///
    /// # Safety
    /// `biome` is a live game `Biome`, called on the game thread.
    pub unsafe fn id(&self, biome: usize, epoch: u64) -> Option<u16> {
        if biome == 0 {
            return None;
        }
        // SAFETY: `biome` is live and `id_at` is the derived offset of its two-byte `mId`.
        let id = unsafe { std::ptr::with_exposed_provenance::<u16>(biome.checked_add(self.id_at)?).read_unaligned() };
        // SAFETY: as above, for the `HashedString` at `hash_at`.
        self.book.meet(id, epoch, || unsafe { self.names.read(biome + self.hash_at) }.map(String::into_bytes));
        Some(id)
    }

    /// The name of the biome `id` in the world `epoch`; `None` for an id not met in it. Any thread.
    pub fn name(&self, id: u16, epoch: u64) -> Option<String> {
        self.book.name(id, epoch)
    }
}
