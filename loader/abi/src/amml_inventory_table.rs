use core::ffi::c_void;

use crate::{
    AmmlBlockHandle, AmmlContainerEntry, AmmlHand, AmmlInventoryMove, AmmlItemDraw, AmmlItemSnapshot,
    AmmlRequestOptions, AmmlStatus, AmmlStr,
};

/// Gives the stack in `hand` of the local player `player` in `out`; null for an empty hand. `player` is a local
/// player handle that a client call gave in the same game frame.
pub type InventoryHeldItemFn =
    unsafe extern "C" fn(player: *const c_void, hand: AmmlHand, out: *mut *const c_void) -> AmmlStatus;
/// Gives the selected hotbar slot (0 to 8) of `player` in `out`.
pub type InventorySelectedSlotFn = unsafe extern "C" fn(player: *const c_void, out: *mut u8) -> AmmlStatus;
/// Fills `out` (as far as its `size` covers) with the fields of `stack`, a stack handle `held_item` gave for
/// `player` in the same game frame.
pub type InventoryItemReadFn =
    unsafe extern "C" fn(player: *const c_void, stack: *const c_void, out: *mut AmmlItemSnapshot) -> AmmlStatus;
/// Copies the type name of `stack` (UTF-8, for example `minecraft:iron_pickaxe`) into `buf`, at most `cap`
/// bytes, and gives its full length in `out_len`. A name longer than `cap` is cut; call again with a larger
/// buffer. `NOT_AVAILABLE` when the name cannot be read on this build.
pub type InventoryItemNameFn = unsafe extern "C" fn(
    player: *const c_void,
    stack: *const c_void,
    buf: *mut u8,
    cap: usize,
    out_len: *mut usize,
) -> AmmlStatus;

/// Gives the light emission (0 to 15) of the block `stack` places in `out_emission` and a handle of that block,
/// owned by `module` and bound to the current world, in `out_block`. An item that places no block gives
/// `AmmlBlockHandle::NONE` and emission 0. Version 2.
pub type InventoryItemBlockFn = unsafe extern "C" fn(
    module: AmmlStr,
    player: *const c_void,
    stack: *const c_void,
    out_emission: *mut u8,
    out_block: *mut AmmlBlockHandle,
) -> AmmlStatus;

/// Gives the stack in slot `index` (0 to 35: 0 to 8 the hotbar, 9 to 35 the main inventory) of `player` in `out`; null
/// for an empty slot. `INVALID_ARGUMENT` for an index above 35. The stack handle is lent: it is valid for the
/// functions of this table only while the scope that runs now runs (a `ScreenRender` dispatch, or the game frame of
/// the call outside any event scope), so a handle kept past it is refused. Version 3.
pub type InventorySlotItemFn =
    unsafe extern "C" fn(player: *const c_void, index: u8, out: *mut *const c_void) -> AmmlStatus;

/// Opens the contents of the shulker box or bundle `stack` of `module`: the list of entries in its tag. `out_len` is
/// the number of entries and `out_contents` the contents id for the functions below; the id is 0 (and `out_len` 0)
/// for a stack that holds no contents list. `stack` and `player` are as for `item_read`. Every entry the loader can
/// make a stack of is in the list; an entry of an item the game does not know is left out. The loader keeps a copy of
/// each stack for drawing. At the 128 live draws of `module` the loader keeps the entries of the list and leaves out
/// the contents of the bundles that did not fit (`bundle` 0); the call gives `REFUSED` only when the list itself does
/// not fit. The caller reads the entries it wants and closes the id with `contents_close`. Version 4.
pub type InventoryContentsOpenFn = unsafe extern "C" fn(
    module: AmmlStr,
    player: *const c_void,
    stack: *const c_void,
    out_contents: *mut u64,
    out_len: *mut u32,
) -> AmmlStatus;
/// Writes entry `index` of `contents` of `module` in `out` (as far as its `size` covers). The caller then owns the
/// entry's draw and bundle contents id. Reading an entry twice gives `REFUSED`, and so does an id of another module.
/// `INVALID_ARGUMENT` for an index past the end, `STALE` for an id that was closed or is of a world that was left.
/// Version 4.
pub type InventoryContentsEntryFn =
    unsafe extern "C" fn(module: AmmlStr, contents: u64, index: u32, out: *mut AmmlContainerEntry) -> AmmlStatus;
/// Copies the type name of entry `index` of `contents` of `module` into `buf`, as `InventoryItemNameFn` does. An id of
/// another module gives `REFUSED`. Version 4.
pub type InventoryContentsNameFn = unsafe extern "C" fn(
    module: AmmlStr,
    contents: u64,
    index: u32,
    buf: *mut u8,
    cap: usize,
    out_len: *mut usize,
) -> AmmlStatus;
/// Closes `contents` of `module`: the draws of the entries the caller did not read are freed. Closing an id that is
/// closed already, or an id of another module, does nothing. Version 4.
pub type InventoryContentsCloseFn = unsafe extern "C" fn(module: AmmlStr, contents: u64);
/// Frees `draw` of `module`. Allowed on any thread: the loader frees the game stack at its next UI draw. `STALE` for a
/// draw of an older generation or one freed, `REFUSED` for a draw of another module. Version 4.
pub type InventoryItemDrawReleaseFn = unsafe extern "C" fn(module: AmmlStr, draw: AmmlItemDraw) -> AmmlStatus;

/// Asks for `count` moves between the slots of the local player `player` (area `inventory.request`), as item stack
/// requests the server answers, one request for each move in one packet. `module` is the calling mod. On `OK`
/// `out_id` holds the id of the request; the call sends nothing itself: the loader sends the packets at the next
/// client tick, and `InventoryRequestDone` with the id ends the request. One request runs at a time for the whole
/// loader: a call while another request runs gives `REFUSED`, and so does `hidden_open` while a screen is open.
/// `INVALID_ARGUMENT` for no move, more than 64, a slot above 35, a swap of a slot with itself or a zero count;
/// `NOT_AVAILABLE` for the `PICK` kind; `REFUSED` off the game thread or for a player that is not the local player.
/// Version 5.
pub type InventoryRequestMovesFn = unsafe extern "C" fn(
    module: AmmlStr,
    player: *const c_void,
    moves: *const AmmlInventoryMove,
    count: u32,
    options: AmmlRequestOptions,
    out_id: *mut u64,
) -> AmmlStatus;

/// Asks for the creative pick of the block `block` into the hotbar slot `slot` (0 to 8) of the local player `player`
/// (area `inventory.request` version 2): one item stack request that makes a full stack of the block's creative item
/// and takes it into the slot. A stack already in the slot is destroyed first in the same request (and comes back on
/// the client when the server refuses or does not answer). `module` is the calling mod. As `request_moves`, the call sends
/// nothing itself: the loader predicts the stack on the client and sends the packet at the next client tick,
/// `InventoryRequestDone` with the id in `out_id` ends the request, and one request runs at a time for the whole loader
/// (`REFUSED` while another runs). No hidden inventory is opened. `REFUSED` outside the creative game mode, for a block
/// with no creative entry, for a slot above 8, off the game thread and for a player that is not the local player;
/// `STALE` for a block handle that was taken back. Version 7.
pub type InventoryRequestPickFn = unsafe extern "C" fn(
    module: AmmlStr,
    player: *const c_void,
    block: AmmlBlockHandle,
    slot: u8,
    out_id: *mut u64,
) -> AmmlStatus;

/// A change stamp of `stack` (a stack `player` holds now, or one lent in the running UI scope with a null `player`):
/// 0 for a stack with no user data; else a hash of the address of its user data `CompoundTag` and of every key and
/// value of that tag and of the compounds nested in it, read through the loader's layout mirrors. Two reads of one
/// stack give the same stamp until the stack gets other user data or its tag changes. `NOT_AVAILABLE` on a build whose
/// tag layout the loader cannot read; `REFUSED` off the game thread and for a stack the call may not read.
pub type InventoryItemChangeStampFn =
    unsafe extern "C" fn(player: *const c_void, stack: *const c_void, out: *mut u64) -> AmmlStatus;

/// The inventory functions of `AmmlServices` (area `inventory.read`). The table only grows: a function added
/// later is appended, `size` and `version` grow, and a reader calls a function only when `size` covers it.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct AmmlInventoryTable {
    pub size: u32,
    pub version: u32,
    pub held_item: InventoryHeldItemFn,
    pub selected_slot: InventorySelectedSlotFn,
    pub item_read: InventoryItemReadFn,
    pub item_name: InventoryItemNameFn,
    /// `None` in a v1 table.
    pub item_block: Option<InventoryItemBlockFn>,
    /// `None` in a v1 or v2 table.
    pub slot_item: Option<InventorySlotItemFn>,
    /// `None` in a table before v4.
    pub contents_open: Option<InventoryContentsOpenFn>,
    /// `None` in a table before v4.
    pub contents_entry: Option<InventoryContentsEntryFn>,
    /// `None` in a table before v4.
    pub contents_name: Option<InventoryContentsNameFn>,
    /// `None` in a table before v4.
    pub contents_close: Option<InventoryContentsCloseFn>,
    /// `None` in a table before v4.
    pub item_draw_release: Option<InventoryItemDrawReleaseFn>,
    /// `None` in a table before v5.
    pub request_moves: Option<InventoryRequestMovesFn>,
    /// `None` in a table before v6.
    pub item_change_stamp: Option<InventoryItemChangeStampFn>,
    /// `None` in a table before v7.
    pub request_pick: Option<InventoryRequestPickFn>,
}

impl AmmlInventoryTable {
    /// `size` of a v1 inventory table. A literal, so it keeps its meaning when a later version adds fields.
    pub const SIZE_V1: u32 = 40;
    /// `size` of a v2 inventory table: v1 plus `item_block`; `held_item` also serves `AmmlHand::OFF`.
    pub const SIZE_V2: u32 = 48;
    /// `size` of a v3 inventory table: v2 plus `slot_item`.
    pub const SIZE_V3: u32 = 56;
    /// `size` of a v4 inventory table: v3 plus the contents functions and `item_draw_release`.
    pub const SIZE_V4: u32 = 96;
    /// `size` of a v5 inventory table: v4 plus `request_moves`.
    pub const SIZE_V5: u32 = 104;
    /// `size` of a v6 inventory table: v5 plus `item_change_stamp`.
    pub const SIZE_V6: u32 = 112;
    /// `size` of a v7 inventory table: v6 plus `request_pick`.
    pub const SIZE_V7: u32 = 120;

    /// A copy of the table at `table`, read only as far as its own `size` covers; the later fields of a
    /// shorter table are `None`. `None` for a null table or one shorter than v1.
    ///
    /// # Safety
    /// A non-null `table` points to a table readable for its own `size` bytes, at least 4.
    #[allow(unsafe_code)]
    pub unsafe fn read(table: *const AmmlInventoryTable) -> Option<AmmlInventoryTable> {
        if table.is_null() {
            return None;
        }
        // SAFETY: the caller guarantees the leading `size` field is readable.
        let size = unsafe { table.cast::<u32>().read_unaligned() };
        if size < Self::SIZE_V1 {
            return None;
        }
        let covered = (size as usize).min(core::mem::size_of::<AmmlInventoryTable>());
        let mut copy = core::mem::MaybeUninit::<AmmlInventoryTable>::zeroed();
        // SAFETY: the caller guarantees `covered` bytes are readable at `table`; `copy` has room for the whole
        // struct and the two do not overlap.
        unsafe { core::ptr::copy_nonoverlapping(table.cast::<u8>(), copy.as_mut_ptr().cast::<u8>(), covered) };
        // SAFETY: the copied bytes cover every v1 field, written from a valid table; every later field is an
        // `Option` of a function pointer, for which all zero bytes are `None`.
        Some(unsafe { copy.assume_init() })
    }
}

const _: () = assert!(core::mem::size_of::<AmmlInventoryTable>() == AmmlInventoryTable::SIZE_V7 as usize);
