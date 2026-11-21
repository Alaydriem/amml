use std::ffi::c_void;
use std::marker::PhantomData;

use amml_abi::{AmmlInventoryMove, AmmlInventoryTable, AmmlRequestOptions, AmmlStr};

use crate::{ApiError, InventoryMove, ItemSnapshot, ItemView, RequestId, RequestOptions};

/// The inventory of the local player, as a frame handle from `LocalPlayer::inventory`. It lives no longer than the
/// event that gave its client.
pub struct Inventory<'f> {
    table: AmmlInventoryTable,
    player: *const c_void,
    module: &'static str,
    _event: PhantomData<&'f ()>,
}

impl<'f> Inventory<'f> {
    /// The slots of the inventory: 0 to 8 the hotbar, 9 to 35 the main inventory.
    pub const SLOTS: u8 = 36;

    pub(crate) fn new(table: AmmlInventoryTable, player: *const c_void, module: &'static str) -> Self {
        Self { table, player, module, _event: PhantomData }
    }

    /// The selected hotbar slot, 0 to 8.
    pub fn selected_slot(&self) -> Option<u8> {
        let mut out = 0u8;
        // SAFETY: the loader checks the player handle before it reads game memory, and `out` is a live local.
        #[allow(unsafe_code)]
        let status = unsafe { (self.table.selected_slot)(self.player, &mut out) };
        status.is_ok().then_some(out)
    }

    /// The stack in slot `index` (0 to 35); `None` for an empty slot, an index above 35, a refused read or a loader
    /// before `Area::INVENTORY_READ.at(3)`. The view is valid only in the event that gave this inventory: a stack
    /// the loader lent in one frame is refused in the next.
    pub fn slot(&self, index: u8) -> Option<ItemView<'f>> {
        let stack = self.slot_stack(index)??;
        Some(ItemView::new(self.table, self.player, stack, self.module))
    }

    /// All 36 slots in one call, each as a snapshot; `None` for an empty slot. `None` as a whole when the loader
    /// refuses a read or serves no slots (a loader before `Area::INVENTORY_READ.at(3)`).
    pub fn slots(&self) -> Option<[Option<ItemSnapshot>; 36]> {
        let mut slots = Vec::with_capacity(usize::from(Self::SLOTS));
        for index in 0..Self::SLOTS {
            let stack = self.slot_stack(index)?;
            slots.push(stack.map(|stack| ItemView::new(self.table, self.player, stack, self.module).snapshot()));
        }
        slots.try_into().ok()
    }

    /// The most moves of one request.
    pub const MAX_MOVES: usize = 64;

    /// Asks for `moves` between the slots of the player, as item stack requests the server answers: one request
    /// for each move, in one packet. The call sends nothing itself; the loader sends the packets at the next client
    /// tick, and `InventoryRequestDone` with the returned id ends the request (never inside this call).
    ///
    /// One request runs at a time for the whole loader: a call while another request runs is `Refused`, and so is
    /// `RequestOptions::hidden_open` while a screen is open. `Loader(INVALID_ARGUMENT)` names a request
    /// with no move, with more than `MAX_MOVES`, with a slot above 35, a swap of a slot with itself or a zero count.
    /// `NotAvailable` without area `Area::INVENTORY_REQUEST` or on a loader before inventory table v5. Game thread.
    pub fn request_moves(&self, moves: &[InventoryMove], options: RequestOptions) -> Result<RequestId, ApiError> {
        let request = self.table.request_moves.ok_or(ApiError::NotAvailable)?;
        let moves: Vec<AmmlInventoryMove> = moves.iter().copied().map(AmmlInventoryMove::from).collect();
        let count = u32::try_from(moves.len()).map_err(|_| ApiError::Loader(amml_abi::AmmlStatus::INVALID_ARGUMENT))?;
        let mut id = 0u64;
        // SAFETY: the loader checks the module, the player handle and the moves before it acts; the module string
        // and the move list live for the call, and `id` is a live local.
        #[allow(unsafe_code)]
        let status = unsafe {
            request(
                AmmlStr::from_str(self.module),
                self.player,
                moves.as_ptr(),
                count,
                AmmlRequestOptions::from(options),
                &mut id,
            )
        };
        ApiError::check(status).map(|()| RequestId(id))
    }

    /// Swaps the stacks of slots `a` and `b`: one `Swap` with `hidden_open` and `predict` on. See `request_moves`.
    pub fn request_swap(&self, a: u8, b: u8) -> Result<RequestId, ApiError> {
        self.request_moves(&[InventoryMove::Swap { a, b }], RequestOptions { hidden_open: true, predict: true })
    }

    // The stack handle of slot `index`: `None` for a refused call, `Some(None)` for an empty slot.
    fn slot_stack(&self, index: u8) -> Option<Option<*const c_void>> {
        let read = self.table.slot_item?;
        if index >= Self::SLOTS {
            return None;
        }
        let mut stack: *const c_void = std::ptr::null();
        // SAFETY: the loader checks the player handle and the index before it reads game memory, and `stack` is a
        // live local.
        #[allow(unsafe_code)]
        let status = unsafe { read(self.player, index, &mut stack) };
        status.is_ok().then_some((!stack.is_null()).then_some(stack))
    }
}
