#[derive(Debug, thiserror::Error)]
pub enum MsvcError {
    #[error("the game allocator returned null for {bytes} bytes")]
    AllocationFailed { bytes: usize },
    #[error("string bytes are not valid UTF-8")]
    InvalidUtf8(#[from] std::str::Utf8Error),
}
