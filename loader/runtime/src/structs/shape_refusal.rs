/// Why the shape book refuses a batch.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum ShapeRefusal {
    #[error("a batch of {count} shape changes is more than {max}")]
    TooMany { count: usize, max: usize },
}
