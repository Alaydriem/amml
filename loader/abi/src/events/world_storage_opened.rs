use crate::{AmmlEvent, AmmlEventHeader, AmmlStr};

/// The game opened the storage of a world: raised After the `DBStorage` constructor. `path` is the folder of the
/// storage (the derived `DBStorage.mFullPath`), valid only inside the listener call.
///
/// The constructor runs on a storage thread. The loader raises the event at once when that is the game thread, and
/// at the next `FrameUpdate` (After) otherwise, so listeners run on the game thread in both cases.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct WorldStorageOpened {
    pub header: AmmlEventHeader,
    pub path: AmmlStr,
}

impl AmmlEvent for WorldStorageOpened {
    const ID: &'static str = "world.storage_opened";
}

const _: () = assert!(core::mem::size_of::<WorldStorageOpened>() == 32);
