/// A Paper breakpoint token: a window width in points.
pub trait BreakpointToken {
    /// The Paper token name, with its leading `--`.
    const NAME: &'static str;
    const VALUE: f32;
}
