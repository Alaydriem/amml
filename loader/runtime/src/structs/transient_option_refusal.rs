/// Why the transient option book refuses a call.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum TransientOptionRefusal {
    #[error("another mod holds a transient value of the option")]
    Taken,
}
