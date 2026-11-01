use amml_game::{Block, EntryId, GameEntries, MaterialLayout};

use crate::support::{FakeBlock, FakeEntries, GameMemory};

// The offsets of the 26.51 and 26.60 derive data (`data/`): `Block.mBlockType`, the Material of `BlockType`
// (a pointer on 26.51, a value on 26.60), `Material.mSolid` (26.51), `Material.mFlags` and `Material.solidMask`
// (26.60).
const BLOCK_TYPE_AT: usize = 0x68;
const MATERIAL_AT: usize = 0x28;
const SOLID_AT: usize = 5;
const FLAGS_AT: usize = 1;
const SOLID_MASK: u8 = 0x10;

fn layout_entries(inline: u64) -> GameEntries {
    FakeEntries::build(&[(EntryId::BLOCK_TYPE_MATERIAL_INLINE, inline)])
}

// A block whose block type is `block_type`.
fn block_with(block_type: &mut GameMemory) -> GameMemory {
    let mut block = GameMemory::new(0x80);
    block.put(BLOCK_TYPE_AT, block_type.ptr());
    block
}

#[test]
fn type_name_reads_a_short_string_and_a_heap_string() {
    let entries = FakeEntries::build(&[]);
    let heap_name = "minecraft:a_block_with_a_rather_long_name";
    assert_eq!(heap_name.len(), 41);

    for name in ["minecraft:stone", heap_name] {
        let mut fake = FakeBlock::named(name);
        // SAFETY: a live fake block laid out as the game's.
        let block = unsafe { Block::from_raw(fake.ptr(), &entries) }.unwrap();

        assert_eq!(block.type_name().unwrap().to_str().unwrap(), name);
    }
}

#[test]
fn a_null_block_type_gives_none() {
    let entries = FakeEntries::build(&[]);
    let mut no_type = FakeBlock::without_type();
    // SAFETY: a live fake block laid out as the game's.
    let block = unsafe { Block::from_raw(no_type.ptr(), &entries) }.unwrap();
    assert!(block.type_name().is_none());
    assert!(block.block_type_ptr().is_none());
}

#[test]
fn a_misaligned_block_type_gives_no_name() {
    let entries = FakeEntries::build(&[]);
    let mut block = GameMemory::new(0x80);
    block.put(BLOCK_TYPE_AT, 0x1001_usize);
    // SAFETY: a live fake block; the misaligned block type is never read.
    let wrapper = unsafe { Block::from_raw(block.ptr(), &entries) }.unwrap();

    assert_eq!(wrapper.block_type_ptr(), Some(0x1001 as *mut std::ffi::c_void));
    assert!(wrapper.type_name().is_none());
}

#[test]
fn is_solid_on_26_51_follows_the_material_pointer_and_reads_its_bool() {
    let entries = layout_entries(0);
    for (solid, expected) in [(0u8, false), (1, true), (2, true)] {
        let mut material = GameMemory::new(0x10);
        material.put(SOLID_AT, solid);
        let mut block_type = GameMemory::new(0x110);
        block_type.put(MATERIAL_AT, material.ptr());
        let mut block = block_with(&mut block_type);
        // SAFETY: a live fake block laid out as the game's on 26.51.
        let wrapper = unsafe { Block::from_raw(block.ptr(), &entries) }.unwrap();
        assert_eq!(wrapper.is_solid(), Some(expected), "solid byte {solid}");
    }

    let mut no_material = GameMemory::new(0x110);
    let mut block = block_with(&mut no_material);
    // SAFETY: as above; the null Material is never followed.
    let wrapper = unsafe { Block::from_raw(block.ptr(), &entries) }.unwrap();
    assert_eq!(wrapper.is_solid(), None);
}

#[test]
fn is_solid_on_26_60_reads_the_solid_bit_of_the_material_held_by_value() {
    let entries = layout_entries(1);
    // solid, blocksMotion only, solid and liquid, nothing
    for (flags, expected) in [(SOLID_MASK, true), (0x04, false), (SOLID_MASK | 0x02, true), (0x00, false)] {
        let mut block_type = GameMemory::new(0x110);
        block_type.put(MATERIAL_AT, 5_u8);
        block_type.put(MATERIAL_AT + FLAGS_AT, flags);
        let mut block = block_with(&mut block_type);
        // SAFETY: a live fake block laid out as the game's on 26.60.
        let wrapper = unsafe { Block::from_raw(block.ptr(), &entries) }.unwrap();
        assert_eq!(wrapper.is_solid(), Some(expected), "flags {flags:#x}");
    }
}

#[test]
fn is_solid_is_none_for_a_null_block_type_an_unknown_layout_or_no_layout_entry() {
    let value_layout = layout_entries(1);
    let mut block = GameMemory::new(0x80);
    // SAFETY: a live fake block with a null block type.
    let wrapper = unsafe { Block::from_raw(block.ptr(), &value_layout) }.unwrap();
    assert_eq!(wrapper.is_solid(), None);

    let mut block_type = GameMemory::new(0x110);
    block_type.put(MATERIAL_AT + FLAGS_AT, SOLID_MASK);
    let mut block = block_with(&mut block_type);
    for entries in [layout_entries(2), FakeEntries::build(&[])] {
        // SAFETY: a live fake block; no layout is selected, so the block type is not read.
        let wrapper = unsafe { Block::from_raw(block.ptr(), &entries) }.unwrap();
        assert_eq!(wrapper.is_solid(), None);
    }
    assert!(MaterialLayout::of(&layout_entries(2)).is_err());
}
