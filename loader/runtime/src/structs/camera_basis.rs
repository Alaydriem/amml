/// The view rotation of a camera at the origin: its basis vectors, the view matrix as `glm::lookAt` builds it (right
/// handed, stored by columns: the rows of the rotation are right, up and minus forward) and its inverse.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CameraBasis {
    pub forward: [f32; 3],
    pub right: [f32; 3],
    pub up: [f32; 3],
    pub view: [[f32; 4]; 4],
    pub inverse: [[f32; 4]; 4],
}
