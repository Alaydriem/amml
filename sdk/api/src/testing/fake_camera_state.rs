use std::cell::{Cell, RefCell};

use amml_abi::{AmmlPose, AmmlStatus};

use crate::testing::FakeCameraRun;
use crate::testing::fake_camera::FakeCamera;
use crate::{Pose, Projection, Vec3};

/// The camera side of a `FakeGame`: the frame counter and the dispatch that runs now. A camera call answers only inside
/// the dispatch of its frame id, as the loader does; the projection answers in After only.
#[derive(Default)]
pub(crate) struct FakeCameraState {
    frame: Cell<u64>,
    running: RefCell<Option<FakeCameraRun>>,
    body_hidden: Cell<bool>,
}

impl FakeCameraState {
    /// A `hide_actor` of the fake mod: the local player's body is hidden (`on`) or shown.
    pub(crate) fn set_body_hidden(&self, on: bool) {
        self.body_hidden.set(on);
    }

    /// Whether the fake mod hides the local player's body.
    pub(crate) fn body_hidden(&self) -> bool {
        self.body_hidden.get()
    }

    /// A dispatch starts: a Before dispatch starts a new frame, an After dispatch keeps the frame of the last Before (the
    /// first frame when none ran). Gives the frame id.
    pub(crate) fn enter(&self, after: bool, camera: &FakeCamera) -> u64 {
        if !after || self.frame.get() == 0 {
            self.frame.set(self.frame.get() + 1);
        }
        let frame = self.frame.get();
        *self.running.borrow_mut() = Some(FakeCameraRun {
            frame,
            after,
            projection: camera.projection,
            rendered: camera.rendered,
            poses: Vec::new(),
        });
        frame
    }

    /// The running dispatch ends; gives the poses set in it.
    pub(crate) fn leave(&self) -> Vec<Pose> {
        self.running.borrow_mut().take().map(|run| run.poses).unwrap_or_default()
    }

    // Runs `body` on the dispatch of `frame`; `REFUSED` outside it.
    fn in_frame<R>(
        &self,
        frame: u64,
        body: impl FnOnce(&mut FakeCameraRun) -> Result<R, AmmlStatus>,
    ) -> Result<R, AmmlStatus> {
        let mut running = self.running.borrow_mut();
        let run = running.as_mut().filter(|run| run.frame == frame).ok_or(AmmlStatus::REFUSED)?;
        body(run)
    }

    /// A `set_pose` of the loader in the dispatch of `frame`: a pose that is not finite is an invalid argument.
    pub(crate) fn set_pose(&self, frame: u64, pose: AmmlPose) -> Result<(), AmmlStatus> {
        self.in_frame(frame, |run| {
            if !pose.is_finite() {
                return Err(AmmlStatus::INVALID_ARGUMENT);
            }
            run.poses.push(Pose::from(pose));
            Ok(())
        })
    }

    pub(crate) fn rendered(&self, frame: u64) -> Result<Vec3, AmmlStatus> {
        self.in_frame(frame, |run| run.rendered.ok_or(AmmlStatus::FAILED))
    }

    pub(crate) fn projection(&self, frame: u64) -> Result<Projection, AmmlStatus> {
        self.in_frame(frame, |run| {
            if !run.after {
                return Err(AmmlStatus::REFUSED);
            }
            run.projection.ok_or(AmmlStatus::FAILED)
        })
    }
}
