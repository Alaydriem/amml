pub mod cursor_shapes;
pub mod message_call;
pub mod router_state;
pub mod size_message;
pub mod size_requests;
pub mod surrogate_join;
pub mod window_proc;
pub mod window_proc_depth;
pub mod window_service;

pub use cursor_shapes::CursorShapes;
pub use message_call::MessageCall;
pub use router_state::RouterState;
pub use size_message::SizeMessage;
pub use size_requests::SizeRequests;
pub use surrogate_join::SurrogateJoin;
pub use window_proc::WindowProc;
pub use window_proc_depth::WindowProcDepth;
pub use window_service::WindowService;

/// What the loader's window procedure does with one message after the listeners ran.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProcOutcome {
    /// The game's procedure gets the message unchanged.
    Pass,
    /// The game's procedure gets the message with this `lParam` (a written `WM_SIZE`).
    PassWith { lparam: isize },
    /// The game's procedure gets a `WM_CHAR` of this high surrogate first, then the message (the low half).
    PassPair { high: usize },
    /// The game's procedure does not run; the caller gets this result.
    Answer(isize),
}
