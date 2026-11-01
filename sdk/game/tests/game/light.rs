use std::ffi::c_void;

use amml_game::{Aabb, Block, BlockPos, BlockSource, Brightness, Dimension, EntryId};

use crate::support::{FakeBlock, FakeEntries, GameMemory};

// A fake BlockSource whose `getBrightnessPair` records the position it saw and writes sky then block light.
#[repr(C)]
struct LightSource {
    seen: BlockPos,
    bytes: [u8; 2],
}

extern "C" fn get_brightness_pair(this: *mut c_void, out: *mut [u8; 2], pos: *const BlockPos) -> *mut [u8; 2] {
    // SAFETY: `this` is a LightSource, `pos` is the caller's position and `out` is the caller's return buffer.
    unsafe {
        let source = this.cast::<LightSource>();
        (*source).seen = *pos;
        *out = (*source).bytes;
    }
    out
}

fn light_entries() -> amml_game::GameEntries {
    FakeEntries::build(&[(
        EntryId::BLOCK_SOURCE_GET_BRIGHTNESS_PAIR,
        get_brightness_pair as extern "C" fn(*mut c_void, *mut [u8; 2], *const BlockPos) -> *mut [u8; 2] as usize
            as u64,
    )])
}

#[test]
fn brightness_passes_the_position_by_reference_and_reads_the_hidden_buffer() {
    assert_eq!(size_of::<BlockPos>(), 12);
    assert_eq!(size_of::<Brightness>(), 2);

    let entries = light_entries();
    let mut fake = LightSource { seen: BlockPos { x: 0, y: 0, z: 0 }, bytes: [15, 7] };
    // SAFETY: a live fake with the function the entry names.
    let source = unsafe { BlockSource::from_raw((&raw mut fake).cast(), &entries) }.unwrap();

    let light = source.brightness(&BlockPos { x: 1, y: -64, z: 2 }).unwrap();

    assert_eq!(light, Brightness { sky: 15, block: 7 });
    assert_eq!(fake.seen, BlockPos { x: 1, y: -64, z: 2 });
}

#[test]
fn brightness_is_none_without_the_entry_or_with_a_zero_address() {
    let mut fake = LightSource { seen: BlockPos { x: 0, y: 0, z: 0 }, bytes: [1, 2] };
    for entries in [FakeEntries::build(&[]), FakeEntries::build(&[(EntryId::BLOCK_SOURCE_GET_BRIGHTNESS_PAIR, 0)])] {
        // SAFETY: the function is never called.
        let source = unsafe { BlockSource::from_raw((&raw mut fake).cast(), &entries) }.unwrap();
        assert!(source.brightness(&BlockPos { x: 0, y: 0, z: 0 }).is_none());
    }
}

// A fake Block laid out as the game's: the bytes before `mBlockType`, the block type pointer at 68h (26.51 and
// 26.60), then the last face the support function saw.
#[repr(C)]
struct SupportBlock {
    pad: [u8; 0x68],
    block_type: *mut c_void,
    last_face: u8,
    answer: u8,
}

extern "C" fn can_provide_full_support(this: *mut c_void, face: u8) -> u8 {
    // SAFETY: `this` is a SupportBlock.
    unsafe {
        let block = this.cast::<SupportBlock>();
        (*block).last_face = face;
        (*block).answer
    }
}

#[test]
fn full_support_passes_the_face_byte_and_reads_the_answer() {
    let entries = FakeEntries::build(&[(
        EntryId::BLOCK_CAN_PROVIDE_FULL_SUPPORT,
        can_provide_full_support as extern "C" fn(*mut c_void, u8) -> u8 as usize as u64,
    )]);
    for (answer, expected) in [(1, true), (0, false)] {
        let mut fake = SupportBlock { pad: [0; 0x68], block_type: std::ptr::null_mut(), last_face: 99, answer };
        let raw = &raw mut fake;
        // SAFETY: a live fake with the function the entry names.
        let block = unsafe { Block::from_raw(raw.cast(), &entries) }.unwrap();

        assert_eq!(block.can_provide_full_support(1), Some(expected));
        // SAFETY: `raw` points to the live fake; the call is over.
        assert_eq!(unsafe { (*raw).last_face }, 1);
    }
}

#[test]
fn full_support_is_none_without_the_entry() {
    let entries = FakeEntries::build(&[]);
    let mut fake = SupportBlock { pad: [0; 0x68], block_type: std::ptr::null_mut(), last_face: 0, answer: 1 };
    // SAFETY: the function is never called.
    let block = unsafe { Block::from_raw((&raw mut fake).cast(), &entries) }.unwrap();

    assert!(block.can_provide_full_support(1).is_none());
}

// A fake BlockType whose `getCollisionShape` is slot 4: it records its arguments and writes `shape`.
#[repr(C)]
struct ShapeType {
    vtable: *const c_void,
    shape: [f32; 6],
    block: *const c_void,
    region: *const c_void,
    pos: BlockPos,
    entity: *const c_void,
}

type ShapeFn = extern "C" fn(
    *mut c_void,
    *mut [f32; 6],
    *const c_void,
    *const c_void,
    *const BlockPos,
    *const c_void,
) -> *mut [f32; 6];

#[repr(C)]
struct ShapeTable([usize; 4], ShapeFn);

extern "C" fn get_collision_shape(
    this: *mut c_void,
    out: *mut [f32; 6],
    block: *const c_void,
    region: *const c_void,
    pos: *const BlockPos,
    entity: *const c_void,
) -> *mut [f32; 6] {
    // SAFETY: `this` is a ShapeType, `pos` is the caller's position and `out` is the caller's return buffer.
    unsafe {
        let block_type = this.cast::<ShapeType>();
        (*block_type).block = block;
        (*block_type).region = region;
        (*block_type).pos = *pos;
        (*block_type).entity = entity;
        *out = (*block_type).shape;
    }
    out
}

static SHAPE_TABLE: ShapeTable = ShapeTable([0; 4], get_collision_shape);

fn shape_entries() -> amml_game::GameEntries {
    FakeEntries::build(&[(EntryId::BLOCK_TYPE_GET_COLLISION_SHAPE, 4)])
}

fn shape_type(shape: [f32; 6]) -> ShapeType {
    ShapeType {
        vtable: (&raw const SHAPE_TABLE).cast(),
        shape,
        block: std::ptr::null(),
        region: std::ptr::null(),
        pos: BlockPos { x: 0, y: 0, z: 0 },
        entity: std::ptr::dangling(),
    }
}

#[test]
fn collision_shape_passes_the_arguments_with_a_null_entity_and_reads_the_box() {
    assert_eq!(size_of::<Aabb>(), 24);

    let entries = shape_entries();
    let mut block_type = shape_type([0.0, 0.0, 0.0, 1.0, 0.5, 1.0]);
    let mut block = SupportBlock { pad: [0; 0x68], block_type: (&raw mut block_type).cast(), last_face: 0, answer: 0 };
    let mut region = LightSource { seen: BlockPos { x: 0, y: 0, z: 0 }, bytes: [0, 0] };
    // SAFETY: live fakes with the layouts the entries name.
    let (wrapped_block, wrapped_region) = unsafe {
        (
            Block::from_raw((&raw mut block).cast(), &entries).unwrap(),
            BlockSource::from_raw((&raw mut region).cast(), &entries).unwrap(),
        )
    };

    let shape = wrapped_block.collision_shape(&wrapped_region, &BlockPos { x: 4, y: 70, z: -9 }).unwrap();

    assert_eq!(shape, Aabb { min: [0.0, 0.0, 0.0], max: [1.0, 0.5, 1.0] });
    assert!(!shape.is_empty());
    assert_eq!(block_type.block, (&raw const block).cast::<c_void>());
    assert_eq!(block_type.region, (&raw const region).cast::<c_void>());
    assert_eq!(block_type.pos, BlockPos { x: 4, y: 70, z: -9 });
    assert!(block_type.entity.is_null());
}

#[test]
fn a_box_whose_minimum_equals_its_maximum_is_empty_at_any_position() {
    for (shape, empty) in [
        ([0.0; 6], true),
        ([4.0, 70.0, -9.0, 4.0, 70.0, -9.0], true),
        ([0.0, 0.0, 0.0, 1.0, 0.0, 1.0], false),
        ([4.0, 70.0, -9.0, 4.0, 70.5, -9.0], false),
    ] {
        let entries = shape_entries();
        let mut block_type = shape_type(shape);
        let mut block =
            SupportBlock { pad: [0; 0x68], block_type: (&raw mut block_type).cast(), last_face: 0, answer: 0 };
        let mut region = LightSource { seen: BlockPos { x: 0, y: 0, z: 0 }, bytes: [0, 0] };
        // SAFETY: live fakes with the layouts the entries name.
        let (wrapped_block, wrapped_region) = unsafe {
            (
                Block::from_raw((&raw mut block).cast(), &entries).unwrap(),
                BlockSource::from_raw((&raw mut region).cast(), &entries).unwrap(),
            )
        };

        let found = wrapped_block.collision_shape(&wrapped_region, &BlockPos { x: 0, y: 0, z: 0 }).unwrap();

        assert_eq!(found.is_empty(), empty, "{shape:?}");
    }
}

#[test]
fn collision_shape_is_none_for_a_null_block_type_or_a_missing_entry() {
    let entries = shape_entries();
    let mut block = SupportBlock { pad: [0; 0x68], block_type: std::ptr::null_mut(), last_face: 0, answer: 0 };
    let mut region = LightSource { seen: BlockPos { x: 0, y: 0, z: 0 }, bytes: [0, 0] };
    // SAFETY: live fakes; the null block type is never followed.
    let (wrapped_block, wrapped_region) = unsafe {
        (
            Block::from_raw((&raw mut block).cast(), &entries).unwrap(),
            BlockSource::from_raw((&raw mut region).cast(), &entries).unwrap(),
        )
    };
    assert!(wrapped_block.collision_shape(&wrapped_region, &BlockPos { x: 0, y: 0, z: 0 }).is_none());

    let none = FakeEntries::build(&[]);
    let mut block_type = shape_type([1.0; 6]);
    block.block_type = (&raw mut block_type).cast();
    // SAFETY: the slot entry is missing, so the block type is not called.
    let wrapped_block = unsafe { Block::from_raw((&raw mut block).cast(), &none) }.unwrap();
    assert!(wrapped_block.collision_shape(&wrapped_region, &BlockPos { x: 0, y: 0, z: 0 }).is_none());
}

#[test]
fn sky_darken_reads_the_byte_at_its_place_on_both_builds() {
    // `Dimension.mSkyDarken` in the 26.51 and 26.60 derive data
    const SKY_DARKEN_AT: usize = 0x1A9;
    let mut memory = GameMemory::new(0x1C0);
    memory.put(SKY_DARKEN_AT - 1, 1_u8);
    memory.put(SKY_DARKEN_AT, 11_u8);
    memory.put(SKY_DARKEN_AT + 1, 99_u8);
    let entries = FakeEntries::build(&[]);
    // SAFETY: a live buffer laid out as the game's Dimension up to `mSkyDarken`.
    let dimension = unsafe { Dimension::from_raw(memory.ptr(), &entries) }.unwrap();
    assert_eq!(dimension.sky_darken(), Some(11));
}

#[test]
fn is_air_is_true_only_for_the_air_type_name() {
    let entries = FakeEntries::build(&[]);
    for (name, expected) in
        [("minecraft:air", true), ("minecraft:stone", false), ("minecraft:air_bubble_with_a_long_name_x", false)]
    {
        let mut fake = FakeBlock::named(name);
        // SAFETY: a live fake block with the layout the entries name.
        let block = unsafe { Block::from_raw(fake.ptr(), &entries) }.unwrap();

        assert_eq!(block.is_air(), Some(expected), "{name}");
    }

    let mut no_type = FakeBlock::without_type();
    // SAFETY: a live fake block with a null block type.
    let block = unsafe { Block::from_raw(no_type.ptr(), &entries) }.unwrap();
    assert_eq!(block.is_air(), None);
}
