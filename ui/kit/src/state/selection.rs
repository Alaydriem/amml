/// The selection of a control that can be on, off or partly on (checkbox, toggle, filter chip).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Selection {
    #[default]
    Off,
    On,
    /// Some children are on and some are off (a parent checkbox).
    Mixed,
}
