use std::cell::{Cell, RefCell};

use amml_abi::{AmmlPose, AmmlStatus};

/// The free camera side of a `FakeGame`: whether a handler was captured (`FakeGame::capture_handler`), whether the
/// world has the free preset (`FakeGame::set_free_available`, true at first) and the calls of the fake mod, one line
/// each. The fake has one mod, so no other mod holds the free camera; `held` is whether the fake mod holds it.
pub(crate) struct FakeCameraFree {
    captured: Cell<bool>,
    available: Cell<bool>,
    held: Cell<bool>,
    calls: RefCell<Vec<String>>,
}

impl Default for FakeCameraFree {
    fn default() -> Self {
        Self {
            captured: Cell::new(false),
            available: Cell::new(true),
            held: Cell::new(false),
            calls: RefCell::new(Vec::new()),
        }
    }
}

impl FakeCameraFree {
    pub(crate) fn capture(&self) {
        self.captured.set(true);
    }

    pub(crate) fn set_available(&self, available: bool) {
        self.available.set(available);
    }

    pub(crate) fn available(&self) -> bool {
        self.available.get()
    }

    pub(crate) fn calls(&self) -> Vec<String> {
        self.calls.borrow().clone()
    }

    /// A `set_free` of the fake mod, after the area, gate and thread checks: `NOT_READY` before a capture,
    /// `NOT_AVAILABLE` without the free preset, `INVALID_ARGUMENT` for a pose that is not finite.
    pub(crate) fn set_free(&self, pose: AmmlPose) -> Result<(), AmmlStatus> {
        if !pose.is_finite() {
            return Err(AmmlStatus::INVALID_ARGUMENT);
        }
        if !self.captured.get() {
            return Err(AmmlStatus::NOT_READY);
        }
        if !self.available.get() {
            return Err(AmmlStatus::NOT_AVAILABLE);
        }
        let position = pose.position;
        self.calls.borrow_mut().push(format!(
            "set_free {},{},{} yaw={} pitch={}",
            position.x, position.y, position.z, pose.rotation.yaw, pose.rotation.pitch
        ));
        self.held.set(true);
        Ok(())
    }

    /// A `release_free` of the fake mod: `NOT_READY` before a capture. Gives true when the mod held the free camera
    /// (the clear is recorded and goes out), false when it did not (nothing goes out).
    pub(crate) fn release_free(&self) -> Result<bool, AmmlStatus> {
        if !self.captured.get() {
            return Err(AmmlStatus::NOT_READY);
        }
        if !self.held.replace(false) {
            return Ok(false);
        }
        self.calls.borrow_mut().push("release_free".to_string());
        Ok(true)
    }
}
