use crate::{AmmlDirKind, AmmlLogLevel, AmmlStatus, AmmlStr};

/// Writes one log line for `module` with structured fields: `count` entries in `keys` and `values`.
pub type LogFieldsFn = unsafe extern "C" fn(
    module: AmmlStr,
    level: AmmlLogLevel,
    message: AmmlStr,
    keys: *const AmmlStr,
    values: *const AmmlStr,
    count: usize,
);
/// Gives a folder of `module` in `out`. The loader owns the bytes; they stay valid for the life of
/// the process.
pub type DirFn = unsafe extern "C" fn(module: AmmlStr, kind: AmmlDirKind, out: *mut AmmlStr) -> AmmlStatus;
/// Gives `OK` when the loader can serve `event` at `version`, otherwise `NOT_AVAILABLE`.
pub type AvailableFn = unsafe extern "C" fn(event: AmmlStr, version: u32) -> AmmlStatus;

/// Gives 1 while the game runs and may be called (from the first `FrameUpdate` until shutdown), else 0.
/// A mod reads it in `amml_mod_disable` to know whether it may still call game actions.
pub type GameAliveFn = unsafe extern "C" fn() -> u8;

/// The core functions of `AmmlServices`.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct AmmlCoreTable {
    pub size: u32,
    pub version: u32,
    pub log: LogFieldsFn,
    pub dir: DirFn,
    pub available: AvailableFn,
    pub game_alive: GameAliveFn,
}

impl AmmlCoreTable {
    /// `size` of a v1 core table. A literal, so it keeps its meaning when a later version adds fields.
    pub const SIZE_V1: u32 = 40;
}

const _: () = assert!(core::mem::size_of::<AmmlCoreTable>() == AmmlCoreTable::SIZE_V1 as usize);
