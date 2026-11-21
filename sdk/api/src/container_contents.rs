use crate::ContainerEntry;

/// The contents of a shulker box or a bundle: one entry for each stack the game could make from its tag, in the order
/// of the tag. An entry of an item the game does not know is left out. Dropping the contents drops every draw handle
/// in it.
#[derive(Debug, Default)]
pub struct ContainerContents {
    pub entries: Vec<ContainerEntry>,
}
