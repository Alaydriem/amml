use std::cell::RefCell;

use crate::{MeshId, Vec3};

/// What a `FakeGame` gives one `WorldRendered` dispatch (`FakeGame::raise_world_rendered`): the camera position of the
/// frame (`None` for a frame the game has no view in) and the graphics mode. `frees` holds the meshes the listeners
/// freed through the canvas during the dispatch, in order. `calls` holds one line per `build` and `draw` that the fake
/// accepted, in order: `build quads=<n> style=<blend>/<depth test>/<depth write>/<sides>/<light>/<texture>` and
/// `draw <id> <x> <y> <z> yaw=<degrees>`, the position being `origin` + `offset`.
#[derive(Debug, Default)]
pub struct FakeCanvas {
    pub camera_position: Option<Vec3>,
    pub vibrant: bool,
    pub frees: RefCell<Vec<MeshId>>,
    pub calls: RefCell<Vec<String>>,
}
