/// One event a mod declares in a `uses` feature.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EventUse {
    pub name: String,
    pub version: u32,
    pub required: bool,
}
