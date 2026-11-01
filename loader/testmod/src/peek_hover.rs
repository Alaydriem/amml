/// The container slot the pointer is over: the controller that reported it, the collection name and the slot index.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PeekHover {
    pub controller: usize,
    pub collection: String,
    pub slot: i32,
}
