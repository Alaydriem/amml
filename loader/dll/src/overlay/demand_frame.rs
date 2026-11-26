use amml_abi::AmmlDrawSurface;
use windows::Win32::Graphics::Dxgi::IDXGISwapChain;

use super::{DrawSurface, MadeSurface};

/// The frame of a `FramePresent` raise as `prepare_surface` sees it. On demand: the chain presenting, the surface the
/// listeners get before one asks (no context, target or back buffer), and the surface once one asked; a frame whose
/// surface could not be made is not tried again. Settled (no chain): the frame's surface, or none, was decided before
/// the listeners, and `prepare_surface` gives that surface or `NOT_READY`.
pub struct DemandFrame {
    pub chain: Option<IDXGISwapChain>,
    pub waiting: AmmlDrawSurface,
    pub made: Option<MadeSurface>,
    pub refused: bool,
}

impl DemandFrame {
    /// An on-demand frame of `chain`, whose listeners get `waiting` until one asks.
    pub fn new(chain: IDXGISwapChain, waiting: AmmlDrawSurface) -> Self {
        Self { chain: Some(chain), waiting, made: None, refused: false }
    }

    /// A frame whose surface was made before the listeners (`lent`), or that has none.
    pub fn settled(lent: Option<AmmlDrawSurface>) -> Self {
        Self { chain: None, waiting: lent.unwrap_or_else(DrawSurface::empty), made: None, refused: lent.is_none() }
    }

    /// The surface the next listener gets: the made one once a listener asked, else the waiting (or lent) one.
    pub fn current(&self) -> AmmlDrawSurface {
        self.made.as_ref().map_or(self.waiting, MadeSurface::abi)
    }
}
