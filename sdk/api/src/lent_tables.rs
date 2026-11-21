use amml_abi::{
    AmmlCameraTable, AmmlInventoryTable, AmmlNetworkTable, AmmlRenderTable, AmmlUiTable, OverlayCopyBackBufferFn,
};

/// The loader functions a listener trampoline lends to the event of a running call: the copy function of a
/// `FramePresent` surface, the ui and inventory tables of a `ScreenRender` view, and the network table of a
/// `PlayerListUpdate` or `PlayerSkinChanged`, the render table of a `WorldRendered` canvas and the camera table of a `CameraSetup` camera.
#[derive(Clone, Copy)]
pub(crate) struct LentTables {
    pub(crate) copy_back_buffer: Option<OverlayCopyBackBufferFn>,
    pub(crate) ui: Option<AmmlUiTable>,
    pub(crate) inventory: Option<AmmlInventoryTable>,
    pub(crate) network: Option<AmmlNetworkTable>,
    pub(crate) render: Option<AmmlRenderTable>,
    pub(crate) camera: Option<AmmlCameraTable>,
}
