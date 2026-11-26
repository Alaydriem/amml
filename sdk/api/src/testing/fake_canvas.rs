use std::cell::RefCell;

use crate::{MeshId, Vec3};

/// What a `FakeGame` gives one `WorldRendered` dispatch (`FakeGame::raise_world_rendered`): the camera position of the
/// frame (`None` for a frame the game has no view in), its clip matrix (projection times view by columns; `None` for a
/// frame without a camera) and the graphics mode. `frees` holds the meshes the listeners
/// freed through the canvas during the dispatch, in order. `calls` holds one line per `build` and `draw` that the fake
/// accepted, in order: `build quads=<n> style=<blend>/<depth test>/<depth write>/<sides>/<light>/<texture>` and
/// `draw <id> <x> <y> <z> yaw=<degrees>`, the position being `origin` + `offset`, and `mesher build cells=<n> quads=<n>` for a
/// `BlockMesher::build` (the fake draws each non-empty inner cell as six quads and gives a new mesh for each build that drew
/// a cell).
#[derive(Debug, Default)]
pub struct FakeCanvas {
    pub camera_position: Option<Vec3>,
    pub clip_matrix: Option<[[f32; 4]; 4]>,
    pub vibrant: bool,
    pub frees: RefCell<Vec<MeshId>>,
    pub calls: RefCell<Vec<String>>,
}
