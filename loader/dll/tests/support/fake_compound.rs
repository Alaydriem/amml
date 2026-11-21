use std::ffi::c_void;

use amml_msvc::MsvcString;

use super::{FakeValue, TestHeap};

// words per node: left, parent, right, flags (color, isnil), the key (4 words), the variant (5 words of tag
// storage, then the index byte in the first byte of the next word), 2 words of padding to 0x70 bytes
const NODE_WORDS: usize = 14;
const LEFT: usize = 0;
const PARENT: usize = 1;
const RIGHT: usize = 2;
const FLAGS: usize = 3;
const KEY: usize = 4;
const TAG: usize = 8;
const INDEX: usize = 13;

// The vtables of a compound tag and a byte tag. No slot is filled: the loader calls no virtual function of a tag.
static COMPOUND_VTABLE: [usize; 6] = [0; 6];
static BYTE_VTABLE: [usize; 6] = [0; 6];

/// A `CompoundTag` laid out the way the 26.60 game lays it out: the vtable, then the `std::map` of nodes whose
/// value is a variant with the Tag object first and the index byte at node + 0x68.
pub struct FakeCompound {
    object: Box<[u64; 3]>,
    head: Box<[u64; NODE_WORDS]>,
    // boxed, so a node keeps its address while the vector grows
    #[allow(clippy::vec_box)]
    nodes: Vec<Box<[u64; NODE_WORDS]>>,
    // the element blocks of the lists and the byte tags, and the compounds in the lists, kept alive
    owned: Vec<Box<[u64]>>,
    children: Vec<FakeCompound>,
    // the key of each node, by the node's index in `nodes`
    keys: Vec<String>,
}

impl FakeCompound {
    pub fn build(entries: Vec<(&str, FakeValue)>) -> Self {
        let mut entries = entries;
        entries.sort_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));
        let mut compound = Self {
            object: Box::new([Self::compound_vtable() as u64, 0, entries.len() as u64]),
            head: Box::new([0; NODE_WORDS]),
            nodes: Vec::new(),
            owned: Vec::new(),
            children: Vec::new(),
            keys: Vec::new(),
        };
        let head = std::ptr::from_mut(&mut *compound.head) as u64;
        compound.head[FLAGS] = 1 << 8;
        let mut slots: Vec<Option<(&str, FakeValue)>> = entries.into_iter().map(Some).collect();
        let root = compound.attach(&mut slots[..], head, head);
        compound.head[PARENT] = root;
        compound.head[LEFT] = if root == head { head } else { compound.edge(root, LEFT, head) };
        compound.head[RIGHT] = if root == head { head } else { compound.edge(root, RIGHT, head) };
        compound.object[1] = head;
        compound
    }

    /// The `CompoundTag*` of the fake.
    pub fn ptr(&self) -> *mut c_void {
        std::ptr::from_ref(&*self.object).cast_mut().cast()
    }

    /// The compound at `index` of the lists of this tag, in the order they were built, to change in place.
    pub fn child_mut(&mut self, index: usize) -> &mut FakeCompound {
        &mut self.children[index]
    }

    /// Writes `value` into the byte tag under `key` in place: the tag keeps its address, as a game tag that a
    /// change edits.
    pub fn set_byte(&mut self, key: &str, value: i8) {
        let index = self.keys.iter().position(|name| name == key).unwrap();
        self.nodes[index][TAG + 1] = u64::from(value as u8);
    }

    /// The address of the vtable every fake compound tag has: the `CompoundTag.vtable` value of the fake game.
    pub fn compound_vtable() -> usize {
        COMPOUND_VTABLE.as_ptr().addr()
    }

    // The address of the vtable of the fake byte tags.
    fn byte_vtable() -> usize {
        BYTE_VTABLE.as_ptr().addr()
    }

    fn edge(&self, node: u64, side: usize, head: u64) -> u64 {
        let mut current = node;
        loop {
            // SAFETY: every node pointer is a boxed node of this tag.
            let next = unsafe { (*(current as *const [u64; NODE_WORDS]))[side] };
            if next == head {
                return current;
            }
            current = next;
        }
    }

    fn attach(&mut self, slots: &mut [Option<(&str, FakeValue)>], parent: u64, head: u64) -> u64 {
        if slots.is_empty() {
            return head;
        }
        let middle = slots.len() / 2;
        let (key, value) = slots[middle].take().unwrap();
        let mut node = Box::new([0u64; NODE_WORDS]);
        node[PARENT] = parent;
        // SAFETY: the test heap is the Rust allocator.
        let text = unsafe { MsvcString::new_in(&TestHeap::heap(), key) }.unwrap();
        // SAFETY: a string is four words and the key slot is four words of the node.
        unsafe { node[KEY..].as_mut_ptr().cast::<MsvcString>().write(text) };
        self.fill_value(&mut node, value);
        let ptr = std::ptr::from_mut(&mut *node) as u64;
        self.nodes.push(node);
        self.keys.push(key.to_string());
        let (below, above) = slots.split_at_mut(middle);
        let left = self.attach(below, ptr, head);
        let right = self.attach(&mut above[1..], ptr, head);
        // SAFETY: `ptr` is the boxed node pushed above, which does not move.
        unsafe {
            (*(ptr as *mut [u64; NODE_WORDS]))[LEFT] = left;
            (*(ptr as *mut [u64; NODE_WORDS]))[RIGHT] = right;
        }
        ptr
    }

    fn fill_value(&mut self, node: &mut [u64; NODE_WORDS], value: FakeValue) {
        match value {
            FakeValue::Byte(byte) => {
                node[TAG] = Self::byte_vtable() as u64;
                node[TAG + 1] = u64::from(byte as u8);
                node[INDEX] = 1;
            }
            FakeValue::Int(int) => {
                node[TAG + 1] = u64::from(int as u32);
                node[INDEX] = 3;
            }
            FakeValue::List(items) => self.fill_list(node, items, false),
            FakeValue::ListWithAByte(items) => self.fill_list(node, items, true),
        }
    }

    fn fill_list(&mut self, node: &mut [u64; NODE_WORDS], items: Vec<FakeCompound>, with_byte: bool) {
        let mut pointers: Vec<u64> = items.iter().map(|item| item.ptr() as u64).collect();
        if with_byte {
            let byte: Box<[u64]> = Box::new([Self::byte_vtable() as u64, 7]);
            pointers.insert(0, byte.as_ptr() as u64);
            self.owned.push(byte);
        }
        let block: Box<[u64]> = pointers.into_boxed_slice();
        let first = block.as_ptr() as u64;
        let last = first + (block.len() * 8) as u64;
        // the ListTag: vtable, then the vector's first, last and end
        node[TAG] = Self::compound_vtable() as u64;
        node[TAG + 1] = first;
        node[TAG + 2] = last;
        node[TAG + 3] = last;
        node[INDEX] = 9;
        self.owned.push(block);
        self.children.extend(items);
    }
}

impl Drop for FakeCompound {
    fn drop(&mut self) {
        let heap = TestHeap::heap();
        for node in &mut self.nodes {
            // SAFETY: the key was made by `new_in` with the test heap in `attach`.
            unsafe { (*node[KEY..].as_mut_ptr().cast::<MsvcString>()).release(&heap) };
        }
    }
}
