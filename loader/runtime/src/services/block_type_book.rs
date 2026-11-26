use std::collections::HashMap;
use std::sync::Mutex;

/// The block type ids the loader gives out: one id per `BlockType` address, counted from 1, never given twice in
/// the process. The name of a type is read once, when its id is made, and kept, so `name` works on any thread and
/// never reads game memory.
///
/// An address is checked again the first time it is seen in a new world epoch: the game may have freed a block type of
/// the world before and put another at the same address. When the name read differs, the address gets a new id; the old
/// id keeps its name and is never given again.
pub struct BlockTypeBook {
    inner: Mutex<Inner>,
}

struct Inner {
    next: u32,
    // The id of an address and the world epoch it was last checked in.
    ids: HashMap<usize, (u32, u64)>,
    names: HashMap<u32, String>,
}

impl BlockTypeBook {
    /// The id that means no block type.
    pub const NONE: u32 = 0;

    pub fn new() -> Self {
        Self { inner: Mutex::new(Inner { next: 1, ids: HashMap::new(), names: HashMap::new() }) }
    }

    /// The id of the block type at `address`. The first time an address is seen, `read_name` gives the bytes of its
    /// full name once; a name that is not UTF-8 or that `read_name` does not give stays unknown (`name` gives
    /// `None`). A zero `address` is no block type: `NONE`. In a later `epoch` than the one the address was checked in,
    /// `read_name` runs again: the same name keeps the id, another name gives a new id.
    pub fn id_of(&self, address: usize, epoch: u64, read_name: impl FnOnce() -> Option<Vec<u8>>) -> u32 {
        if address == 0 {
            return Self::NONE;
        }
        let mut inner = self.lock();
        let known = inner.ids.get(&address).copied();
        if let Some((id, checked)) = known {
            if checked == epoch {
                return id;
            }
            let name = read_name().and_then(|bytes| String::from_utf8(bytes).ok());
            if name == inner.names.get(&id).cloned() {
                inner.ids.insert(address, (id, epoch));
                return id;
            }
            return Self::make(&mut inner, address, epoch, name);
        }
        let name = read_name().and_then(|bytes| String::from_utf8(bytes).ok());
        Self::make(&mut inner, address, epoch, name)
    }

    fn make(inner: &mut Inner, address: usize, epoch: u64, name: Option<String>) -> u32 {
        let id = inner.next;
        inner.next = inner.next.saturating_add(1);
        inner.ids.insert(address, (id, epoch));
        if let Some(name) = name {
            inner.names.insert(id, name);
        }
        id
    }

    /// The name read for `id`; `None` for `NONE`, an id never given and a name that was not readable.
    pub fn name(&self, id: u32) -> Option<String> {
        self.lock().names.get(&id).cloned()
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Inner> {
        self.inner.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

impl Default for BlockTypeBook {
    fn default() -> Self {
        Self::new()
    }
}
