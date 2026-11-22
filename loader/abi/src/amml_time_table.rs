use crate::{AmmlHoldRequest, AmmlStatus, AmmlStr, AmmlTimeHold, AmmlTimeState};

/// The mod `module` takes the game clock with `request` and gets its handle in `out`. While held, the game's four
/// client timers (the sim and real timers of `MinecraftGame` and of the client) hand out the ticks and the alpha of the
/// stepped game time, not of the wall clock: the game time moves only by `step_us`. One mod holds the clock: `REFUSED`
/// while any hold exists, and off the game thread; `NOT_READY` before a world is joined; `NOT_AVAILABLE` when the area
/// is not offered; `INVALID_ARGUMENT` for a null `request` or `out`. Area `time.control`.
pub type TimeHoldFn =
    unsafe extern "C" fn(module: AmmlStr, request: *const AmmlHoldRequest, out: *mut AmmlTimeHold) -> AmmlStatus;

/// Advances the held clock by exactly `micros` microseconds of game time at the next advance of the timers; steps
/// queued in one frame reach the timers together. Each call, 0 included, feeds the watchdog. `STALE` for a hold that
/// ended; `INVALID_ARGUMENT` for a negative `micros`; `REFUSED` off the game thread. Area `time.control`.
pub type TimeStepUsFn = unsafe extern "C" fn(hold: AmmlTimeHold, micros: i64) -> AmmlStatus;

/// Ends the hold: the timers run on the wall clock again from the next frame. Off the game thread the release is
/// queued for the next frame and the call gives `OK` without checking the hold (a stale hold is dropped from the queue
/// then). On the game thread, `STALE` for a hold that ended. Area `time.control`.
pub type TimeReleaseFn = unsafe extern "C" fn(hold: AmmlTimeHold) -> AmmlStatus;

/// Gives in `out` the state of the game clock. Any thread. `NOT_AVAILABLE` when the area is not offered;
/// `INVALID_ARGUMENT` for a null `out`. Area `time.control`.
pub type TimeStateFn = unsafe extern "C" fn(out: *mut AmmlTimeState) -> AmmlStatus;

/// The game clock functions of `AmmlServices` (area `time.control`). The table only grows: a function added later is
/// appended, `size` and `version` grow, and a reader calls a function only when `size` covers it.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct AmmlTimeTable {
    pub size: u32,
    pub version: u32,
    pub hold: TimeHoldFn,
    pub step_us: TimeStepUsFn,
    pub release: TimeReleaseFn,
    pub state: TimeStateFn,
}

impl AmmlTimeTable {
    /// `size` of a v1 time table. A literal, so it keeps its meaning when a later version adds fields.
    pub const SIZE_V1: u32 = 40;

    /// A copy of the table at `table`, read as far as its own `size` covers. A null table, or one shorter than v1,
    /// gives `None`.
    ///
    /// # Safety
    /// A non-null `table` points to a table readable for its own `size` bytes, at least 4.
    #[allow(unsafe_code)]
    pub unsafe fn read(table: *const AmmlTimeTable) -> Option<AmmlTimeTable> {
        if table.is_null() {
            return None;
        }
        // SAFETY: the caller guarantees the leading `size` field is readable.
        let size = unsafe { table.cast::<u32>().read_unaligned() };
        if size < Self::SIZE_V1 {
            return None;
        }
        let covered = (size as usize).min(core::mem::size_of::<AmmlTimeTable>());
        let mut copy = core::mem::MaybeUninit::<AmmlTimeTable>::zeroed();
        // SAFETY: the caller guarantees `covered` bytes are readable at `table`; `copy` has room for the whole struct
        // and the two do not overlap.
        unsafe { core::ptr::copy_nonoverlapping(table.cast::<u8>(), copy.as_mut_ptr().cast::<u8>(), covered) };
        // SAFETY: `size` covers every v1 field, so the copy holds a valid table.
        Some(unsafe { copy.assume_init() })
    }
}

const _: () = assert!(core::mem::size_of::<AmmlTimeTable>() == AmmlTimeTable::SIZE_V1 as usize);
