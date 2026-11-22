use std::ffi::c_void;
use std::hint::black_box;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

use amml_data::{EntryKind, Resolved};

use super::{FakeVtable, Harness};

// The bytes the stand-in receive writes into its out string, the sends and receives the original game ran, and the
// calls of the stand-in `isServer`.
static INBOUND: Mutex<Vec<u8>> = Mutex::new(Vec::new());
static SENT: AtomicUsize = AtomicUsize::new(0);
static RECEIVED: AtomicUsize = AtomicUsize::new(0);
static IS_SERVER_CALLS: AtomicUsize = AtomicUsize::new(0);

type SendFn = extern "C" fn(*mut c_void, *const c_void, *const c_void, *const c_void);
type ReceiveFn = extern "C" fn(*mut c_void, *mut [u64; 4], *const c_void) -> i32;

#[inline(never)]
extern "C" fn send_stand_in(system: *mut c_void, id: *const c_void, packet: *const c_void, data: *const c_void) {
    black_box((system, id, packet, data));
    SENT.fetch_add(black_box(1), Ordering::SeqCst);
}

#[inline(never)]
extern "C" fn receive_stand_in(this: *mut c_void, out: *mut [u64; 4], timepoint: *const c_void) -> i32 {
    black_box((this, timepoint));
    RECEIVED.fetch_add(black_box(1), Ordering::SeqCst);
    let bytes = INBOUND.lock().unwrap().clone();
    // SAFETY: `out` is the 32-byte string the caller of the stand-in gives.
    unsafe { out.write(WireGame::words(WireGame::string(&bytes))) };
    black_box(0)
}

#[inline(never)]
extern "C" fn other_receive(this: *mut c_void, out: *mut [u64; 4], timepoint: *const c_void) -> i32 {
    black_box((this, out, timepoint));
    black_box(1)
}

#[inline(never)]
extern "C" fn is_server_stand_in(system: *const u8) -> bool {
    IS_SERVER_CALLS.fetch_add(black_box(1), Ordering::SeqCst);
    // SAFETY: `system` is a stand-in system of `WireGame::system`, at least `SERVER_FLAG_AT + 1` bytes.
    unsafe { system.add(WireGame::SERVER_FLAG_AT).read() != 0 }
}

#[inline(never)]
extern "C" fn destroy_stand_in(value: *mut u8) {
    black_box(value);
}

const NULL: *const () = std::ptr::null();
static COVERED: FakeVtable<9> =
    FakeVtable([NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, receive_stand_in as *const ()]);
static UNCOVERED: FakeVtable<9> =
    FakeVtable([NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, other_receive as *const ()]);

/// Stand-in game memory for the wire adapter: a `NetworkSystem` with its `mConnections` vector, `NetworkConnection`s
/// with their identifier, type and peer, the peers with their vtables, and the hooked `_sendInternal` and
/// `PacketTraceNetworkPeer::_receivePacket` the game calls. Laid out as the 26.63 derived entries say. Everything is
/// leaked.
pub struct WireGame;

impl WireGame {
    pub const CONNECTIONS_AT: usize = 0xc8;
    pub const TYPE_AT: usize = 0xb0;
    pub const PEER_AT: usize = 0x118;
    pub const SERVER_FLAG_AT: usize = 0x10;
    pub const PROTOCOL: u64 = 2225;
    /// `mType` of a remote and of a local connection.
    pub const REMOTE: i32 = 0;
    pub const LOCAL: i32 = 1;
    // `NetworkIdentifier.mType` of an IPv4 identifier, and where its port and address are.
    const ID_ADDRESS: i32 = 1;
    const ID_KIND_AT: usize = 0xa8;
    const ID_SOCK_AT: usize = 0x28;
    const ID_SIZE: usize = 0xb0;

    /// The entries of a build that has every wire row.
    pub fn entries() -> Vec<(&'static str, Resolved)> {
        let function = |address: usize| Resolved {
            kind: EntryKind::Function,
            value: address as u64 - Harness::BASE,
            recipes_agreeing: 2,
        };
        let slot = |value| Resolved { kind: EntryKind::VSlot, value, recipes_agreeing: 2 };
        let field = |value: u64| Resolved { kind: EntryKind::Field, value, recipes_agreeing: 2 };
        vec![
            ("NetworkSystem.sendInternal", function(send_stand_in as *const () as usize)),
            ("NetworkSystem.isServer", function(is_server_stand_in as *const () as usize)),
            ("NetworkSystem.mConnections", field(Self::CONNECTIONS_AT as u64)),
            ("NetworkConnection.mPeer", field(Self::PEER_AT as u64)),
            ("NetworkConnection.mType", field(Self::TYPE_AT as u64)),
            ("PacketTraceNetworkPeer.receivePacket", function(receive_stand_in as *const () as usize)),
            ("NetworkPeer.receivePacket", slot(8)),
            ("SharedConstants.NetworkProtocolVersion", field(Self::PROTOCOL)),
            ("ClientInstance.getGameConnectionInfo", slot(0x30)),
            ("GameConnectionInfo.size", field(0x2f0)),
            ("GameConnectionInfo.dtor", function(destroy_stand_in as *const () as usize)),
        ]
    }

    /// The entries of `entries` but `without`.
    pub fn entries_without(without: &[&str]) -> Vec<(&'static str, Resolved)> {
        Self::entries().into_iter().filter(|(name, _)| !without.contains(name)).collect()
    }

    /// A `NetworkIdentifier` of an IPv4 peer whose port and address bytes are all `peer`; gives its address.
    pub fn id(peer: u8) -> usize {
        Box::leak(Self::id_bytes(peer).into_boxed_slice()).as_ptr().expose_provenance()
    }

    fn id_bytes(peer: u8) -> Vec<u8> {
        let mut id = vec![0u8; Self::ID_SIZE];
        id[Self::ID_KIND_AT..Self::ID_KIND_AT + 4].copy_from_slice(&Self::ID_ADDRESS.to_le_bytes());
        id[Self::ID_SOCK_AT + 2..Self::ID_SOCK_AT + 8].fill(peer);
        id
    }

    /// A connection to the peer `peer` (as `id`) of type `connection_type`, whose peer object's receive slot is the
    /// hooked function when `covered`. Gives the connection and the peer object.
    pub fn connection(peer: u8, connection_type: i32, covered: bool) -> (usize, usize) {
        let vtable = if covered { std::ptr::from_ref(&COVERED) } else { std::ptr::from_ref(&UNCOVERED) };
        let object: &'static mut [usize; 2] = Box::leak(Box::new([vtable.expose_provenance(), 0]));
        let object = object.as_ptr().expose_provenance();
        let mut connection = vec![0u8; 0x200];
        connection[..Self::ID_SIZE].copy_from_slice(&Self::id_bytes(peer));
        connection[Self::TYPE_AT..Self::TYPE_AT + 4].copy_from_slice(&connection_type.to_le_bytes());
        connection[Self::PEER_AT..Self::PEER_AT + 8].copy_from_slice(&object.to_le_bytes());
        (Self::leak_aligned(&connection), object)
    }

    /// A network system that holds `connections`; a server system when `server`.
    pub fn system(server: bool, connections: &[usize]) -> usize {
        let elements: &'static mut [usize] = Box::leak(connections.to_vec().into_boxed_slice());
        let begin = elements.as_ptr().expose_provenance();
        let end = begin + elements.len() * 8;
        let mut system = vec![0u8; 0x200];
        system[Self::SERVER_FLAG_AT] = u8::from(server);
        system[Self::CONNECTIONS_AT..Self::CONNECTIONS_AT + 8].copy_from_slice(&begin.to_le_bytes());
        system[Self::CONNECTIONS_AT + 8..Self::CONNECTIONS_AT + 16].copy_from_slice(&end.to_le_bytes());
        Self::leak_aligned(&system)
    }

    // Leaks `bytes` in an 8-aligned block, as the game's objects are; gives its address.
    fn leak_aligned(bytes: &[u8]) -> usize {
        let words: Vec<u64> = bytes
            .chunks(8)
            .map(|chunk| chunk.iter().rev().fold(0u64, |word, byte| word << 8 | u64::from(*byte)))
            .collect();
        Box::leak(words.into_boxed_slice()).as_ptr().expose_provenance()
    }

    /// A `std::string` of `bytes`: inline up to 15 bytes, else on a leaked block.
    pub fn string(bytes: &[u8]) -> [u8; 32] {
        let mut out = [0u8; 32];
        if bytes.len() <= 15 {
            out[..bytes.len()].copy_from_slice(bytes);
            out[0x18..0x20].copy_from_slice(&15usize.to_le_bytes());
        } else {
            let heap: &'static mut [u8] = Box::leak(bytes.to_vec().into_boxed_slice());
            out[..8].copy_from_slice(&heap.as_ptr().expose_provenance().to_le_bytes());
            out[0x18..0x20].copy_from_slice(&bytes.len().to_le_bytes());
        }
        out[0x10..0x18].copy_from_slice(&bytes.len().to_le_bytes());
        out
    }

    /// The 32 bytes of a `std::string` as the four 8-aligned qwords the game keeps them in.
    pub fn words(bytes: [u8; 32]) -> [u64; 4] {
        std::array::from_fn(|index| u64::from_le_bytes(bytes[index * 8..index * 8 + 8].try_into().unwrap()))
    }

    /// The game sends `bytes` to the peer `id` through the hooked `_sendInternal` of `system`. Gives the address of the
    /// string's bytes.
    pub fn send(system: usize, id: usize, bytes: &[u8]) -> usize {
        let data: &'static [u64; 4] = Box::leak(Box::new(Self::words(Self::string(bytes))));
        let at = if bytes.len() <= 15 { data.as_ptr().expose_provenance() } else { data[0] as usize };
        let function = black_box(send_stand_in as SendFn);
        function(
            std::ptr::with_exposed_provenance_mut(system),
            std::ptr::with_exposed_provenance(id),
            std::ptr::null(),
            data.as_ptr().cast(),
        );
        at
    }

    /// The peer `peer` hands out `bytes` through the hooked `receivePacket`; gives its status.
    pub fn receive(peer: usize, bytes: &[u8]) -> i32 {
        *INBOUND.lock().unwrap() = bytes.to_vec();
        let mut out = [0u64; 4];
        let function = black_box(receive_stand_in as ReceiveFn);
        function(std::ptr::with_exposed_provenance_mut(peer), &mut out, std::ptr::null())
    }

    /// How often the original `_sendInternal` ran.
    pub fn sent() -> usize {
        SENT.load(Ordering::SeqCst)
    }

    /// How often the original `receivePacket` ran.
    pub fn received() -> usize {
        RECEIVED.load(Ordering::SeqCst)
    }

    /// How often the stand-in `isServer` ran.
    pub fn is_server_calls() -> usize {
        IS_SERVER_CALLS.load(Ordering::SeqCst)
    }
}
