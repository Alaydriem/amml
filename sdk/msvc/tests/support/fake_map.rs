use amml_msvc::{MsvcMap, MsvcString};

use super::Fixture;

// words per node: the header (left, parent, right, color and isnil), the key (4 words), one value word
const NODE_WORDS: usize = 9;
const LEFT: usize = 0;
const PARENT: usize = 1;
const RIGHT: usize = 2;
const FLAGS: usize = 3;
const KEY: usize = 4;
const VALUE: usize = 8;

/// A `std::map<std::string, u64>` laid out the way the MSVC STL lays it out: the object is the head pointer and
/// the size, the head node is a nil node whose parent is the root, and every missing child points at the head.
pub struct FakeMap {
    object: Box<[u64; 2]>,
    head: Box<[u64; NODE_WORDS]>,
    // boxed, so a node keeps its address while the vector grows
    #[allow(clippy::vec_box)]
    nodes: Vec<Box<[u64; NODE_WORDS]>>,
}

impl FakeMap {
    /// A tree of `entries`, balanced by their sorted order.
    pub fn build(entries: &[(&str, u64)]) -> Self {
        let mut sorted = entries.to_vec();
        sorted.sort_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));
        let mut map = Self { object: Box::new([0; 2]), head: Box::new([0; NODE_WORDS]), nodes: Vec::new() };
        let head = map.head_ptr();
        map.head[FLAGS] = 1 << 8;
        let root = map.attach(&sorted, head, head);
        map.head[PARENT] = root;
        map.head[LEFT] = if root == head { head } else { map.edge(root, LEFT, head) };
        map.head[RIGHT] = if root == head { head } else { map.edge(root, RIGHT, head) };
        map.object[0] = head;
        map.object[1] = sorted.len() as u64;
        map
    }

    /// The map object, as the game holds it inside a `CompoundTag`.
    pub fn map(&self) -> &MsvcMap<u64> {
        // SAFETY: the object is the head pointer and the size, which `MsvcMap` mirrors.
        unsafe { &*std::ptr::from_ref(&*self.object).cast::<MsvcMap<u64>>() }
    }

    /// The node pointers of the tree, to corrupt it in a test.
    pub fn root(&self) -> *mut [u64; NODE_WORDS] {
        self.head[PARENT] as *mut [u64; NODE_WORDS]
    }

    /// Points the left child of the root at the root itself: a cycle.
    pub fn make_cyclic(&mut self) {
        let root = self.head[PARENT];
        // SAFETY: the root is one of the boxed nodes of this map.
        unsafe { (*self.root())[LEFT] = root };
    }

    fn head_ptr(&mut self) -> u64 {
        std::ptr::from_mut(&mut *self.head) as u64
    }

    // follows `side` down from `node` until the nil head
    fn edge(&self, node: u64, side: usize, head: u64) -> u64 {
        let mut current = node;
        loop {
            // SAFETY: every node pointer is a boxed node of this map.
            let next = unsafe { (*(current as *const [u64; NODE_WORDS]))[side] };
            if next == head {
                return current;
            }
            current = next;
        }
    }

    fn attach(&mut self, sorted: &[(&str, u64)], parent: u64, head: u64) -> u64 {
        if sorted.is_empty() {
            return head;
        }
        let middle = sorted.len() / 2;
        let (key, value) = sorted[middle];
        let mut node = Box::new([0u64; NODE_WORDS]);
        node[PARENT] = parent;
        node[FLAGS] = 0;
        node[VALUE] = value;
        // SAFETY: the fixture heap is the real operator new and delete.
        let Ok(text) = (unsafe { MsvcString::new_in(&Fixture::heap(), key) }) else {
            panic!("the fixture heap failed to allocate");
        };
        // SAFETY: a string is four words and the key slot is four words of the node.
        unsafe { node[KEY..].as_mut_ptr().cast::<MsvcString>().write(text) };
        let ptr = std::ptr::from_mut(&mut *node) as u64;
        self.nodes.push(node);
        let left = self.attach(&sorted[..middle], ptr, head);
        let right = self.attach(&sorted[middle + 1..], ptr, head);
        // SAFETY: `ptr` is the boxed node pushed above, which does not move.
        unsafe {
            (*(ptr as *mut [u64; NODE_WORDS]))[LEFT] = left;
            (*(ptr as *mut [u64; NODE_WORDS]))[RIGHT] = right;
        }
        ptr
    }
}

impl Drop for FakeMap {
    fn drop(&mut self) {
        let heap = Fixture::heap();
        for node in &mut self.nodes {
            // SAFETY: the key was made by `new_in` with the fixture heap in `attach`.
            unsafe { (*node[KEY..].as_mut_ptr().cast::<MsvcString>()).release(&heap) };
        }
    }
}
