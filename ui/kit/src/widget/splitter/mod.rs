use std::ops::RangeInclusive;

use egui::{Color32, CursorIcon, Id, Painter, Rect, Response, Sense, Ui, Vec2, vec2};

use crate::draw::{Corner, Draw};
use crate::state::InteractionState;
use crate::tokens::{ColorToken, SpacingToken, color, spacing};

/// Paper: Editor, `Editor / Splitter + scroll`, the splitter. A `--spacing-splitter` high bar with
/// a grip in its centre; a drag up or down moves `position` within its range.
pub struct Splitter<'a> {
    id: Id,
    position: &'a mut f32,
    range: RangeInclusive<f32>,
}

impl<'a> Splitter<'a> {
    /// Paper: Splitter + scroll, the grip.
    const GRIP: Vec2 = vec2(32.0, 4.0);

    pub fn new(id: Id, position: &'a mut f32, range: RangeInclusive<f32>) -> Self {
        Self { id, position, range }
    }

    /// Takes the full width. A drag down makes `position` larger.
    pub fn show(self, ui: &mut Ui) -> Response {
        let height = <spacing::Splitter as SpacingToken>::VALUE;
        let rect = ui.allocate_space(vec2(ui.available_width(), height)).1;
        let mut response = ui.interact(rect, self.id, Sense::drag());
        if response.dragged() {
            let delta = response.drag_delta().y;
            if delta != 0.0 {
                // a reversed range (a window shorter than both panes) holds at its start
                let start = *self.range.start();
                let moved = (*self.position + delta).clamp(start, self.range.end().max(start));
                if moved != *self.position {
                    *self.position = moved;
                    response.mark_changed();
                }
            }
        }
        Self::paint(ui.painter(), rect, InteractionState::of(&response, true));
        response.on_hover_and_drag_cursor(CursorIcon::ResizeVertical)
    }

    /// Paints the bar in `state`: Hover is the pointer over it, Pressed is a drag.
    pub fn paint(painter: &Painter, rect: Rect, state: InteractionState) {
        let (bar, grip) = Self::colors(state);
        Draw::fill(painter, rect, Corner::None, bar);
        Draw::fill(painter, Rect::from_center_size(rect.center(), Self::GRIP), Corner::Full, grip);
    }

    // (bar, grip)
    fn colors(state: InteractionState) -> (Color32, Color32) {
        match state {
            InteractionState::Hover => (<color::BgHover as ColorToken>::VALUE, <color::TextFaint as ColorToken>::VALUE),
            InteractionState::Pressed => {
                (<color::AccentSoft as ColorToken>::VALUE, <color::Accent as ColorToken>::VALUE)
            }
            InteractionState::Rest | InteractionState::Focused | InteractionState::Disabled => {
                (<color::BgPanel as ColorToken>::VALUE, <color::LineStrong as ColorToken>::VALUE)
            }
        }
    }
}
