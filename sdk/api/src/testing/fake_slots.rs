use crate::testing::FakeItem;

/// The three slot collections of a container screen a `FakeGame` serves (see `FakeScreen`): the slots of the
/// container, the nine hotbar slots and the 27 main inventory slots, `None` for an empty slot. A collection the test
/// leaves empty (a `Vec` of length 0) has no slots: its `ContainerScreen::slots` gives an empty list, and a slot index
/// in it is refused as outside the collection.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FakeSlots {
    pub container: Vec<Option<FakeItem>>,
    pub hotbar: Vec<Option<FakeItem>>,
    pub inventory: Vec<Option<FakeItem>>,
}
