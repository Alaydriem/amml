use std::ffi::c_void;
use std::hint::black_box;
use std::sync::Mutex;
use std::sync::atomic::{AtomicI32, AtomicUsize, Ordering};

use amml_data::{EntryKind, Resolved};
use amml_runtime::ImageLayout;

use super::{ChunkGame, Harness};

// The stand-in base listener table: the locator word, nine distinct slot values that no call uses, and the word of the
// next object, which a test sets.
static BASE_TABLE: [AtomicUsize; BlockGame::SLOTS + 2] = [
    AtomicUsize::new(0xA0),
    AtomicUsize::new(0xB0),
    AtomicUsize::new(0xB1),
    AtomicUsize::new(0xB2),
    AtomicUsize::new(0xB3),
    AtomicUsize::new(0xB4),
    AtomicUsize::new(0xB5),
    AtomicUsize::new(0xB6),
    AtomicUsize::new(0xB7),
    AtomicUsize::new(0xB8),
    AtomicUsize::new(0),
];
static ADDED: Mutex<Vec<usize>> = Mutex::new(Vec::new());
static REMOVED: Mutex<Vec<usize>> = Mutex::new(Vec::new());
static DIMENSION: AtomicI32 = AtomicI32::new(0);

/// A stand-in block source listener list for the block listener: `BlockSource::addListener`, `removeListener` and
/// `getDimensionId` in slots 4 to 6 of the stand-in block source vtable of `ActorGame` (they record the listener they
/// get), the base listener vtable, and calls that do what the game's own loops do with a listener (`onBlockChanged`
/// from `fireBlockChanged`, `onSourceDestroyed` from the block source destructor).
pub struct BlockGame;

impl BlockGame {
    pub const SLOTS: usize = 9;
    pub const DESTROYED_SLOT: usize = 2;
    pub const CHANGED_SLOT: usize = 4;

    /// The chunk entries (and so the actor entries) and the listener entries; `without` names entries to leave out.
    /// Clears the recorded calls.
    pub fn entries(without: &[&str]) -> Vec<(&'static str, Resolved)> {
        ADDED.lock().unwrap().clear();
        REMOVED.lock().unwrap().clear();
        DIMENSION.store(0, Ordering::SeqCst);
        Self::set_word_after_table(false);
        let mut entries = ChunkGame::entries(without, 0x1C8, 1);
        let more = [
            (
                "BlockSourceListener.vtable",
                (EntryKind::Global, std::ptr::from_ref(&BASE_TABLE[1]).addr() as u64 - Harness::BASE),
            ),
            ("BlockSourceListener.onSourceDestroyed", (EntryKind::VSlot, Self::DESTROYED_SLOT as u64)),
            ("BlockSourceListener.onBlockChanged", (EntryKind::VSlot, Self::CHANGED_SLOT as u64)),
            ("BlockSource.addListener", (EntryKind::VSlot, 4)),
            ("BlockSource.removeListener", (EntryKind::VSlot, 5)),
            ("BlockSource.getDimensionId", (EntryKind::VSlot, 6)),
        ];
        for (name, (kind, value)) in more {
            if !without.contains(&name) {
                entries.push((name, Resolved { kind, value, recipes_agreeing: 2 }));
            }
        }
        entries
    }

    /// The words of the stand-in base listener vtable.
    pub fn base() -> [usize; Self::SLOTS] {
        std::array::from_fn(|index| BASE_TABLE[index + 1].load(Ordering::SeqCst))
    }

    /// The locator word before the stand-in base table.
    pub fn locator() -> usize {
        BASE_TABLE[0].load(Ordering::SeqCst)
    }

    /// Sets the word after the ninth slot: a pointer into the stand-in code section (`code_pointer`) or into data.
    pub fn set_word_after_table(code_pointer: bool) {
        let word = if code_pointer { Self::add_listener as *const () as usize } else { 0xDA7A };
        BASE_TABLE[Self::SLOTS + 1].store(word, Ordering::SeqCst);
    }

    /// An image layout whose one executable range is the function `add_listener`.
    pub fn image() -> ImageLayout {
        let at = Self::add_listener as *const () as usize as u64 - Harness::BASE;
        ImageLayout::new(u64::MAX / 2, std::iter::once(at..at + 16).collect())
    }

    /// The word before the vtable of the listener object at `listener`.
    pub fn locator_of(listener: usize) -> usize {
        // SAFETY: the test passes a listener object the loader made; the word before its vtable is its locator.
        unsafe {
            let vtable = std::ptr::with_exposed_provenance::<usize>(listener).read();
            std::ptr::with_exposed_provenance::<usize>(vtable).sub(1).read()
        }
    }

    /// The listeners `addListener` got, in order, and those `removeListener` got.
    pub fn added() -> Vec<usize> {
        ADDED.lock().unwrap().clone()
    }

    pub fn removed() -> Vec<usize> {
        REMOVED.lock().unwrap().clone()
    }

    /// The dimension id `getDimensionId` gives from now on.
    pub fn set_dimension(id: i32) {
        DIMENSION.store(id, Ordering::SeqCst);
    }

    /// The words of the vtable of the listener object at `listener`.
    ///
    /// The listener must be live: the call reads its vtable.
    pub fn slots_of(listener: usize) -> [usize; Self::SLOTS] {
        // SAFETY: the test passes a listener the loader attached and has not freed; its first word points to a vtable
        // of `SLOTS` words.
        unsafe {
            let vtable = std::ptr::with_exposed_provenance::<usize>(listener).read();
            std::array::from_fn(|index| std::ptr::with_exposed_provenance::<usize>(vtable).add(index).read())
        }
    }

    /// The game's `fireBlockChanged` loop body for the listener at `listener`: `onBlockChanged` through the vtable with
    /// the block source, the position, the layer, the new and old blocks and the game's own flags.
    pub fn fire_changed(listener: usize, pos: [i32; 3], layer: u32, new_block: usize, old_block: usize) {
        type Changed = unsafe extern "C" fn(usize, usize, *const [i32; 3], u32, usize, usize, i32, usize, u8, usize);
        let function = Self::slots_of(listener)[Self::CHANGED_SLOT];
        // SAFETY: the slot holds the loader's `onBlockChanged` thunk, which has this signature.
        let function = unsafe { std::mem::transmute::<usize, Changed>(function) };
        let source = super::ActorGame::block_source();
        // SAFETY: the thunk reads `pos` for the call only.
        unsafe { black_box(function)(listener, source, &raw const pos, layer, new_block, old_block, 3, 0, 0, 0) };
    }

    /// The block source destructor's loop body: `onSourceDestroyed` of the listener at `listener` for `source`.
    pub fn fire_destroyed(listener: usize, source: usize) {
        type Destroyed = unsafe extern "C" fn(usize, usize);
        let function = Self::slots_of(listener)[Self::DESTROYED_SLOT];
        // SAFETY: the slot holds the loader's `onSourceDestroyed` thunk, which has this signature.
        let function = unsafe { std::mem::transmute::<usize, Destroyed>(function) };
        // SAFETY: as above.
        unsafe { black_box(function)(listener, source) };
    }

    /// `void BlockSource::addListener(BlockSourceListener&)`.
    pub extern "C" fn add_listener(_this: *mut c_void, listener: usize) {
        ADDED.lock().unwrap().push(listener);
    }

    /// `void BlockSource::removeListener(BlockSourceListener&)`.
    pub extern "C" fn remove_listener(_this: *mut c_void, listener: usize) {
        REMOVED.lock().unwrap().push(listener);
    }

    /// `DimensionType BlockSource::getDimensionId() const`: the id through the hidden pointer in rdx.
    pub extern "C" fn get_dimension_id(_this: *mut c_void, out: *mut i32) -> *mut i32 {
        // SAFETY: the adapter passes a writable int buffer.
        unsafe { out.write(DIMENSION.load(Ordering::SeqCst)) };
        out
    }
}
