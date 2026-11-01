/// A Paper corner radius token, in points.
pub trait RadiusToken {
    /// The Paper token name, with its leading `--`.
    const NAME: &'static str;
    const VALUE: f32;
}
