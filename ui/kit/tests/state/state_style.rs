use amml_ui::state::{InteractionState, StateStyle};
use amml_ui::tokens::{ColorToken, color};
use egui::Color32;

const STYLES: [StateStyle; 5] =
    [StateStyle::Field, StateStyle::Clear, StateStyle::Accent, StateStyle::Soft, StateStyle::Danger];

#[test]
fn field_ladder_steps_up() {
    let fill = |state| StateStyle::Field.fill(state);
    assert_eq!(fill(InteractionState::Rest), <color::BgField as ColorToken>::VALUE);
    assert_eq!(fill(InteractionState::Hover), <color::BgHover as ColorToken>::VALUE);
    assert_eq!(fill(InteractionState::Pressed), <color::BgActive as ColorToken>::VALUE);
}

#[test]
fn every_ladder_matches_paper() {
    let ladders = [
        (
            StateStyle::Clear,
            [Color32::TRANSPARENT, <color::BgHover as ColorToken>::VALUE, <color::BgActive as ColorToken>::VALUE],
        ),
        (
            StateStyle::Accent,
            [
                <color::Accent as ColorToken>::VALUE,
                <color::AccentHover as ColorToken>::VALUE,
                <color::AccentPress as ColorToken>::VALUE,
            ],
        ),
        (
            StateStyle::Soft,
            [
                <color::AccentSoft as ColorToken>::VALUE,
                <color::AccentSoftHover as ColorToken>::VALUE,
                <color::AccentSoftPress as ColorToken>::VALUE,
            ],
        ),
        (
            StateStyle::Danger,
            [
                <color::Danger as ColorToken>::VALUE,
                <color::DangerHover as ColorToken>::VALUE,
                <color::DangerPress as ColorToken>::VALUE,
            ],
        ),
    ];
    for (style, [rest, hover, pressed]) in ladders {
        assert_eq!(style.fill(InteractionState::Rest), rest, "{style:?}");
        assert_eq!(style.fill(InteractionState::Hover), hover, "{style:?}");
        assert_eq!(style.fill(InteractionState::Pressed), pressed, "{style:?}");
    }
}

#[test]
fn focus_and_disabled_keep_the_rest_fill() {
    for style in STYLES {
        let rest = style.fill(InteractionState::Rest);
        assert_eq!(style.fill(InteractionState::Focused), rest, "{style:?}");
        assert_eq!(style.fill(InteractionState::Disabled), rest, "{style:?}");
    }
}
