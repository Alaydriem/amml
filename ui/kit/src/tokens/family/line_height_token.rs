/// A Paper line height token, in points.
pub trait LineHeightToken {
    /// The Paper token name, with its leading `--`.
    const NAME: &'static str;
    const VALUE: f32;
}
