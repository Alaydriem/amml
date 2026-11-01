use egui::Color32;

use super::InteractionState;
use crate::tokens::{ColorToken, color};

/// A fill ladder: hover and pressed go one step up from the rest fill (Paper state model).
/// Focused and Disabled keep the rest fill; the ring and the opacity do the rest.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StateStyle {
    /// `bg-field > bg-hover > bg-active`: fields and secondary controls.
    Field,
    /// `none > bg-hover > bg-active`: ghost buttons, menu items, rows.
    Clear,
    /// `accent > accent-hover > accent-press`: primary controls and on states.
    Accent,
    /// `accent-soft > accent-soft-hover > accent-soft-press`: selected and toggled-on controls.
    Soft,
    /// `danger > danger-hover > danger-press`.
    Danger,
}

impl StateStyle {
    pub fn fill(self, state: InteractionState) -> Color32 {
        let [rest, hover, pressed] = self.ladder();
        match state {
            InteractionState::Hover => hover,
            InteractionState::Pressed => pressed,
            InteractionState::Rest | InteractionState::Focused | InteractionState::Disabled => rest,
        }
    }

    fn ladder(self) -> [Color32; 3] {
        match self {
            Self::Field => [
                <color::BgField as ColorToken>::VALUE,
                <color::BgHover as ColorToken>::VALUE,
                <color::BgActive as ColorToken>::VALUE,
            ],
            Self::Clear => {
                [Color32::TRANSPARENT, <color::BgHover as ColorToken>::VALUE, <color::BgActive as ColorToken>::VALUE]
            }
            Self::Accent => [
                <color::Accent as ColorToken>::VALUE,
                <color::AccentHover as ColorToken>::VALUE,
                <color::AccentPress as ColorToken>::VALUE,
            ],
            Self::Soft => [
                <color::AccentSoft as ColorToken>::VALUE,
                <color::AccentSoftHover as ColorToken>::VALUE,
                <color::AccentSoftPress as ColorToken>::VALUE,
            ],
            Self::Danger => [
                <color::Danger as ColorToken>::VALUE,
                <color::DangerHover as ColorToken>::VALUE,
                <color::DangerPress as ColorToken>::VALUE,
            ],
        }
    }
}
