use amml_abi::AmmlScreen;
use amml_msvc::MemberCall;
use amml_runtime::{ProtocolValues, RequestSlot, RequestStep};

use crate::{ClientAdapter, InventoryAdapter, PacketIo, PacketLayout, RequestCalls, RequestPort};

/// The request port of the running game: the screen of the client, the slots of the local player and the packets of
/// the request path. Built for one client tick or one call; it keeps nothing. Game thread only.
pub struct GameRequestPort<'a> {
    calls: &'a RequestCalls,
    io: &'a PacketIo,
    client: usize,
}

impl<'a> GameRequestPort<'a> {
    pub fn new(calls: &'a RequestCalls, io: &'a PacketIo, client: usize) -> Self {
        Self { calls, io, client }
    }

    fn player() -> Option<usize> {
        ClientAdapter::get().and_then(ClientAdapter::primary_local_player)
    }

    // `Interact(OpenInventory)` for the local player, as the game's own inventory key sends it.
    fn interact(&self) -> bool {
        let Some(player) = Self::player() else { return false };
        // SAFETY: the entry is the address of `getRuntimeID`, a member function that returns an 8-byte class; the
        // player is the live local player, on the game thread.
        let runtime_id = unsafe {
            MemberCall::call_sret0::<u64>(
                std::ptr::with_exposed_provenance(self.calls.get_runtime_id),
                std::ptr::with_exposed_provenance_mut(player),
            )
        };
        let Some(made) = self.io.make(ProtocolValues::INTERACT_ID) else { return false };
        let Some(packet) = PacketIo::address(&made) else {
            PacketIo::release(made);
            return false;
        };
        // SAFETY: the packet is the `InteractPacket` the game's factory made for its id.
        unsafe { PacketLayout::write_interact(packet, ProtocolValues::OPEN_INVENTORY, runtime_id) };
        let sent = self.send(packet);
        PacketIo::release(made);
        sent
    }

    // One `ItemStackRequest` packet filled from the payload.
    fn requests(&self, bytes: &[u8]) -> bool {
        let Some(made) = self.io.make(ProtocolValues::ITEM_STACK_REQUEST_ID) else {
            return false;
        };
        let Some(packet) = PacketIo::address(&made) else {
            PacketIo::release(made);
            return false;
        };
        let sent = if self.io.fill(packet, bytes) {
            self.send(packet)
        } else {
            curia::warn!("the game did not accept the item stack request payload", { bytes: bytes.len() });
            false
        };
        PacketIo::release(made);
        sent
    }

    // The client's `ContainerClose` (type none) for the inventory the server opened.
    fn close(&self, container_id: i8) -> bool {
        let Some(made) = self.io.make(ProtocolValues::CONTAINER_CLOSE_ID) else {
            return false;
        };
        let Some(packet) = PacketIo::address(&made) else {
            PacketIo::release(made);
            return false;
        };
        // SAFETY: the packet is the `ContainerClosePacket` the game's factory made for its id.
        unsafe { PacketLayout::write_close(packet, container_id) };
        let sent = self.send(packet);
        PacketIo::release(made);
        sent
    }

    fn send(&self, packet: usize) -> bool {
        self.io.sender(self.client).is_some_and(|sender| self.io.send(sender, packet))
    }

    fn swap(&self, a: u8, b: u8, expect_at_b: Option<i32>) -> bool {
        let (Some(player), Some(inventory)) = (Self::player(), InventoryAdapter::get()) else { return false };
        let expect = expect_at_b.map(|net_id| (net_id, self.calls.net_id_at));
        inventory.swap_player_slots(player, self.calls.swap_slots, a, b, expect)
    }
}

impl RequestPort for GameRequestPort<'_> {
    fn screen_open(&mut self, strict: bool) -> bool {
        let handle = std::ptr::with_exposed_provenance(self.client);
        match ClientAdapter::get().map(|adapter| adapter.top_screen(handle)) {
            Some(Ok(AmmlScreen::HUD)) => false,
            Some(Ok(AmmlScreen::TOAST)) => strict,
            _ => true,
        }
    }

    fn read_slots(&mut self) -> Option<Vec<Option<RequestSlot>>> {
        InventoryAdapter::get()?.request_slots(Self::player()?, self.calls.net_id_at)
    }

    fn run(&mut self, step: &RequestStep) -> bool {
        match step {
            RequestStep::SendInteract => self.interact(),
            RequestStep::SendRequests(bytes) => self.requests(bytes),
            RequestStep::SendClose(container_id) => self.close(*container_id),
            RequestStep::Predict { a, b } => self.swap(*a, *b, None),
            RequestStep::Unpredict { a, b, net_id } => self.swap(*a, *b, Some(*net_id)),
        }
    }
}
