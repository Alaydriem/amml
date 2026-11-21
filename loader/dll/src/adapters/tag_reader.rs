use amml_msvc::{MsvcMap, MsvcVec};

use crate::{ObjectRead, TagVariantLayout};

/// Reads the tags of a game `CompoundTag` by walking its `std::map` through the layout mirrors: no game call, no
/// allocation and no throw. The reads are not thread-checked, and a tag must not change while a read runs.
///
/// A reader exists only when the `CompoundTag.nodeVariantIndex` entry has the value the mirror fits. It calls no
/// virtual function of a tag: a compound element is known by its vtable, which is the `CompoundTag.vtable` entry, and
/// byte, int and list values are read by the variant index of the map node.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TagReader {
    compound_vtable: usize,
}

impl TagReader {
    // The offset of the map in a `CompoundTag` and of the element vector in a `ListTag`: both come after the vtable.
    const MEMBER: usize = 8;
    // The offset of the value of a `ByteTag` or an `IntTag`.
    const VALUE: usize = 8;
    // The most elements one list gives; a shulker box holds 27 and a bundle at most 64.
    const MAX_ELEMENTS: usize = 256;
    // The deepest nesting a stamp walk follows: the user data, its item list, an item, a bundle's list, an item in it.
    const STAMP_DEPTH: usize = 5;
    // The most map nodes one stamp walk visits; a box of 27 bundles of 64 items stays far below it.
    const STAMP_NODES: usize = 1 << 16;
    const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;
    // The index of a nested `CompoundTag` held in a map node.
    const COMPOUND: u8 = 10;

    /// A reader for the derived `CompoundTag.nodeVariantIndex` value and the address of the `CompoundTag` vtable;
    /// `None` for an index the mirror does not fit or a null vtable.
    pub fn new(node_variant_index: u64, compound_vtable: usize) -> Option<Self> {
        (node_variant_index == TagVariantLayout::NODE_INDEX_OFFSET && compound_vtable != 0)
            .then_some(Self { compound_vtable })
    }

    // The value of `key` in the compound at `tag` with the variant index `index`.
    //
    // Safety: `tag` is a live `CompoundTag`.
    unsafe fn value<'a>(tag: usize, key: &str, index: u8) -> Option<&'a TagVariantLayout> {
        let map = tag.checked_add(Self::MEMBER)?;
        // SAFETY: the caller guarantees a live CompoundTag, whose map starts at the member offset.
        let map = unsafe { &*std::ptr::with_exposed_provenance::<MsvcMap<TagVariantLayout>>(map) };
        // SAFETY: the map is a live std::map<std::string, CompoundTagVariant>, which no one changes during the read.
        let value = unsafe { map.find(key) }?;
        (value.index == index).then_some(value)
    }

    /// The element tags of the list under `key` of the compound at `tag`: the addresses of the `Tag` objects, at most
    /// `MAX_ELEMENTS`. `None` for a missing key and for a tag of another type.
    ///
    /// # Safety
    /// `tag` is a live `CompoundTag` of the game build the reader was made for.
    pub unsafe fn list(&self, tag: usize, key: &str) -> Option<Vec<usize>> {
        // SAFETY: the caller's guarantee is passed on.
        let value = unsafe { Self::value(tag, key, TagVariantLayout::LIST) }?;
        let items = value.tag().checked_add(Self::MEMBER)?;
        // SAFETY: the variant index says the Tag object is a ListTag, whose element vector follows the vtable.
        let items = unsafe { &*std::ptr::with_exposed_provenance::<MsvcVec<usize>>(items) };
        Some(items.as_slice().iter().copied().take(Self::MAX_ELEMENTS).collect())
    }

    /// `element`, an element of a list, when it is a compound tag: the same address. `None` for a null element and
    /// for an element of another type (its vtable is not the `CompoundTag` vtable), so a malformed list loses only the
    /// bad elements.
    ///
    /// # Safety
    /// `element` is zero or a live `Tag` of the game.
    pub unsafe fn compound(&self, element: usize) -> Option<usize> {
        // SAFETY: a non-null element of a live list is a live Tag, whose first qword is its vtable.
        let vtable = unsafe { ObjectRead::pointer(element, 0) }?;
        (vtable == self.compound_vtable).then_some(element)
    }

    /// The byte tag under `key` of the compound at `tag`. `None` for a missing key and for a tag of another type.
    ///
    /// # Safety
    /// `tag` is a live `CompoundTag` of the game build the reader was made for.
    pub unsafe fn byte(&self, tag: usize, key: &str) -> Option<i8> {
        // SAFETY: the caller's guarantee is passed on.
        let value = unsafe { Self::value(tag, key, TagVariantLayout::BYTE) }?;
        // SAFETY: the variant index says the Tag object is a ByteTag, whose value is the byte after the vtable.
        unsafe { ObjectRead::value::<i8>(value.tag(), Self::VALUE) }
    }

    /// A change stamp of the compound at `tag`: an FNV-1a hash of its address and, for it and every compound nested in
    /// it (a compound value, or a compound element of a list), the number of keys and each key, variant index and the
    /// bytes of the variant's storage after the vtable; each element address of a list is hashed too. Two walks of a tag
    /// that did not change give the same stamp. `None` for a walk that meets a map that is not a tree or passes
    /// `STAMP_NODES`. A nesting deeper than `STAMP_DEPTH` is not followed.
    ///
    /// # Safety
    /// `tag` is a live `CompoundTag` of the game build the reader was made for, which no one changes during the walk.
    pub unsafe fn stamp(&self, tag: usize) -> Option<u64> {
        let mut hash = Self::FNV_OFFSET;
        Self::mix(&mut hash, &(tag as u64).to_le_bytes());
        let mut budget = Self::STAMP_NODES;
        // SAFETY: the caller's guarantee is passed on.
        unsafe { self.walk(tag, 0, &mut hash, &mut budget) }.then_some(hash)
    }

    fn mix(hash: &mut u64, bytes: &[u8]) {
        for byte in bytes {
            *hash = (*hash ^ u64::from(*byte)).wrapping_mul(Self::FNV_PRIME);
        }
    }

    // Hashes the compound at `tag` into `hash`; false when a map is not a tree or the node budget runs out.
    //
    // Safety: `tag` is a live `CompoundTag` that no one changes during the walk.
    unsafe fn walk(&self, tag: usize, depth: usize, hash: &mut u64, budget: &mut usize) -> bool {
        let Some(map) = tag.checked_add(Self::MEMBER) else { return false };
        // SAFETY: the caller guarantees a live CompoundTag, whose map starts at the member offset.
        let map = unsafe { &*std::ptr::with_exposed_provenance::<MsvcMap<TagVariantLayout>>(map) };
        Self::mix(hash, &(map.len() as u64).to_le_bytes());
        let mut nested = Vec::new();
        let mut within = true;
        // SAFETY: the map is a live std::map<std::string, CompoundTagVariant>, which no one changes during the walk.
        let complete = unsafe {
            map.for_each(|key, value| {
                if *budget == 0 {
                    within = false;
                    return;
                }
                *budget -= 1;
                Self::mix(hash, key);
                Self::mix(hash, &[value.index]);
                Self::mix(hash, value.storage_after_vtable());
                if depth + 1 < Self::STAMP_DEPTH && matches!(value.index, Self::COMPOUND | TagVariantLayout::LIST) {
                    nested.push((value.index, value.tag()));
                }
            })
        };
        if !complete || !within {
            return false;
        }
        for (index, item) in nested {
            let ok = if index == Self::COMPOUND {
                // SAFETY: the variant index says the storage holds a CompoundTag, vtable first; its vtable is checked
                // before the walk follows it.
                match unsafe { self.compound(item) } {
                    // SAFETY: a live CompoundTag of the same tag tree.
                    Some(compound) => unsafe { self.walk(compound, depth + 1, hash, budget) },
                    None => true,
                }
            } else {
                // SAFETY: the variant index says the storage holds a ListTag.
                unsafe { self.walk_list(item, depth + 1, hash, budget) }
            };
            if !ok {
                return false;
            }
        }
        true
    }

    // Hashes the element addresses of the list at `list` and walks each compound element.
    //
    // Safety: `list` is a live `ListTag` that no one changes during the walk.
    unsafe fn walk_list(&self, list: usize, depth: usize, hash: &mut u64, budget: &mut usize) -> bool {
        let Some(items) = list.checked_add(Self::MEMBER) else { return false };
        // SAFETY: a ListTag's element vector follows the vtable.
        let items = unsafe { &*std::ptr::with_exposed_provenance::<MsvcVec<usize>>(items) };
        for element in items.as_slice().iter().copied().take(Self::MAX_ELEMENTS) {
            Self::mix(hash, &(element as u64).to_le_bytes());
            // SAFETY: an element of a live list is zero or a live Tag.
            let Some(compound) = (unsafe { self.compound(element) }) else { continue };
            // SAFETY: a compound element of a live list is a live CompoundTag of the same tag tree.
            if !unsafe { self.walk(compound, depth, hash, budget) } {
                return false;
            }
        }
        true
    }

    /// The int tag under `key` of the compound at `tag`. `None` for a missing key and for a tag of another type.
    ///
    /// # Safety
    /// `tag` is a live `CompoundTag` of the game build the reader was made for.
    pub unsafe fn int(&self, tag: usize, key: &str) -> Option<i32> {
        // SAFETY: the caller's guarantee is passed on.
        let value = unsafe { Self::value(tag, key, TagVariantLayout::INT) }?;
        // SAFETY: the variant index says the Tag object is an IntTag, whose value is the i32 after the vtable.
        unsafe { ObjectRead::value::<i32>(value.tag(), Self::VALUE) }
    }
}
