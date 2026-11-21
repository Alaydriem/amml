use core::ffi::c_void;

use crate::{AmmlBand, AmmlEventContext, AmmlEventHeader, AmmlStatus, AmmlStr};

/// Receives one event. `event` points to the event struct, which starts with an `AmmlEventHeader`.
pub type EventCallbackFn = unsafe extern "C" fn(
    event: *mut AmmlEventHeader,
    context: *const AmmlEventContext,
    user: *mut c_void,
) -> AmmlStatus;
/// Registers `callback` for `event` at `version`; `handle` receives the id for `UnlistenFn`.
///
/// A non-zero `required` makes the call fail when the loader cannot serve the event. A null
/// `callback` gives `INVALID_ARGUMENT`.
pub type ListenFn = unsafe extern "C" fn(
    event: AmmlStr,
    version: u32,
    band: AmmlBand,
    required: u8,
    callback: Option<EventCallbackFn>,
    user: *mut c_void,
    handle: *mut u64,
) -> AmmlStatus;
/// Removes the listener behind `handle`.
pub type UnlistenFn = unsafe extern "C" fn(handle: u64) -> AmmlStatus;

/// The event functions of `AmmlServices`.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct AmmlEventsTable {
    pub size: u32,
    pub version: u32,
    pub listen: ListenFn,
    pub unlisten: UnlistenFn,
}

impl AmmlEventsTable {
    /// `size` of a v1 events table. A literal, so it keeps its meaning when a later version adds
    /// fields: a reader accepts any `size` at or above it and reads a later field only when `size`
    /// covers it.
    pub const SIZE_V1: u32 = 24;
}

const _: () = assert!(core::mem::size_of::<AmmlEventsTable>() == AmmlEventsTable::SIZE_V1 as usize);
