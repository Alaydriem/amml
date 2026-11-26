use std::cell::RefCell;

use crate::{Pose, Projection, Vec3};

/// What a `FakeGame` gives one `CameraSetup` dispatch (`FakeGame::raise_camera_setup`): the projection of the camera
/// (answered in After only, `None` for a matrix without usable focal lengths) and the position the loader wrote in the
/// frame before (`None` for none). `recorded` holds the poses the listeners set through the camera during the
/// dispatches of this fake camera, in order; `poses` gives a copy.
#[derive(Debug, Default)]
pub struct FakeCamera {
    pub projection: Option<Projection>,
    pub rendered: Option<Vec3>,
    pub recorded: RefCell<Vec<Pose>>,
}

impl FakeCamera {
    /// The poses set so far, in order.
    pub fn poses(&self) -> Vec<Pose> {
        self.recorded.borrow().clone()
    }
}
