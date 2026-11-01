/// A Paper opacity token, from 0 to 1.
pub trait OpacityToken {
    /// The Paper token name, with its leading `--`.
    const NAME: &'static str;
    const VALUE: f32;
}
