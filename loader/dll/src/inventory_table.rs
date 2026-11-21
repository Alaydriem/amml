use std::ffi::c_void;

use amml_abi::{
    AmmlBlockHandle, AmmlContainerEntry, AmmlHand, AmmlInventoryMove, AmmlInventoryTable, AmmlItemDraw,
    AmmlItemSnapshot, AmmlRequestOptions, AmmlStatus, AmmlStr,
};

use crate::{BlockHandles, Boundary, InventoryAdapter, InventoryLayout, ItemDrawAdapter, LoaderState, RequestAdapter};

const TABLE_VERSION: u32 = 6;

pub(crate) static INVENTORY: AmmlInventoryTable = AmmlInventoryTable {
    size: AmmlInventoryTable::SIZE_V6,
    version: TABLE_VERSION,
    held_item: InventoryTable::held_item,
    selected_slot: InventoryTable::selected_slot,
    item_read: InventoryTable::item_read,
    item_name: InventoryTable::item_name,
    item_block: Some(InventoryTable::item_block),
    slot_item: Some(InventoryTable::slot_item),
    contents_open: Some(InventoryTable::contents_open),
    contents_entry: Some(InventoryTable::contents_entry),
    contents_name: Some(InventoryTable::contents_name),
    contents_close: Some(InventoryTable::contents_close),
    item_draw_release: Some(InventoryTable::item_draw_release),
    request_moves: Some(InventoryTable::request_moves),
    item_change_stamp: Some(InventoryTable::item_change_stamp),
};

/// The inventory table behind `AmmlServices::inventory`, and its `extern "C"` functions. Each function checks its
/// pointers and delegates to `InventoryAdapter` inside the boundary, so a panic never crosses it. With no inventory
/// adapter installed (a host without one) every call gives `NOT_AVAILABLE`.
pub struct InventoryTable;

impl InventoryTable {
    // The most moves the table reads from the caller: a count above it names a request the book refuses anyway.
    const MAX_MOVES_READ: usize = 1024;
}

impl InventoryTable {
    unsafe extern "C" fn held_item(player: *const c_void, hand: AmmlHand, out: *mut *const c_void) -> AmmlStatus {
        Boundary::guard(|| {
            if out.is_null() {
                return AmmlStatus::INVALID_ARGUMENT;
            }
            match Self::adapter().and_then(|adapter| adapter.held_item(player, hand)) {
                Ok(stack) => {
                    // SAFETY: `out` is non-null and the caller promises it is writable.
                    unsafe { out.write(std::ptr::with_exposed_provenance(stack)) };
                    AmmlStatus::OK
                }
                Err(status) => status,
            }
        })
    }

    unsafe extern "C" fn selected_slot(player: *const c_void, out: *mut u8) -> AmmlStatus {
        Boundary::guard(|| {
            if out.is_null() {
                return AmmlStatus::INVALID_ARGUMENT;
            }
            match Self::adapter().and_then(|adapter| adapter.selected_slot(player)) {
                Ok(slot) => {
                    // SAFETY: `out` is non-null and the caller promises it is writable.
                    unsafe { out.write(slot) };
                    AmmlStatus::OK
                }
                Err(status) => status,
            }
        })
    }

    unsafe extern "C" fn slot_item(player: *const c_void, index: u8, out: *mut *const c_void) -> AmmlStatus {
        Boundary::guard(|| {
            if out.is_null() || usize::from(index) >= InventoryLayout::SLOTS {
                return AmmlStatus::INVALID_ARGUMENT;
            }
            match Self::adapter().and_then(|adapter| adapter.slot_item(player, index)) {
                Ok(stack) => {
                    // SAFETY: `out` is non-null and the caller promises it is writable.
                    unsafe { out.write(std::ptr::with_exposed_provenance(stack)) };
                    AmmlStatus::OK
                }
                Err(status) => status,
            }
        })
    }

    unsafe extern "C" fn item_read(
        player: *const c_void,
        stack: *const c_void,
        out: *mut AmmlItemSnapshot,
    ) -> AmmlStatus {
        Boundary::guard(|| {
            if out.is_null() {
                return AmmlStatus::INVALID_ARGUMENT;
            }
            // SAFETY: `out` is non-null and the caller promises its first `size` bytes are readable and writable.
            let size = unsafe { (&raw const (*out).size).read() };
            if size < AmmlItemSnapshot::SIZE_V1 {
                return AmmlStatus::INVALID_ARGUMENT;
            }
            match Self::adapter().and_then(|adapter| adapter.item_read(player, stack)) {
                Ok(snapshot) => {
                    // SAFETY: the caller's `size` covers the whole v1 struct, which is all the loader writes.
                    unsafe { out.write(snapshot) };
                    AmmlStatus::OK
                }
                Err(status) => status,
            }
        })
    }

    unsafe extern "C" fn item_change_stamp(player: *const c_void, stack: *const c_void, out: *mut u64) -> AmmlStatus {
        Boundary::guard(|| {
            if out.is_null() {
                return AmmlStatus::INVALID_ARGUMENT;
            }
            match Self::adapter().and_then(|adapter| adapter.item_change_stamp(player, stack)) {
                Ok(stamp) => {
                    // SAFETY: `out` is non-null and the caller promises it is writable for a u64.
                    unsafe { out.write(stamp) };
                    AmmlStatus::OK
                }
                Err(status) => status,
            }
        })
    }

    unsafe extern "C" fn item_name(
        player: *const c_void,
        stack: *const c_void,
        buf: *mut u8,
        cap: usize,
        out_len: *mut usize,
    ) -> AmmlStatus {
        Boundary::guard(|| {
            if out_len.is_null() || (buf.is_null() && cap != 0) {
                return AmmlStatus::INVALID_ARGUMENT;
            }
            match Self::adapter().and_then(|adapter| adapter.item_name(player, stack)) {
                Ok(name) => {
                    let copied = name.len().min(cap);
                    // SAFETY: `buf` is writable for `cap` bytes (the caller's promise) and `copied <= cap`; `out_len`
                    // is non-null and writable.
                    unsafe {
                        if copied > 0 {
                            std::ptr::copy_nonoverlapping(name.as_ptr(), buf, copied);
                        }
                        out_len.write(name.len());
                    }
                    AmmlStatus::OK
                }
                Err(status) => status,
            }
        })
    }

    unsafe extern "C" fn item_block(
        module: AmmlStr,
        player: *const c_void,
        stack: *const c_void,
        out_emission: *mut u8,
        out_block: *mut AmmlBlockHandle,
    ) -> AmmlStatus {
        Boundary::guard(|| {
            let Some(state) = LoaderState::get() else {
                return AmmlStatus::FAILED;
            };
            if out_emission.is_null() || out_block.is_null() {
                return AmmlStatus::INVALID_ARGUMENT;
            }
            // SAFETY: the caller promises `module` is valid for the call.
            let module = match unsafe { module.as_str() } {
                Ok(module) => module,
                Err(status) => return status,
            };
            let owner = match state.live_mod(module) {
                Ok(owner) => owner,
                Err(status) => return status,
            };
            let (address, emission) = match Self::adapter().and_then(|adapter| adapter.item_block(player, stack)) {
                Ok(found) => found,
                Err(status) => return status,
            };
            let handle = if address == 0 {
                AmmlBlockHandle::NONE
            } else {
                match BlockHandles::handle(state, owner, address) {
                    Ok(handle) => handle,
                    Err(status) => return status,
                }
            };
            // SAFETY: both outputs are non-null and the caller promises they are writable.
            unsafe {
                out_emission.write(emission);
                out_block.write(handle);
            }
            AmmlStatus::OK
        })
    }

    unsafe extern "C" fn contents_open(
        module: AmmlStr,
        player: *const c_void,
        stack: *const c_void,
        out_contents: *mut u64,
        out_len: *mut u32,
    ) -> AmmlStatus {
        Boundary::guard(|| {
            if out_contents.is_null() || out_len.is_null() {
                return AmmlStatus::INVALID_ARGUMENT;
            }
            // SAFETY: the caller promises `module` is valid for the call.
            let owner = match unsafe { ItemDrawAdapter::live_owner(module) } {
                Ok(owner) => owner,
                Err(status) => return status,
            };
            match ItemDrawAdapter::with(Err(AmmlStatus::NOT_AVAILABLE), |adapter| adapter.open(owner, player, stack)) {
                Ok((contents, len)) => {
                    // SAFETY: both outputs are non-null and the caller promises they are writable.
                    unsafe {
                        out_contents.write(contents);
                        out_len.write(len);
                    }
                    AmmlStatus::OK
                }
                Err(status) => status,
            }
        })
    }

    unsafe extern "C" fn contents_entry(
        module: AmmlStr,
        contents: u64,
        index: u32,
        out: *mut AmmlContainerEntry,
    ) -> AmmlStatus {
        Boundary::guard(|| {
            if out.is_null() {
                return AmmlStatus::INVALID_ARGUMENT;
            }
            // SAFETY: the caller promises `module` is valid for the call.
            let owner = match unsafe { ItemDrawAdapter::live_owner(module) } {
                Ok(owner) => owner,
                Err(status) => return status,
            };
            // SAFETY: `out` is non-null and the caller promises its first `size` bytes are readable and writable.
            let size = unsafe { (&raw const (*out).size).read() };
            if size < AmmlContainerEntry::SIZE_V1 {
                return AmmlStatus::INVALID_ARGUMENT;
            }
            match ItemDrawAdapter::with(Err(AmmlStatus::NOT_AVAILABLE), |adapter| adapter.entry(owner, contents, index))
            {
                Ok(entry) => {
                    // SAFETY: the size of the caller covers the whole v1 struct, which is all the loader writes.
                    unsafe { out.write(entry) };
                    AmmlStatus::OK
                }
                Err(status) => status,
            }
        })
    }

    unsafe extern "C" fn contents_name(
        module: AmmlStr,
        contents: u64,
        index: u32,
        buf: *mut u8,
        cap: usize,
        out_len: *mut usize,
    ) -> AmmlStatus {
        Boundary::guard(|| {
            if out_len.is_null() || (buf.is_null() && cap != 0) {
                return AmmlStatus::INVALID_ARGUMENT;
            }
            // SAFETY: the caller promises `module` is valid for the call.
            let owner = match unsafe { ItemDrawAdapter::live_owner(module) } {
                Ok(owner) => owner,
                Err(status) => return status,
            };
            match ItemDrawAdapter::with(Err(AmmlStatus::NOT_AVAILABLE), |adapter| {
                adapter.entry_name(owner, contents, index)
            }) {
                Ok(name) => {
                    let copied = name.len().min(cap);
                    // SAFETY: `buf` is writable for `cap` bytes (the promise of the caller) and `copied <= cap`;
                    // `out_len` is non-null and writable.
                    unsafe {
                        if copied > 0 {
                            std::ptr::copy_nonoverlapping(name.as_ptr(), buf, copied);
                        }
                        out_len.write(name.len());
                    }
                    AmmlStatus::OK
                }
                Err(status) => status,
            }
        })
    }

    unsafe extern "C" fn contents_close(module: AmmlStr, contents: u64) {
        Boundary::run((), || {
            // SAFETY: the caller promises `module` is valid for the call.
            let Ok(owner) = (unsafe { ItemDrawAdapter::live_owner(module) }) else { return };
            ItemDrawAdapter::with((), |adapter| adapter.close(owner, contents));
        });
    }

    unsafe extern "C" fn item_draw_release(module: AmmlStr, draw: AmmlItemDraw) -> AmmlStatus {
        Boundary::guard(|| {
            // SAFETY: the caller promises `module` is valid for the call.
            let owner = match unsafe { ItemDrawAdapter::live_owner(module) } {
                Ok(owner) => owner,
                Err(status) => return status,
            };
            match ItemDrawAdapter::with(Err(AmmlStatus::NOT_AVAILABLE), |adapter| adapter.release(owner, draw)) {
                Ok(()) => AmmlStatus::OK,
                Err(status) => status,
            }
        })
    }

    unsafe extern "C" fn request_moves(
        module: AmmlStr,
        player: *const c_void,
        moves: *const AmmlInventoryMove,
        count: u32,
        options: AmmlRequestOptions,
        out_id: *mut u64,
    ) -> AmmlStatus {
        Boundary::guard(|| {
            if out_id.is_null() || (moves.is_null() && count != 0) {
                return AmmlStatus::INVALID_ARGUMENT;
            }
            // SAFETY: the caller promises `module` is valid for the call.
            let owner = match unsafe { ItemDrawAdapter::live_owner(module) } {
                Ok(owner) => owner,
                Err(status) => return status,
            };
            // The adapter bounds the count; a count past `u32` bytes of moves is refused here before the slice is made.
            if count as usize > Self::MAX_MOVES_READ {
                return AmmlStatus::INVALID_ARGUMENT;
            }
            let moves: &[AmmlInventoryMove] = if count == 0 {
                &[]
            } else {
                // SAFETY: the caller promises `moves` holds `count` moves; the count is at most `MAX_MOVES_READ`.
                unsafe { std::slice::from_raw_parts(moves, count as usize) }
            };
            match RequestAdapter::with(Err(AmmlStatus::NOT_AVAILABLE), |adapter| {
                adapter.request_moves(owner, player, moves, options)
            }) {
                Ok(id) => {
                    // SAFETY: `out_id` is non-null and the caller promises it is writable.
                    unsafe { out_id.write(id) };
                    AmmlStatus::OK
                }
                Err(status) => status,
            }
        })
    }

    fn adapter() -> Result<&'static InventoryAdapter, AmmlStatus> {
        InventoryAdapter::get().ok_or(AmmlStatus::NOT_AVAILABLE)
    }
}
