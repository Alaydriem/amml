use amml_abi::{AmmlRgba, AmmlShapeOp, AmmlVec3};

use crate::{Rgba, Vec3};

/// One change of `WorldShapes::apply`. `id` is the mod's own shape id: a draw of a live id moves or recolours that
/// shape, a draw of a live id with another kind replaces it, and `Remove` takes it away.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ShapeOp {
    /// A line from `from` to `to`.
    Line { id: u64, from: Vec3, to: Vec3, color: Rgba },
    /// A box with its lowest corner at `from` and its size along each axis in `to`.
    Box { id: u64, from: Vec3, to: Vec3, color: Rgba },
    /// An arrow from `from` to `to`, with the head at `to`.
    Arrow { id: u64, from: Vec3, to: Vec3, color: Rgba },
    /// Removes the shape `id`; nothing when no shape has that id.
    Remove { id: u64 },
}

impl From<ShapeOp> for AmmlShapeOp {
    fn from(op: ShapeOp) -> Self {
        let (kind, id, from, to, color) = match op {
            ShapeOp::Line { id, from, to, color } => (AmmlShapeOp::LINE, id, from, to, color),
            ShapeOp::Box { id, from, to, color } => (AmmlShapeOp::BOX, id, from, to, color),
            ShapeOp::Arrow { id, from, to, color } => (AmmlShapeOp::ARROW, id, from, to, color),
            ShapeOp::Remove { id } => (AmmlShapeOp::REMOVE, id, Vec3::default(), Vec3::default(), Rgba::default()),
        };
        AmmlShapeOp {
            kind,
            _reserved: [0; 7],
            id,
            from: AmmlVec3::from(from),
            to: AmmlVec3::from(to),
            color: AmmlRgba::from(color),
            _pad: 0,
        }
    }
}
