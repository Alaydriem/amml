use egui::{Color32, Pos2};

/// What a curve plot shows. Values are in the unit square: x is time from 0 to 1, y is progress with 0 at the
/// bottom and 1 at the top; an overshooting curve goes past 0 or 1.
pub struct CurvePlotView<'a> {
    /// The curve's y at even steps of x from 0 to 1.
    pub samples: &'a [f32],
    /// The track colour of the key the curve belongs to.
    pub color: Color32,
    /// The two bezier handles of a custom curve.
    pub handles: Option<[Pos2; 2]>,
}
