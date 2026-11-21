use amml::{MapColorShape, MceColor};
use amml_abi::AmmlRgba;

const COMMON: [&str; 5] = [
    "Block.mBlockType",
    "Dimension.getBlockSourceFromMainChunkSource",
    "BiomeColorSampling.getWaterColor",
    "BiomeColorSampling.getMapGrassColor",
    "LevelChunk.getBiome",
];
const COMPONENT: [&str; 4] = [
    "Block.tryGetComponent.BlockMapColorComponent",
    "BiomeColorSampling.getMapPolicy",
    "MapPolicy.get",
    "BlockType.getMapColor",
];

fn pick(extra: &[&str]) -> Option<MapColorShape> {
    MapColorShape::pick(&|name| COMMON.contains(&name) || extra.contains(&name))
}

#[test]
fn float_color_converts_to_rgba_bytes() {
    let color = MceColor { r: 0.5, g: 0.25, b: 1.0, a: 0.1 };

    assert_eq!(color.to_rgba(), AmmlRgba { r: 128, g: 64, b: 255, a: 255 });
}

#[test]
fn float_color_outside_the_range_is_clamped_and_nan_is_zero() {
    let color = MceColor { r: 1.7, g: -0.3, b: f32::NAN, a: 1.0 };

    assert_eq!(color.to_rgba(), AmmlRgba { r: 255, g: 0, b: 0, a: 255 });
}

#[test]
fn a_tint_scales_each_channel_and_keeps_the_alpha_of_the_colour() {
    let base = MceColor { r: 0.5, g: 0.5, b: 0.5, a: 1.0 };

    let tinted = base.tinted(MceColor { r: 0.5, g: 1.0, b: 0.0, a: 0.2 });

    assert_eq!(tinted, MceColor { r: 0.25, g: 0.5, b: 0.0, a: 1.0 });
}

#[test]
fn shape_pick_follows_the_resolved_entries() {
    // The 26.63 set has the component chain and no `Block.getMapColor`; the 70.26 set the reverse.
    assert_eq!(pick(&COMPONENT), Some(MapColorShape::Component));
    assert_eq!(pick(&["Block.getMapColor"]), Some(MapColorShape::Block));
    assert_eq!(pick(&[]), None);
}

#[test]
fn a_component_chain_with_an_entry_missing_is_not_a_shape() {
    let partial = &COMPONENT[..3];

    assert_eq!(pick(partial), None);
}
