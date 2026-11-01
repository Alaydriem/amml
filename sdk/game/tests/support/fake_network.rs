use std::ffi::c_void;

use amml_game::{EntryId, GameEntries, NetworkIdentifierLayout};

use super::{FakeEntries, GameMemory};

/// A fake `NetworkSystem` with a `mConnections` vector, and the connections it points at.
pub struct FakeNetwork {
    system: GameMemory,
    connections: Vec<GameMemory>,
    vector: Vec<u64>,
}

impl FakeNetwork {
    pub const CONNECTIONS_AT: usize = 0xA0;
    pub const PEER_AT: usize = 0x118;
    pub const RECEIVE_SLOT: u64 = 8;

    /// A system whose vector holds one element for each of `ids`; `None` makes a null element.
    pub fn new(ids: &[Option<&NetworkIdentifierLayout>]) -> Self {
        let mut connections: Vec<GameMemory> = Vec::new();
        for id in ids {
            let mut connection = GameMemory::new(0x1B0);
            if let Some(id) = id {
                Self::put_id(&mut connection, id);
            }
            connections.push(connection);
        }
        let mut network = Self { system: GameMemory::new(0x200), connections, vector: Vec::new() };
        let pointers: Vec<u64> = ids
            .iter()
            .zip(network.connections.iter_mut())
            .map(|(id, connection)| if id.is_some() { connection.ptr() as u64 } else { 0 })
            .collect();
        network.vector = pointers;
        network.point_system_at_vector();
        network
    }

    /// A system whose vector is not null and holds no element (begin equals end), as the game leaves a cleared one.
    pub fn cleared() -> Self {
        let mut network = Self::new(&[]);
        network.vector = Vec::with_capacity(4);
        let begin = network.vector.as_ptr() as u64;
        network.system.put::<u64>(Self::CONNECTIONS_AT, begin);
        network.system.put::<u64>(Self::CONNECTIONS_AT + 8, begin);
        network
    }

    fn put_id(connection: &mut GameMemory, id: &NetworkIdentifierLayout) {
        // SAFETY: the mirror is plain data of 0xB0 bytes.
        let bytes = unsafe { std::slice::from_raw_parts((id as *const NetworkIdentifierLayout).cast::<u8>(), 0xB0) };
        for (at, byte) in bytes.iter().enumerate() {
            connection.put::<u8>(at, *byte);
        }
    }

    fn point_system_at_vector(&mut self) {
        let begin = if self.vector.is_empty() { 0 } else { self.vector.as_ptr() as u64 };
        let end = begin + (self.vector.len() * 8) as u64;
        self.system.put::<u64>(Self::CONNECTIONS_AT, begin);
        self.system.put::<u64>(Self::CONNECTIONS_AT + 8, end);
    }

    pub fn system_ptr(&mut self) -> *mut c_void {
        self.system.ptr()
    }

    /// The address of connection `index`, whether or not the vector holds it.
    pub fn connection_ptr(&mut self, index: usize) -> *mut c_void {
        self.connections[index].ptr()
    }

    pub fn connection(&mut self, index: usize) -> &mut GameMemory {
        &mut self.connections[index]
    }

    pub fn entries() -> GameEntries {
        FakeEntries::build(&[
            (EntryId::NETWORK_SYSTEM_CONNECTIONS, Self::CONNECTIONS_AT as u64),
            (EntryId::NETWORK_CONNECTION_PEER, Self::PEER_AT as u64),
            (EntryId::NETWORK_PEER_RECEIVE_PACKET, Self::RECEIVE_SLOT),
        ])
    }
}
