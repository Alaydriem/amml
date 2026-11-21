use std::mem::offset_of;

use amml_abi::AmmlInstructionKind;

use crate::SetInstructionLayout;

/// The game's `CameraInstruction` (150h bytes) inside a `CameraInstructionPacket` (at
/// `CameraInstructionPacket.mCameraInstruction`). The members that hold strings or vectors (attach, spline, target) are
/// padding: the loader never writes them, and they stay as `createPacket` made them. Only `mSet` and `mClear` are
/// written, member by member, never the whole struct.
///
/// Read from the exe on 26.63 and 70.26 (the camera instruction function 0x5eb2150 / 0x43fa760, called by the handler
/// with the packet in rcx): it copies the first B0h bytes of the instruction (packet+30h), tests the set engaged byte
/// (+ACh) against 1 before it reads the set, copies the members from +128h to +14Eh, and tests the clear engaged byte
/// (+149h) against 1, then the clear value (+148h) against 0. The `CameraInstruction.mSpline` entry checks the front
/// and `CameraInstructionPacket.size` the whole.
#[repr(C)]
pub struct CameraInstructionLayout {
    _attach_to_entity: [u64; 3],   // 0x000 mAttachToEntity
    _fade: [u32; 9],               // 0x018 mFade
    _field_of_view: [u32; 5],      // 0x03C mFieldOfView
    pub set: SetInstructionLayout, // 0x050 mSet value
    pub set_engaged: u8,           // 0x0AC mSet engaged
    _set_pad: [u8; 3],             // 0x0AD
    _spline: [u64; 15],            // 0x0B0 mSpline
    _target: [u64; 4],             // 0x128 mTarget
    pub clear: u8,                 // 0x148 mClear value
    pub clear_engaged: u8,         // 0x149 mClear engaged
    _detach_from_entity: [u8; 2],  // 0x14A mDetachFromEntity
    _remove_target: [u8; 2],       // 0x14C mRemoveTarget
    _pad: [u8; 2],                 // 0x14E
}

impl CameraInstructionLayout {
    /// The offset of `mSpline`, the value the `CameraInstruction.mSpline` entry must have.
    pub const SPLINE: u64 = offset_of!(CameraInstructionLayout, _spline) as u64;
    /// The bytes the packet holds after the instruction: the serialization mode, then the padding to 8.
    pub const PACKET_TAIL: u64 = 8;

    /// Whether the entries of a build fit this mirror: `mSpline` at `SPLINE`, and a packet of `packet_size` bytes that
    /// ends `PACKET_TAIL` bytes after an instruction at `instruction_at`.
    pub fn fits(spline: u64, instruction_at: u64, packet_size: u64) -> bool {
        let whole =
            instruction_at.checked_add(size_of::<Self>() as u64).and_then(|end| end.checked_add(Self::PACKET_TAIL));
        spline == Self::SPLINE && whole == Some(packet_size)
    }

    /// Writes the set instruction `set` (engaged) and a disengaged clear. No other member is written.
    ///
    /// # Safety
    /// `instruction` points to the live instruction of a `CameraInstructionPacket` of a build whose entries `fits`.
    pub unsafe fn write_set(instruction: *mut Self, set: SetInstructionLayout) {
        // SAFETY: the caller guarantees a live instruction laid out as this mirror; every written member is trivially
        // copyable and written alone.
        unsafe {
            (&raw mut (*instruction).set).write(set);
            (&raw mut (*instruction).set_engaged).write(1);
            (&raw mut (*instruction).clear).write(0);
            (&raw mut (*instruction).clear_engaged).write(0);
        }
    }

    /// Writes the clear instruction (engaged and true) and a disengaged, zeroed set. No other member is written.
    ///
    /// # Safety
    /// As `write_set`.
    pub unsafe fn write_clear(instruction: *mut Self) {
        // SAFETY: as in `write_set`.
        unsafe {
            (&raw mut (*instruction).set).write(SetInstructionLayout::default());
            (&raw mut (*instruction).set_engaged).write(0);
            (&raw mut (*instruction).clear).write(1);
            (&raw mut (*instruction).clear_engaged).write(1);
        }
    }

    /// What the instruction asks: `SET` when only the set part is present, `CLEAR` when only the clear part is present
    /// and true, `OTHER` for both or neither.
    ///
    /// # Safety
    /// As `write_set`, for reading.
    pub unsafe fn kind(instruction: *const Self) -> AmmlInstructionKind {
        // SAFETY: as in `write_set`; the three bytes are read alone.
        let (set, clear_engaged, clear) = unsafe {
            (
                (&raw const (*instruction).set_engaged).read(),
                (&raw const (*instruction).clear_engaged).read(),
                (&raw const (*instruction).clear).read(),
            )
        };
        match (set == 1, clear_engaged == 1 && clear != 0) {
            (true, false) => AmmlInstructionKind::SET,
            (false, true) => AmmlInstructionKind::CLEAR,
            _ => AmmlInstructionKind::OTHER,
        }
    }

    /// Whether the instruction at `instruction` sets a camera preset or clears the camera: its set part is present, or
    /// its clear part is present and true.
    ///
    /// # Safety
    /// As `kind`.
    pub unsafe fn takes_camera(instruction: *const Self) -> bool {
        // SAFETY: passed on from the caller.
        let kind = unsafe { Self::kind(instruction) };
        // SAFETY: as in `kind`; the set engaged byte is read alone.
        let both = unsafe { (&raw const (*instruction).set_engaged).read() } == 1;
        kind != AmmlInstructionKind::OTHER || both
    }
}

const _: () = assert!(offset_of!(CameraInstructionLayout, set) == 0x50);
const _: () = assert!(offset_of!(CameraInstructionLayout, set_engaged) == 0xAC);
const _: () = assert!(offset_of!(CameraInstructionLayout, _spline) == 0xB0);
const _: () = assert!(offset_of!(CameraInstructionLayout, clear) == 0x148);
const _: () = assert!(offset_of!(CameraInstructionLayout, clear_engaged) == 0x149);
const _: () = assert!(size_of::<CameraInstructionLayout>() == 0x150);
