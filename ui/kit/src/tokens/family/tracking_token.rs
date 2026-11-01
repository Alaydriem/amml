/// A Paper letter spacing token, in em.
pub trait TrackingToken {
    /// The Paper token name, with its leading `--`.
    const NAME: &'static str;
    const VALUE: f32;
}
