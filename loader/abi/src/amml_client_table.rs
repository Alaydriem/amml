use core::ffi::c_void;

use crate::{AmmlClientFlag, AmmlMenu, AmmlScreen, AmmlStartWorldOutcome, AmmlStatus, AmmlStr};

/// Gives the primary client in `out`. Game thread only.
pub type ClientCurrentFn = unsafe extern "C" fn(out: *mut *const c_void) -> AmmlStatus;
/// Gives the local player of `client` in `out`; a null `out` value means no player.
pub type ClientLocalPlayerFn = unsafe extern "C" fn(client: *const c_void, out: *mut *const c_void) -> AmmlStatus;
/// Gives a client flag as 0 or 1 in `out`.
pub type ClientFlagFn = unsafe extern "C" fn(client: *const c_void, flag: AmmlClientFlag, out: *mut u8) -> AmmlStatus;
/// Gives the screen on top of `client` in `out`.
pub type ClientTopScreenFn = unsafe extern "C" fn(client: *const c_void, out: *mut AmmlScreen) -> AmmlStatus;
/// Sets a client flag for `module`. Only `MOUSE_CAPTURED` and `HIDE_HUD` may be set; any other flag
/// gives `INVALID_ARGUMENT`.
pub type ClientSetFlagFn =
    unsafe extern "C" fn(module: AmmlStr, client: *const c_void, flag: AmmlClientFlag, value: u8) -> AmmlStatus;
/// Opens the pause screen of `client`.
pub type ClientOpenPauseFn = unsafe extern "C" fn(client: *const c_void) -> AmmlStatus;

/// Asks the game to leave the joined world of `client` (area `client.leave_world`).
pub type ClientLeaveWorldFn = unsafe extern "C" fn(client: *const c_void) -> AmmlStatus;
/// Gives 1 in `out` when the game's primary client is ready for a join (area `client.can_join`).
pub type ClientCanJoinFn = unsafe extern "C" fn(client: *const c_void, out: *mut u8) -> AmmlStatus;
/// Starts the local world `level_id` (its folder name); the game's answer in `out` (area `client.start_local_world`).
pub type ClientStartLocalWorldFn =
    unsafe extern "C" fn(client: *const c_void, level_id: AmmlStr, out: *mut AmmlStartWorldOutcome) -> AmmlStatus;
/// Joins the server `host`:`port` for `module`, shown as `name` (the host when `name` is empty); the join id in `out`
/// (area `client.join_server`). The answer comes as `JoinServerResult`.
pub type ClientJoinServerFn = unsafe extern "C" fn(
    module: AmmlStr,
    client: *const c_void,
    host: AmmlStr,
    port: u16,
    name: AmmlStr,
    out: *mut u64,
) -> AmmlStatus;
/// Opens the menu page `menu` (area `client.open_menu`).
pub type ClientOpenMenuFn = unsafe extern "C" fn(client: *const c_void, menu: AmmlMenu) -> AmmlStatus;
/// Sends `text` as a chat command, with its slash (area `client.send_command`).
pub type ClientSendCommandFn = unsafe extern "C" fn(client: *const c_void, text: AmmlStr) -> AmmlStatus;
/// Selects hotbar slot `slot` (0 to 8) through the game's HUD (area `hud.select_hotbar`).
pub type ClientSelectHotbarSlotFn = unsafe extern "C" fn(client: *const c_void, slot: u8) -> AmmlStatus;
/// Runs the game's any-input handler of `client` once, as the game does for each input event, with no input event
/// (area `client.activity`). `REFUSED` when the game is not ready for it.
pub type ClientNoteActivityFn = unsafe extern "C" fn(client: *const c_void) -> AmmlStatus;

/// The client functions of `AmmlServices`. The table only grows: a function added later is appended as an `Option`,
/// `size` and `version` grow, and `read` copies only what `size` covers (a later function of an older table is
/// `None`).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct AmmlClientTable {
    pub size: u32,
    pub version: u32,
    pub current: ClientCurrentFn,
    pub local_player: ClientLocalPlayerFn,
    pub flag: ClientFlagFn,
    pub top_screen: ClientTopScreenFn,
    pub set_flag: ClientSetFlagFn,
    pub open_pause_screen: ClientOpenPauseFn,
    pub leave_world: Option<ClientLeaveWorldFn>,
    pub can_join: Option<ClientCanJoinFn>,
    pub start_local_world: Option<ClientStartLocalWorldFn>,
    pub join_server: Option<ClientJoinServerFn>,
    pub open_menu: Option<ClientOpenMenuFn>,
    pub send_command: Option<ClientSendCommandFn>,
    pub select_hotbar_slot: Option<ClientSelectHotbarSlotFn>,
    /// From v3.
    pub note_activity: Option<ClientNoteActivityFn>,
}

impl AmmlClientTable {
    /// `size` of a v1 client table. A literal, so it keeps its meaning when a later version adds fields.
    pub const SIZE_V1: u32 = 56;
    /// `size` of a v2 client table: v1 plus the session actions and the hotbar select.
    pub const SIZE_V2: u32 = 112;
    /// `size` of a v3 client table: v2 plus `note_activity`.
    pub const SIZE_V3: u32 = 120;

    /// A copy of the table at `table`, read as far as its own `size` covers; a later field is `None`. A null
    /// table, or one shorter than v1, gives `None`.
    ///
    /// # Safety
    /// A non-null `table` points to a table readable for its own `size` bytes, at least 4.
    #[allow(unsafe_code)]
    pub unsafe fn read(table: *const AmmlClientTable) -> Option<AmmlClientTable> {
        if table.is_null() {
            return None;
        }
        // SAFETY: the caller guarantees the leading `size` field is readable.
        let size = unsafe { table.cast::<u32>().read_unaligned() };
        if size < Self::SIZE_V1 {
            return None;
        }
        let covered = (size as usize).min(core::mem::size_of::<AmmlClientTable>());
        let mut copy = core::mem::MaybeUninit::<AmmlClientTable>::zeroed();
        // SAFETY: the caller guarantees `covered` bytes are readable at `table`; `copy` has room for the whole
        // struct and the two do not overlap.
        unsafe { core::ptr::copy_nonoverlapping(table.cast::<u8>(), copy.as_mut_ptr().cast::<u8>(), covered) };
        // SAFETY: the copied bytes cover every v1 field, written from a valid table; every later field is an
        // `Option` of a function pointer, for which all zero bytes are `None`.
        Some(unsafe { copy.assume_init() })
    }
}

const _: () = assert!(core::mem::size_of::<AmmlClientTable>() == AmmlClientTable::SIZE_V3 as usize);
