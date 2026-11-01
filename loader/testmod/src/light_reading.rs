use amml_game::{Aabb, Brightness};

/// What the probe read about the cell the player stands in and the block under it.
pub struct LightReading {
    /// The block position of the player's feet (the cell above the ground).
    pub feet: [i32; 3],
    /// The light at the feet cell.
    pub brightness: Option<Brightness>,
    /// Whether the block under the feet is air.
    pub under_air: Option<bool>,
    /// Whether the block under the feet gives full support on its top face.
    pub under_full_support: Option<bool>,
    /// Whether the block at the feet is air.
    pub feet_air: Option<bool>,
    /// The collision box of the block at the feet.
    pub feet_shape: Option<Aabb>,
    /// The sky darkness of the dimension.
    pub sky_darken: Option<u8>,
}
