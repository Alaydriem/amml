use std::sync::atomic::{AtomicU32, Ordering};

/// Joins the two UTF-16 units of a character outside the basic plane, as `WM_CHAR` gives them. Used on the window
/// thread only.
pub struct SurrogateJoin {
    // The high half waiting for its low half; 0 when none.
    high: AtomicU32,
}

impl Default for SurrogateJoin {
    fn default() -> Self {
        Self::new()
    }
}

impl SurrogateJoin {
    const HIGH_FIRST: u32 = 0xD800;
    const HIGH_LAST: u32 = 0xDBFF;
    const LOW_FIRST: u32 = 0xDC00;
    const LOW_LAST: u32 = 0xDFFF;
    const PLANE_BASE: u32 = 0x10000;
    const SHIFT: u32 = 10;

    pub const fn new() -> Self {
        Self { high: AtomicU32::new(0) }
    }

    /// Whether `unit` is the high half of a pair.
    pub fn is_high(unit: u32) -> bool {
        (Self::HIGH_FIRST..=Self::HIGH_LAST).contains(&unit)
    }

    /// The high half of the pair that encodes `codepoint`; `None` for a character of the basic plane.
    pub fn high_of(codepoint: u32) -> Option<u32> {
        (codepoint >= Self::PLANE_BASE).then(|| Self::HIGH_FIRST + ((codepoint - Self::PLANE_BASE) >> Self::SHIFT))
    }

    /// The character `unit` completes, or `None` for a high half, which waits, and for a low half with no high
    /// half before it. A waiting high half followed by a plain unit is dropped.
    pub fn feed(&self, unit: u32) -> Option<u32> {
        if Self::is_high(unit) {
            self.high.store(unit, Ordering::Relaxed);
            return None;
        }
        let high = self.high.swap(0, Ordering::Relaxed);
        if (Self::LOW_FIRST..=Self::LOW_LAST).contains(&unit) {
            if high == 0 {
                return None;
            }
            return Some(Self::PLANE_BASE + ((high - Self::HIGH_FIRST) << Self::SHIFT) + (unit - Self::LOW_FIRST));
        }
        Some(unit)
    }

    /// Forgets a waiting high half, on focus loss.
    pub fn reset(&self) {
        self.high.store(0, Ordering::Relaxed);
    }
}
