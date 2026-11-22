use amml_abi::AmmlMesherOptions;

/// How `BlockMesher::build` builds a grid. `alpha` is written into every vertex colour (255 opaque). With
/// `skip_matching_world` a cell whose world block is already the same block state (the same game `Block`; stairs of another
/// facing are drawn) is not drawn, and its index is listed
/// in `MeshBuilt::skipped`. `water_tint` gives water the biome colour (`Area::RENDER_WATER_TINT`; ignored without it).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MesherOptions {
    pub alpha: u8,
    pub skip_matching_world: bool,
    pub water_tint: bool,
}

impl From<&MesherOptions> for AmmlMesherOptions {
    fn from(options: &MesherOptions) -> Self {
        Self {
            alpha: options.alpha,
            skip_matching_world: u8::from(options.skip_matching_world),
            water_tint: u8::from(options.water_tint),
            _reserved: 0,
        }
    }
}
