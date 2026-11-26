/// Layout mirror of one node of the release MSVC `std::list<T>`, which `std::unordered_map` keeps its elements
/// in: the next and previous node pointers, then the value. The list's head is a sentinel node with no value;
/// a walk from the head's `next` back to the head visits every element once. For reads only.
#[repr(C)]
pub struct MsvcListNode<T> {
    next: *const MsvcListNode<T>,
    prev: *const MsvcListNode<T>,
    value: T,
}

impl<T> MsvcListNode<T> {
    pub fn next(&self) -> *const MsvcListNode<T> {
        self.next
    }

    pub fn prev(&self) -> *const MsvcListNode<T> {
        self.prev
    }

    /// The value of an element node. The head node has none: its value bytes are not initialised.
    pub fn value(&self) -> &T {
        &self.value
    }
}
