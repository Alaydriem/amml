// What a `Minecraft Bedrock` folder holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DataFigures {
    pub worlds: u32,
    pub resource_packs: u32,
    pub behavior_packs: u32,
    pub bytes: u64,
}
