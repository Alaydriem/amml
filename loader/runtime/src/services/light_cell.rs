use std::sync::atomic::{AtomicI32, AtomicI64, AtomicU8, AtomicU32, AtomicUsize, Ordering, fence};

use amml_abi::AmmlLightFalloff;

use crate::services::LightBook;
use crate::structs::{LightPoint, VvPlan};

/// The lights as the mesh worker threads read them, without a lock: one writer at a time (`publish`, from
/// `LightBook::set` under its lock) and many readers. A sequence counter keeps a reader from mixing an old
/// and a new light: the writer makes it odd while it writes, and a reader retries when it saw an odd or a
/// changed value.
///
/// Beside the lights it keeps the box (inclusive block min, max) that holds the reach of all of them: a getter
/// outside it gets no light, so it skips the read. The box is read without the counter and may be stale by one
/// publish.
pub struct LightCell {
    seq: AtomicU32,
    count: AtomicU8,
    slots: [CellSlot; LightBook::SLOTS],
    reach_low: [AtomicI32; 3],
    reach_high: [AtomicI32; 3],
}

struct CellSlot {
    steps: [AtomicI64; 3],
    level: AtomicU8,
    falloff: AtomicU8,
    block: AtomicUsize,
}

impl CellSlot {
    const fn new() -> Self {
        Self {
            steps: [AtomicI64::new(0), AtomicI64::new(0), AtomicI64::new(0)],
            level: AtomicU8::new(0),
            falloff: AtomicU8::new(0),
            block: AtomicUsize::new(0),
        }
    }
}

impl Default for LightCell {
    fn default() -> Self {
        Self::new()
    }
}

impl LightCell {
    // Tries of a reader before it gives up, and reads no light, while a writer keeps writing.
    const READ_TRIES: usize = 8;
    const EMPTY: LightPoint = LightPoint { steps: [0; 3], level: 0, falloff: AmmlLightFalloff::ROUND, block: 0 };

    pub const fn new() -> Self {
        Self {
            seq: AtomicU32::new(0),
            count: AtomicU8::new(0),
            slots: [CellSlot::new(), CellSlot::new(), CellSlot::new(), CellSlot::new()],
            reach_low: [AtomicI32::new(i32::MAX), AtomicI32::new(i32::MAX), AtomicI32::new(i32::MAX)],
            reach_high: [AtomicI32::new(i32::MIN), AtomicI32::new(i32::MIN), AtomicI32::new(i32::MIN)],
        }
    }

    /// Replaces the lights with the first `LightBook::SLOTS` of `points`. One writer at a time.
    pub fn publish(&self, points: &[LightPoint]) {
        let points = &points[..points.len().min(LightBook::SLOTS)];
        self.seq.fetch_add(1, Ordering::AcqRel);
        fence(Ordering::Release);
        for (slot, point) in self.slots.iter().zip(points) {
            for axis in 0..3 {
                slot.steps[axis].store(point.steps[axis], Ordering::Relaxed);
            }
            slot.level.store(point.level, Ordering::Relaxed);
            slot.falloff.store(point.falloff.0, Ordering::Relaxed);
            slot.block.store(point.block, Ordering::Relaxed);
        }
        let mut low = [i32::MAX; 3];
        let mut high = [i32::MIN; 3];
        for (min, max) in points.iter().map(LightPoint::reach_box) {
            for axis in 0..3 {
                low[axis] = low[axis].min(min[axis]);
                high[axis] = high[axis].max(max[axis]);
            }
        }
        for axis in 0..3 {
            self.reach_low[axis].store(low[axis], Ordering::Relaxed);
            self.reach_high[axis].store(high[axis], Ordering::Relaxed);
        }
        let count = u8::try_from(points.len()).unwrap_or(0);
        self.count.store(count, Ordering::Relaxed);
        self.seq.fetch_add(1, Ordering::Release);
    }

    /// Whether any light is set: one relaxed load, the fast exit of the hot path.
    pub fn is_lit(&self) -> bool {
        self.count.load(Ordering::Relaxed) != 0
    }

    /// Whether `pos` is inside the reach of the lights, so it may get light: six relaxed loads, no counter and
    /// no copy. `false` with no light.
    pub fn might_reach(&self, pos: [i32; 3]) -> bool {
        (0..3).all(|axis| {
            (self.reach_low[axis].load(Ordering::Relaxed)..=self.reach_high[axis].load(Ordering::Relaxed))
                .contains(&pos[axis])
        })
    }

    /// A consistent copy of the lights and their count; no light when a writer kept the counter moving.
    pub fn read(&self) -> ([LightPoint; LightBook::SLOTS], usize) {
        let mut out = [Self::EMPTY; LightBook::SLOTS];
        for _ in 0..Self::READ_TRIES {
            let before = self.seq.load(Ordering::Acquire);
            if before % 2 == 1 {
                std::hint::spin_loop();
                continue;
            }
            let count = usize::from(self.count.load(Ordering::Relaxed)).min(LightBook::SLOTS);
            for (slot, point) in self.slots.iter().zip(out.iter_mut()).take(count) {
                *point = LightPoint {
                    steps: [0, 1, 2].map(|axis| slot.steps[axis].load(Ordering::Relaxed)),
                    level: slot.level.load(Ordering::Relaxed),
                    falloff: AmmlLightFalloff(slot.falloff.load(Ordering::Relaxed)),
                    block: slot.block.load(Ordering::Relaxed),
                };
            }
            fence(Ordering::Acquire);
            if self.seq.load(Ordering::Relaxed) == before {
                return (out, count);
            }
        }
        (out, 0)
    }

    /// The block light byte of a getter result at `pos`: `current`, or the brightest light's value there
    /// when that is higher. A position outside the reach of the lights returns before the read.
    pub fn boost_at(&self, pos: [i32; 3], current: u8) -> u8 {
        if !self.is_lit() || !self.might_reach(pos) {
            return current;
        }
        let (points, count) = self.read();
        points[..count].iter().map(|point| point.boost(pos)).fold(current, u8::max)
    }

    /// The light pair (`[sky, block]`) of an entity light lookup at `pos`: the block byte raised as by
    /// `boost_at`; the sky byte never changes.
    pub fn raise_pair(&self, pair: [u8; 2], pos: [i32; 3]) -> [u8; 2] {
        [pair[0], self.boost_at(pos, pair[1])]
    }

    /// What the Vibrant Visuals static light pass does for the subchunk at `subchunk`; `variant` 0 is the
    /// mesher's pass. With `emitter_ready` and variant 0, the brightest light with a block within one
    /// subchunk of `subchunk` is added as an emitter. Otherwise a light that reaches the subchunk skips the
    /// pass, so the raised vertex light stays.
    pub fn vv_plan(&self, subchunk: [i32; 3], variant: u16, emitter_ready: bool) -> VvPlan {
        if !self.is_lit() {
            return VvPlan::Normal;
        }
        let (points, count) = self.read();
        VvPlan::of(&points[..count], subchunk, variant, emitter_ready)
    }
}
