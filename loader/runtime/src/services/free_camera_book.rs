use crate::{FreeCameraRefusal, FreeCameraRelease};

/// Which mod holds the free camera of the joined world.
///
/// One mod holds it at a time: the first `set` takes it, a `set` or `release` of another mod is refused, and the
/// holder's `release` frees it. A disabled holder frees it, and the loader sends the clear instruction only while the
/// game runs and the disable runs on the game thread; the kept packet goes back either way. A server instruction that
/// sets or clears the camera forgets the holder (the server took the camera), and so does a world leave (the camera of
/// the left world is gone with it). The book makes no game call; the game thread, the captured handler and the preset
/// are checked by the loader.
#[derive(Debug, Default)]
pub struct FreeCameraBook {
    owner: Option<usize>,
}

impl FreeCameraBook {
    pub fn new() -> Self {
        Self { owner: None }
    }

    /// The mod that holds the free camera.
    pub fn owner(&self) -> Option<usize> {
        self.owner
    }

    /// A `set_free` of the mod `owner`: it holds the free camera now. `Taken` when another mod holds it.
    pub fn set(&mut self, owner: usize) -> Result<(), FreeCameraRefusal> {
        match self.owner {
            Some(held) if held != owner => Err(FreeCameraRefusal::Taken),
            _ => {
                self.owner = Some(owner);
                Ok(())
            }
        }
    }

    /// A `release_free` of the mod `owner`: true when it held the free camera (the clear goes out and the camera is
    /// free), false when no mod holds it (nothing goes out). `Taken` when another mod holds it.
    pub fn release(&mut self, owner: usize) -> Result<bool, FreeCameraRefusal> {
        match self.owner {
            None => Ok(false),
            Some(held) if held != owner => Err(FreeCameraRefusal::Taken),
            Some(_) => {
                self.owner = None;
                Ok(true)
            }
        }
    }

    /// The mod `owner` is disabled. When it held the free camera the camera is free and the packet goes back; the clear
    /// goes out only when `live` (the game runs and this is the game thread). Any other mod's disable changes nothing.
    pub fn disable(&mut self, owner: usize, live: bool) -> FreeCameraRelease {
        if self.owner != Some(owner) {
            return FreeCameraRelease::default();
        }
        self.owner = None;
        FreeCameraRelease { clear: live, drop_packet: true }
    }

    /// The server sent a camera instruction with a set or a clear part: it took the camera, so no mod holds it now.
    /// The old holder's disable then sends no clear.
    pub fn server_instruction(&mut self) {
        self.owner = None;
    }

    /// Before `WorldLeft`: the holder is forgotten.
    pub fn world_left(&mut self) {
        self.owner = None;
    }
}
