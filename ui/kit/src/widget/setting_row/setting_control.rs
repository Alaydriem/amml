use super::CustomControl;
use crate::state::Selection;

/// The control at the right of a `SettingRow`. For a toggle or a checkbox the whole row is the
/// click target; another control takes its own clicks and gets the row's state to draw with.
pub enum SettingControl<'a> {
    Toggle(&'a mut bool),
    Checkbox(&'a mut Selection),
    Other(CustomControl<'a>),
}
