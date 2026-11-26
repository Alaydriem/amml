use super::{MaterialChoice, MaterialWrite};

/// What the render thread does before one world mesh draw: the material writes, in order, then the draw with
/// `choice`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaterialPlan {
    pub writes: Vec<MaterialWrite>,
    pub choice: MaterialChoice,
}
