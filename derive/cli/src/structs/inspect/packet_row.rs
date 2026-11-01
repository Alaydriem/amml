/// One packet that `createPacket` makes: its id, the packet vtable, `getId`, the name from `getName`, the object
/// size from the `make_shared` allocation, and the `NetEventCallback` slot that its handler stub calls.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PacketRow {
    pub id: u32,
    pub vtable: u32,
    pub get_id: u32,
    pub name: Option<String>,
    pub size: Option<u32>,
    pub slot: Option<u32>,
}

/// The packets of one build, with the counts of the jump table: every case, the cases that go to the default
/// (no packet), and the cases that make something this reader did not recognise.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PacketList {
    pub rows: Vec<PacketRow>,
    pub cases: u32,
    pub empty: u32,
    pub skipped: u32,
}
