use amml_abi::AmmlWireConnection;

/// The client connections the packet wire saw in the current join, and the answer of each network system to
/// `isServer`.
///
/// A connection is the pair of a `NetworkConnection` address and its peer address. The first `outbound` of a pair gives
/// it a new id (never 0, never given again in the process); a new peer at a known connection address is a new
/// connection. Each connection keeps a pending start until `mark_started`: the loader marks it only when a
/// `ConnectionStarted` listener got it. `inbound` finds the connection of a covered peer. `keep_only` forgets the
/// connections that are no longer in the system's vector; `forget` forgets every connection and system (at a level
/// exit); the ids go on counting. Holds at most `MAX_CONNECTIONS` connections: a new one past that
/// forgets the oldest.
pub struct WireConnections {
    next_id: u64,
    // (connection address, peer address, the connection, whether its start was given), oldest first.
    connections: Vec<(u64, u64, AmmlWireConnection, bool)>,
    // (system address, whether it is a server).
    systems: Vec<(u64, bool)>,
}

impl WireConnections {
    /// `NetworkConnection::Type` of a connection to another machine.
    pub const REMOTE_TYPE: i32 = 0;
    /// `NetworkConnection::Type` of the client's connection to a world on this machine.
    pub const LOCAL_TYPE: i32 = 1;
    pub const MAX_CONNECTIONS: usize = 64;
    const MAX_SYSTEMS: usize = 8;

    pub fn new() -> Self {
        Self { next_id: 1, connections: Vec::new(), systems: Vec::new() }
    }

    /// Whether the pair of `address` and `peer` is a known connection.
    pub fn knows(&self, address: u64, peer: u64) -> bool {
        self.connections.iter().any(|(at, of, _, _)| *at == address && *of == peer)
    }

    /// The connection at `address` with peer `peer`, of the game's type `connection_type` (`LOCAL_TYPE` is local, any
    /// other value remote), whose peer the receive hook covers when `covered`; true when this call created it.
    pub fn outbound(
        &mut self,
        address: u64,
        peer: u64,
        connection_type: i32,
        covered: bool,
    ) -> (AmmlWireConnection, bool) {
        if let Some((_, _, connection, _)) =
            self.connections.iter().find(|(at, of, _, _)| *at == address && *of == peer)
        {
            return (*connection, false);
        }
        self.connections.retain(|(at, _, _, _)| *at != address);
        if self.connections.len() >= Self::MAX_CONNECTIONS {
            self.connections.remove(0);
        }
        let kind =
            if connection_type == Self::LOCAL_TYPE { AmmlWireConnection::LOCAL } else { AmmlWireConnection::REMOTE };
        let connection = AmmlWireConnection { id: self.next_id, kind, covered: u8::from(covered), _reserved: [0; 6] };
        self.next_id = self.next_id.saturating_add(1);
        self.connections.push((address, peer, connection, false));
        (connection, true)
    }

    /// Whether the connection `id` still waits for its start.
    pub fn start_pending(&self, id: u64) -> bool {
        self.connections.iter().any(|(_, _, connection, started)| connection.id == id && !started)
    }

    /// The start of the connection `id` was given to a listener.
    pub fn mark_started(&mut self, id: u64) {
        for (_, _, connection, started) in &mut self.connections {
            if connection.id == id {
                *started = true;
            }
        }
    }

    /// The connection whose covered peer is `peer`; `None` for a peer no `outbound` named or an uncovered one.
    pub fn inbound(&self, peer: u64) -> Option<AmmlWireConnection> {
        self.connections
            .iter()
            .find(|(_, of, connection, _)| *of == peer && connection.covered != 0)
            .map(|(_, _, connection, _)| *connection)
    }

    /// Forgets every connection whose address is not in `live` (the connection addresses the client system holds now).
    pub fn keep_only(&mut self, live: &[u64]) {
        self.connections.retain(|(at, _, _, _)| live.contains(at));
    }

    /// Whether the system at `address` is a server, when `note_system` gave it.
    pub fn system(&self, address: u64) -> Option<bool> {
        self.systems.iter().find(|(at, _)| *at == address).map(|(_, server)| *server)
    }

    /// Keeps the answer of the system at `address` to `isServer`.
    pub fn note_system(&mut self, address: u64, server: bool) {
        self.systems.retain(|(at, _)| *at != address);
        if self.systems.len() >= Self::MAX_SYSTEMS {
            self.systems.remove(0);
        }
        self.systems.push((address, server));
    }

    /// Forgets every connection and system.
    pub fn forget(&mut self) {
        self.connections.clear();
        self.systems.clear();
    }
}

impl Default for WireConnections {
    fn default() -> Self {
        Self::new()
    }
}
