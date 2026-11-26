/// The object the loader gives the game as a `BlockSourceListener`: a vtable word, a zero word, the RTTI locator of the
/// game's table, then a copy of the `BlockSourceListener.vtable` entry's table with two slots replaced. The vtable word
/// points at the copied slots, so the word before slot 0 is the locator, as in the game's own tables.
///
/// There is one object for the process: it is made at the first attach and reused at every attach, and it is never
/// freed. A detach only takes it out of the block source's listener list, so a game call on another thread that already
/// loaded its pointer still reads a live vtable.
///
/// The slots the game can call on a listener are safe on this object (`logs/review6/`, read from the exe on 26.63 and
/// 70.26): slot 2 and slot 4 are the loader's thunks; slots 1, 3, 4, 6, 7 and 8 of the table are `ret`; slot 5 is the
/// default `onBrightnessChanged`, which calls slot 3 of the object's own copy (`ret`). Slot 0 is the deleting
/// destructor of `BlockSourceHandle`: it reads the word at +8 (zero here, so it calls nothing), and with its delete
/// flag it would free the object with the game's `operator delete`. The game does not call it: the `BlockSource`
/// destructor, the only owner of the listener list seen, calls slot 2 for each listener and never deletes one
/// (`logs/task6/dtor-body-63.txt`, `dtor-body-70.txt`).
///
/// The entry names the table of `BlockSourceHandle`, the smallest listener class, which keeps its block source at +8.
#[repr(C)]
pub struct BlockListenerObject {
    vtable: usize,
    _handle_source: usize,
    _locator: usize,
    slots: [usize; Self::SLOTS],
}

impl BlockListenerObject {
    /// The slots of the `BlockSourceListener` vtable (slot 0 to 8; the listing of `inspect vtable` stops after slot 8 on
    /// 26.63 and 70.26, and `BlockListener::install` refuses a build whose table goes on).
    pub const SLOTS: usize = 9;

    /// A listener object whose vtable is the `SLOTS` words at `base` with `slots` replaced, each by (slot, function),
    /// and the locator word before `base` copied. `None` for a replaced slot outside the table.
    ///
    /// # Safety
    /// `base` is the address of the `BlockSourceListener` vtable of the running game, readable for `SLOTS` words and
    /// for the word before it.
    pub unsafe fn new(base: usize, replaced: [(usize, usize); 2]) -> Option<Box<Self>> {
        if replaced.iter().any(|(slot, _)| *slot >= Self::SLOTS) {
            return None;
        }
        let mut object = Box::new(Self { vtable: 0, _handle_source: 0, _locator: 0, slots: [0; Self::SLOTS] });
        // SAFETY: the caller guarantees the word before the table is readable.
        object._locator = unsafe { std::ptr::with_exposed_provenance::<usize>(base).sub(1).read_unaligned() };
        for (index, slot) in object.slots.iter_mut().enumerate() {
            // SAFETY: the caller guarantees `SLOTS` readable words at `base`.
            *slot = unsafe { std::ptr::with_exposed_provenance::<usize>(base).add(index).read_unaligned() };
        }
        for (slot, function) in replaced {
            object.slots[slot] = function;
        }
        object.vtable = (&raw const object.slots).expose_provenance();
        Some(object)
    }

    /// Gives the allocation up as the address the game gets. The object is never taken back.
    pub fn into_address(self: Box<Self>) -> usize {
        Box::into_raw(self).expose_provenance()
    }
}
