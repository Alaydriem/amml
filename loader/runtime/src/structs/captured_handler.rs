use std::thread::ThreadId;

/// The client network handler a handler detour saw last in this world: its address, the address of the
/// `NetworkIdentifier` it was called with, the thread of that call and the generation of the capture.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CapturedHandler {
    pub handler: usize,
    pub id: usize,
    pub thread: ThreadId,
    pub generation: u32,
}
