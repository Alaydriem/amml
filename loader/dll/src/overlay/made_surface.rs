use std::sync::atomic::AtomicBool;

use amml_abi::AmmlDrawSurface;

use super::{BackBufferCopy, DrawSurface, DrawWindow, StateBackup, WrappedAcquire};

/// A surface made for the listeners of one frame, with its guards: the wrapped back buffer acquired, the state saved,
/// the copy window open and the draw flag set. The fields drop in the order they are declared, the reverse of the
/// order they were set up, so the draw flag clears first and the wrapper is released before the surface's references
/// go.
pub struct MadeSurface {
    _drawing: DrawWindow<'static>,
    _copy: BackBufferCopy,
    _state: StateBackup,
    _acquired: Option<WrappedAcquire>,
    surface: DrawSurface,
}

impl MadeSurface {
    /// Sets up the guards of `surface`; `drawing` is the flag of the time listeners draw.
    pub fn open(surface: DrawSurface, drawing: &'static AtomicBool) -> Self {
        let acquired = surface.acquire();
        let state = StateBackup::capture(&surface.context);
        let copy = BackBufferCopy::open(&surface);
        let drawing = DrawWindow::open(drawing);
        Self { _drawing: drawing, _copy: copy, _state: state, _acquired: acquired, surface }
    }

    /// The surface as `FramePresent` carries it; the pointers are borrowed from this value.
    pub fn abi(&self) -> AmmlDrawSurface {
        self.surface.abi()
    }
}
