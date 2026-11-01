use egui::{Context, Id};

/// The time parts that move: the toggle knob, the selection slides and the spinner. Colours
/// change in steps between states, as Paper draws them; they do not fade.
pub struct Motion;

impl Motion {
    /// The studio's motion time. Paper has no motion tokens.
    pub const DURATION: f32 = 0.12;
    /// Spinner speed in full turns per second (the studio's 5 radians per second, rounded).
    pub const TURNS_PER_SECOND: f64 = 0.8;

    /// Eases from 0 to 1 while `on`, and back, with a cubic ease-out.
    pub fn flag(ctx: &Context, id: Id, on: bool) -> f32 {
        ctx.animate_bool_with_time_and_easing(id, on, Self::DURATION, egui::emath::easing::cubic_out)
    }

    /// Eases towards `target` in `DURATION`.
    pub fn value(ctx: &Context, id: Id, target: f32) -> f32 {
        ctx.animate_value_with_time(id, target, Self::DURATION)
    }

    /// The spinner turn now, in 0..1. Asks for the next frame, so the spinner keeps turning.
    pub fn turn(ctx: &Context) -> f32 {
        ctx.request_repaint();
        let time = ctx.input(|input| input.time);
        (time * Self::TURNS_PER_SECOND).rem_euclid(1.0) as f32
    }
}
