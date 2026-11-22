/// The creative menu entry of an item, as a pick needs it: its net id and the most items of one stack.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CreativeEntry {
    pub creative_net_id: u32,
    pub count: u8,
}
