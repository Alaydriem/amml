use core::ffi::c_void;

use crate::AmmlThread;

/// What the loader passes to a listener beside the event.
///
/// `client` is an opaque handle to the game client, or null when there is none.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct AmmlEventContext {
    pub size: u32,
    pub version: u32,
    pub thread: AmmlThread,
    pub game_alive: u8,
    pub _reserved: [u8; 6],
    pub epoch: u64,
    pub client: *const c_void,
}

impl AmmlEventContext {
    /// `size` of a v1 event context. A literal, so it keeps its meaning when a later version adds
    /// fields: a reader accepts any `size` at or above it and reads a later field only when `size`
    /// covers it.
    pub const SIZE_V1: u32 = 32;
}

const _: () = assert!(core::mem::size_of::<AmmlEventContext>() == AmmlEventContext::SIZE_V1 as usize);
