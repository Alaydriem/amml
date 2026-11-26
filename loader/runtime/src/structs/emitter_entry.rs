/// One entry of the game's thread-local light emitter vector (16 bytes): the `Block` pointer at 0, the
/// window-local x, y, z at 8, 9, 10 and the emission at 11. The window is the 48x48x48 block grid of the
/// 3x3x3 subchunks around the subchunk the light pass runs for.
pub struct EmitterEntry;

impl EmitterEntry {
    pub const SIZE: usize = 16;
    const WINDOW: i32 = 48;

    /// The window-local position of the world block `position` for the pass of `subchunk`; `None` outside
    /// the window.
    pub fn local(position: [i32; 3], subchunk: [i32; 3]) -> Option<[u8; 3]> {
        let mut out = [0u8; 3];
        for axis in 0..3 {
            let origin = subchunk[axis].checked_sub(1)?.checked_mul(16)?;
            let local = position[axis].checked_sub(origin)?;
            if !(0..Self::WINDOW).contains(&local) {
                return None;
            }
            out[axis] = u8::try_from(local).ok()?;
        }
        Some(out)
    }

    /// The bytes of one entry; the emission is capped at 15.
    pub fn bytes(block: usize, local: [u8; 3], emission: u8) -> [u8; Self::SIZE] {
        let mut out = [0u8; Self::SIZE];
        out[..8].copy_from_slice(&(block as u64).to_le_bytes());
        out[8..11].copy_from_slice(&local);
        out[11] = emission.min(15);
        out
    }
}
