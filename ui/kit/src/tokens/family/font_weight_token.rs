/// A Paper font weight token, from 100 to 900.
pub trait FontWeightToken {
    /// The Paper token name, with its leading `--`.
    const NAME: &'static str;
    const VALUE: u16;
}
