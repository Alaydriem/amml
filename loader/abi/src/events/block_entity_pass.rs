use crate::{AmmlBlockEntityCamera, AmmlEvent, AmmlEventHeader};

/// The game drew the opaque block entities of one level renderer camera (`LevelRendererCamera::renderBlockEntities`,
/// the shared body).
///
/// `frame_id` names this dispatch: `AmmlRenderTable::block_entity_draw` takes it and gives `REFUSED` outside the
/// dispatch of that pass on the thread that raised it. `camera` is `PLAYER` when the camera is the local player's level
/// renderer, else `OTHER`. Raised After the game's call, for the opaque pass only, only while a world is joined, on the
/// thread that runs the pass (the game thread on both gate builds). `_reserved` stays reserved; a later version appends
/// its fields after it, so its size is 32 or more.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct BlockEntityPass {
    pub header: AmmlEventHeader,
    pub frame_id: u64,
    pub camera: AmmlBlockEntityCamera,
    pub _reserved: [u8; 7],
}

impl AmmlEvent for BlockEntityPass {
    const ID: &'static str = "render.block_entity_pass";
}

const _: () = assert!(core::mem::size_of::<BlockEntityPass>() == 32);
