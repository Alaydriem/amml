/// A Paper container token: a fixed panel width in points.
pub trait ContainerToken {
    /// The Paper token name, with its leading `--`.
    const NAME: &'static str;
    const VALUE: f32;
}
