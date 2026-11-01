// the look of a rail tile: Active is the accent tile (Resume), Danger the red one (Discard)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RailTone {
    Normal,
    Active,
    Danger,
}
