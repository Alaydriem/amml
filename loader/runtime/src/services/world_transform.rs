use crate::WorldPlacement;

/// The world matrix of one mesh draw, relative to the camera of the frame.
///
/// The game draws the world relative to the camera position, as f32. The position of the mesh (`origin` + `offset` -
/// camera) is computed in f64 first, so a mesh far from the world origin keeps its sub-block precision; then
/// `translate(position) * rotate_y(yaw) * scale` is built as an f32 matrix stored by columns (as `glm::mat4`). The yaw
/// has the signs of the actor rotation: 0 keeps +Z, 90 turns +Z to -X.
pub struct WorldTransform;

impl WorldTransform {
    /// The position of the mesh point (0, 0, 0) relative to `camera`.
    pub fn position(placement: &WorldPlacement, camera: [f32; 3]) -> [f64; 3] {
        std::array::from_fn(|axis| {
            f64::from(placement.origin[axis]) + f64::from(placement.offset[axis]) - f64::from(camera[axis])
        })
    }

    /// `translate(position) * rotate_y(yaw_degrees) * scale`, stored by columns.
    pub fn matrix(placement: &WorldPlacement, camera: [f32; 3]) -> [[f32; 4]; 4] {
        let [x, y, z] = Self::position(placement, camera);
        let (sin, cos) = f64::from(placement.yaw_degrees).to_radians().sin_cos();
        let scale = f64::from(placement.scale);
        let narrow = |value: f64| value as f32;
        [
            [narrow(cos * scale), 0.0, narrow(sin * scale), 0.0],
            [0.0, narrow(scale), 0.0, 0.0],
            [narrow(-sin * scale), 0.0, narrow(cos * scale), 0.0],
            [narrow(x), narrow(y), narrow(z), 1.0],
        ]
    }

    /// `top * matrix`, both stored by columns: the stack top with the mesh matrix applied first.
    pub fn compose(top: &[[f32; 4]; 4], matrix: &[[f32; 4]; 4]) -> [[f32; 4]; 4] {
        std::array::from_fn(|column| std::array::from_fn(|row| (0..4).map(|k| top[k][row] * matrix[column][k]).sum()))
    }

    /// Where `matrix` (stored by columns) maps `point`.
    pub fn apply(matrix: &[[f32; 4]; 4], point: [f32; 3]) -> [f32; 3] {
        std::array::from_fn(|row| {
            matrix[0][row] * point[0] + matrix[1][row] * point[1] + matrix[2][row] * point[2] + matrix[3][row]
        })
    }
}
