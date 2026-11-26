use amml_abi::{AmmlBlockHit, AmmlFace};

use crate::{BlockPos, Face, Vec3};

/// The block a `BlockSource::raycast` ray met first: the block, the point on it where the ray met it, and the face the
/// ray entered through.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BlockHit {
    pub pos: BlockPos,
    pub point: Vec3,
    pub face: Face,
}

impl From<AmmlBlockHit> for BlockHit {
    fn from(hit: AmmlBlockHit) -> Self {
        Self { pos: hit.pos.into(), point: hit.point.into(), face: hit.face.into() }
    }
}

impl From<BlockHit> for AmmlBlockHit {
    fn from(hit: BlockHit) -> Self {
        Self { hit: 1, face: AmmlFace::from(hit.face), _reserved: [0; 2], pos: hit.pos.into(), point: hit.point.into() }
    }
}
