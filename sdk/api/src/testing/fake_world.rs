use std::collections::HashMap;

use crate::testing::{FakeBlock, FakeChunk, FakeItem};
use crate::{ActorBox, BlockHandle, BlockHit, BlockPos, BlockTypeId, SurfaceSample};

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
    /// The block states `Blocks::from_state` finds exactly, by the block type name and the states asked for, sorted by
    /// state name, each value as text (`2`, `true`, `north`). Needs `Area::WORLD_BLOCK_STATES`.
    pub states: HashMap<(String, Vec<(String, String)>), BlockHandle>,
    /// The default state of each block type: what `Blocks::from_state` gives, with `exact` false, for states that are
    /// not in `states`. A name in neither map gives `None`.
    pub default_states: HashMap<String, BlockHandle>,
    /// What `Blocks::rotated` gives for a handle and a quarter count (0 to 3); a pair not here gives the same handle.
    /// `FakeGame::rotations` lists the calls.
    pub rotated: HashMap<(BlockHandle, u8), BlockHandle>,
    /// The names `Blocks::name` gives.
    pub block_names: HashMap<BlockHandle, String>,
    /// The items `Blocks::item` gives.
    pub block_items: HashMap<BlockHandle, FakeItem>,
    /// The blocks `BlockSource::block` gives, by position; a position not here reads as air (`air_block`), as the game gives the registry's air for any position. Needs
    /// `Area::WORLD_BLOCKS`.
    pub blocks: HashMap<BlockPos, FakeBlock>,
}

impl FakeWorld {
    /// The block a position not in `blocks` reads as: `minecraft:air` (its type id is the one `names` gives that name, else
    /// `BlockTypeId::NONE`), not interactive, with no item and no handle. Its `can_be_built_over` is true, as the game
    /// places blocks into air. On 26.63 and 70.26 no block type overrides `BlockType::canBeBuiltOver` (slot 84 is one
    /// body in all 270 / 272 BlockType vtables, survey section 8 row W1); that body answers from the type's component and
    /// tag data, which the registry sets at run time, so `true` is the fake's choice, not an exe reading. Map the position
    /// to say otherwise.
    pub fn air_block(&self) -> FakeBlock {
        let type_id =
            self.names.iter().find(|(_, name)| name.as_str() == "minecraft:air").map(|(id, _)| *id).unwrap_or_default();
        FakeBlock { type_id, name: "minecraft:air", is_air: true, built_over: true, ..FakeBlock::default() }
    }
}
