use amml_game::{BlockPos, HitResultLayout};

use crate::support::GameMemory;

#[test]
fn reads_the_type_facing_block_and_position_at_the_hit_result_offsets() {
    let mut memory = GameMemory::new(0x80);
    // HitResult: mType 0x18, mFacing 0x1C, mBlock 0x20, mPos 0x2C
    memory.put::<i32>(0x18, 0);
    memory.put::<u8>(0x1C, 5);
    memory.put::<[i32; 3]>(0x20, [10, -64, 300]);
    memory.put::<[f32; 3]>(0x2C, [10.5, -63.0, 300.25]);

    // SAFETY: the buffer is 16-byte aligned, zeroed and larger than the mirror's prefix.
    let hit = unsafe { &*memory.ptr().cast::<HitResultLayout>() };

    assert_eq!(hit.kind, 0);
    assert_eq!(hit.facing, 5);
    assert_eq!(hit.block, BlockPos { x: 10, y: -64, z: 300 });
    assert_eq!(hit.pos, [10.5, -63.0, 300.25]);
}

#[test]
fn writes_land_at_the_hit_result_offsets() {
    let mut memory = GameMemory::new(0x80);

    // SAFETY: as above; the mirror is the only view of the buffer while it lives.
    let hit = unsafe { &mut *memory.ptr().cast::<HitResultLayout>() };
    hit.kind = 3;
    hit.facing = 1;
    hit.block = BlockPos { x: -1, y: 2, z: -3 };
    hit.pos = [0.5, 1.5, 2.5];

    assert_eq!(memory.get::<i32>(0x18), 3);
    assert_eq!(memory.get::<u8>(0x1C), 1);
    assert_eq!(memory.get::<[i32; 3]>(0x20), [-1, 2, -3]);
    assert_eq!(memory.get::<[f32; 3]>(0x2C), [0.5, 1.5, 2.5]);
}
