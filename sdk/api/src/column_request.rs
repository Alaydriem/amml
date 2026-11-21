use amml_abi::AmmlColumnRequest;

/// Which columns of a chunk `LevelChunk::read_columns` reads, and from where it starts.
///
/// A column `(x, z)` of the chunk (each 0 to 15) is bit `z * 16 + x` of `columns`: word `bit / 64`, bit `bit % 64`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ColumnRequest {
    /// The block y the walk of each column starts from (the camera height).
    pub camera_y: i32,
    /// Water that is shown gives its depth in `ColumnSample::water_depth` and the first block under it that is not
    /// water or air as `shown`.
    pub transparent_water: bool,
    /// Ask for the game's own map colours.
    pub vanilla_colors: bool,
    pub columns: [u64; 4],
}

impl ColumnRequest {
    /// Every column of the chunk, with no transparent water and no vanilla colours.
    pub fn all(camera_y: i32) -> Self {
        Self { camera_y, transparent_water: false, vanilla_colors: false, columns: [u64::MAX; 4] }
    }

    /// The column `(x, z)` only; no column at all when `x` or `z` is 16 or more.
    pub fn one(x: u8, z: u8, camera_y: i32) -> Self {
        let mut columns = [0u64; 4];
        if x < 16 && z < 16 {
            let bit = usize::from(z) * 16 + usize::from(x);
            columns[bit / 64] = 1 << (bit % 64);
        }
        Self { camera_y, transparent_water: false, vanilla_colors: false, columns }
    }

    /// Whether the column `(x, z)` is asked for.
    pub fn selects(&self, x: u8, z: u8) -> bool {
        x < 16 && z < 16 && self.selects_index(usize::from(z) * 16 + usize::from(x))
    }

    /// Whether the column of `index` (`z * 16 + x`) is asked for.
    pub fn selects_index(&self, index: usize) -> bool {
        index < 256 && (self.columns[index / 64] >> (index % 64)) & 1 != 0
    }
}

impl From<&ColumnRequest> for AmmlColumnRequest {
    fn from(request: &ColumnRequest) -> Self {
        let mut flags = 0;
        if request.transparent_water {
            flags |= AmmlColumnRequest::TRANSPARENT_WATER;
        }
        if request.vanilla_colors {
            flags |= AmmlColumnRequest::VANILLA_COLORS;
        }
        Self { camera_y: request.camera_y, flags, _reserved: [0; 3], columns: request.columns }
    }
}
