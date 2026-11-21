use crate::{AmmlEvent, AmmlEventHeader, AmmlModifiers};

/// A key went down or up. A listener may cancel it.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct KeyInput {
    pub header: AmmlEventHeader,
    pub key: u16,
    pub down: u8,
    pub repeat: u8,
    pub modifiers: AmmlModifiers,
}

impl AmmlEvent for KeyInput {
    const ID: &'static str = "input.key";
    const CANCELLABLE: bool = true;
}

const _: () = assert!(core::mem::size_of::<KeyInput>() == 24);
