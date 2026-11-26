use amml_abi::AmmlHitResult;

use crate::{BlockHit, Vec3};

/// What the crosshair of the local player points at, from the game's own hit result (`LocalPlayer::hit_result`): a
/// block (the block, the point on it and the face the pick entered), an entity (the point only: the game keeps the
/// entity as a weak reference, not as a runtime id), or nothing (also an entity out of reach).
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum HitResult {
    Nothing,
    Block(BlockHit),
    Entity { point: Vec3 },
}

impl From<AmmlHitResult> for HitResult {
    fn from(hit: AmmlHitResult) -> Self {
        match hit.kind {
            AmmlHitResult::BLOCK => {
                Self::Block(BlockHit { pos: hit.pos.into(), point: hit.point.into(), face: hit.face.into() })
            }
            AmmlHitResult::ENTITY => Self::Entity { point: hit.point.into() },
            _ => Self::Nothing,
        }
    }
}

impl From<HitResult> for AmmlHitResult {
    fn from(hit: HitResult) -> Self {
        match hit {
            HitResult::Block(block) => Self {
                kind: Self::BLOCK,
                face: block.face.into(),
                _reserved: [0; 2],
                pos: block.pos.into(),
                point: block.point.into(),
            },
            HitResult::Entity { point } => Self { kind: Self::ENTITY, point: point.into(), ..Self::default() },
            HitResult::Nothing => Self::default(),
        }
    }
}
