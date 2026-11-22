use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

use amml_data::{EntryKind, Resolved};

use super::{BlockGame, ChunkGame, Harness};

// `canBeBuiltOver` calls: (block type, block, block source, position).
type BuiltOverCall = (usize, usize, usize, [i32; 3]);
static BUILT_OVER_CALLS: Mutex<Vec<BuiltOverCall>> = Mutex::new(Vec::new());
static INTERACTIVE_CALLS: AtomicUsize = AtomicUsize::new(0);

/// The `world.blocks` stand-ins over `BlockGame`: `BlockSource::getBlock` (`ChunkGame::block_source_get_block`), the
/// `canBeBuiltOver` and `isInteractiveBlock` slots of the stand-in block types, and the two entries of the item path.
pub struct BlockReadGame;

impl BlockReadGame {
    pub const BUILT_OVER_SLOT: usize = 0x54;
    pub const INTERACTIVE_SLOT: usize = 0x78;
    pub const AS_ITEM_SLOT: u64 = 0x4E;

    /// The listener and chunk entries and the `world.blocks` entries; `without` names entries to leave out. Puts the
    /// two functions in the slots of every stand-in block type and clears the recorded calls.
    pub fn entries(without: &[&str]) -> Vec<(&'static str, Resolved)> {
        let mut entries = BlockGame::entries(without);
        BUILT_OVER_CALLS.lock().unwrap().clear();
        INTERACTIVE_CALLS.store(0, Ordering::SeqCst);
        ChunkGame::set_type_slot(Self::BUILT_OVER_SLOT, Self::can_be_built_over as *const () as usize);
        ChunkGame::set_type_slot(Self::INTERACTIVE_SLOT, Self::is_interactive as *const () as usize);
        let function = |address: usize| (EntryKind::Function, address as u64 - Harness::BASE);
        let more = [
            ("BlockSource.getBlock", function(ChunkGame::block_source_get_block as *const () as usize)),
            ("BlockType.canBeBuiltOver", (EntryKind::VSlot, Self::BUILT_OVER_SLOT as u64)),
            ("BlockType.isInteractiveBlock", (EntryKind::VSlot, Self::INTERACTIVE_SLOT as u64)),
            ("BlockType.asItemInstance", (EntryKind::VSlot, Self::AS_ITEM_SLOT)),
            ("ItemStackBase.dtor", function(Self::item_dtor as *const () as usize)),
        ];
        for (name, (kind, value)) in more {
            if !without.contains(&name) {
                entries.push((name, Resolved { kind, value, recipes_agreeing: 2 }));
            }
        }
        entries
    }

    /// The `canBeBuiltOver` calls, in order: the block type, the block, the block source and the position.
    pub fn built_over_calls() -> Vec<BuiltOverCall> {
        BUILT_OVER_CALLS.lock().unwrap().clone()
    }

    /// The number of `isInteractiveBlock` calls.
    pub fn interactive_calls() -> usize {
        INTERACTIVE_CALLS.load(Ordering::SeqCst)
    }

    // `bool BlockType::canBeBuiltOver(Block const&, BlockSource const&, BlockPos const&) const`: true.
    extern "C" fn can_be_built_over(this: usize, block: usize, source: usize, pos: *const [i32; 3]) -> u8 {
        // SAFETY: the adapter passes a live `BlockPos` of three ints.
        let pos = unsafe { pos.read() };
        BUILT_OVER_CALLS.lock().unwrap().push((this, block, source, pos));
        1
    }

    // `bool BlockType::isInteractiveBlock() const`: true.
    extern "C" fn is_interactive(_this: usize) -> u8 {
        INTERACTIVE_CALLS.fetch_add(1, Ordering::SeqCst);
        1
    }

    // `ItemStackBase::~ItemStackBase`: nothing.
    extern "C" fn item_dtor(_stack: usize) {}
}
