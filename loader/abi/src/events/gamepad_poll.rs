use crate::{AmmlEvent, AmmlEventHeader};

/// The game polls the gamepad. A listener may cancel it.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct GamepadPoll {
    pub header: AmmlEventHeader,
}

impl AmmlEvent for GamepadPoll {
    const ID: &'static str = "input.gamepad_poll";
    const CANCELLABLE: bool = true;
}

const _: () = assert!(core::mem::size_of::<GamepadPoll>() == 16);
