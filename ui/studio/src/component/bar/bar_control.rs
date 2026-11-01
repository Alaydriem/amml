use amml_ui::state::InteractionState;
use amml_ui::widget::TextButton;
use egui::{CursorIcon, Response, Sense, Ui, vec2};

/// The text buttons, chip and switch in the top bar and transport. Paper draws them 30 high, under the
/// `--spacing-field` height amml-ui buttons use elsewhere.
pub(crate) struct BarControl;

impl BarControl {
    /// Paper: `G-0` and `UW-0`, the height of a bar's text button, scene chip and Edit / Preview switch.
    pub(crate) const HEIGHT: f32 = 30.0;

    /// Shows `button` at its own width and the bar height. `enabled` false takes no clicks and mutes it.
    pub(crate) fn text(ui: &mut Ui, button: &TextButton, enabled: bool) -> Response {
        let width = button.desired_width(ui.ctx());
        let sense = if enabled { Sense::click() } else { Sense::hover() };
        let (rect, response) = ui.allocate_exact_size(vec2(width, Self::HEIGHT), sense);
        button.paint(ui.painter(), rect, InteractionState::of(&response, enabled));
        if enabled { response.on_hover_cursor(CursorIcon::PointingHand) } else { response }
    }
}
