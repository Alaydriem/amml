/// What a running network event holds: the entries of a player list update, as the addresses of their elements in the
/// game's packet, or the uuid of a player skin change.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NetworkEventKind {
    List { elements: Vec<usize> },
    Skin { uuid: [u8; 16] },
}
