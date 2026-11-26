use std::mem::offset_of;

/// The game's `CameraInstructionOptions::SetInstruction` (5Ch bytes): the preset index, then one `std::optional` per
/// option (the value, then the engaged byte and its padding). Every member is trivially copyable, so a whole value is
/// written over the game's. The loader sets the preset, the position and the rotation; every other option is zero,
/// which is a disengaged optional.
///
/// Read from the exe on 26.63 and 70.26 (the camera instruction function that the `CameraInstructionPacket` handler
/// calls, 0x5eb2150 / 0x43fa760, on its copy of the instruction at `rbp-50h`): the preset index (+0) indexes the preset
/// vector with stride 130h, the ease engaged byte (+0Ch) is tested, the rotation engaged byte (+28h) is tested and the
/// pitch (+20h) is clamped. The position fills 10h..20h between the ease and the rotation.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SetInstructionLayout {
    pub preset_index: u32,    // 0x00 mPresetIndex
    _ease: [u32; 3],          // 0x04 mEase (value 04h, engaged 0Ch)
    pub pos: [f32; 3],        // 0x10 mPos
    pub pos_engaged: u32,     // 0x1C mPos engaged byte, then padding
    pub rot: [f32; 2],        // 0x20 mRot (x pitch, y yaw)
    pub rot_engaged: u32,     // 0x28 mRot engaged byte, then padding
    _facing: [u32; 4],        // 0x2C mFacing (value 2Ch, engaged 38h)
    _view_offset: [u32; 3],   // 0x3C mViewOffset (value 3Ch, engaged 44h)
    _entity_offset: [u32; 4], // 0x48 mEntityOffset (value 48h, engaged 54h)
    _default: [u8; 2],        // 0x58 mDefault
    _remove_flag: [u8; 2],    // 0x5A mRemoveIgnoreStartingValuesComponent, then padding
}

impl SetInstructionLayout {
    /// The set instruction of the preset at `preset_index` at `position`, looking at `pitch` and `yaw` (degrees).
    pub fn new(preset_index: u32, position: [f32; 3], pitch: f32, yaw: f32) -> Self {
        Self { preset_index, pos: position, pos_engaged: 1, rot: [pitch, yaw], rot_engaged: 1, ..Self::default() }
    }
}

const _: () = assert!(offset_of!(SetInstructionLayout, pos) == 0x10);
const _: () = assert!(offset_of!(SetInstructionLayout, rot) == 0x20);
const _: () = assert!(offset_of!(SetInstructionLayout, rot_engaged) == 0x28);
const _: () = assert!(offset_of!(SetInstructionLayout, _default) == 0x58);
const _: () = assert!(size_of::<SetInstructionLayout>() == 0x5C);
