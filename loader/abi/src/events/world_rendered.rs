use crate::{AmmlEvent, AmmlEventHeader};

/// The game drew the world of the primary client, after its level pass (`LevelRenderer::renderLevel`).
///
/// `frame_id` names this dispatch: the canvas calls of `AmmlRenderTable` take it and give `REFUSED` outside the
/// dispatch of that frame on the thread that raised it. `vibrant` is 1 while the game renders in the Vibrant Visuals
/// (deferred) graphics mode. Raised only while a world is joined, on the thread that runs `renderLevel`: the game
/// thread (`GAME`, with the primary client in the context) when the game renders there, else `RENDER`.
/// `_reserved` stays reserved; a later version appends its fields after it, so its size is 40 or more.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct WorldRendered {
    pub header: AmmlEventHeader,
    pub frame_id: u64,
    pub vibrant: u8,
    pub _reserved: [u8; 7],
}

impl AmmlEvent for WorldRendered {
    const ID: &'static str = "render.world_rendered";
}

const _: () = assert!(core::mem::size_of::<WorldRendered>() == 32);
