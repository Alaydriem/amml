use std::collections::BTreeSet;

/// The subchunks whose meshes must be rebuilt after one or more light changes, each once, and the light
/// evaluations it took to find them.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LightChange {
    // Keyed `[y, z, x]`, so the subchunks of one x row are next to each other.
    subchunks: BTreeSet<[i32; 3]>,
    evaluations: u64,
}

impl LightChange {
    pub const SUBCHUNK: i32 = 16;

    pub fn is_empty(&self) -> bool {
        self.subchunks.is_empty()
    }

    /// Marks the subchunk at `subchunk` (subchunk coordinates) unless it is already in.
    pub fn mark(&mut self, subchunk: [i32; 3]) {
        self.subchunks.insert([subchunk[1], subchunk[2], subchunk[0]]);
    }

    /// Whether the subchunk at `subchunk` is marked.
    pub fn contains(&self, subchunk: [i32; 3]) -> bool {
        self.subchunks.contains(&[subchunk[1], subchunk[2], subchunk[0]])
    }

    /// The number of marked subchunks.
    pub fn marks(&self) -> usize {
        self.subchunks.len()
    }

    /// The light evaluations (`LightPoint::boost` calls) it took to find the marks.
    pub fn evaluations(&self) -> u64 {
        self.evaluations
    }

    pub fn add_evaluations(&mut self, count: u64) {
        self.evaluations = self.evaluations.saturating_add(count);
    }

    /// The marked subchunks as block boxes (inclusive min, max): one box per run of neighbouring subchunks
    /// along x, so each subchunk is in exactly one box.
    pub fn boxes(&self) -> Vec<([i32; 3], [i32; 3])> {
        let mut boxes = Vec::new();
        let mut run: Option<([i32; 3], i32)> = None;
        for &[y, z, x] in &self.subchunks {
            run = match run {
                Some((start, end)) if start[1] == y && start[2] == z && end + 1 == x => Some((start, x)),
                Some((start, end)) => {
                    boxes.push(Self::run_box(start, end));
                    Some(([x, y, z], x))
                }
                None => Some(([x, y, z], x)),
            };
        }
        if let Some((start, end)) = run {
            boxes.push(Self::run_box(start, end));
        }
        boxes
    }

    /// Adds every subchunk of `other` that is not in yet.
    pub fn join(&mut self, other: LightChange) {
        self.subchunks.extend(other.subchunks);
        self.add_evaluations(other.evaluations);
    }

    fn run_box(start: [i32; 3], end_x: i32) -> ([i32; 3], [i32; 3]) {
        let low = start.map(|c| c.saturating_mul(Self::SUBCHUNK));
        let mut high = start;
        high[0] = end_x;
        (low, high.map(|c| c.saturating_mul(Self::SUBCHUNK).saturating_add(Self::SUBCHUNK - 1)))
    }
}
