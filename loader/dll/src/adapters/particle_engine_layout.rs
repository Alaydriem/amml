use std::mem::offset_of;

/// Layout mirror of the game's `ParticleSystemEngine` as far as the loader reads it, and the same on 26.63 and 70.26
/// (survey section 8, rows N4 to N6): the valid byte `mValid` at B8h, the steady-clock time of the last tick
/// `mLastTickTime` at C0h (nanoseconds) and the particle count of the last tick `mParticleCount` at 10F8h.
///
/// The mirror is never built; the loader reads and writes the three fields in the live engine only.
#[repr(C)]
pub struct ParticleEngineLayout {
    _before: [u8; 0xB8],
    valid: u8,
    _align: [u8; 7],
    last_tick_time: i64,
    _between: [u8; 0x10F8 - 0xC8],
    particle_count: u64,
}

impl ParticleEngineLayout {
    pub const VALID_AT: usize = offset_of!(ParticleEngineLayout, valid);
    pub const LAST_TICK_AT: usize = offset_of!(ParticleEngineLayout, last_tick_time);
    pub const COUNT_AT: usize = offset_of!(ParticleEngineLayout, particle_count);

    /// Whether the engine at `engine` has ticked before (the game measures a step only then).
    ///
    /// # Safety
    /// `engine` is the address of a live game `ParticleSystemEngine` that only this thread uses now.
    pub unsafe fn valid(engine: usize) -> bool {
        let engine = std::ptr::with_exposed_provenance::<ParticleEngineLayout>(engine);
        // SAFETY: the caller guarantees a live engine, which the mirror covers up to the count.
        unsafe { (&raw const (*engine).valid).read() != 0 }
    }

    /// The last tick time of the engine at `engine`, in nanoseconds.
    ///
    /// # Safety
    /// As for `valid`.
    pub unsafe fn last_tick_time(engine: usize) -> i64 {
        let engine = std::ptr::with_exposed_provenance::<ParticleEngineLayout>(engine);
        // SAFETY: as in `valid`.
        unsafe { (&raw const (*engine).last_tick_time).read() }
    }

    /// Writes the last tick time of the engine at `engine`.
    ///
    /// # Safety
    /// As for `valid`.
    pub unsafe fn set_last_tick_time(engine: usize, nanos: i64) {
        let engine = std::ptr::with_exposed_provenance_mut::<ParticleEngineLayout>(engine);
        // SAFETY: as in `valid`.
        unsafe { (&raw mut (*engine).last_tick_time).write(nanos) }
    }

    /// The particle count of the last tick of the engine at `engine`.
    ///
    /// # Safety
    /// As for `valid`.
    pub unsafe fn particle_count(engine: usize) -> u64 {
        let engine = std::ptr::with_exposed_provenance::<ParticleEngineLayout>(engine);
        // SAFETY: as in `valid`.
        unsafe { (&raw const (*engine).particle_count).read() }
    }
}

const _: () = assert!(ParticleEngineLayout::VALID_AT == 0xB8);
const _: () = assert!(ParticleEngineLayout::LAST_TICK_AT == 0xC0);
const _: () = assert!(ParticleEngineLayout::COUNT_AT == 0x10F8);
