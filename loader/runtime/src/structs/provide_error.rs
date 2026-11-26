/// Why the provider registry refuses a table.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum ProvideError {
    #[error("another mod provides this name and major version")]
    Taken { owner: usize },
}
