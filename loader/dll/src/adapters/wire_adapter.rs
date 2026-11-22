use std::ffi::c_void;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::Instant;

use amml_abi::{
    AmmlEvent, AmmlEventContext, AmmlPhase, AmmlStr, AmmlThread, AmmlWireConnection, AmmlWireDir, ConnectionStarted,
    PacketWire,
};
use amml_data::{AdapterCatalog, Availability, EntryId, NeedKind};
use amml_msvc::MsvcString;
use amml_runtime::WireConnections;

use crate::{EventHub, LoaderState, NetworkConnectionLayout, NetworkIdEquality, WireDetours, WorldInfoAdapter};

static ADAPTER: OnceLock<WireAdapter> = OnceLock::new();

/// `bool NetworkSystem::isServer() const`: rcx the system, the result in al (survey section 8, row C5).
type IsServerFn = unsafe extern "C" fn(*mut c_void) -> u8;

/// The `network.packet_wire` and `network.connection_started` events.
///
/// `outbound` runs Before `_sendInternal` on the sending thread: a system that answers `isServer` true is left alone
/// (the answer is kept per system until `WorldLeft`); otherwise the connection whose identifier equals the send's is
/// found in `mConnections` (8-byte elements) with the game's per-type compare, and `WireConnections` gives its id, its
/// kind (`mType`) and its coverage (slot `NetworkPeer.receivePacket` of its peer's vtable is the hooked receive). A new
/// connection raises `ConnectionStarted` first (at its first packet sent while a `ConnectionStarted` listener exists:
/// a `PacketWire` listener alone does not use the start up), then the packet raises `PacketWire` `OUT`. `inbound` runs After the
/// outer peer's receive gave a packet, on the network thread, and raises `PacketWire` `IN` for a peer of a known
/// connection only. The bytes are lent from the game's string for the dispatch and cleared after it; no lock is held
/// while a listener runs.
pub struct WireAdapter {
    state: &'static LoaderState,
    is_server: usize,
    connections_at: usize,
    receive_body: usize,
    receive_slot: usize,
    protocol: u32,
    book: Mutex<WireConnections>,
    out_thread_logged: AtomicBool,
    in_thread_logged: AtomicBool,
    first_out_timed: AtomicBool,
    first_in_timed: AtomicBool,
}

impl WireAdapter {
    pub const WIRE: &'static str = PacketWire::ID;
    pub const STARTED: &'static str = ConnectionStarted::ID;
    /// A dispatch slower than this is logged while a mod listens.
    pub const SLOW_NANOS: u64 = 50_000;
    const TYPE_ENTRY: &'static str = "NetworkConnection.mType";
    const PEER_ENTRY: &'static str = "NetworkConnection.mPeer";
    // Loader hooks run before every mod hook on the same function.
    const PRIORITY: i32 = i32::MAX;
    // The size of one element of `mConnections`: a `unique_ptr<NetworkConnection>`.
    const ELEMENT_SIZE: usize = 8;
    // The most connections a walk reads; the client holds one or two.
    const MAX_WALK: usize = 256;

    /// Checks the connection mirror against the derived entries, hooks the send and the receive when both rows resolved,
    /// offers each row whose hooks stand, and logs one line per row. Works once per process: a second call hooks nothing
    /// and gives the catalog rows.
    pub fn install(state: &'static LoaderState) -> Vec<Availability> {
        let resolved = |name: &str| state.table().get(&EntryId(name.to_string())).is_some();
        let mut rows: Vec<Availability> = AdapterCatalog::availability(&resolved)
            .into_iter()
            .filter(|row| row.kind == NeedKind::Event && (row.id == Self::WIRE || row.id == Self::STARTED))
            .collect();
        let value = |name: &str| state.resolve(name).ok().and_then(|value| usize::try_from(value).ok());
        let adapter = Self {
            state,
            is_server: value("NetworkSystem.isServer").unwrap_or(0),
            connections_at: value("NetworkSystem.mConnections").unwrap_or(0),
            receive_body: value(WireDetours::RECEIVE).unwrap_or(0),
            receive_slot: value("NetworkPeer.receivePacket").unwrap_or(0),
            protocol: value("SharedConstants.NetworkProtocolVersion").and_then(|v| u32::try_from(v).ok()).unwrap_or(0),
            book: Mutex::new(WireConnections::new()),
            out_thread_logged: AtomicBool::new(false),
            in_thread_logged: AtomicBool::new(false),
            first_out_timed: AtomicBool::new(false),
            first_in_timed: AtomicBool::new(false),
        };
        if ADAPTER.set(adapter).is_err() {
            curia::warn!("the wire adapter is already installed");
            return rows;
        }
        let mismatched: Vec<&'static str> = [
            (Self::TYPE_ENTRY, NetworkConnectionLayout::TYPE_AT),
            (Self::PEER_ENTRY, NetworkConnectionLayout::PEER_AT),
        ]
        .into_iter()
        .filter(|(name, at)| value(name).is_some_and(|derived| derived != *at))
        .map(|(name, _)| name)
        .collect();
        if !mismatched.is_empty() {
            curia::warn!("the connection mirror does not match the build", { entries: mismatched.join(", ") });
        }
        let wanted = mismatched.is_empty() && rows.iter().any(|row| row.shape.is_some());
        let refused: Vec<&'static str> = if wanted {
            [WireDetours::send_hook(), WireDetours::receive_hook()]
                .iter()
                .filter(|hook| !hook.install(state, Self::PRIORITY))
                .map(|hook| hook.entry)
                .collect()
        } else {
            Vec::new()
        };
        for row in &mut rows {
            let missing = if !mismatched.is_empty() { mismatched.clone() } else { refused.clone() };
            if row.shape.is_some() && !missing.is_empty() {
                row.shape = None;
                row.missing = missing;
            }
            match row.shape {
                Some(shape) => {
                    state.events().offer(row.id, row.version);
                    curia::info!("event offered", { event: row.id, shape: shape });
                }
                None => curia::info!("event not available", { event: row.id, missing: row.missing.join(", ") }),
            }
        }
        rows
    }

    /// The installed adapter; `None` before `install`.
    pub fn get() -> Option<&'static WireAdapter> {
        ADAPTER.get()
    }

    /// Runs `body` on the installed adapter; `fallback` before `install`.
    pub(crate) fn with<R>(fallback: R, body: impl FnOnce(&WireAdapter) -> R) -> R {
        match ADAPTER.get() {
            Some(adapter) => body(adapter),
            None => fallback,
        }
    }

    /// At every `ClientExitLevel` (with or without a joined world): every connection and system answer is forgotten, so
    /// a connection of the next join that reuses freed addresses starts again. The join itself forgets nothing, so the
    /// book does not depend on when the game calls `onStartJoinGame` relative to the first packets of a connection; a
    /// connection that left `mConnections` is forgotten when the next new connection is seen (`keep_only`).
    pub(crate) fn forget(&self) {
        self.book.lock().unwrap_or_else(std::sync::PoisonError::into_inner).forget();
    }

    /// Before `_sendInternal` of the system at `system` to the identifier at `id` with the string at `data`.
    pub(crate) fn outbound(&self, system: usize, id: usize, data: usize) {
        let hub = self.state.events();
        let wire = hub.has_listeners(Self::WIRE);
        if !wire && !hub.has_listeners(Self::STARTED) {
            return;
        }
        let begun = Instant::now();
        let Some(connection) = self.connection_of(system, id) else { return };
        let game = hub.gate().on_game_thread();
        if !self.out_thread_logged.swap(true, Ordering::Relaxed) {
            curia::info!("packet wire out thread", { game: game });
        }
        let context = hub.context_on(if game { AmmlThread::GAME } else { AmmlThread::NETWORK });
        self.started(connection, game, &context);
        if !wire || data == 0 {
            return;
        }
        // SAFETY: `data` is the non-null `std::string const&` the game passed to `_sendInternal`, live for the call.
        let bytes = unsafe { &*std::ptr::with_exposed_provenance::<MsvcString>(data) }.as_bytes();
        self.raise(AmmlWireDir::OUT, connection, bytes, &context);
        self.note_time(AmmlWireDir::OUT, &self.first_out_timed, begun);
    }

    /// After the peer at `peer` gave one packet in the string at `out`.
    pub(crate) fn inbound(&self, peer: usize, out: usize) {
        let hub = self.state.events();
        if out == 0 || !hub.has_listeners(Self::WIRE) {
            return;
        }
        let begun = Instant::now();
        let connection = self.book.lock().unwrap_or_else(std::sync::PoisonError::into_inner).inbound(peer as u64);
        let Some(connection) = connection else { return };
        // SAFETY: `out` is the non-null `std::string&` the game passed to the receive, which filled it with one packet.
        let bytes = unsafe { &*std::ptr::with_exposed_provenance::<MsvcString>(out) }.as_bytes();
        if bytes.is_empty() {
            return;
        }
        let game = hub.gate().on_game_thread();
        if !self.in_thread_logged.swap(true, Ordering::Relaxed) {
            curia::info!("packet wire in thread", { game: game });
        }
        let context = hub.context_on(if game { AmmlThread::GAME } else { AmmlThread::NETWORK });
        self.raise(AmmlWireDir::IN, connection, bytes, &context);
        self.note_time(AmmlWireDir::IN, &self.first_in_timed, begun);
    }

    // The client connection the send goes to; `None` for a server system, an identifier no connection has, or a null
    // argument. Before a connection is added, the book forgets the connections the system no longer holds.
    fn connection_of(&self, system: usize, id: usize) -> Option<AmmlWireConnection> {
        if system == 0 || id == 0 {
            return None;
        }
        let known = self.book.lock().unwrap_or_else(std::sync::PoisonError::into_inner).system(system as u64);
        let server = match known {
            Some(server) => server,
            None => {
                let server = self.ask_server(system)?;
                self.book.lock().unwrap_or_else(std::sync::PoisonError::into_inner).note_system(system as u64, server);
                server
            }
        };
        if server {
            return None;
        }
        // SAFETY: `id` is the `NetworkIdentifier const&` the game passed to `_sendInternal`, B0h live bytes.
        let id = unsafe { &*std::ptr::with_exposed_provenance::<[u8; NetworkIdEquality::SIZE]>(id) };
        let connection = self.find(system, id)?;
        // SAFETY: `find` gives a live connection of the system, which starts with the mirror's head (checked against
        // the derived entries at install).
        let head = unsafe { &*std::ptr::with_exposed_provenance::<NetworkConnectionLayout>(connection) };
        let peer = head.peer.expose_provenance();
        let covered = self.covers(peer);
        let mut book = self.book.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        if !book.knows(connection as u64, peer as u64) {
            let live: Vec<u64> = self.connections(system).map(|address| address as u64).collect();
            book.keep_only(&live);
        }
        let (wire, created) = book.outbound(connection as u64, peer as u64, head.kind, covered);
        drop(book);
        if created && !covered {
            curia::warn!("packet wire peer not covered", { id: wire.id, kind: wire.kind });
        }
        Some(wire)
    }

    // The answer of the live system at `system` to `isServer`.
    fn ask_server(&self, system: usize) -> Option<bool> {
        if self.is_server == 0 {
            return None;
        }
        // SAFETY: the entry is `bool NetworkSystem::isServer() const`; `system` is the live system the game passed to
        // `_sendInternal`, on the thread that sends.
        let server = unsafe {
            let call = std::mem::transmute::<usize, IsServerFn>(self.is_server);
            call(std::ptr::with_exposed_provenance_mut(system))
        };
        Some(server != 0)
    }

    // The first connection of the system at `system` whose identifier equals `id`.
    fn find(&self, system: usize, id: &[u8; NetworkIdEquality::SIZE]) -> Option<usize> {
        self.connections(system).find(|connection| {
            // SAFETY: `connections` gives live connections owned by the vector, which start with their identifier.
            let head = unsafe { &*std::ptr::with_exposed_provenance::<NetworkConnectionLayout>(*connection) };
            NetworkIdEquality::equals(&head.id, id)
        })
    }

    // The non-null connection pointers of `mConnections` of the system at `system`, at most `MAX_WALK`.
    fn connections(&self, system: usize) -> impl Iterator<Item = usize> {
        let (begin, end) = match system.checked_add(self.connections_at) {
            // SAFETY: the entry is the offset of `mConnections` in the live system: its begin and end pointers are
            // the first two qwords.
            Some(vector) => unsafe {
                let at = std::ptr::with_exposed_provenance::<usize>(vector);
                (at.read_unaligned(), at.add(1).read_unaligned())
            },
            None => (0, 0),
        };
        let count =
            if begin == 0 || end < begin { 0 } else { ((end - begin) / Self::ELEMENT_SIZE).min(Self::MAX_WALK) };
        (0..count)
            .map(move |index| {
                // SAFETY: the index is inside the vector the game holds; each element is a connection pointer.
                unsafe {
                    std::ptr::with_exposed_provenance::<usize>(begin + index * Self::ELEMENT_SIZE).read_unaligned()
                }
            })
            .filter(|connection| *connection != 0)
    }

    // Whether the receive slot of the peer at `peer` is the hooked receive.
    fn covers(&self, peer: usize) -> bool {
        if peer == 0 || self.receive_body == 0 {
            return false;
        }
        // SAFETY: a non-null `mPeer` of a live connection points to a `NetworkPeer`, whose first qword is its vtable;
        // the entry is the slot of `receivePacket` in it.
        let slot = unsafe {
            let vtable = std::ptr::with_exposed_provenance::<usize>(peer).read_unaligned();
            if vtable == 0 {
                return false;
            }
            std::ptr::with_exposed_provenance::<usize>(vtable + self.receive_slot * 8).read_unaligned()
        };
        slot == self.receive_body
    }

    // Raises `ConnectionStarted` for `connection` while its start is pending and a mod listens to it, and marks it given;
    // the host and port of a remote one come from the client's connection info when the send runs on the game thread.
    fn started(&self, connection: AmmlWireConnection, game: bool, context: &AmmlEventContext) {
        let hub = self.state.events();
        if !hub.has_listeners(Self::STARTED) {
            return;
        }
        {
            let mut book = self.book.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
            if !book.start_pending(connection.id) {
                return;
            }
            book.mark_started(connection.id);
        }
        let (port, host) = if connection.kind == AmmlWireConnection::REMOTE && game {
            WorldInfoAdapter::get().and_then(WorldInfoAdapter::primary_connection).unwrap_or_default()
        } else {
            (0, String::new())
        };
        let protocol = self.protocol;
        let fill = |event: &mut ConnectionStarted| {
            event.connection = connection;
            event.protocol = protocol;
            event.port = port;
            event._reserved = 0;
            event.host = AmmlStr::from_str(&host);
        };
        let mut event = ConnectionStarted {
            header: EventHub::header::<ConnectionStarted>(AmmlPhase::BEFORE),
            connection,
            protocol,
            port,
            _reserved: 0,
            host: AmmlStr::from_str(&host),
        };
        hub.raise_keeping(&mut event, context, &fill);
    }

    // Raises `PacketWire` with `bytes` lent for the dispatch; the loader's fields are written back after every listener.
    fn raise(&self, dir: AmmlWireDir, connection: AmmlWireConnection, bytes: &[u8], context: &AmmlEventContext) {
        let fill = |event: &mut PacketWire| {
            event.dir = dir;
            event.connection = connection;
            event.bytes = bytes.as_ptr();
            event.len = bytes.len();
        };
        let phase = if dir == AmmlWireDir::OUT { AmmlPhase::BEFORE } else { AmmlPhase::AFTER };
        let mut event = PacketWire {
            header: EventHub::header::<PacketWire>(phase),
            dir,
            _reserved: [0; 7],
            connection,
            bytes: bytes.as_ptr(),
            len: bytes.len(),
        };
        self.state.events().raise_keeping(&mut event, context, &fill);
        event.bytes = std::ptr::null();
        event.len = 0;
        std::hint::black_box(&event);
    }

    // Logs `packet wire time` for the first packet of `dir` and each one slower than `SLOW_NANOS`.
    fn note_time(&self, dir: AmmlWireDir, first: &AtomicBool, begun: Instant) {
        let nanos = u64::try_from(begun.elapsed().as_nanos()).unwrap_or(u64::MAX);
        if !first.swap(true, Ordering::Relaxed) || nanos > Self::SLOW_NANOS {
            curia::info!("packet wire time", { dir: dir.0, nanos: nanos });
        }
    }
}
