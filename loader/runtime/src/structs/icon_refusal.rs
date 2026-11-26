/// Why the inline icon book refuses a call.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum IconRefusal {
    #[error(
        "the advance is not above 0 and at most 16, the image is over 16 by 16, the icon is of another mod, or no cell is free"
    )]
    Refused,
    #[error("the icon is of an older generation, or was freed")]
    Stale,
    #[error("the size is zero or the pixels do not fit it")]
    BadImage,
}
