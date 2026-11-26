/// How many draw surfaces the overlay made since the process started: `made` counts every surface (each one is a
/// back-buffer wrap or buffer read, and a render target view), `asked` the ones a listener asked for with
/// `AmmlOverlayTable::prepare_surface`, `state_saves` the pipeline state saves before listeners.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SurfaceCounts {
    pub made: u64,
    pub asked: u64,
    pub state_saves: u64,
}
