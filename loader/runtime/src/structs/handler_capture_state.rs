use crate::CapturedHandler;

// The state of a `HandlerCapture`: the handler held now, the generation of the last capture (0 before the first) and
// whether captures are refused (from a world leave to the next join).
#[derive(Default)]
pub(crate) struct HandlerCaptureState {
    pub(crate) held: Option<CapturedHandler>,
    pub(crate) generation: u32,
    pub(crate) closed: bool,
}
