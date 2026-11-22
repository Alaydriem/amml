use crate::{AmmlEvent, AmmlEventHeader};

/// The storage of a world that `WorldStorageOpened` named finished its shutdown: raised After
/// `DBStorage::checkShutdownDone` first reports done for it. It comes once per opened storage, on the game thread,
/// at once or at the next `FrameUpdate` (After) when the call ran on another thread.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct WorldStorageClosed {
    pub header: AmmlEventHeader,
}

impl AmmlEvent for WorldStorageClosed {
    const ID: &'static str = "world.storage_closed";
}

const _: () = assert!(core::mem::size_of::<WorldStorageClosed>() == 16);
