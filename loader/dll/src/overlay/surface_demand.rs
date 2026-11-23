use std::cell::RefCell;

use amml_abi::{AmmlDrawSurface, AmmlStatus};
use windows::Win32::Graphics::Dxgi::IDXGISwapChain;

use super::{DemandFrame, MadeSurface};

thread_local! {
    // The on-demand frame of the `FramePresent` this thread is raising now; none outside one.
    static FRAME: RefCell<Option<DemandFrame>> = const { RefCell::new(None) };
}

/// The window of a `FramePresent` raise for `AmmlOverlayTable::prepare_surface`. The value `open` gives marks that
/// window on this thread; when it drops, also when a panic unwinds through the listeners, the frame goes, and with it
/// any surface made on demand: the state is put back and the wrapped back buffer released before the original
/// Present. A settled frame (`DemandFrame::settled`) lends the surface made before the listeners.
pub struct SurfaceDemand {
    open: bool,
}

impl SurfaceDemand {
    /// Opens the window for `frame` on this thread until the value drops.
    pub fn open(frame: DemandFrame) -> Self {
        let open = FRAME.try_with(|slot| *slot.borrow_mut() = Some(frame)).is_ok();
        Self { open }
    }

    /// The surface the next listener gets, inside the window on this thread.
    pub fn current() -> Option<AmmlDrawSurface> {
        FRAME.try_with(|slot| slot.borrow().as_ref().map(DemandFrame::current)).ok().flatten()
    }

    /// The frame's surface: the one made already (on demand or before the listeners), or the one `make` makes for
    /// the chain now. `REFUSED` outside the window on this thread; `NOT_READY` for a settled frame with no surface,
    /// and when `make` made none, then and for the rest of the frame.
    pub fn prepare(make: impl FnOnce(&IDXGISwapChain) -> Option<MadeSurface>) -> Result<AmmlDrawSurface, AmmlStatus> {
        let step = FRAME
            .try_with(|slot| match slot.borrow().as_ref() {
                None => Err(AmmlStatus::REFUSED),
                Some(frame) if frame.refused => Err(AmmlStatus::NOT_READY),
                Some(frame) => match (&frame.made, &frame.chain) {
                    (Some(made), _) => Ok(Ok(made.abi())),
                    (None, None) => Ok(Ok(frame.waiting)),
                    (None, Some(chain)) => Ok(Err(chain.clone())),
                },
            })
            .unwrap_or(Err(AmmlStatus::REFUSED))?;
        let chain = match step {
            Ok(made) => return Ok(made),
            Err(chain) => chain,
        };
        // No borrow of the slot is held while the surface is made.
        let made = make(&chain);
        FRAME
            .try_with(|slot| {
                let mut slot = slot.borrow_mut();
                let Some(frame) = slot.as_mut() else {
                    return Err(AmmlStatus::REFUSED);
                };
                match made {
                    Some(made) => {
                        let surface = made.abi();
                        frame.made = Some(made);
                        Ok(surface)
                    }
                    None => {
                        frame.refused = true;
                        Err(AmmlStatus::NOT_READY)
                    }
                }
            })
            .unwrap_or(Err(AmmlStatus::REFUSED))
    }
}

impl Drop for SurfaceDemand {
    fn drop(&mut self) {
        if self.open {
            // The frame drops after the slot's borrow ends, so its guards run with the slot free.
            let frame = FRAME.try_with(|slot| slot.borrow_mut().take()).ok().flatten();
            drop(frame);
        }
    }
}
