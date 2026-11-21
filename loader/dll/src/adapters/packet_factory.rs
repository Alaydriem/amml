use std::ffi::c_void;

use amml_msvc::SharedPtr;

/// `std::shared_ptr<Packet> MinecraftPackets::createPacket(MinecraftPacketIds)`, static: rcx the buffer, edx the id,
/// the buffer back in rax.
type CreatePacketFn = unsafe extern "C" fn(*mut SharedPtr<c_void>, u32) -> *mut SharedPtr<c_void>;

/// The game's packet factory, `MinecraftPackets::createPacket`. A made packet is held with one use of the game's
/// control block.
///
/// The packet is not checked by name: no vtable slot of a packet is called, because no derived entry proves one. The
/// factory and the id tie the packet to its type; the game test of the deriver
/// (`the_world_packets_have_the_ids_names_and_sizes_the_loader_makes`) checks the id, the name and the size on each
/// gate build.
#[derive(Clone, Copy, Debug)]
pub struct PacketFactory {
    create: usize,
}

impl PacketFactory {
    /// The factory at `create`, the address of the `MinecraftPackets.createPacket` entry.
    pub fn new(create: usize) -> Self {
        Self { create }
    }

    /// A packet of `id` from the game's factory; `None` when the factory makes none.
    ///
    /// # Safety
    /// On the game thread, with `create` the resolved `MinecraftPackets.createPacket` and `id` a packet id the game
    /// knows.
    pub unsafe fn make(&self, id: u32) -> Option<SharedPtr<c_void>> {
        // SAFETY: an all-zero shared_ptr is the empty one.
        let mut made: SharedPtr<c_void> = unsafe { std::mem::zeroed() };
        // SAFETY: the caller guarantees the entry is `MinecraftPackets::createPacket`, with the signature of
        // `CreatePacketFn`.
        let create = unsafe { std::mem::transmute::<usize, CreatePacketFn>(self.create) };
        // SAFETY: `made` is 16 writable bytes; the caller guarantees the thread and the id.
        unsafe { create(&raw mut made, id) };
        made.get().is_some().then_some(made)
    }
}
