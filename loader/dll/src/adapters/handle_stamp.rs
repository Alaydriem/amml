/// A counter value kept in the top 16 bits of a handle that the loader gives a mod. A game address uses the low 48
/// bits, so the handle is still one integer; a mod never reads it as memory. The loader checks the stamp on each
/// call: a handle sealed in an older world (block source) or an older game frame (chunk) is refused.
pub struct HandleStamp;

impl HandleStamp {
    const SHIFT: u32 = 48;
    const ADDRESS_MASK: usize = (1 << Self::SHIFT) - 1;

    /// `address` with the low 16 bits of `counter` in its top bits; `None` for an address that does not fit 48 bits.
    pub fn seal(address: usize, counter: u64) -> Option<usize> {
        (address & !Self::ADDRESS_MASK == 0).then(|| address | (Self::stamp(counter) << Self::SHIFT))
    }

    /// The address in `handle` when its stamp is that of `counter`; `None` for another stamp or no address.
    pub fn open(handle: usize, counter: u64) -> Option<usize> {
        let address = handle & Self::ADDRESS_MASK;
        (address != 0 && handle >> Self::SHIFT == Self::stamp(counter)).then_some(address)
    }

    fn stamp(counter: u64) -> usize {
        (counter & 0xFFFF) as usize
    }
}
