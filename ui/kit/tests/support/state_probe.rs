use amml_ui::state::InteractionState;
use egui::{Pos2, Rect, Sense, pos2, vec2};
use egui_kittest::Harness;

// one 100 x 40 click target at (20, 20); records the state the target reads each pass
pub struct StateProbe;

impl StateProbe {
    pub const TARGET: Rect = Rect::from_min_max(pos2(20.0, 20.0), pos2(120.0, 60.0));

    pub fn harness(enabled: bool) -> Harness<'static, Option<InteractionState>> {
        Harness::new_ui_state(
            move |ui, state: &mut Option<InteractionState>| {
                let response = ui.allocate_rect(Self::TARGET, Sense::click());
                *state = Some(InteractionState::of(&response, enabled));
            },
            None,
        )
    }

    pub fn centre() -> Pos2 {
        Self::TARGET.center()
    }

    pub fn outside() -> Pos2 {
        Self::TARGET.max + vec2(100.0, 100.0)
    }
}
