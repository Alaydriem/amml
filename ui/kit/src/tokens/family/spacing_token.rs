/// A Paper spacing token, in points.
pub trait SpacingToken {
    /// The Paper token name, with its leading `--`.
    const NAME: &'static str;
    const VALUE: f32;
}
