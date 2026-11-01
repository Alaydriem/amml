/// One of a custom curve's two bezier handles: the first leaves the start key, the second comes into the end key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HandleIndex {
    First,
    Second,
}
