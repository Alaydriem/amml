use std::mem::offset_of;

use amml::{ChunkLayout2651, ChunkLayout2660, ChunkLayoutKind, SubChunkLayout};

#[test]
fn layout_2651_offsets_match_the_amml_game_mirror() {
    // The offsets of `LevelChunkLayout2660` of `amml-game`, which fits the build whose `mSubChunks` is at 1C8h.
    assert_eq!(offset_of!(ChunkLayout2651, dimension), 0x58);
    assert_eq!(offset_of!(ChunkLayout2651, min), 0x60);
    assert_eq!(offset_of!(ChunkLayout2651, max), 0x6C);
    assert_eq!(offset_of!(ChunkLayout2651, position), 0x78);
    assert_eq!(offset_of!(ChunkLayout2651, load_state), 0xF0);
    assert_eq!(offset_of!(ChunkLayout2651, sub_chunks), 0x1C8);
    assert_eq!(offset_of!(ChunkLayout2651, heightmap), 0xC08);
    assert_eq!(offset_of!(ChunkLayout2651, render_heightmap), 0xE08);
    assert_eq!(SubChunkLayout::SIZE, 0x68);
    let offsets = ChunkLayoutKind::Build2651.offsets();
    assert_eq!((offsets.min, offsets.max, offsets.load_state), (0x60, 0x6C, 0xF0));
    assert_eq!((offsets.sub_chunks, offsets.heightmap, offsets.render_heightmap), (0x1C8, 0xC08, 0xE08));
}

#[test]
fn layout_2660_offsets_are_the_values_read_from_the_70_26_exe() {
    assert_eq!(offset_of!(ChunkLayout2660, min), 0x60);
    assert_eq!(offset_of!(ChunkLayout2660, load_state), 0xF0);
    assert_eq!(offset_of!(ChunkLayout2660, sub_chunks), 0x228);
    assert_eq!(offset_of!(ChunkLayout2660, heightmap), 0xC68);
    assert_eq!(offset_of!(ChunkLayout2660, render_heightmap), 0xE68);
    let offsets = ChunkLayoutKind::Build2660.offsets();
    assert_eq!((offsets.sub_chunks, offsets.heightmap, offsets.render_heightmap), (0x228, 0xC68, 0xE68));
}

#[test]
fn the_sub_chunks_value_picks_the_layout() {
    assert_eq!(ChunkLayoutKind::of(0x1C8), Some(ChunkLayoutKind::Build2651));
    assert_eq!(ChunkLayoutKind::of(0x228), Some(ChunkLayoutKind::Build2660));
}

#[test]
fn unknown_sub_chunks_value_offers_no_layout() {
    // 140h is the value of 26.51, which the loader does not support; 0 is an entry that was never derived.
    for value in [0, 0x140, 0x1C0, 0x230, u64::MAX] {
        assert_eq!(ChunkLayoutKind::of(value), None, "{value:#x}");
    }
}
