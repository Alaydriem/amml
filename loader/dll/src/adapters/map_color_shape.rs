use amml_data::AdapterCatalog;

/// How a build reaches the map colour of a block.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MapColorShape {
    /// 26.63: the map colour component of the block, tinted by its biome policy; the block type's own colour when the
    /// block has no component.
    Component,
    /// 70.x: `Block::getMapColor`.
    Block,
}

impl MapColorShape {
    /// The shape the resolved entries give: `Component` when the component chain resolved, `Block` when
    /// `Block.getMapColor` did, `None` when neither. `resolved` says whether an entry name resolved.
    pub fn pick(resolved: &impl Fn(&str) -> bool) -> Option<Self> {
        AdapterCatalog::availability(resolved)
            .into_iter()
            .find(|row| row.id == "world.map_color")
            .and_then(|row| row.shape)
            .and_then(Self::named)
    }

    // The shape a catalog shape name stands for.
    fn named(name: &str) -> Option<Self> {
        match name {
            AdapterCatalog::MAP_COLOR_COMPONENT_SHAPE => Some(Self::Component),
            AdapterCatalog::MAP_COLOR_BLOCK_SHAPE => Some(Self::Block),
            _ => None,
        }
    }
}
