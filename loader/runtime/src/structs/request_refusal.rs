/// Why the request book does not take a request.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum RequestRefusal {
    #[error("another request runs")]
    Busy,
    #[error("a screen is open")]
    ScreenOpen,
    #[error("the request has no move")]
    Empty,
    #[error("the request has too many moves")]
    TooMany,
    #[error("a move names a slot that does not exist, or has no effect")]
    BadMove,
    #[error("the kind of move is not run yet")]
    Unsupported,
}
