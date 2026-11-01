use std::ffi::c_void;
use std::sync::atomic::{AtomicUsize, Ordering};

use amml_game::{ContainerScreenController, EntryId, ItemStackBaseRef};
use amml_msvc::MsvcString;

use crate::support::{FakeEntries, FakeVirtual, GameMemory, TestHeap};

const VISUAL_ITEM_SLOT: u64 = 71;

static THIS: AtomicUsize = AtomicUsize::new(0);
static COLLECTION: AtomicUsize = AtomicUsize::new(0);
static INDEX: AtomicUsize = AtomicUsize::new(0);
static STACK: AtomicUsize = AtomicUsize::new(0);

// returns the stack in STACK, and records rcx, rdx and r8d
extern "C" fn visual_item(this: *mut c_void, collection: *const MsvcString, index: i32) -> *const c_void {
    THIS.store(this as usize, Ordering::SeqCst);
    COLLECTION.store(collection as usize, Ordering::SeqCst);
    INDEX.store(index as u32 as usize, Ordering::SeqCst);
    STACK.load(Ordering::SeqCst) as *const c_void
}

#[test]
fn visual_item_passes_the_collection_by_reference_and_the_index_and_returns_the_fakes_stack() {
    let entries =
        FakeEntries::build(&[(EntryId::CONTAINER_SCREEN_CONTROLLER_GET_VISUAL_ITEM_STACK_IMPL, VISUAL_ITEM_SLOT)]);
    let mut controller = FakeVirtual::new(0x20, &[(VISUAL_ITEM_SLOT, visual_item as *const () as usize)]);
    let mut stack = GameMemory::new(0x80);
    STACK.store(stack.ptr() as usize, Ordering::SeqCst);
    // SAFETY: the test heap is the Rust allocator.
    let mut collection = unsafe { MsvcString::new_in(&TestHeap::heap(), "container_items") }.unwrap();
    let collection_ptr = &raw const collection as usize;
    // SAFETY: a live fake whose vtable holds the slot.
    let controller = unsafe { ContainerScreenController::from_raw(controller.ptr(), &entries) }.unwrap();

    let item: Option<ItemStackBaseRef> = controller.visual_item(&collection, -3);

    assert_eq!(item.map(|item| item.as_ptr() as usize), Some(STACK.load(Ordering::SeqCst)));
    assert_eq!(THIS.load(Ordering::SeqCst), controller.address());
    assert_eq!(COLLECTION.load(Ordering::SeqCst), collection_ptr);
    assert_eq!(INDEX.load(Ordering::SeqCst), (-3i32) as u32 as usize);
    // SAFETY: the string was made with the test heap.
    unsafe { collection.release(&TestHeap::heap()) };
}

#[test]
fn visual_item_is_none_when_the_entry_is_off_or_the_game_returns_null() {
    let off = FakeEntries::build(&[]);
    let mut controller = FakeVirtual::new(0x20, &[(VISUAL_ITEM_SLOT, visual_item as *const () as usize)]);
    let ptr = controller.ptr();
    // SAFETY: the test heap is the Rust allocator.
    let mut collection = unsafe { MsvcString::new_in(&TestHeap::heap(), "x") }.unwrap();
    // SAFETY: a live fake; the vtable is never read because the entry is off.
    let wrapper = unsafe { ContainerScreenController::from_raw(ptr, &off) }.unwrap();
    assert!(wrapper.visual_item(&collection, 0).is_none());

    let on = FakeEntries::build(&[(EntryId::CONTAINER_SCREEN_CONTROLLER_GET_VISUAL_ITEM_STACK_IMPL, VISUAL_ITEM_SLOT)]);
    STACK.store(0, Ordering::SeqCst);
    // SAFETY: a live fake whose vtable holds the slot.
    let wrapper = unsafe { ContainerScreenController::from_raw(ptr, &on) }.unwrap();
    assert!(wrapper.visual_item(&collection, 0).is_none());
    // SAFETY: the string was made with the test heap.
    unsafe { collection.release(&TestHeap::heap()) };
}
