use std::ffi::c_void;

use amml_game::{
    Actor, BlockSource, ChunkBlockPos, ChunkState, ClientInstance, EntryId, GameEntries, LevelChunk, SubChunk,
};

use crate::support::{FakeEntries, GameMemory};

// A fake object whose first qword points to a table of function pointers, as an MSVC object does. The slots are
// numbered from 0; slot 0 is unused.
#[repr(C)]
struct VirtualObject {
    vtable: *const c_void,
    value: *mut c_void,
    flag: u8,
    last_x: i32,
    last_z: i32,
}

impl VirtualObject {
    fn new(table: &'static impl Sized, value: *mut c_void, flag: u8) -> Self {
        Self { vtable: (&raw const *table).cast(), value, flag, last_x: i32::MIN, last_z: i32::MIN }
    }

    fn ptr(&mut self) -> *mut c_void {
        (&raw mut *self).cast()
    }
}

extern "C" fn returns_value(this: *mut c_void) -> *mut c_void {
    // SAFETY: `this` is a VirtualObject.
    unsafe { (*this.cast::<VirtualObject>()).value }
}

extern "C" fn returns_flag(this: *mut c_void) -> u8 {
    // SAFETY: `this` is a VirtualObject.
    unsafe { (*this.cast::<VirtualObject>()).flag }
}

extern "C" fn get_chunk(this: *mut c_void, x: i32, z: i32) -> *mut c_void {
    let object = this.cast::<VirtualObject>();
    // SAFETY: `this` is a VirtualObject.
    unsafe {
        (*object).last_x = x;
        (*object).last_z = z;
        (*object).value
    }
}

// The fake vtables: slot 0 is unused, slot 1 is the call.
#[repr(C)]
struct ValueTable(usize, extern "C" fn(*mut c_void) -> *mut c_void);
#[repr(C)]
struct FlagTable(usize, extern "C" fn(*mut c_void) -> u8);
#[repr(C)]
struct ChunkTable(usize, extern "C" fn(*mut c_void, i32, i32) -> *mut c_void);

static VALUE_TABLE: ValueTable = ValueTable(0, returns_value);
static FLAG_TABLE: FlagTable = FlagTable(0, returns_flag);
static CHUNK_TABLE: ChunkTable = ChunkTable(0, get_chunk);

#[test]
fn region_calls_the_slot_and_a_null_result_or_missing_entry_gives_none() {
    let entries = FakeEntries::build(&[(EntryId::CLIENT_INSTANCE_GET_REGION, 1)]);
    let source = 0x9000 as *mut c_void;
    let mut fake = VirtualObject::new(&VALUE_TABLE, source, 0);
    // SAFETY: a live fake with the slot the entry names.
    let client = unsafe { ClientInstance::from_raw(fake.ptr(), &entries) }.unwrap();
    assert_eq!(client.region().map(|region| region.as_ptr()), Some(source));

    let mut no_world = VirtualObject::new(&VALUE_TABLE, std::ptr::null_mut(), 0);
    // SAFETY: as above.
    let client = unsafe { ClientInstance::from_raw(no_world.ptr(), &entries) }.unwrap();
    assert!(client.region().is_none());

    let none = FakeEntries::build(&[]);
    // SAFETY: the entry is missing, so the vtable is never read.
    let client = unsafe { ClientInstance::from_raw(fake.ptr(), &none) }.unwrap();
    assert!(client.region().is_none());
}

#[test]
fn is_alive_reads_the_bool_of_the_slot() {
    let entries = FakeEntries::build(&[(EntryId::ACTOR_IS_ALIVE, 1)]);
    for alive in [false, true] {
        let mut fake = VirtualObject::new(&FLAG_TABLE, std::ptr::null_mut(), u8::from(alive));
        // SAFETY: a live fake with the slot the entry names.
        let actor = unsafe { Actor::from_raw(fake.ptr(), &entries) }.unwrap();
        assert_eq!(actor.is_alive(), Some(alive));
    }

    let none = FakeEntries::build(&[]);
    let mut fake = VirtualObject::new(&FLAG_TABLE, std::ptr::null_mut(), 1);
    // SAFETY: the entry is missing, so the vtable is never read.
    let actor = unsafe { Actor::from_raw(fake.ptr(), &none) }.unwrap();
    assert_eq!(actor.is_alive(), None);
}

#[test]
fn chunk_passes_exactly_x_and_z_and_a_null_result_gives_none() {
    let entries = FakeEntries::build(&[(EntryId::BLOCK_SOURCE_GET_CHUNK, 1)]);
    let chunk = 0xA000 as *mut c_void;
    let mut fake = VirtualObject::new(&CHUNK_TABLE, chunk, 0);
    // SAFETY: a live fake with the slot the entry names.
    let source = unsafe { BlockSource::from_raw(fake.ptr(), &entries) }.unwrap();

    assert_eq!(source.chunk(3, -2).map(|chunk| chunk.as_ptr()), Some(chunk));
    assert_eq!((fake.last_x, fake.last_z), (3, -2));

    let mut empty = VirtualObject::new(&CHUNK_TABLE, std::ptr::null_mut(), 0);
    // SAFETY: as above.
    let source = unsafe { BlockSource::from_raw(empty.ptr(), &entries) }.unwrap();
    assert!(source.chunk(1, 1).is_none());
    assert_eq!((empty.last_x, empty.last_z), (1, 1));
}

// The `LevelChunk` offsets of one build, from its derive data (`data/`).
struct ChunkOffsets {
    sub_chunks: usize,
    heightmap: usize,
    render_heightmap: usize,
}

// The members that are at the same place on both builds.
const DIMENSION_AT: usize = 0x58;
const MIN_AT: usize = 0x60;
const MAX_AT: usize = 0x6C;
const POSITION_AT: usize = 0x78;
const LOAD_STATE_AT: usize = 0xF0;

const BUILD_2651: ChunkOffsets = ChunkOffsets { sub_chunks: 0x140, heightmap: 0xB80, render_heightmap: 0xD80 };
const BUILD_2660: ChunkOffsets = ChunkOffsets { sub_chunks: 0x1C8, heightmap: 0xC08, render_heightmap: 0xE08 };
const BUILDS: [ChunkOffsets; 2] = [BUILD_2651, BUILD_2660];

/// `SubChunk.size` on both builds.
const SUB_CHUNK_SIZE: usize = 0x68;

// Past the mirrored members: where the fake `getBlock` keeps the block it returns and the position it saw.
const BLOCK_AT: usize = 0x1100;
const LAST_POS_AT: usize = 0x1108;
const CHUNK_LEN: usize = 0x1110;

extern "C" fn fake_get_block(this: *mut c_void, pos: *const ChunkBlockPos) -> *mut c_void {
    let chunk = this.cast::<u8>();
    // SAFETY: `this` is a fake chunk of CHUNK_LEN bytes and `pos` points to the caller's ChunkBlockPos.
    unsafe {
        chunk.add(LAST_POS_AT).cast::<ChunkBlockPos>().write_unaligned(*pos);
        chunk.add(BLOCK_AT).cast::<*mut c_void>().read_unaligned()
    }
}

extern "C" fn fake_is_placeholder(this: *mut c_void) -> u8 {
    // SAFETY: `this` is an element of the fake sub-chunk vector; the first byte is its index.
    u8::from(unsafe { *this.cast::<u8>() } % 2 == 1)
}

fn chunk_entries_with(build: &ChunkOffsets, sub_chunk_size: usize) -> GameEntries {
    FakeEntries::build(&[
        (EntryId::LEVEL_CHUNK_SUB_CHUNKS, build.sub_chunks as u64),
        (
            EntryId::LEVEL_CHUNK_GET_BLOCK,
            fake_get_block as extern "C" fn(*mut c_void, *const ChunkBlockPos) -> *mut c_void as usize as u64,
        ),
        (EntryId::SUB_CHUNK_SIZE, sub_chunk_size as u64),
        (
            EntryId::SUB_CHUNK_IS_PLACE_HOLDER_SUB_CHUNK,
            fake_is_placeholder as extern "C" fn(*mut c_void) -> u8 as usize as u64,
        ),
    ])
}

fn chunk_entries(build: &ChunkOffsets) -> GameEntries {
    chunk_entries_with(build, SUB_CHUNK_SIZE)
}

// A chunk laid out as the game's on `build`, with `count` sub-chunks. The storage has one guard element past the
// end, filled with 0xEE, to show a read past the vector end.
fn fake_chunk(build: &ChunkOffsets, storage: &mut Vec<u8>, count: usize) -> GameMemory {
    *storage = vec![0xEE; (count + 1) * SUB_CHUNK_SIZE];
    for i in 0..count {
        storage[i * SUB_CHUNK_SIZE] = i as u8;
    }
    let begin = storage.as_mut_ptr();
    let end = begin.wrapping_add(count * SUB_CHUNK_SIZE);
    let mut chunk = GameMemory::new(CHUNK_LEN);
    chunk.put(MIN_AT, [0_i32, -64, 0]);
    chunk.put(MAX_AT, [15_i32, 319, 15]);
    chunk.put(POSITION_AT, [-7_i32, 12]);
    chunk.put(LOAD_STATE_AT, ChunkState::LOADED);
    chunk.put(build.sub_chunks, [begin, end, end]);
    for i in 0..256 {
        chunk.put(build.heightmap + 2 * i, i as i16 - 64);
        chunk.put(build.render_heightmap + 2 * i, 2 * i as i16);
    }
    chunk
}

#[test]
fn the_members_come_from_the_mirror_of_each_build() {
    for build in &BUILDS {
        let entries = chunk_entries(build);
        let mut storage = Vec::new();
        let mut fake = fake_chunk(build, &mut storage, 24);
        let dimension = 0xD000 as *mut c_void;
        fake.put(DIMENSION_AT, dimension);
        // SAFETY: a live fake chunk laid out as the game's on this build.
        let chunk = unsafe { LevelChunk::from_raw(fake.ptr(), &entries) }.unwrap();

        assert_eq!(chunk.position(), Some([-7, 12]));
        assert_eq!(chunk.min_y(), Some(-64));
        assert_eq!(chunk.max_y(), Some(319));
        assert_eq!(chunk.dimension().map(|dimension| dimension.as_ptr()), Some(dimension));
        assert!(chunk.loaded());
        assert_eq!(chunk.height(0), Some(-64));
        assert_eq!(chunk.height(255), Some(191));
        assert_eq!(chunk.render_height(255), Some(510));
        assert_eq!(chunk.sub_chunk_count(), 24);
    }
}

#[test]
fn an_unknown_sub_chunk_offset_or_a_missing_entry_selects_no_mirror() {
    let unknown = FakeEntries::build(&[(EntryId::LEVEL_CHUNK_SUB_CHUNKS, 0x150), (EntryId::SUB_CHUNK_SIZE, 0x68)]);
    let none = FakeEntries::build(&[]);
    let mut storage = Vec::new();
    let mut fake = fake_chunk(&BUILD_2651, &mut storage, 4);
    for entries in [unknown, none] {
        // SAFETY: no mirror is selected, so nothing is read.
        let chunk = unsafe { LevelChunk::from_raw(fake.ptr(), &entries) }.unwrap();

        assert_eq!(chunk.position(), None);
        assert_eq!(chunk.min_y(), None);
        assert_eq!(chunk.max_y(), None);
        assert!(chunk.dimension().is_none());
        assert!(!chunk.loaded());
        assert_eq!(chunk.height(0), None);
        assert_eq!(chunk.sub_chunk_count(), 0);
        assert!(chunk.sub_chunk(0).is_none());
        assert!(chunk.block(ChunkBlockPos { x: 0, z: 0, y: 0 }).is_none());
    }
}

#[test]
fn height_at_256_gives_none() {
    let entries = chunk_entries(&BUILD_2651);
    let mut storage = Vec::new();
    let mut fake = fake_chunk(&BUILD_2651, &mut storage, 1);
    // SAFETY: a live fake chunk laid out as the game's.
    let chunk = unsafe { LevelChunk::from_raw(fake.ptr(), &entries) }.unwrap();

    assert_eq!(chunk.height(256), None);
    assert_eq!(chunk.render_height(256), None);
    assert_eq!(chunk.height(usize::MAX), None);
}

#[test]
fn loaded_is_false_for_state_10_and_true_for_11() {
    let entries = chunk_entries(&BUILD_2660);
    let mut storage = Vec::new();
    let mut fake = fake_chunk(&BUILD_2660, &mut storage, 1);
    let raw = fake.ptr();
    // SAFETY: a live fake chunk laid out as the game's.
    let chunk = unsafe { LevelChunk::from_raw(raw, &entries) }.unwrap();
    assert!(chunk.loaded());

    for (state, loaded) in [(10u8, false), (11, true), (0, false), (255, false)] {
        // SAFETY: `raw` points to the live fake; the wrapper only reads.
        unsafe { raw.cast::<u8>().add(LOAD_STATE_AT).write(state) };
        assert_eq!(chunk.loaded(), loaded, "state {state}");
    }
}

#[test]
fn sub_chunk_at_the_count_gives_none_and_no_read_happens_past_the_end() {
    for build in &BUILDS {
        let entries = chunk_entries(build);
        let mut storage = Vec::new();
        let mut fake = fake_chunk(build, &mut storage, 3);
        // SAFETY: a live fake chunk laid out as the game's.
        let chunk = unsafe { LevelChunk::from_raw(fake.ptr(), &entries) }.unwrap();

        assert_eq!(chunk.sub_chunk_count(), 3);
        for i in 0..3 {
            let sub: SubChunk = chunk.sub_chunk(i).unwrap();
            // The stride is the SubChunk mirror size: each wrapper starts at its element, whose first byte the
            // fake placeholder test reads.
            assert_eq!(sub.as_ptr(), storage.as_mut_ptr().wrapping_add(i * SUB_CHUNK_SIZE).cast());
            assert_eq!(sub.is_placeholder(), Some(i % 2 == 1));
        }
        // The element past the end holds 0xEE, which the fake would call a placeholder (odd), so a wrapper for it
        // would show as `Some(false)` or `Some(true)` and not as `None`.
        assert!(chunk.sub_chunk(3).is_none());
        assert!(chunk.sub_chunk(4).is_none());
    }
}

#[test]
fn a_sub_chunk_size_entry_other_than_the_mirror_gives_no_sub_chunks() {
    let entries = chunk_entries_with(&BUILD_2651, 0x70);
    let mut storage = Vec::new();
    let mut fake = fake_chunk(&BUILD_2651, &mut storage, 3);
    // SAFETY: a live fake chunk laid out as the game's.
    let chunk = unsafe { LevelChunk::from_raw(fake.ptr(), &entries) }.unwrap();

    assert_eq!(chunk.sub_chunk_count(), 0);
    assert!(chunk.sub_chunk(0).is_none());
    assert_eq!(chunk.min_y(), Some(-64));
}

#[test]
fn an_empty_or_unallocated_vector_has_no_sub_chunks() {
    let entries = chunk_entries(&BUILD_2651);
    let mut storage = Vec::new();
    let mut fake = fake_chunk(&BUILD_2651, &mut storage, 0);
    let raw = fake.ptr();
    // SAFETY: a live fake chunk laid out as the game's.
    let chunk = unsafe { LevelChunk::from_raw(raw, &entries) }.unwrap();
    assert_eq!(chunk.sub_chunk_count(), 0);
    assert!(chunk.sub_chunk(0).is_none());

    // SAFETY: `raw` points to the live fake; the wrapper only reads.
    unsafe { raw.cast::<u8>().add(BUILD_2651.sub_chunks).cast::<[usize; 3]>().write_unaligned([0; 3]) };
    assert_eq!(chunk.sub_chunk_count(), 0);
    assert!(chunk.sub_chunk(0).is_none());
}

#[test]
fn block_passes_the_four_byte_position_by_reference() {
    assert_eq!(size_of::<ChunkBlockPos>(), 4);

    let entries = chunk_entries(&BUILD_2651);
    let mut storage = Vec::new();
    let mut fake = fake_chunk(&BUILD_2651, &mut storage, 1);
    let block = 0xB000 as *mut c_void;
    fake.put(BLOCK_AT, block);
    let raw = fake.ptr();
    // SAFETY: a live fake chunk laid out as the game's.
    let chunk = unsafe { LevelChunk::from_raw(raw, &entries) }.unwrap();
    let found = chunk.block(ChunkBlockPos { x: 15, z: 0, y: -64 }).map(|block| block.as_ptr());

    assert_eq!(found, Some(block));
    assert_eq!(fake.get::<ChunkBlockPos>(LAST_POS_AT), ChunkBlockPos { x: 15, z: 0, y: -64 });

    fake.put(BLOCK_AT, std::ptr::null_mut::<c_void>());
    // SAFETY: as above.
    let chunk = unsafe { LevelChunk::from_raw(raw, &entries) }.unwrap();
    assert!(chunk.block(ChunkBlockPos { x: 1, z: 2, y: 3 }).is_none());
}

#[test]
fn block_gives_none_for_a_zero_function_entry() {
    let entries = FakeEntries::build(&[(EntryId::LEVEL_CHUNK_GET_BLOCK, 0)]);
    let mut storage = Vec::new();
    let mut fake = fake_chunk(&BUILD_2651, &mut storage, 1);
    fake.put(BLOCK_AT, 0xB000 as *mut c_void);
    // SAFETY: a live fake chunk; the zero entry is never called.
    let chunk = unsafe { LevelChunk::from_raw(fake.ptr(), &entries) }.unwrap();
    assert!(chunk.block(ChunkBlockPos { x: 0, z: 0, y: 0 }).is_none());
}
