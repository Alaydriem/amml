use std::cell::Cell;

thread_local! {
    static DEPTH: Cell<u32> = const { Cell::new(0) };
}

/// The ResizeBuffers nesting of this thread while the guard lives. `ResizeBuffers1` may reach `ResizeBuffers` inside
/// DXGI, and only the outermost call raises the resize events.
pub struct ResizeDepth {
    depth: u32,
}

impl ResizeDepth {
    /// Counts one more resize on this thread. On a thread whose locals are torn down the resize counts as nested, so
    /// it raises nothing.
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

impl Drop for ResizeDepth {
    fn drop(&mut self) {
        if self.depth != 0 {
            let _ = DEPTH.try_with(|cell| cell.set(cell.get().saturating_sub(1)));
        }
    }
}
