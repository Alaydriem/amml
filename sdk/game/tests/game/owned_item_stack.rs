use std::ffi::c_void;
use std::sync::atomic::{AtomicUsize, Ordering};

use amml_game::{CompoundTagRef, EntryId, GameEntries, OwnedItemStack};

use crate::support::{FakeCompound, FakeEntries, FakeValue, TestHeap};

static STACK_VTABLE: AtomicUsize = AtomicUsize::new(0);
static ITEM_COUNTER: AtomicUsize = AtomicUsize::new(0);
static RETURN_BUFFER: AtomicUsize = AtomicUsize::new(0);
static TAG_ARGUMENT: AtomicUsize = AtomicUsize::new(0);
static FROM_TAG_CALLS: AtomicUsize = AtomicUsize::new(0);
static DTOR_CALLS: AtomicUsize = AtomicUsize::new(0);
static DTOR_THIS: AtomicUsize = AtomicUsize::new(0);
static DTOR_FLAGS: AtomicUsize = AtomicUsize::new(usize::MAX);

// builds a stack in the hidden buffer in rcx from the tag in rdx and returns the buffer, as fromTag does; mItem is
// the counter in ITEM_COUNTER (zero is the empty stack)
extern "C" fn from_tag(buffer: *mut c_void, tag: *const c_void) -> *mut c_void {
    FROM_TAG_CALLS.fetch_add(1, Ordering::SeqCst);
    RETURN_BUFFER.store(buffer as usize, Ordering::SeqCst);
    TAG_ARGUMENT.store(tag as usize, Ordering::SeqCst);
    // SAFETY: the buffer is the 0x98 bytes the wrapper allocated.
    unsafe {
        buffer.cast::<usize>().write(STACK_VTABLE.load(Ordering::SeqCst));
        buffer.cast::<usize>().add(1).write(ITEM_COUNTER.load(Ordering::SeqCst));
    }
    buffer
}

// the scalar deleting destructor in slot 0 of the stack's vtable
extern "C" fn dtor(this: *mut c_void, flags: u32) -> *mut c_void {
    DTOR_CALLS.fetch_add(1, Ordering::SeqCst);
    DTOR_THIS.store(this as usize, Ordering::SeqCst);
    DTOR_FLAGS.store(flags as usize, Ordering::SeqCst);
    this
}

struct Setup {
    entries: GameEntries,
    tag: FakeCompound,
    // the vtable of the fake stack and the SharedCounter<Item> whose first member is the Item pointer
    _vtable: Box<[usize; 1]>,
    _counter: Box<[usize; 1]>,
}

impl Setup {
    fn new(with_item: bool) -> Self {
        let vtable = Box::new([dtor as *const () as usize]);
        let counter = Box::new([0x4000usize]);
        STACK_VTABLE.store(vtable.as_ptr() as usize, Ordering::SeqCst);
        ITEM_COUNTER.store(if with_item { counter.as_ptr() as usize } else { 0 }, Ordering::SeqCst);
        let entries = FakeEntries::build(&[
            (EntryId::ITEM_STACK_FROM_TAG, from_tag as *const () as usize as u64),
            (EntryId::COMPOUND_TAG_NODE_VARIANT_INDEX, 0x68),
        ]);
        let tag = FakeCompound::build(vec![("Slot", FakeValue::Byte(0))]);
        Self { entries, tag, _vtable: vtable, _counter: counter }
    }

    fn tag(&self) -> CompoundTagRef<'_> {
        // SAFETY: a live fake laid out as the 26.60 CompoundTag.
        unsafe { CompoundTagRef::from_raw(self.tag.ptr(), &self.entries) }.unwrap()
    }
}

#[test]
fn from_tag_calls_fromtag_with_the_hidden_buffer_and_drop_calls_the_dtor_once_without_the_free_flag() {
    let setup = Setup::new(true);

    let stack = OwnedItemStack::from_tag_in(setup.tag(), TestHeap::heap()).unwrap();

    assert_eq!(FROM_TAG_CALLS.load(Ordering::SeqCst), 1);
    assert_eq!(TAG_ARGUMENT.load(Ordering::SeqCst), setup.tag.ptr() as usize);
    let buffer = RETURN_BUFFER.load(Ordering::SeqCst);
    assert_ne!(buffer, 0);
    assert_eq!(buffer % 16, 0);
    assert_eq!(stack.as_base().as_ptr() as usize, buffer);
    assert_eq!(DTOR_CALLS.load(Ordering::SeqCst), 0);

    drop(stack);

    assert_eq!(DTOR_CALLS.load(Ordering::SeqCst), 1);
    assert_eq!(DTOR_THIS.load(Ordering::SeqCst), buffer);
    assert_eq!(DTOR_FLAGS.load(Ordering::SeqCst), 0);
}

#[test]
fn a_null_stack_gives_none_and_is_destroyed_once() {
    let setup = Setup::new(false);

    assert!(OwnedItemStack::from_tag_in(setup.tag(), TestHeap::heap()).is_none());

    assert_eq!(FROM_TAG_CALLS.load(Ordering::SeqCst), 1);
    assert_eq!(DTOR_CALLS.load(Ordering::SeqCst), 1);
    assert_eq!(DTOR_FLAGS.load(Ordering::SeqCst), 0);
}

#[test]
fn from_tag_without_the_game_heap_gives_none_and_calls_nothing() {
    let setup = Setup::new(true);

    // no mod runtime is loaded in a test, so the game heap is not there
    assert!(OwnedItemStack::from_tag(setup.tag()).is_none());

    assert_eq!(FROM_TAG_CALLS.load(Ordering::SeqCst), 0);
}

#[test]
fn from_tag_with_the_fromtag_entry_off_gives_none() {
    let tag = FakeCompound::build(vec![("Slot", FakeValue::Byte(0))]);
    let entries = FakeEntries::build(&[(EntryId::COMPOUND_TAG_NODE_VARIANT_INDEX, 0x68)]);
    // SAFETY: a live fake laid out as the 26.60 CompoundTag.
    let wrapper = unsafe { CompoundTagRef::from_raw(tag.ptr(), &entries) }.unwrap();

    assert!(OwnedItemStack::from_tag_in(wrapper, TestHeap::heap()).is_none());
    assert_eq!(FROM_TAG_CALLS.load(Ordering::SeqCst), 0);
}
