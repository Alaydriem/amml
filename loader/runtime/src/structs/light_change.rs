/// The block boxes (inclusive min, max) whose meshes must be rebuilt after a light change: the reach of
/// the old and of the new light, each once.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LightChange {
    pub boxes: Vec<([i32; 3], [i32; 3])>,
}

impl LightChange {
    pub fn is_empty(&self) -> bool {
        self.boxes.is_empty()
    }

    /// Adds `found` unless it is already in.
    pub fn add(&mut self, found: ([i32; 3], [i32; 3])) {
        if !self.boxes.contains(&found) {
            self.boxes.push(found);
        }
    }

    /// Adds every box of `other` that is not in yet.
    pub fn join(&mut self, other: LightChange) {
        for found in other.boxes {
            self.add(found);
        }
    }
}
