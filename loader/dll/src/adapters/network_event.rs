use std::collections::HashMap;

use amml_msvc::{MsvcHeap, MsvcString};

use crate::NetworkEventKind;

/// The state of one running network event: its kind, the skins its entries hold (lent as handles) and the geometry texts
/// the loader built for them. It lives in a `NetworkScope` and ends with it.
///
/// A skin handle is `serial << 16 | (index + 1)`: the serial of the event and the position of the skin in it. A handle
/// of another event, or of an entry with no skin, is not valid.
pub struct NetworkEvent {
    serial: u64,
    heap: MsvcHeap,
    kind: NetworkEventKind,
    // The address of the `SerializedSkinRef` of each entry; 0 for an entry that has none.
    skins: Vec<usize>,
    // The geometry text of each skin read so far; `None` for a skin whose text was refused. Kept until the event ends.
    geometry: HashMap<usize, Option<Box<MsvcString>>>,
}

impl NetworkEvent {
    const INDEX_BITS: u32 = 16;

    pub fn new(serial: u64, heap: MsvcHeap, kind: NetworkEventKind, skins: Vec<usize>) -> Self {
        Self { serial, heap, kind, skins, geometry: HashMap::new() }
    }

    pub fn heap(&self) -> MsvcHeap {
        self.heap
    }

    pub fn kind(&self) -> &NetworkEventKind {
        &self.kind
    }

    /// The handle of the skin of entry `index`; 0 when the entry has none.
    pub fn handle(&self, index: usize) -> u64 {
        match self.skins.get(index) {
            Some(skin_ref) if *skin_ref != 0 => (self.serial << Self::INDEX_BITS) | (index as u64 + 1),
            _ => 0,
        }
    }

    /// The position and the `SerializedSkinRef` address of the skin `handle` names; `None` for a handle this event did
    /// not give.
    pub fn skin_of(&self, handle: u64) -> Option<(usize, usize)> {
        if handle >> Self::INDEX_BITS != self.serial {
            return None;
        }
        let index = usize::try_from(handle & ((1 << Self::INDEX_BITS) - 1)).ok()?.checked_sub(1)?;
        self.skins.get(index).copied().filter(|skin_ref| *skin_ref != 0).map(|skin_ref| (index, skin_ref))
    }

    /// The geometry text kept for the skin at `index`: `Some(None)` for a skin whose text was refused, `None` before the
    /// first read.
    pub fn geometry(&self, index: usize) -> Option<Option<&MsvcString>> {
        self.geometry.get(&index).map(|text| text.as_deref())
    }

    /// Keeps the result of the first geometry read of the skin at `index`.
    pub fn keep_geometry(&mut self, index: usize, text: Option<Box<MsvcString>>) {
        self.geometry.insert(index, text);
    }

    /// Takes every kept geometry text, to free.
    pub fn take_geometry(&mut self) -> Vec<Box<MsvcString>> {
        self.geometry.drain().filter_map(|(_, text)| text).collect()
    }
}
