use std::mem::offset_of;

/// Layout mirror of the first 10h bytes of the game's `PredictedMovementSystem::SystemUniforms`, the same on 26.63 and
/// 70.26 (survey section 8, row N1): `mNow`, the steady-clock count of the tick in nanoseconds, at 8.
///
/// The mirror is never built; the loader reads and writes `mNow` in the live struct only.
#[repr(C)]
pub struct SystemUniformsLayout {
    _first: u64,
    now: i64,
}

impl SystemUniformsLayout {
    pub const NOW_AT: usize = offset_of!(SystemUniformsLayout, now);

    /// The `mNow` of the uniforms at `uniforms`, in nanoseconds.
    ///
    /// # Safety
    /// `uniforms` is the address of a live `SystemUniforms` that only this thread uses now.
    pub unsafe fn now(uniforms: usize) -> i64 {
        let uniforms = std::ptr::with_exposed_provenance::<SystemUniformsLayout>(uniforms);
        // SAFETY: the caller guarantees a live struct, which the mirror covers.
        unsafe { (&raw const (*uniforms).now).read() }
    }

    /// Writes the `mNow` of the uniforms at `uniforms`.
    ///
    /// # Safety
    /// As for `now`.
    pub unsafe fn set_now(uniforms: usize, nanos: i64) {
        let uniforms = std::ptr::with_exposed_provenance_mut::<SystemUniformsLayout>(uniforms);
        // SAFETY: as in `now`.
        unsafe { (&raw mut (*uniforms).now).write(nanos) }
    }
}

const _: () = assert!(SystemUniformsLayout::NOW_AT == 8);
