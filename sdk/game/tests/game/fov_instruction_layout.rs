use amml_game::FovInstructionLayout;
use amml_msvc::Optional;

use crate::support::GameMemory;

// `mFieldOfView` is an `optional<FovInstruction>` at CameraInstruction +0x3C: the instruction (16 bytes), then the
// engaged flag at +0x4C. The buffer here starts at that optional.

#[test]
fn reads_the_four_fields_when_the_optional_is_engaged() {
    let mut memory = GameMemory::new(0x14);
    memory.put::<f32>(0x00, 70.0); // mFov
    memory.put::<u8>(0x04, 1); // mFovClear
    memory.put::<f32>(0x08, 0.5); // mFovEaseTime
    memory.put::<i32>(0x0C, 0); // mFovEaseType (Linear)
    memory.put::<u8>(0x10, 1); // engaged (CameraInstruction +0x4C)

    // SAFETY: the buffer is 16-byte aligned, zeroed and as large as the optional.
    let optional = unsafe { &*memory.ptr().cast::<Optional<FovInstructionLayout>>() };
    let fov = optional.get().unwrap();

    assert_eq!(fov.fov, 70.0);
    assert!(fov.fov_clear);
    assert_eq!(fov.ease_time, 0.5);
    assert_eq!(fov.ease_type, 0);
}

#[test]
fn a_clear_flag_means_no_instruction() {
    let mut memory = GameMemory::new(0x14);
    memory.put::<f32>(0x00, 70.0);

    // SAFETY: as above.
    let optional = unsafe { &*memory.ptr().cast::<Optional<FovInstructionLayout>>() };

    assert!(optional.get().is_none());
}

#[test]
fn an_optional_instruction_is_20_bytes_with_the_flag_at_0x10() {
    assert_eq!(size_of::<Optional<FovInstructionLayout>>(), 0x14);

    let mut optional = Optional::some(FovInstructionLayout::new(60.0, false, 0.0, 0));
    // SAFETY: `Optional` is `repr(C)` and 20 bytes, so byte 0x10 is its flag.
    let flag = unsafe { *(&raw mut optional).cast::<u8>().add(0x10) };
    assert_eq!(flag, 1);
}
