use amml_game::{EntryId, GameEntries, ItemStackLayout, PlayerInventoryRef};

use crate::support::{FakeEntries, GameMemory};

// the values of 6aea437e-7eff13daabb430da (26.62)
const PLAYER_INVENTORY: u64 = 0x5E0;
const ITEMS: u64 = 0x198;
// PlayerInventory::mSelected and PlayerInventory::mInventory
const SELECTED_AT: usize = 0x10;
const INVENTORY_AT: usize = 0xB8;
const STACK: usize = size_of::<ItemStackLayout>();

// A player, its PlayerInventory and the Inventory (a FillingContainer) with a vector of `stacks` ItemStacks.
struct Setup {
    player: GameMemory,
    supplies: GameMemory,
    container: GameMemory,
    stacks: GameMemory,
}

impl Setup {
    fn new(stacks: usize, selected: i32) -> Self {
        let mut setup = Self {
            player: GameMemory::new(0x600),
            supplies: GameMemory::new(0x100),
            container: GameMemory::new(0x200),
            stacks: GameMemory::new(stacks * STACK),
        };
        let first = setup.stacks.ptr() as usize;
        setup.container.put::<usize>(ITEMS as usize, first);
        setup.container.put::<usize>(ITEMS as usize + 8, first + stacks * STACK);
        setup.supplies.put::<i32>(SELECTED_AT, selected);
        setup.supplies.put::<usize>(INVENTORY_AT, setup.container.ptr() as usize);
        setup.player.put::<usize>(PLAYER_INVENTORY as usize, setup.supplies.ptr() as usize);
        setup
    }

    fn entries() -> GameEntries {
        FakeEntries::build(&[
            (EntryId::PLAYER_PLAYER_INVENTORY, PLAYER_INVENTORY),
            (EntryId::FILLING_CONTAINER_ITEMS, ITEMS),
        ])
    }
}

#[test]
fn selected_reads_m_selected() {
    let entries = Setup::entries();
    let mut setup = Setup::new(9, 4);
    // SAFETY: a live fake player whose inventory chain is laid out at the game's offsets.
    let inventory = unsafe { PlayerInventoryRef::of_player(setup.player.ptr(), &entries) }.unwrap();

    assert_eq!(inventory.selected(), Some(4));
}

#[test]
fn item_past_end_is_none() {
    let entries = Setup::entries();
    let mut setup = Setup::new(9, 0);
    // SAFETY: a live fake player whose inventory chain is laid out at the game's offsets.
    let inventory = unsafe { PlayerInventoryRef::of_player(setup.player.ptr(), &entries) }.unwrap();

    assert!(inventory.item(9).is_none());
    assert!(inventory.item(8).is_some());
}

#[test]
fn null_vector_is_none() {
    let entries = Setup::entries();
    let mut setup = Setup::new(9, 0);
    setup.container.put::<usize>(ITEMS as usize, 0);
    setup.container.put::<usize>(ITEMS as usize + 8, 0);
    // SAFETY: a live fake player whose container holds an empty vector.
    let inventory = unsafe { PlayerInventoryRef::of_player(setup.player.ptr(), &entries) }.unwrap();

    assert!(inventory.item(0).is_none());
}

#[test]
fn null_inventory_pointer_is_none() {
    let entries = Setup::entries();
    let mut setup = Setup::new(9, 0);
    setup.supplies.put::<usize>(INVENTORY_AT, 0);

    // SAFETY: a live fake player whose PlayerInventory holds no Inventory.
    assert!(unsafe { PlayerInventoryRef::of_player(setup.player.ptr(), &entries) }.is_none());
}

#[test]
fn missing_entry_is_none() {
    let entries = FakeEntries::build(&[(EntryId::PLAYER_PLAYER_INVENTORY, PLAYER_INVENTORY)]);
    let mut setup = Setup::new(9, 0);

    // SAFETY: a live fake player; without the mItems entry no container is read.
    assert!(unsafe { PlayerInventoryRef::of_player(setup.player.ptr(), &entries) }.is_none());
}

#[test]
fn selected_item_points_at_slot() {
    let entries = Setup::entries();
    let mut setup = Setup::new(9, 4);
    let first = setup.stacks.ptr() as usize;
    // SAFETY: a live fake player whose inventory chain is laid out at the game's offsets.
    let inventory = unsafe { PlayerInventoryRef::of_player(setup.player.ptr(), &entries) }.unwrap();

    assert_eq!(inventory.selected_item().unwrap().as_ptr() as usize, first + 4 * 0x98);
}
