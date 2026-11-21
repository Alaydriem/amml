use amml_abi::AmmlRotation;

use crate::CameraBasis;

/// The matrix rules of the camera: the view basis of a pose and the field of view of a projection matrix.
///
/// Angles are in degrees with the signs of `AmmlRotation`: yaw 0 looks along +Z and 90 along -X, pitch 0 is level and
/// positive pitch looks down. The right vector comes from the yaw alone, so a camera that looks straight up or down
/// keeps a finite basis. A positive roll turns the camera clockwise about its forward vector as the viewer sees it: at
/// 90 the camera's up is the right vector of the level camera.
pub struct CameraMath;

impl CameraMath {
    /// The basis and the view matrix of a camera at the origin with `rotation` and `roll`.
    pub fn basis(rotation: AmmlRotation, roll: f32) -> CameraBasis {
        let (sin_yaw, cos_yaw) = rotation.yaw.to_radians().sin_cos();
        let (sin_pitch, cos_pitch) = rotation.pitch.to_radians().sin_cos();
        let forward = [-sin_yaw * cos_pitch, -sin_pitch, cos_yaw * cos_pitch];
        let level_right = [-cos_yaw, 0.0, -sin_yaw];
        let level_up = Self::cross(level_right, forward);
        let (sin_roll, cos_roll) = roll.to_radians().sin_cos();
        let right: [f32; 3] = std::array::from_fn(|axis| level_right[axis] * cos_roll - level_up[axis] * sin_roll);
        let up: [f32; 3] = std::array::from_fn(|axis| level_up[axis] * cos_roll + level_right[axis] * sin_roll);
        let mut view = [[0.0f32; 4]; 4];
        for axis in 0..3 {
            view[axis][0] = right[axis];
            view[axis][1] = up[axis];
            view[axis][2] = -forward[axis];
        }
        view[3][3] = 1.0;
        // The eye is at the origin, so the view is a pure rotation and its inverse is its transpose.
        let inverse = std::array::from_fn(|column| std::array::from_fn(|row| view[row][column]));
        CameraBasis { forward, right, up, view, inverse }
    }

    /// The vertical field of view in degrees and the aspect ratio of a perspective `matrix` stored by columns: element
    /// [1][1] is the vertical focal length and [1][1] / [0][0] the aspect ratio. `None` when either focal length is not
    /// finite and positive.
    pub fn projection(matrix: &[[f32; 4]; 4]) -> Option<(f32, f32)> {
        let (focal_x, focal_y) = (matrix[0][0], matrix[1][1]);
        let usable = focal_x.is_finite() && focal_y.is_finite() && focal_x > 0.0 && focal_y > 0.0;
        usable.then(|| ((2.0 * (1.0 / focal_y).atan()).to_degrees(), focal_y / focal_x))
    }

    fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
        [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
    }
}
