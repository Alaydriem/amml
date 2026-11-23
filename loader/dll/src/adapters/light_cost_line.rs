/// One `perf` line of the held light over a window of `span_ms`: per second, the light changes, the subchunks
/// they marked dirty, the `_setDirty` calls, the light evaluations that found the marks, and the light getter
/// calls (terrain and entity) while a light was set (`lit`) and of those the ones inside the reach of the lights
/// (`near`; the rest skip the light read).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LightCostLine {
    pub span_ms: u64,
    pub changes_per_s: u64,
    pub subchunks_per_s: u64,
    pub set_dirty_per_s: u64,
    pub evaluations_per_s: u64,
    pub lit_getters_per_s: u64,
    pub near_getters_per_s: u64,
}

impl LightCostLine {
    pub fn log(&self) {
        curia::info!("perf", {
            area: "lighting.held",
            span_ms: self.span_ms,
            changes_per_s: self.changes_per_s,
            subchunks_per_s: self.subchunks_per_s,
            set_dirty_per_s: self.set_dirty_per_s,
            evaluations_per_s: self.evaluations_per_s,
            lit_getters_per_s: self.lit_getters_per_s,
            near_getters_per_s: self.near_getters_per_s
        });
    }
}
