use amml_api::{Dirs, Events, Overlay, Window};

/// The loader services the overlay checks call from the client tick, taken from the `Ctx` at enable.
#[derive(Clone, Copy)]
pub struct OverlayTools {
    pub events: Events,
    pub overlay: Overlay,
    pub window: Window,
    pub dirs: Dirs,
}
