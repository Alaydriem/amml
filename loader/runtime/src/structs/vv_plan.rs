use crate::structs::LightPoint;

/// What the Vibrant Visuals static light pass does for one subchunk.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VvPlan {
    /// Run it as the game does.
    Normal,
    /// Skip it, so the raised vertex light stays.
    Skip,
    /// Run it with the held block added as an emitter at the world block `position`.
    Emit { block: usize, position: [i32; 3], emission: u8 },
}

impl VvPlan {
    /// The plan for the subchunk at `subchunk` under `points`; `variant` 0 is the mesher's pass. With
    /// `emitter_ready` and variant 0, the brightest light with a block within one subchunk of `subchunk` is
    /// added as an emitter. Otherwise a light that reaches the subchunk skips the pass, so the raised vertex
    /// light stays.
    pub fn of(points: &[LightPoint], subchunk: [i32; 3], variant: u16, emitter_ready: bool) -> Self {
        if points.is_empty() {
            return Self::Normal;
        }
        let emits = emitter_ready && variant == 0;
        let near = |point: &LightPoint| {
            let own = point.subchunk();
            point.block != 0 && (0..3).all(|axis| (subchunk[axis] - own[axis]).abs() <= 1)
        };
        if emits && let Some(point) = points.iter().filter(|point| near(point)).max_by_key(|point| point.level) {
            return Self::Emit { block: point.block, position: point.block_pos(), emission: point.level };
        }
        // A light with a block is drawn by its emitter where the emitter runs; elsewhere its boost must stay.
        let skipped = points.iter().any(|point| !(emits && point.block != 0) && point.reaches_subchunk(subchunk));
        if skipped { Self::Skip } else { Self::Normal }
    }
}
