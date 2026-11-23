/// One `perf` line of the overlay: the presents of a window, the mean and max time of `before_present` and of its
/// surface part (make, wrap, view, acquire, state save and copy window; the same parts on both paths) in
/// microseconds, and the surfaces made with no listener asking (`unasked`: made before the listeners, whether or not
/// one drew, which the loader cannot see) and on a listener's request (`asked`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CostLine {
    pub frames: u64,
    pub before_us_mean: u64,
    pub before_us_max: u64,
    pub surface_us_mean: u64,
    pub surface_us_max: u64,
    pub unasked: u64,
    pub asked: u64,
}

impl CostLine {
    pub fn log(&self) {
        curia::info!("perf", {
            area: "overlay.present",
            frames: self.frames,
            before_us_mean: self.before_us_mean,
            before_us_max: self.before_us_max,
            surface_us_mean: self.surface_us_mean,
            surface_us_max: self.surface_us_max,
            unasked_surfaces: self.unasked,
            asked_surfaces: self.asked
        });
    }
}
