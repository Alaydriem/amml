use std::ffi::c_void;
use std::mem::MaybeUninit;

use amml_msvc::SharedPtr;

use crate::{ClientAdapter, RequestCalls};

/// `std::shared_ptr<Packet> MinecraftPackets::createPacket(MinecraftPacketIds)`, static: rcx the buffer, edx the id,
/// the buffer back in rax.
type CreatePacketFn = unsafe extern "C" fn(*mut SharedPtr<c_void>, u32) -> *mut SharedPtr<c_void>;
/// `Bedrock::Result<void> Packet::read(ReadOnlyBinaryStream&)`: rcx the packet, rdx the result, r8 the stream.
type ReadFn = unsafe extern "C" fn(*mut c_void, *mut u8, *mut c_void) -> *mut u8;
/// `ReadOnlyBinaryStream::~ReadOnlyBinaryStream()`: rcx the stream.
type StreamDtorFn = unsafe extern "C" fn(*mut c_void);
/// `void PacketSender::send(Packet&)`: rcx the sender, rdx the packet.
type SendFn = unsafe extern "C" fn(*mut c_void, *mut c_void);

/// The game's packets for the request path: a packet made by the game's factory, filled from
/// payload bytes through the game's own `Packet::read`, and sent through the client's packet sender. Game thread only.
///
/// Nothing here holds a lock of the loader: a send may bring the server's answer back into a packet detour before it
/// returns.
pub struct PacketIo {
    calls: RequestCalls,
}

impl PacketIo {
    // The buffer of the `Result<void>`: the exe's shared read body writes the error in the first 0x40 bytes and the
    // has-value flag at 0x40; the buffer is larger, so a longer result still fits.
    const RESULT_BYTES: usize = 0x80;
    const RESULT_HAS_VALUE: usize = 0x40;
    // The mirror of the game's `ReadOnlyBinaryStream` (0x48 bytes): the vtable, an owned buffer (an empty MSVC
    // string), the view of the bytes, the read pointer and the overflow flag.
    const STREAM_WORDS: usize = 9;
    const STREAM_BUFFER: usize = 1;
    const STREAM_VIEW: usize = 5;
    const STREAM_VIEW_LEN: usize = 6;
    // An empty MSVC std::string: 16 zero inline bytes, size 0, capacity 15.
    const EMPTY_STRING: [u64; 4] = [0, 0, 0, 15];

    pub fn new(calls: RequestCalls) -> Self {
        Self { calls }
    }

    /// A packet of `id` from the game's factory, held with one use; `None` when the factory makes none. Give it back
    /// with `release`.
    ///
    /// The packet is not checked by name: no vtable slot of a packet is called, because no derived entry proves one.
    /// What ties an id to its packet is the factory and the packet ids of `ProtocolValues`, which the game test of the
    /// deriver (`the_request_packets_have_the_ids_names_and_sizes_the_loader_uses`) checks on each gate build.
    pub fn make(&self, id: u32) -> Option<SharedPtr<c_void>> {
        // SAFETY: an all-zero shared_ptr is the empty one.
        let mut made: SharedPtr<c_void> = unsafe { std::mem::zeroed() };
        // SAFETY: the entry is `MinecraftPackets::createPacket`, with the signature of `CreatePacketFn`.
        let create = unsafe { std::mem::transmute::<usize, CreatePacketFn>(self.calls.create_packet) };
        // SAFETY: `made` is 16 writable bytes, on the game thread, and `id` is a packet id the game knows.
        unsafe { create(&raw mut made, id) };
        Self::address(&made)?;
        Some(made)
    }

    /// The address of the packet in `held`.
    pub fn address(held: &SharedPtr<c_void>) -> Option<usize> {
        held.get().map(|packet| std::ptr::from_ref(packet).expose_provenance())
    }

    /// Gives back the use `held` owns.
    pub fn release(mut held: SharedPtr<c_void>) {
        // SAFETY: `held` owns the one use the factory gave, which this call gives back once.
        unsafe { held.release() };
    }

    /// The client's packet sender; `None` when the client gives none.
    pub fn sender(&self, client: usize) -> Option<usize> {
        ClientAdapter::call_pointer(client, Some(self.calls.sender_slot)).ok().filter(|sender| *sender != 0)
    }

    /// Sends the live `packet` through `sender`.
    pub fn send(&self, sender: usize, packet: usize) -> bool {
        let Some(function) = ClientAdapter::virtual_fn(sender, Some(self.calls.send_slot)) else { return false };
        // SAFETY: the slot holds `void PacketSender::send(Packet&)`.
        let send = unsafe { std::mem::transmute::<usize, SendFn>(function) };
        // SAFETY: the sender and the packet are live, on the game thread; the call does not take the packet.
        unsafe { send(std::ptr::with_exposed_provenance_mut(sender), std::ptr::with_exposed_provenance_mut(packet)) };
        true
    }

    /// Fills the live `packet` from the payload `bytes` through its one-argument `Packet::read`, as the network layer
    /// does. True when the game accepted the payload. The stream is destroyed once before the call returns.
    pub fn fill(&self, packet: usize, bytes: &[u8]) -> bool {
        let Some(function) = ClientAdapter::virtual_fn(packet, Some(self.calls.read_slot)) else { return false };
        // SAFETY: the slot holds `Bedrock::Result<void> Packet::read(ReadOnlyBinaryStream&)`.
        let read = unsafe { std::mem::transmute::<usize, ReadFn>(function) };
        let mut stream = [0u64; Self::STREAM_WORDS];
        stream[0] = self.calls.stream_vtable as u64;
        stream[Self::STREAM_BUFFER..Self::STREAM_BUFFER + 4].copy_from_slice(&Self::EMPTY_STRING);
        stream[Self::STREAM_VIEW] = bytes.as_ptr().expose_provenance() as u64;
        stream[Self::STREAM_VIEW_LEN] = bytes.len() as u64;
        let mut result = MaybeUninit::<[u64; Self::RESULT_BYTES / 8]>::zeroed();
        let out = result.as_mut_ptr().cast::<u8>();
        // SAFETY: the packet, a result buffer larger than the game's `Result<void>`, and a stream over `bytes` that
        // lives for the call.
        unsafe { read(std::ptr::with_exposed_provenance_mut(packet), out, stream.as_mut_ptr().cast()) };
        // SAFETY: the entry is the address of the stream destructor; the stream was made with the game's vtable and an
        // empty owned buffer, and is destroyed once.
        let destroy = unsafe { std::mem::transmute::<usize, StreamDtorFn>(self.calls.stream_dtor) };
        // SAFETY: as above.
        unsafe { destroy(stream.as_mut_ptr().cast()) };
        // SAFETY: the flag byte lies inside the zeroed buffer.
        unsafe { out.add(Self::RESULT_HAS_VALUE).read() == 1 }
    }
}
