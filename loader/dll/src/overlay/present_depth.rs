use std::cell::Cell;

thread_local! {
    static DEPTH: Cell<u32> = const { Cell::new(0) };
}

/// The Present nesting of this thread while the guard lives. `Present1` can reach `Present` inside DXGI, and only
/// the outermost call is a frame.
pub struct PresentDepth {
    depth: u32,
}

impl PresentDepth {
    /// Counts one more Present on this thread. On a thread whose locals are torn down the present counts as nested,
    /// so it is not a frame.
    pub fn enter() -> Self {
        let depth = DEPTH
            .try_with(|cell| {
                let depth = cell.get() + 1;
                cell.set(depth);
                depth
            })
            .unwrap_or(0);
        Self { depth }
    }

    pub fn outermost(&self) -> bool {
        self.depth == 1
    }
}

impl Drop for PresentDepth {
    fn drop(&mut self) {
        if self.depth != 0 {
            let _ = DEPTH.try_with(|cell| cell.set(cell.get().saturating_sub(1)));
        }
    }
}
