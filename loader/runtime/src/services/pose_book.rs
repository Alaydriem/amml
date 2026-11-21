use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, MutexGuard};

use amml_abi::{AmmlPose, AmmlVec3};

use crate::PoseRefusal;
use crate::structs::held_pose::HeldPose;
use crate::structs::pose_book_state::PoseBookState;

/// The camera pose a mod set for the current camera frame, and the position the loader wrote with it.
///
/// A camera frame starts at the first `setupCamera` of a new frame id (`begin_frame`). A pose is for the frame it was
/// set in only: the next frame drops it, so a frame without a `set` writes nothing and the game camera shows; a mod
/// that is disabled or panics leaves no pose behind for more than its own frame. One mod writes per frame: once a mod
/// set the pose of a frame, a `set` of another mod in that frame is refused; the same mod may set it again.
///
/// The loader writes the pose (`pose` for the rotation, `write_position` for the positions) from its camera detours;
/// the position written in a frame is the `rendered` position of the next frame. The book makes no game call.
pub struct PoseBook {
    state: Mutex<PoseBookState>,
    // True while a pose is held; the detours read it without the lock.
    held: AtomicBool,
}

impl Default for PoseBook {
    fn default() -> Self {
        Self::new()
    }
}

impl PoseBook {
    pub fn new() -> Self {
        Self {
            state: Mutex::new(PoseBookState { frame: 0, held: None, written: None, rendered: None }),
            held: AtomicBool::new(false),
        }
    }

    // The state holds plain data, so a lock poisoned by a panic is recovered.
    fn lock(&self) -> MutexGuard<'_, PoseBookState> {
        self.state.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// A `setupCamera` of frame `frame` starts. For a new frame the pose of the frame before is dropped and the
    /// position written in it becomes the rendered position; gives true then (the first setup of the frame), false for
    /// a later setup of the same frame.
    pub fn begin_frame(&self, frame: u64) -> bool {
        let mut state = self.lock();
        if state.frame == frame {
            return false;
        }
        state.frame = frame;
        state.held = None;
        state.rendered = state.written.take();
        self.held.store(false, Ordering::Release);
        true
    }

    /// Sets `pose` for the mod `owner` in frame `frame`. `OtherFrame` unless `frame` is the current camera frame,
    /// `Taken` when another mod set the pose of this frame first.
    pub fn set(&self, owner: usize, frame: u64, pose: AmmlPose) -> Result<(), PoseRefusal> {
        let mut state = self.lock();
        if frame == 0 || state.frame != frame {
            return Err(PoseRefusal::OtherFrame);
        }
        if state.held.is_some_and(|held| held.owner != owner) {
            return Err(PoseRefusal::Taken);
        }
        state.held = Some(HeldPose { owner, pose });
        self.held.store(true, Ordering::Release);
        Ok(())
    }

    /// The pose of the current camera frame.
    pub fn pose(&self) -> Option<AmmlPose> {
        self.lock().held.map(|held| held.pose)
    }

    /// The position of the pose of the current camera frame, recorded as written in this frame.
    pub fn write_position(&self) -> Option<AmmlVec3> {
        let mut state = self.lock();
        let position = state.held.map(|held| held.pose.position)?;
        state.written = Some(position);
        Some(position)
    }

    /// The position the loader wrote in the camera frame before the current one.
    pub fn rendered(&self) -> Option<AmmlVec3> {
        self.lock().rendered
    }

    /// Whether a pose is held, read without the lock.
    pub fn is_held(&self) -> bool {
        self.held.load(Ordering::Relaxed)
    }

    /// The mod `owner` is disabled: its pose is dropped.
    pub fn disable_owner(&self, owner: usize) {
        let mut state = self.lock();
        if state.held.is_some_and(|held| held.owner == owner) {
            state.held = None;
            self.held.store(false, Ordering::Release);
        }
    }

    /// The world is left: the pose and both positions are forgotten.
    pub fn clear(&self) {
        let mut state = self.lock();
        state.held = None;
        state.written = None;
        state.rendered = None;
        self.held.store(false, Ordering::Release);
    }
}
