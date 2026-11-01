/// A Paper font family token: the family name.
pub trait FontFamilyToken {
    /// The Paper token name, with its leading `--`.
    const NAME: &'static str;
    const VALUE: &'static str;
}
