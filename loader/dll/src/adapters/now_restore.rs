use crate::SystemUniformsLayout;

/// The wall value of a `SystemUniforms.mNow` that the loader replaced for one call: dropping it writes the wall value
/// back, on every path out of the detour.
pub struct NowRestore {
    uniforms: usize,
    wall: i64,
}

impl NowRestore {
    /// Puts `virtual_now` into the `mNow` of the uniforms at `uniforms` and keeps the value that was there.
    ///
    /// # Safety
    /// `uniforms` is the address of a live `SystemUniforms` that only this thread uses until the guard drops.
    pub unsafe fn replace(uniforms: usize, virtual_now: i64) -> Self {
        // SAFETY: the caller guarantees a live struct used by this thread only.
        let wall = unsafe { SystemUniformsLayout::now(uniforms) };
        // SAFETY: as above.
        unsafe { SystemUniformsLayout::set_now(uniforms, virtual_now) };
        Self { uniforms, wall }
    }
}

impl Drop for NowRestore {
    fn drop(&mut self) {
        // SAFETY: `replace` was given a live struct that stays live for the call the guard covers.
        unsafe { SystemUniformsLayout::set_now(self.uniforms, self.wall) };
    }
}
