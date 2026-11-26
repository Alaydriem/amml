use amml_abi::AmmlCameraTable;

/// One `CameraSetup` call in progress: the address of the event the listener got, the loader camera table and the
/// module of the listener.
#[derive(Clone, Copy)]
pub(crate) struct LentCameraEvent {
    pub(crate) event: usize,
    pub(crate) table: AmmlCameraTable,
    pub(crate) module: &'static str,
}
