use amml_i18n::text::Text;

/// The unit after a drag field's value (Paper: Drag field sheet).
#[derive(Debug, Clone, Default)]
pub enum DragUnit {
    #[default]
    None,
    /// "°", joined to the number with no gap.
    Degrees,
    /// A word unit ("m", "s", "m/s"), `--spacing-xs` after the number.
    Text(Text),
    /// A one-character unit ("x", "%") joined to the number with no gap, as "3.00x" and "50%".
    Joined(Text),
}
