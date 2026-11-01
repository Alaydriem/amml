/// A Paper colour token.
pub trait ColorToken {
    /// The Paper token name, with its leading `--`.
    const NAME: &'static str;
    const VALUE: egui::Color32;
}
