use amml_abi::AmmlFace;

/// A face of a block. A value this crate does not know is `Other`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Face {
    Down,
    Up,
    North,
    South,
    West,
    East,
    Other(u8),
}

impl From<AmmlFace> for Face {
    fn from(face: AmmlFace) -> Self {
        match face {
            AmmlFace::DOWN => Self::Down,
            AmmlFace::UP => Self::Up,
            AmmlFace::NORTH => Self::North,
            AmmlFace::SOUTH => Self::South,
            AmmlFace::WEST => Self::West,
            AmmlFace::EAST => Self::East,
            AmmlFace(other) => Self::Other(other),
        }
    }
}

impl From<Face> for AmmlFace {
    fn from(face: Face) -> Self {
        match face {
            Face::Down => Self::DOWN,
            Face::Up => Self::UP,
            Face::North => Self::NORTH,
            Face::South => Self::SOUTH,
            Face::West => Self::WEST,
            Face::East => Self::EAST,
            Face::Other(other) => Self(other),
        }
    }
}
