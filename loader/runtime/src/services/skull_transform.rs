use crate::{SkullFace, WorldTransform};

/// The transform of a skull ghost on builds whose skull renderer has no pose arguments (70.x).
///
/// 70.26 `SkullBlockRenderer::render` (0x8939da0) takes the face from the block and the turn from the actor, adds the
/// offset of the face to the position, translates the matrix stack top by it, negates columns 0 and 1 and turns about
/// (0, 1, 0) (survey section 7.6, rows K1..K5). `renderSkullUpright` (0x893abe0) does the same with the floor offset
/// (0, 0.0001, 0) and the turn pi (row K7). The loader puts `matrix` on the stack top before `renderSkullUpright`, so
/// the two give the pose of `render`. Matrices are stored by columns, as `glm::mat4`; the turn is `glm::rotate`
/// (0xf954e0: column 0 becomes (cos, 0, -sin)).
pub struct SkullTransform;

impl SkullTransform {
    /// The floor face (`facing_direction` 1).
    pub const FLOOR: u8 = 1;
    /// The y offset of a floor skull (0x12dfdd6c, 0x893a166 and 0x893ac7a).
    pub const FLOOR_Y: f32 = f32::from_bits(0x38D1_B717);
    /// The y offset of a wall skull (0x12c77cd0, 0x893a41f).
    pub const WALL_Y: f32 = f32::from_bits(0x3E80_0000);
    /// The offset to the centre of the block along x and z.
    pub const CENTRE: f32 = f32::from_bits(0x3F00_0000);
    /// The offset of a wall skull away from the block it hangs on (0x1313fd40, 0x1313fd50).
    pub const FAR: f32 = f32::from_bits(0x3F3D_70A4);
    /// The offset of a wall skull towards the block it hangs on (0x1313fd30, 0x1313fd60).
    pub const NEAR: f32 = f32::from_bits(0x3E85_1EB8);
    /// Degrees to radians (0x12dfdc50, `mulss` at 0x893a18e).
    pub const DEGREES_TO_RADIANS: f32 = f32::from_bits(0x3C8E_FA35);
    /// pi: the turn of face 3 (0x893a5ae) and of `renderSkullUpright` (0x893ad02).
    pub const PI: f32 = f32::from_bits(0x4049_0FDB);
    /// 3 pi / 2: the turn of face 4 (0x893a44f).
    pub const THREE_HALF_PI: f32 = f32::from_bits(0x4096_CBE4);
    /// pi / 2: the turn of face 5 and of any other face (0x893a5d3).
    pub const HALF_PI: f32 = f32::from_bits(0x3FC9_0FDB);
    /// Degrees per sixteenth of a turn.
    pub const DEGREES_PER_TURN: f32 = 22.5;

    /// The rotation in degrees a skull of `face` with `turn` sixteenths has: `turn * 22.5` on the floor, 0 on a wall (the
    /// 26.63 `render` passes the actor's rotation for face 1 and 0 for any other face).
    pub fn degrees(face: u8, turn: u8) -> f32 {
        if face == Self::FLOOR { f32::from(turn) * Self::DEGREES_PER_TURN } else { 0.0 }
    }

    /// The offset and turn of `face` with `turn` sixteenths (rows K1..K5); face 0 and any face above 5 take the face 5
    /// row, as the game's `else` branch does.
    pub fn face(face: u8, turn: u8) -> SkullFace {
        let (offset, angle) = match face {
            Self::FLOOR => {
                ([Self::CENTRE, Self::FLOOR_Y, Self::CENTRE], Self::degrees(face, turn) * Self::DEGREES_TO_RADIANS)
            }
            2 => ([Self::CENTRE, Self::WALL_Y, Self::FAR], 0.0),
            3 => ([Self::CENTRE, Self::WALL_Y, Self::NEAR], Self::PI),
            4 => ([Self::FAR, Self::WALL_Y, Self::CENTRE], Self::THREE_HALF_PI),
            _ => ([Self::NEAR, Self::WALL_Y, Self::CENTRE], Self::HALF_PI),
        };
        SkullFace { offset, angle }
    }

    /// The transform 70.26 `render` applies to the stack top for `face` and `turn` at the render position `pos`, step by
    /// step as the game does it on an identity top.
    pub fn game(face: u8, turn: u8, pos: [f32; 3]) -> [[f32; 4]; 4] {
        let pose = Self::face(face, turn);
        Self::steps(std::array::from_fn(|axis| pos[axis] + pose.offset[axis]), pose.angle)
    }

    /// The transform `renderSkullUpright` applies to the stack top at the render position `pos`.
    pub fn upright(pos: [f32; 3]) -> [[f32; 4]; 4] {
        Self::steps([pos[0], pos[1] + Self::FLOOR_Y, pos[2]], Self::PI)
    }

    /// The transform the loader puts on the stack top before `renderSkullUpright(pos)` so that the skull takes the pose
    /// of `face` and `turn`: `translate(pos + offset) * rotate_y(pi - angle) * translate(-(pos + floor))`. The negation of
    /// columns 0 and 1 cancels between the two (`N * rotate_y(a) * N = rotate_y(-a)`).
    pub fn matrix(face: u8, turn: u8, pos: [f32; 3]) -> [[f32; 4]; 4] {
        let pose = Self::face(face, turn);
        let to = Self::translate(std::array::from_fn(|axis| pos[axis] + pose.offset[axis]));
        let back = Self::translate([-pos[0], -(pos[1] + Self::FLOOR_Y), -pos[2]]);
        let turned = WorldTransform::compose(&Self::rotate_y(Self::PI - pose.angle), &back);
        WorldTransform::compose(&to, &turned)
    }

    // An identity top translated by `at`, its columns 0 and 1 negated, then turned by `angle` about +Y.
    fn steps(at: [f32; 3], angle: f32) -> [[f32; 4]; 4] {
        let mut top = Self::translate(at);
        for column in &mut top[..2] {
            for value in column.iter_mut() {
                *value = -*value;
            }
        }
        WorldTransform::compose(&top, &Self::rotate_y(angle))
    }

    fn translate(at: [f32; 3]) -> [[f32; 4]; 4] {
        [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [at[0], at[1], at[2], 1.0]]
    }

    // `glm::rotate(identity, angle, (0, 1, 0))`.
    fn rotate_y(angle: f32) -> [[f32; 4]; 4] {
        let (sin, cos) = angle.sin_cos();
        [[cos, 0.0, -sin, 0.0], [0.0, 1.0, 0.0, 0.0], [sin, 0.0, cos, 0.0], [0.0, 0.0, 0.0, 1.0]]
    }
}
