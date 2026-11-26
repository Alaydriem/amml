use std::collections::BTreeSet;

use crate::structs::{LightChange, LightPoint, VvPlan};

/// Finds the subchunks to rebuild when the lights change from one set to another: a subchunk is marked when
/// a block in it, or within `BORDER` blocks of it, gets a different light from the changed lights, or when
/// its Vibrant Visuals plan differs. Every other subchunk meshes the same with the old and the new lights.
#[derive(Clone, Copy, Debug, Default)]
pub struct LightDiff;

impl LightDiff {
    // A mesh reads the light of the blocks next to its own (faces and smooth-light corners), so a block whose
    // light differs dirties every subchunk within this many blocks of it.
    pub const BORDER: i32 = 1;
    // The pass variants and emitter states a plan can be asked with: variant 0 is the mesher's pass, any
    // other variant plans as 1.
    const VARIANTS: [u16; 2] = [0, 1];
    const EMITTER: [bool; 2] = [false, true];

    pub const fn new() -> Self {
        Self
    }

    /// The change from the lights `old` to the lights `new`; empty when they hold the same lights.
    pub fn change(&self, old: &[LightPoint], new: &[LightPoint]) -> LightChange {
        let removed = Self::minus(old, new);
        let added = Self::minus(new, old);
        let mut change = LightChange::default();
        let Some(reach) = Self::reach_of(removed.iter().chain(&added)) else {
            return change;
        };
        // A change of the block alone gives every block the same light; only the plans can differ.
        let block_only = Self::only_blocks_differ(&removed, &added);
        for subchunk in Self::candidates(removed.iter().chain(&added)) {
            if Self::plan_differs(old, new, subchunk)
                || (!block_only && Self::light_differs(&removed, &added, subchunk, reach, &mut change))
            {
                change.mark(subchunk);
            }
        }
        change
    }

    // The lights of `from` that `other` does not hold (each kept light taken once).
    fn minus(from: &[LightPoint], other: &[LightPoint]) -> Vec<LightPoint> {
        let mut rest = other.to_vec();
        from.iter()
            .filter(|point| match rest.iter().position(|held| held == *point) {
                Some(index) => {
                    rest.swap_remove(index);
                    false
                }
                None => true,
            })
            .copied()
            .collect()
    }

    // Whether `removed` and `added` hold the same lights once the block is left out.
    fn only_blocks_differ(removed: &[LightPoint], added: &[LightPoint]) -> bool {
        let without_block = |points: &[LightPoint]| -> Vec<LightPoint> {
            points.iter().map(|point| LightPoint { block: 0, ..*point }).collect()
        };
        removed.len() == added.len() && Self::minus(&without_block(removed), &without_block(added)).is_empty()
    }

    // The box (inclusive block min, max) that holds the reach of every light; `None` without a light.
    fn reach_of<'a>(lights: impl Iterator<Item = &'a LightPoint>) -> Option<([i32; 3], [i32; 3])> {
        lights.map(LightPoint::reach_box).reduce(|(low, high), (min, max)| {
            ([0, 1, 2].map(|axis| low[axis].min(min[axis])), [0, 1, 2].map(|axis| high[axis].max(max[axis])))
        })
    }

    // The subchunks a changed light can affect: its reach and border, and for a light with a block the
    // subchunks within one of its own, where its emitter runs.
    fn candidates<'a>(lights: impl Iterator<Item = &'a LightPoint>) -> BTreeSet<[i32; 3]> {
        let mut found = BTreeSet::new();
        for light in lights {
            let (low, high) = light.reach_box();
            let low = low.map(|c| c.saturating_sub(Self::BORDER).div_euclid(LightChange::SUBCHUNK));
            let high = high.map(|c| c.saturating_add(Self::BORDER).div_euclid(LightChange::SUBCHUNK));
            Self::add_range(&mut found, low, high);
            if light.block != 0 {
                let own = light.subchunk();
                Self::add_range(&mut found, own.map(|c| c - 1), own.map(|c| c + 1));
            }
        }
        found
    }

    fn add_range(found: &mut BTreeSet<[i32; 3]>, low: [i32; 3], high: [i32; 3]) {
        for x in low[0]..=high[0] {
            for y in low[1]..=high[1] {
                for z in low[2]..=high[2] {
                    found.insert([x, y, z]);
                }
            }
        }
    }

    fn plan_differs(old: &[LightPoint], new: &[LightPoint], subchunk: [i32; 3]) -> bool {
        Self::VARIANTS.iter().any(|&variant| {
            Self::EMITTER
                .iter()
                .any(|&ready| VvPlan::of(old, subchunk, variant, ready) != VvPlan::of(new, subchunk, variant, ready))
        })
    }

    // Whether a block of the subchunk or its border gets a different light from the removed lights than from
    // the added ones. A kept light may hide the difference; the subchunk is marked anyway. Rows of blocks that
    // no changed light can reach are skipped; each row test and each block counts as evaluations.
    fn light_differs(
        removed: &[LightPoint],
        added: &[LightPoint],
        subchunk: [i32; 3],
        reach: ([i32; 3], [i32; 3]),
        change: &mut LightChange,
    ) -> bool {
        let side = LightChange::SUBCHUNK;
        let low = [0, 1, 2].map(|axis| (subchunk[axis] * side - Self::BORDER).max(reach.0[axis]));
        let high = [0, 1, 2].map(|axis| (subchunk[axis] * side + side - 1 + Self::BORDER).min(reach.1[axis]));
        if (0..3).any(|axis| low[axis] > high[axis]) {
            return false;
        }
        let changed = removed.iter().chain(added);
        let lights = u64::try_from(removed.len() + added.len()).unwrap_or(u64::MAX);
        change.add_evaluations(lights);
        if !changed.clone().any(|light| light.could_light(low, high)) {
            return false;
        }
        let brightest = |lights: &[LightPoint], pos: [i32; 3]| lights.iter().map(|light| light.boost(pos)).max();
        for y in low[1]..=high[1] {
            for z in low[2]..=high[2] {
                change.add_evaluations(lights);
                let (row_low, row_high) = ([low[0], y, z], [high[0], y, z]);
                if !changed.clone().any(|light| light.could_light(row_low, row_high)) {
                    continue;
                }
                for x in low[0]..=high[0] {
                    change.add_evaluations(lights);
                    let pos = [x, y, z];
                    if brightest(removed, pos).unwrap_or(0) != brightest(added, pos).unwrap_or(0) {
                        return true;
                    }
                }
            }
        }
        false
    }
}
