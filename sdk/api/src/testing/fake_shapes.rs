use std::cell::{Cell, RefCell};
use std::collections::HashMap;

use amml_abi::{AmmlShapeOp, AmmlStatus};

/// The shapes side of a `FakeGame`: the target epoch (`FakeGame::set_target_epoch`, 0 = no handler captured yet) and
/// the shape ops of the fake mod that went out, one line each, and the kind of each live shape id, so that the number
/// of payloads counts as the loader's: one per draw, two for a kind change of a live id, one for the remove of a live id
/// and none for the remove of an unknown id.
#[derive(Default)]
pub(crate) struct FakeShapes {
    target_epoch: Cell<u32>,
    ops: RefCell<Vec<String>>,
    live: RefCell<HashMap<u64, u8>>,
}

impl FakeShapes {
    /// The most ops one apply takes, as on the loader.
    pub(crate) const MAX_BATCH: usize = 800;

    /// A new handler generation: as on the loader, the game dropped every shape.
    pub(crate) fn set_target_epoch(&self, epoch: u32) {
        if epoch != self.target_epoch.replace(epoch) {
            self.live.borrow_mut().clear();
        }
    }

    /// The target epoch; `NOT_READY` before one is set.
    pub(crate) fn target_epoch(&self) -> Result<u32, AmmlStatus> {
        match self.target_epoch.get() {
            0 => Err(AmmlStatus::NOT_READY),
            epoch => Ok(epoch),
        }
    }

    pub(crate) fn ops(&self) -> Vec<String> {
        self.ops.borrow().clone()
    }

    /// An apply of the fake mod after the area, gate and thread checks: `REFUSED` over the maximum, `NOT_READY` before
    /// a target epoch, `INVALID_ARGUMENT` for an unknown kind or a number that is not finite (nothing is recorded).
    /// Records one line per op and gives the number of payloads, counted as the loader counts them.
    pub(crate) fn apply(&self, dimension: i32, ops: &[AmmlShapeOp]) -> Result<u32, AmmlStatus> {
        if ops.len() > Self::MAX_BATCH {
            return Err(AmmlStatus::REFUSED);
        }
        self.target_epoch()?;
        let lines = ops.iter().map(|op| Self::line(dimension, op)).collect::<Result<Vec<String>, AmmlStatus>>()?;
        let mut live = self.live.borrow_mut();
        let mut payloads = 0u32;
        for op in ops {
            payloads += match (op.kind, live.get(&op.id).copied()) {
                (AmmlShapeOp::REMOVE, None) => 0,
                (AmmlShapeOp::REMOVE, Some(_)) => {
                    live.remove(&op.id);
                    1
                }
                (kind, held) => {
                    live.insert(op.id, kind);
                    if held.is_some_and(|held| held != kind) { 2 } else { 1 }
                }
            };
        }
        self.ops.borrow_mut().extend(lines);
        Ok(payloads)
    }

    fn line(dimension: i32, op: &AmmlShapeOp) -> Result<String, AmmlStatus> {
        let name = match op.kind {
            AmmlShapeOp::LINE => "line",
            AmmlShapeOp::BOX => "box",
            AmmlShapeOp::ARROW => "arrow",
            AmmlShapeOp::REMOVE => return Ok(format!("remove id={} dim={dimension}", op.id)),
            _ => return Err(AmmlStatus::INVALID_ARGUMENT),
        };
        let (from, to, color) = (op.from, op.to, op.color);
        if ![from.x, from.y, from.z, to.x, to.y, to.z].iter().all(|value| value.is_finite()) {
            return Err(AmmlStatus::INVALID_ARGUMENT);
        }
        Ok(format!(
            "{name} id={} from={},{},{} to={},{},{} color={},{},{},{} dim={dimension}",
            op.id, from.x, from.y, from.z, to.x, to.y, to.z, color.r, color.g, color.b, color.a
        ))
    }
}
