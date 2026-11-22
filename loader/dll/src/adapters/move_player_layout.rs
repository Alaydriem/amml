use std::mem::offset_of;

use crate::MovePlayerFields;

/// Layout mirror of the first 51h bytes of the game's `MovePlayerPacket` the loader reads, the same on 26.63 and 70.26
/// (survey section 8, row V2): the packet header, then the runtime id (8 bytes) at 30h, the position (3 floats) at 38h,
/// the rotation (pitch, then yaw) at 44h, the head yaw at 4Ch and the mode byte at 50h. The fields sum to 8 + 12 + 8 + 4
/// + 1 bytes after the 30h header, which ends at 51h; the packet is 80h bytes.
///
/// The mirror is never built; the loader reads the fields in the live packet only.
#[repr(C)]
pub struct MovePlayerLayout {
    _header: [u8; 0x30],
    player_id: u64,
    pos: [f32; 3],
    rot: [f32; 2],
    head_yaw: f32,
    mode: u8,
}

const _: () = assert!(offset_of!(MovePlayerLayout, player_id) == 0x30);
const _: () = assert!(offset_of!(MovePlayerLayout, pos) == 0x38);
const _: () = assert!(offset_of!(MovePlayerLayout, rot) == 0x44);
const _: () = assert!(offset_of!(MovePlayerLayout, head_yaw) == 0x4C);
const _: () = assert!(offset_of!(MovePlayerLayout, mode) == 0x50);

impl MovePlayerLayout {
    /// The fields of the packet at `packet`.
    ///
    /// # Safety
    /// `packet` is the address of a live game `MovePlayerPacket`.
    pub unsafe fn read(packet: usize) -> MovePlayerFields {
        let packet = std::ptr::with_exposed_provenance::<MovePlayerLayout>(packet);
        // SAFETY: the caller guarantees a live packet, which the mirror covers; each field is read in place.
        unsafe {
            MovePlayerFields {
                player_id: (&raw const (*packet).player_id).read_unaligned(),
                pos: (&raw const (*packet).pos).read_unaligned(),
                rot: (&raw const (*packet).rot).read_unaligned(),
                head_yaw: (&raw const (*packet).head_yaw).read_unaligned(),
                mode: (&raw const (*packet).mode).read_unaligned(),
            }
        }
    }
}
