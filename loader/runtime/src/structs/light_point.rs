use amml_abi::AmmlLightFalloff;

/// One held light as the loader keeps it: the position in quarter blocks, the level (1 to 15), the falloff
/// and the address of the block the light is (0 when there is none).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LightPoint {
    pub steps: [i64; 3],
    pub level: u8,
    pub falloff: AmmlLightFalloff,
    pub block: usize,
}

impl LightPoint {
    /// Steps per block: the light moves, and the terrain near it is rebuilt, each time it crosses a quarter
    /// block.
    pub const STEPS: i64 = 4;
    pub const MAX_LEVEL: u8 = 15;
    // World coordinates are at most 30 million; a position past this is not a world position.
    const MAX_COORDINATE: f64 = 1.0e9;

    /// The light at a world position, rounded down to a quarter block. A level of 0, a level above 15 or a
    /// position that is not finite gives `None`.
    pub fn at(position: [f32; 3], level: u8, falloff: AmmlLightFalloff, block: usize) -> Option<Self> {
        if level == 0 || level > Self::MAX_LEVEL {
            return None;
        }
        let mut steps = [0i64; 3];
        for (axis, value) in position.into_iter().enumerate() {
            let value = f64::from(value);
            if !value.is_finite() || value.abs() > Self::MAX_COORDINATE {
                return None;
            }
            #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
            let step = (value * Self::STEPS as f64).floor() as i64;
            steps[axis] = step;
        }
        Some(Self { steps, level, falloff, block })
    }

    /// The block the light is in.
    pub fn block_pos(&self) -> [i32; 3] {
        #[allow(clippy::cast_possible_truncation)]
        self.steps.map(|step| step.div_euclid(Self::STEPS) as i32)
    }

    /// The subchunk the light is in (block >> 4).
    pub fn subchunk(&self) -> [i32; 3] {
        self.block_pos().map(|c| c.div_euclid(16))
    }

    /// The block light (0 to 15) this light gives at the block `pos`, measured to the block centre and
    /// rounded to whole blocks: straight-line distance for `ROUND` (and any unknown value), block steps for
    /// `TAXICAB`.
    pub fn boost(&self, pos: [i32; 3]) -> u8 {
        let steps = Self::STEPS;
        let reach = i64::from(self.level);
        let mut d = [0i64; 3];
        for axis in 0..3 {
            let centre = i64::from(pos[axis]) * steps + steps / 2;
            d[axis] = centre.saturating_sub(self.steps[axis]);
            // Fast reject: `level` blocks or more on one axis gives nothing with either falloff.
            if d[axis].unsigned_abs() >= (reach * steps).unsigned_abs() {
                return 0;
            }
        }
        #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
        let distance = if self.falloff == AmmlLightFalloff::TAXICAB {
            ((d[0].abs() + d[1].abs() + d[2].abs()) as f64 / steps as f64).round() as i64
        } else {
            (((d[0] * d[0] + d[1] * d[1] + d[2] * d[2]) as f64).sqrt() / steps as f64).round() as i64
        };
        u8::try_from((reach - distance).clamp(0, 15)).unwrap_or(0)
    }

    /// The block box (inclusive min, max) the light can reach: its block `+/- level`. A light anywhere in its
    /// block gives nothing at `level` blocks or more on one axis, so the box covers the quarter-block offset.
    pub fn reach_box(&self) -> ([i32; 3], [i32; 3]) {
        let reach = i32::from(self.level);
        let block = self.block_pos();
        (block.map(|c| c.saturating_sub(reach)), block.map(|c| c.saturating_add(reach)))
    }

    /// Whether the reach box touches the subchunk at `subchunk` (subchunk coordinates).
    pub fn reaches_subchunk(&self, subchunk: [i32; 3]) -> bool {
        let (low, high) = self.reach_box();
        (0..3).all(|axis| (low[axis].div_euclid(16)..=high[axis].div_euclid(16)).contains(&subchunk[axis]))
    }
}
