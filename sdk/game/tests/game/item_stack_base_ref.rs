use std::ffi::c_void;
use std::sync::atomic::{AtomicUsize, Ordering};

use amml_game::{EntryId, GameEntries, ItemStackBaseRef};
use amml_msvc::MsvcString;

use crate::support::{FakeCompound, FakeEntries, FakeValue, FakeVirtual, GameMemory, TestHeap};

const MAX_DAMAGE_SLOT: u64 = 36;
const IS_GLINT_SLOT: u64 = 40;
const ANIMATION_FRAME_SLOT: u64 = 119;

static DAMAGE_THIS: AtomicUsize = AtomicUsize::new(0);
static DAMAGED_THIS: AtomicUsize = AtomicUsize::new(0);
static GLINT_ITEM: AtomicUsize = AtomicUsize::new(0);
static GLINT_STACK: AtomicUsize = AtomicUsize::new(0);
static FRAME: [AtomicUsize; 5] =
    [AtomicUsize::new(0), AtomicUsize::new(0), AtomicUsize::new(0), AtomicUsize::new(0), AtomicUsize::new(0)];

extern "C" fn get_damage_value(this: *const c_void) -> i16 {
    DAMAGE_THIS.store(this as usize, Ordering::SeqCst);
    37
}

extern "C" fn is_damaged(this: *const c_void) -> u8 {
    DAMAGED_THIS.store(this as usize, Ordering::SeqCst);
    1
}

extern "C" fn get_max_damage(_: *const c_void) -> i16 {
    250
}

extern "C" fn is_glint(item: *const c_void, stack: *const c_void) -> u8 {
    GLINT_ITEM.store(item as usize, Ordering::SeqCst);
    GLINT_STACK.store(stack as usize, Ordering::SeqCst);
    1
}

// records this, the holder, the first bool, the stack and the second bool
extern "C" fn animation_frame(
    item: *const c_void,
    holder: *const c_void,
    first: u8,
    stack: *const c_void,
    second: u8,
) -> i32 {
    FRAME[0].store(item as usize, Ordering::SeqCst);
    FRAME[1].store(holder as usize, Ordering::SeqCst);
    FRAME[2].store(usize::from(first), Ordering::SeqCst);
    FRAME[3].store(stack as usize, Ordering::SeqCst);
    FRAME[4].store(usize::from(second), Ordering::SeqCst);
    3
}

struct Setup {
    item: FakeVirtual,
    // the SharedCounter<Item>: its first member is the Item pointer
    _counter: Box<usize>,
    stack: GameMemory,
    name_heap_block: MsvcString,
}

impl Setup {
    fn new() -> Self {
        let mut item = FakeVirtual::new(
            0x200,
            &[
                (MAX_DAMAGE_SLOT, get_max_damage as *const () as usize),
                (IS_GLINT_SLOT, is_glint as *const () as usize),
                (ANIMATION_FRAME_SLOT, animation_frame as *const () as usize),
            ],
        );
        // SAFETY: the test heap is the Rust allocator.
        let name = unsafe { MsvcString::new_in(&TestHeap::heap(), "minecraft:red_shulker_box") }.unwrap();
        // Item::mFullName is a HashedString at 0x120: the hash, then the std::string at 0x128
        item.put::<u64>(0x120, 0xABCD);
        // SAFETY: the item buffer holds a 48-byte HashedString at 0x120; the string is copied in bit for bit and the
        // original is released once, in drop.
        unsafe { item.ptr().cast::<u8>().add(0x128).cast::<MsvcString>().write(std::ptr::read(&raw const name)) };
        let mut counter = Box::new(item.ptr() as usize);
        let mut stack = GameMemory::new(0x80);
        stack.put::<usize>(0x08, &raw mut *counter as usize);
        stack.put::<u8>(0x22, 12);
        Self { item, _counter: counter, stack, name_heap_block: name }
    }

    fn entries() -> GameEntries {
        FakeEntries::build(&[
            (EntryId::ITEM_FULL_NAME, 0x120),
            (EntryId::ITEM_GET_MAX_DAMAGE, MAX_DAMAGE_SLOT),
            (EntryId::ITEM_IS_GLINT, IS_GLINT_SLOT),
            (EntryId::ITEM_GET_ANIMATION_FRAME_FOR, ANIMATION_FRAME_SLOT),
            (EntryId::ITEM_STACK_BASE_GET_DAMAGE_VALUE, get_damage_value as *const () as usize as u64),
            (EntryId::ITEM_STACK_BASE_IS_DAMAGED, is_damaged as *const () as usize as u64),
            (EntryId::COMPOUND_TAG_NODE_VARIANT_INDEX, 0x68),
        ])
    }
}

impl Drop for Setup {
    fn drop(&mut self) {
        // SAFETY: the name was made with the test heap and is released once; the copy in the item buffer is not read
        // again.
        unsafe { self.name_heap_block.release(&TestHeap::heap()) };
    }
}

#[test]
fn type_name_and_count_come_from_the_item_and_the_stack() {
    let entries = Setup::entries();
    let mut setup = Setup::new();
    // SAFETY: a live fake stack laid out as ItemStackBase, whose item is a live fake.
    let stack = unsafe { ItemStackBaseRef::from_raw(setup.stack.ptr(), &entries) }.unwrap();

    assert_eq!(stack.type_name().as_deref(), Some("minecraft:red_shulker_box"));
    assert_eq!(stack.count(), Some(12));
}

#[test]
fn an_empty_stack_gives_none_for_every_item_read() {
    let entries = Setup::entries();
    let mut setup = Setup::new();
    setup.stack.put::<usize>(0x08, 0);
    // SAFETY: a live fake stack whose mItem is null, as the game's empty stack has it.
    let stack = unsafe { ItemStackBaseRef::from_raw(setup.stack.ptr(), &entries) }.unwrap();

    assert_eq!(stack.type_name(), None);
    assert_eq!(stack.count(), None);
    assert_eq!(stack.is_damaged(), None);
    assert_eq!(stack.damage(), None);
    assert_eq!(stack.max_damage(), None);
    assert_eq!(stack.is_glint(), None);
    // SAFETY: a null holder is allowed.
    assert_eq!(unsafe { stack.animation_frame(std::ptr::null()) }, None);
}

#[test]
fn the_durability_calls_pass_the_stack_or_the_item_and_widen_the_result() {
    let entries = Setup::entries();
    let mut setup = Setup::new();
    let stack_ptr = setup.stack.ptr();
    // SAFETY: a live fake stack and item.
    let stack = unsafe { ItemStackBaseRef::from_raw(stack_ptr, &entries) }.unwrap();

    assert_eq!(stack.is_damaged(), Some(true));
    assert_eq!(DAMAGED_THIS.load(Ordering::SeqCst), stack_ptr as usize);
    assert_eq!(stack.damage(), Some(37));
    assert_eq!(DAMAGE_THIS.load(Ordering::SeqCst), stack_ptr as usize);
    assert_eq!(stack.max_damage(), Some(250));
    assert_eq!(stack.is_glint(), Some(true));
    assert_eq!(GLINT_ITEM.load(Ordering::SeqCst), setup.item.ptr() as usize);
    assert_eq!(GLINT_STACK.load(Ordering::SeqCst), stack_ptr as usize);
}

#[test]
fn animation_frame_passes_the_holder_false_the_stack_and_true() {
    let entries = Setup::entries();
    let mut setup = Setup::new();
    let stack_ptr = setup.stack.ptr();
    let holder = 0x5150usize as *const c_void;
    // SAFETY: a live fake stack and item.
    let stack = unsafe { ItemStackBaseRef::from_raw(stack_ptr, &entries) }.unwrap();

    // SAFETY: the fake item never reads the holder.
    assert_eq!(unsafe { stack.animation_frame(holder) }, Some(3));
    let seen: Vec<usize> = FRAME.iter().map(|value| value.load(Ordering::SeqCst)).collect();
    assert_eq!(seen, [setup.item.ptr() as usize, 0x5150, 0, stack_ptr as usize, 1]);
}

#[test]
fn a_call_with_its_entry_off_gives_none_and_calls_nothing() {
    let entries = FakeEntries::build(&[]);
    let mut setup = Setup::new();
    // SAFETY: a live fake stack; no entry is present, so no function is read.
    let stack = unsafe { ItemStackBaseRef::from_raw(setup.stack.ptr(), &entries) }.unwrap();

    assert_eq!(stack.type_name(), None);
    assert_eq!(stack.is_damaged(), None);
    assert_eq!(stack.damage(), None);
    assert_eq!(stack.max_damage(), None);
    assert_eq!(stack.is_glint(), None);
    // SAFETY: a null holder is allowed.
    assert_eq!(unsafe { stack.animation_frame(std::ptr::null()) }, None);
    assert_eq!(DAMAGED_THIS.load(Ordering::SeqCst), 0);
    assert_eq!(GLINT_STACK.load(Ordering::SeqCst), 0);
}

#[test]
fn user_data_is_the_compound_at_0x10_and_none_without_the_nbt_entries() {
    let compound = FakeCompound::build(vec![("Items", FakeValue::List(vec![]))]);
    let mut setup = Setup::new();
    setup.stack.put::<usize>(0x10, compound.ptr() as usize);
    let on = Setup::entries();
    // SAFETY: a live fake stack.
    let stack = unsafe { ItemStackBaseRef::from_raw(setup.stack.ptr(), &on) }.unwrap();
    assert_eq!(stack.user_data().map(|data| data.as_ptr() as usize), Some(compound.ptr() as usize));

    let off = FakeEntries::build(&[]);
    // SAFETY: a live fake stack.
    let stack = unsafe { ItemStackBaseRef::from_raw(setup.stack.ptr(), &off) }.unwrap();
    assert!(stack.user_data().is_none());

    setup.stack.put::<usize>(0x10, 0);
    // SAFETY: a live fake stack.
    let stack = unsafe { ItemStackBaseRef::from_raw(setup.stack.ptr(), &on) }.unwrap();
    assert!(stack.user_data().is_none());
}
