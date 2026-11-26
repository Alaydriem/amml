use crate::ContentsEntry;

/// The contents of one shulker box or bundle that a mod opened, and the mod that owns them.
pub struct ContentsList {
    pub owner: usize,
    pub entries: Vec<ContentsEntry>,
}
