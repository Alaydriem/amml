use std::marker::PhantomData;

use crate::structs::string::MsvcString;

/// Layout mirror of the release MSVC `std::map<std::string, V>` (16 bytes), for reads only: the pointer to the head
/// node of the red-black tree, then the number of elements.
///
/// A node holds the left, parent and right pointers, the color byte and the `_Isnil` byte, then the pair of the
/// key (a `std::string`) and the value. The head is a nil node whose parent is the root; every missing child
/// points at the head. The map is never changed from Rust: it has no insert and no allocation.
#[repr(C)]
pub struct MsvcMap<V> {
    head: *const u8,
    size: usize,
    _value: PhantomData<V>,
}

impl<V> MsvcMap<V> {
    const PARENT: usize = 0x08;
    const LEFT: usize = 0x00;
    const RIGHT: usize = 0x10;
    const IS_NIL: usize = 0x19;
    const KEY: usize = 0x20;
    /// The most nodes a walk visits. A red-black tree of any size that fits memory is shallower, so a longer walk
    /// is a loop in memory that is not a tree.
    const MAX_DEPTH: usize = 128;

    /// Compile-time check: the value sits after the 32-byte key, at its own alignment, and the alignment is one
    /// the STL node allocation gives.
    const LAYOUT_OK: () = assert!(align_of::<V>() <= 16, "MsvcMap<V> needs a V with alignment of at most 16");

    /// The offset of the value in a node: after the key, at the alignment of `V`.
    pub const VALUE_OFFSET: usize = (Self::KEY + size_of::<MsvcString>()).next_multiple_of(align_of::<V>());

    /// The number of elements.
    pub fn len(&self) -> usize {
        self.size
    }

    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    /// The value of `key`, found the way `std::map::find` finds it: the first node whose key is not below `key`,
    /// then a check that `key` is not below that node's key. Keys compare as unsigned bytes, then by length, as
    /// `std::string` does. `None` for a missing key, a null head or pointer, and a walk longer than any tree.
    ///
    /// # Safety
    /// The map must be a live `std::map<std::string, V>` of the MSVC STL, and must not change while the returned
    /// reference lives.
    pub unsafe fn find(&self, key: &str) -> Option<&V> {
        let () = Self::LAYOUT_OK;
        if self.head.is_null() {
            return None;
        }
        let head = self.head;
        // SAFETY: the caller guarantees a live head node, whose parent is the root.
        let mut node = unsafe { Self::link(head, Self::PARENT) };
        let mut bound = head;
        let mut steps = 0;
        // SAFETY: the caller guarantees live nodes; `is_nil` rejects a null pointer first.
        while !node.is_null() && !unsafe { Self::is_nil(node) } {
            steps += 1;
            if steps > Self::MAX_DEPTH {
                return None;
            }
            // SAFETY: a non-nil node of a live map holds a key string.
            let node_key = unsafe { Self::key(node) };
            if node_key < key.as_bytes() {
                // SAFETY: as above.
                node = unsafe { Self::link(node, Self::RIGHT) };
            } else {
                bound = node;
                // SAFETY: as above.
                node = unsafe { Self::link(node, Self::LEFT) };
            }
        }
        if node.is_null() || bound == head {
            return None;
        }
        // SAFETY: `bound` is a non-nil node of the live map.
        if key.as_bytes() < unsafe { Self::key(bound) } {
            return None;
        }
        // SAFETY: a node holds its value at `VALUE_OFFSET`; the caller guarantees it is live and a `V`.
        Some(unsafe { &*bound.add(Self::VALUE_OFFSET).cast::<V>() })
    }

    /// Calls `visit` with the key bytes and the value of each element, in key order. Returns `false`, after the
    /// elements visited so far, for a null head or pointer, a tree deeper than any tree and more elements than
    /// `len`, which a loop in memory gives; `true` when the walk ends at the last element.
    ///
    /// # Safety
    /// The map must be a live `std::map<std::string, V>` of the MSVC STL, and must not change during the walk.
    pub unsafe fn for_each(&self, mut visit: impl FnMut(&[u8], &V)) -> bool {
        let () = Self::LAYOUT_OK;
        if self.head.is_null() {
            return false;
        }
        let mut pending = [std::ptr::null::<u8>(); 128];
        let mut depth = 0;
        let mut visited = 0;
        // SAFETY: the caller guarantees a live head node, whose parent is the root.
        let mut node = unsafe { Self::link(self.head, Self::PARENT) };
        loop {
            // SAFETY: the caller guarantees live nodes; `is_nil` is reached only for a non-null pointer.
            while !node.is_null() && !unsafe { Self::is_nil(node) } {
                if depth == pending.len() {
                    return false;
                }
                pending[depth] = node;
                depth += 1;
                // SAFETY: a non-nil node of a live map holds its child pointers.
                node = unsafe { Self::link(node, Self::LEFT) };
            }
            if node.is_null() {
                return false;
            }
            if depth == 0 {
                return true;
            }
            depth -= 1;
            let current = pending[depth];
            visited += 1;
            if visited > self.size {
                return false;
            }
            // SAFETY: `current` is a non-nil node of the live map: its key is a live string and its value a `V`.
            visit(unsafe { Self::key(current) }, unsafe { &*current.add(Self::VALUE_OFFSET).cast::<V>() });
            // SAFETY: as above.
            node = unsafe { Self::link(current, Self::RIGHT) };
        }
    }

    // The node pointer stored at `offset` in `node`.
    unsafe fn link(node: *const u8, offset: usize) -> *const u8 {
        // SAFETY: the caller passes a readable node.
        unsafe { node.add(offset).cast::<*const u8>().read_unaligned() }
    }

    // Whether `node` is the nil node (the head).
    unsafe fn is_nil(node: *const u8) -> bool {
        // SAFETY: the caller passes a readable node.
        unsafe { node.add(Self::IS_NIL).read() != 0 }
    }

    // The key bytes of `node`.
    unsafe fn key<'a>(node: *const u8) -> &'a [u8] {
        // SAFETY: the caller passes a non-nil node, whose key is a live `std::string`.
        unsafe { (*node.add(Self::KEY).cast::<MsvcString>()).as_bytes() }
    }
}
