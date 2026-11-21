/// Why the event bus refuses a listener.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum ListenError {
    #[error("the event is not offered")]
    UnknownEvent,
    #[error("the event is offered only in an older version")]
    VersionTooNew,
    #[error("the owner is being removed")]
    OwnerClosed,
}
