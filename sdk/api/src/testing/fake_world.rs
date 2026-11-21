use std::collections::HashMap;

use crate::testing::FakeChunk;
use crate::{ActorBox, BlockHit, BlockTypeId, SurfaceSample};

/// The world of a `FakeClient`: what `LocalPlayer::block_source` gives a block source for, the chunks that
/// `BlockSource::chunk` finds there (by chunk position), the names `Blocks::type_name` gives and the surfaces
/// `BlockSource::scan_surfaces` finds. A client with no `FakeWorld` has a dimension and no block source, as a loader
/// whose game gives no block source for the dimension.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FakeWorld {
    pub chunks: HashMap<(i32, i32), FakeChunk>,
    pub names: HashMap<BlockTypeId, String>,
    /// The names `Biomes::name` gives, by biome id.
    pub biome_names: HashMap<u16, String>,
    /// The surfaces of each column, by block x and z. `BlockSource::scan_surfaces` gives those of the column within the
    /// block ys asked, lowest first, as the loader does; the chunk of the column must be in `chunks` and loaded, or the
    /// call gives `None`.
    pub surfaces: HashMap<(i32, i32), Vec<SurfaceSample>>,
    /// The hit `BlockSource::raycast` gives for every ray with finite points; `None` for a miss.
    pub ray_hit: Option<BlockHit>,
    /// The actors `BlockSource::actors_in` looks through: it gives those whose boxes meet the box asked, in this order,
    /// at most as many as the room given.
    pub actors: Vec<ActorBox>,
}
