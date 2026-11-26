use crate::AmmlConnectionKind;

/// The facts of the joined world that `client.world_info` gives in one call. Each fact has a flag in `flags`; a fact
/// whose flag is clear was not readable, and its field is zero. The texts (folder, level name, host) are read with
/// `world_text`; their flags say that the text can be read.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AmmlWorldInfo {
    pub flags: u32,
    pub connection_kind: AmmlConnectionKind,
    pub _reserved: u8,
    pub port: u16,
    pub seed: u64,
}

impl AmmlWorldInfo {
    /// `seed` holds the world seed.
    pub const HAS_SEED: u32 = 1;
    /// `world_text` gives the storage folder.
    pub const HAS_STORAGE: u32 = 2;
    /// `connection_kind` and `port` are valid, and `world_text` gives the host.
    pub const HAS_CONNECTION: u32 = 4;
    /// `REALM` says whether the world is a realm.
    pub const HAS_REALM: u32 = 8;
    /// The world is a realm (valid with `HAS_REALM`).
    pub const REALM: u32 = 16;
    /// `COMMANDS` says whether commands are enabled.
    pub const HAS_COMMANDS: u32 = 32;
    /// Commands are enabled (valid with `HAS_COMMANDS`).
    pub const COMMANDS: u32 = 64;
    /// `world_text` gives the level name.
    pub const HAS_LEVEL_NAME: u32 = 128;
}

const _: () = assert!(core::mem::size_of::<AmmlWorldInfo>() == 16);
