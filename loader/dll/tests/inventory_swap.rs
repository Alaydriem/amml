use std::ffi::c_void;
use std::hint::black_box;
use std::sync::Mutex;

use amml::{InventoryAdapter, UiScope};
use amml_abi::AmmlStatus;
use amml_data::{EntryKind, Resolved};
use amml_runtime::UiScopeKind;

use crate::support::{FakeVtable, Harness};

const PLAYER_INVENTORY_AT: usize = 0x5e0;
const INVENTORY_AT: usize = 0xb8;
const ITEMS_AT: usize = 0x198;
const STACK: usize = 0x98;
const SWAP_SLOT: usize = 5;
// The net id variant of a stack: the id at its offset, the index byte 0x10 later.
const NET_ID_AT: usize = 0x80;

static SWAPS: Mutex<Vec<(i32, i32)>> = Mutex::new(Vec::new());
static GLOBAL: [u64; 2] = [0; 2];

#[inline(never)]
extern "C" fn swap_stand_in(this: *mut c_void, a: i32, b: i32) {
    black_box(this);
    SWAPS.lock().unwrap().push((a, b));
}

#[inline(never)]
extern "C" fn function_stand_in() -> i16 {
    black_box(0)
}

static CONTAINER_VTABLE: FakeVtable<6> = FakeVtable([
    std::ptr::null(),
    std::ptr::null(),
    std::ptr::null(),
    std::ptr::null(),
    std::ptr::null(),
    swap_stand_in as *const (),
]);

fn entries() -> Vec<(&'static str, Resolved)> {
    let function = |address: usize| Resolved {
        kind: EntryKind::Function,
        value: address as u64 - Harness::BASE,
        recipes_agreeing: 2,
    };
    let field = |value| Resolved { kind: EntryKind::Field, value, recipes_agreeing: 2 };
    let slot = |value| Resolved { kind: EntryKind::VSlot, value, recipes_agreeing: 2 };
    let global = |address: usize| Resolved {
        kind: EntryKind::Global,
        value: address as u64 - Harness::BASE,
        recipes_agreeing: 2,
    };
    let stand_in = function_stand_in as *const () as usize;
    vec![
        ("ClientInstance.getLocalPlayer", slot(0)),
        ("Player.playerInventory", field(PLAYER_INVENTORY_AT as u64)),
        ("PlayerInventory.mInventory", field(INVENTORY_AT as u64)),
        ("PlayerInventory.mSelected", field(0x10)),
        ("FillingContainer.mItems", field(ITEMS_AT as u64)),
        ("ItemStack.size", field(STACK as u64)),
        ("ItemStackBase.mItem", field(0x8)),
        ("ItemStackBase.mCount", field(0x22)),
        ("ItemStackBase.getDamageValue", function(stand_in)),
        ("Item.getMaxDamage", slot(0x24)),
        ("Item.mFullName", field(0x20)),
        ("HashedString.layout", field(0)),
        ("Actor.getItemInHandSlot", function(stand_in)),
        ("ItemStackBase.mBlock", field(0x18)),
        ("Block.mDirectData.mLightEmission", field(0x10)),
        ("ItemStack.fromTag", function(stand_in)),
        ("ItemStackBase.dtor", function(stand_in)),
        ("CompoundTag.nodeVariantIndex", field(0x68)),
        ("CompoundTag.vtable", global(std::ptr::from_ref(&GLOBAL) as usize)),
    ]
}

// A player whose container holds 36 stacks. Slots 0 and 1 hold a stack (net ids 100 and 101); the others are empty.
struct Player {
    player: Box<[u64; 0xc0]>,
    _inventory: Box<[u64; 0x20]>,
    _container: Box<[u64; 0x40]>,
    stacks: Vec<u8>,
    _counter: Box<[usize; 1]>,
    _item: Box<[u64; 16]>,
}

impl Player {
    fn new() -> Self {
        let item = Box::new([0u64; 16]);
        let counter = Box::new([item.as_ptr().addr()]);
        let mut stacks = vec![0u8; 36 * STACK];
        for (slot, net_id) in [(0usize, 100i32), (1, 101)] {
            let base = slot * STACK;
            stacks[base + 8..base + 16].copy_from_slice(&counter.as_ptr().addr().to_le_bytes());
            // A stack with user data has no identity, so reading it never reads a name.
            stacks[base + 0x10] = 1;
            stacks[base + 0x22] = 8;
            stacks[base + NET_ID_AT..base + NET_ID_AT + 4].copy_from_slice(&net_id.to_le_bytes());
        }
        let first = stacks.as_ptr().addr();
        let mut container = Box::new([0u64; 0x40]);
        container[0] = std::ptr::from_ref(&CONTAINER_VTABLE).addr() as u64;
        container[ITEMS_AT / 8] = first as u64;
        container[ITEMS_AT / 8 + 1] = (first + stacks.len()) as u64;
        let mut inventory = Box::new([0u64; 0x20]);
        let mut player = Box::new([0u64; 0xc0]);
        player[PLAYER_INVENTORY_AT / 8] = inventory.as_ptr().addr() as u64;
        inventory[INVENTORY_AT / 8] = container.as_ptr().addr() as u64;
        Self { player, _inventory: inventory, _container: container, stacks, _counter: counter, _item: item }
    }

    fn address(&self) -> usize {
        self.player.as_ptr().addr()
    }

    fn stack(&self, slot: usize) -> usize {
        self.stacks.as_ptr().addr() + slot * STACK
    }
}

fn start() -> Harness {
    let harness = Harness::start_with_entries(&entries());
    InventoryAdapter::install(harness.state);
    harness.state.events().gate().open();
    SWAPS.lock().unwrap().clear();
    harness
}

#[test]
fn a_swap_forgets_the_stacks_lent_before_it() {
    let _harness = start();
    let player = Player::new();
    let adapter = InventoryAdapter::get().unwrap();
    let _scope = UiScope::enter(UiScopeKind::ScreenRender { view: 1 });
    let lent = adapter.lend_visual(player.stack(0)).unwrap();
    assert_eq!(lent, player.stack(0));
    assert!(adapter.item_read(std::ptr::null(), std::ptr::with_exposed_provenance(lent)).is_ok());

    assert!(adapter.swap_player_slots(player.address(), SWAP_SLOT, 0, 1, None));

    assert_eq!(*SWAPS.lock().unwrap(), [(0, 1)]);
    assert_eq!(
        adapter.item_read(std::ptr::null(), std::ptr::with_exposed_provenance(lent)).unwrap_err(),
        AmmlStatus::REFUSED
    );
}

#[test]
fn a_swap_that_did_not_run_forgets_nothing() {
    let _harness = start();
    let player = Player::new();
    let adapter = InventoryAdapter::get().unwrap();
    let _scope = UiScope::enter(UiScopeKind::ScreenRender { view: 1 });
    let lent = adapter.lend_visual(player.stack(0)).unwrap();

    // Slot 1 does not hold the net id 999: the put-back is refused.
    assert!(!adapter.swap_player_slots(player.address(), SWAP_SLOT, 0, 1, Some((999, NET_ID_AT))));

    assert!(SWAPS.lock().unwrap().is_empty());
    assert!(adapter.item_read(std::ptr::null(), std::ptr::with_exposed_provenance(lent)).is_ok());
}

#[test]
fn a_put_back_runs_while_the_slot_holds_the_moved_stack() {
    let _harness = start();
    let player = Player::new();
    let adapter = InventoryAdapter::get().unwrap();

    assert!(adapter.swap_player_slots(player.address(), SWAP_SLOT, 0, 1, Some((101, NET_ID_AT))));

    assert_eq!(*SWAPS.lock().unwrap(), [(0, 1)]);
}
