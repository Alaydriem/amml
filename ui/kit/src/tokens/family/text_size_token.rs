/// A Paper text size token, in points.
pub trait TextSizeToken {
    /// The Paper token name, with its leading `--`.
    const NAME: &'static str;
    const VALUE: f32;
}
