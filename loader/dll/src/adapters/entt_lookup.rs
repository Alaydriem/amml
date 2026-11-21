use std::ffi::c_void;

use crate::{EnttPoolNodeLayout, EnttRegistryLayout, EnttStorageLayout};

/// The component lookup of the game's entt, as the game's own lookups do it: the storage of a type id in the pool map
/// of the registry, the entity's packed index in the sparse pages of the storage, then the component in the payload
/// pages. The entity index mask and the two page sizes are compile-time constants of the game's entt (read in
/// `setupCamera` on both builds).
///
/// Every step is bounds-checked against the vectors the game keeps: a bucket count that is not a power of two, a node
/// index, a sparse page index or a payload page index outside its vector, or a null page gives `None`, never a read.
/// The game reads the payload page vector without a check; this lookup does not.
pub struct EnttLookup;

impl EnttLookup {
    /// The entity index bits of an entity id; the rest is the version.
    pub const INDEX_MASK: u32 = 0x3_FFFF;
    /// The sparse pages hold 2048 entries.
    pub const SPARSE_PAGE_SHIFT: u32 = 11;
    /// The payload pages hold 128 components.
    pub const PAYLOAD_PAGE_SHIFT: u32 = 7;

    /// The sparse page and the slot in it of `entity`.
    pub fn sparse_position(entity: u32) -> (usize, usize) {
        let index = entity & Self::INDEX_MASK;
        ((index >> Self::SPARSE_PAGE_SHIFT) as usize, (index & ((1 << Self::SPARSE_PAGE_SHIFT) - 1)) as usize)
    }

    /// The payload page and the slot in it of the packed index `packed`.
    pub fn payload_position(packed: u32) -> (usize, usize) {
        ((packed >> Self::PAYLOAD_PAGE_SHIFT) as usize, (packed & ((1 << Self::PAYLOAD_PAGE_SHIFT) - 1)) as usize)
    }

    // The element count of a vector of `first..last`; 0 for an empty, null or inverted one.
    fn count<T>(first: *mut T, last: *mut T) -> usize {
        if first.is_null() || last < first {
            return 0;
        }
        // SAFETY: both pointers are non-null and ordered; the difference is only computed, never read through.
        #[allow(unsafe_code)]
        let count = unsafe { last.offset_from(first) };
        usize::try_from(count).unwrap_or(0)
    }

    /// The storage of the component with entt type id `type_id` in `registry`; `None` when the registry is null, has no
    /// usable buckets, or has no pool of that type.
    ///
    /// # Safety
    /// `registry` is null or a live entt registry of the game, read on the thread that owns it.
    #[allow(unsafe_code)]
    pub unsafe fn storage(registry: *const c_void, type_id: u32) -> Option<*const EnttStorageLayout> {
        if registry.is_null() {
            return None;
        }
        // SAFETY: the caller guarantees a live registry, laid out as the mirror.
        let registry = unsafe { &*registry.cast::<EnttRegistryLayout>() };
        let [buckets, buckets_end, _] = registry.buckets.raw_parts();
        let [nodes, nodes_end, _] = registry.nodes.raw_parts();
        let bucket_count = Self::count(buckets, buckets_end);
        let node_count = Self::count(nodes, nodes_end);
        if !bucket_count.is_power_of_two() {
            return None;
        }
        // SAFETY: the bucket index is below the bucket count, so the read is inside the bucket vector.
        let mut next = unsafe { buckets.add((type_id as usize) & (bucket_count - 1)).read() };
        // A chain is never longer than the node vector; the bound keeps a broken chain from looping.
        for _ in 0..=node_count {
            if next >= node_count {
                return None;
            }
            // SAFETY: `next` is below the node count, so the node is inside the node vector.
            let node: &EnttPoolNodeLayout = unsafe { &*nodes.add(next) };
            if node.type_id == type_id {
                return (!node.pool.is_null()).then_some(node.pool.cast());
            }
            next = node.next;
        }
        None
    }

    /// The packed index of `entity` in `storage`; `None` when the storage does not hold it (another version, a null or
    /// missing sparse page).
    ///
    /// # Safety
    /// `storage` is a live entt storage of the game.
    #[allow(unsafe_code)]
    pub unsafe fn packed_index(storage: *const EnttStorageLayout, entity: u32) -> Option<u32> {
        let (page, slot) = Self::sparse_position(entity);
        // SAFETY: the caller guarantees a live storage, laid out as the mirror.
        let [pages, pages_end, _] = unsafe { (*storage).sparse.raw_parts() };
        if page >= Self::count(pages, pages_end) {
            return None;
        }
        // SAFETY: the page index is below the page count, so the read is inside the page vector.
        let page = unsafe { pages.add(page).read() };
        if page.is_null() {
            return None;
        }
        // SAFETY: a sparse page holds 1 << SPARSE_PAGE_SHIFT entries and `slot` is below that.
        let value = unsafe { page.add(slot).read() };
        ((value ^ (entity & !Self::INDEX_MASK)) < Self::INDEX_MASK).then_some(value & Self::INDEX_MASK)
    }

    /// The address of the component of `entity` in `storage`, whose components are `size` bytes each; `None` when the
    /// storage does not hold the entity or its payload page is outside the pool or null. The address is inside the
    /// payload page.
    ///
    /// # Safety
    /// `storage` is a live entt storage of the game whose components are `size` bytes.
    #[allow(unsafe_code)]
    pub unsafe fn component(storage: *const EnttStorageLayout, entity: u32, size: usize) -> Option<usize> {
        // SAFETY: forwarded to the caller.
        let packed = unsafe { Self::packed_index(storage, entity) }?;
        let (page, slot) = Self::payload_position(packed);
        // SAFETY: the caller guarantees a live storage, laid out as the mirror.
        let [pages, pages_end, _] = unsafe { (*storage).payload.raw_parts() };
        if page >= Self::count(pages, pages_end) {
            return None;
        }
        // SAFETY: the page index is below the page count, so the read is inside the page vector.
        let page = unsafe { pages.add(page).read() };
        if page.is_null() {
            return None;
        }
        page.expose_provenance().checked_add(slot.checked_mul(size)?)
    }
}
