/// What a column read adds to the walk, for the areas the loader offers and the request asks for.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ColumnExtras {
    /// The game's map colours (`world.map_color` offered and `VANILLA_COLORS` asked).
    pub colors: bool,
    /// The biome id of the column (`world.biomes` offered).
    pub biomes: bool,
}
