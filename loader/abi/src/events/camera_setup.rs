use crate::{AmmlEvent, AmmlEventHeader};

/// The game sets up the camera of the primary client for a rendered frame (`LevelRendererPlayer::setupCamera`):
/// Before the game's call and After it.
///
/// `frame_id` is the count of `FrameUpdate` events when the call came, so it changes once per game frame;
/// `first_in_frame` is 1 for the first `setupCamera` of a `frame_id` (Before and After of that call) and 0 for any
/// later one. The camera calls of `AmmlCameraTable` take `frame_id` and give `REFUSED` outside the dispatch of that
/// frame on the thread that raised it. Raised only while a world is joined, on the thread that runs `setupCamera`: the
/// game thread (`GAME`, with the primary client in the context) when the game sets the camera up there, else `RENDER`.
/// No field is writable: the loader writes `frame_id`, `first_in_frame` and `_reserved` back after each listener.
/// `_reserved` stays reserved; a later version appends its fields after it, so its size is 40 or more.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct CameraSetup {
    pub header: AmmlEventHeader,
    pub frame_id: u64,
    pub first_in_frame: u8,
    pub _reserved: [u8; 7],
}

impl AmmlEvent for CameraSetup {
    const ID: &'static str = "camera.setup";
}

const _: () = assert!(core::mem::size_of::<CameraSetup>() == 32);
