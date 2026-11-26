/// Why the free camera book refuses a call.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum FreeCameraRefusal {
    #[error("another mod holds the free camera")]
    Taken,
}
