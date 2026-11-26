/// The steps the block mesher applies to the vertices the game's block tessellator wrote for one block, before the mesh
/// is ended: each quad pushed 0.01 into its own cell along its normal (so a real face over a ghost face always wins), the
/// alpha in every vertex colour, a back face for the one-quad shapes, the positions moved to the grid origin and the box
/// fitted to them. Quads are four vertices in a row, as the tessellator writes a quad list.
pub struct MeshPost;

impl MeshPost {
    /// How far each quad moves into its own cell.
    pub const PUSH: f32 = 0.01;
    /// The vertices of one quad.
    pub const QUAD: usize = 4;
    /// The render layers the chunk builder walks in the extra layer bits: 0 to 15h on both gate builds (`cmp esi,15h;
    /// jbe` with `bt edx,esi`).
    pub const LAYER_COUNT: u32 = 22;
    /// The `BlockShape` values drawn from both sides: the shapes the exe names `cross_texture` (1), `rows` (6), `vine`
    /// (20), `seagrass` (97) and `chain` (123), and the three shapes whose case draws with the cross pair writer of the
    /// chain (121, 122, 129); the same values on both gate builds.
    pub const TWO_SIDED_SHAPES: [i32; 8] = [1, 6, 20, 97, 121, 122, 123, 129];
    // A normal shorter than this has no direction.
    const MIN_NORMAL: f32 = 1e-8;
    // A face whose centre lies closer than this to the cell centre along its normal has no inside.
    const MIN_DEPTH: f32 = 1e-4;
    // A quad faces up when its outward normal is within 60 degrees of +y.
    const UP_COS: f32 = 0.5;

    /// Moves each whole quad of `positions` `PUSH` along its normal towards `centre`, the centre of the block's cell. A
    /// quad with no normal, one whose plane passes through the centre, and vertices that are not whole quads keep their
    /// place.
    pub fn push_in(positions: &mut [[f32; 3]], centre: [f32; 3]) {
        if !positions.len().is_multiple_of(Self::QUAD) {
            return;
        }
        for quad in positions.as_chunks_mut::<{ Self::QUAD }>().0 {
            let normal = Self::cross(quad);
            let length = Self::length(normal);
            if length < Self::MIN_NORMAL {
                continue;
            }
            let face: [f32; 3] =
                std::array::from_fn(|axis| quad.iter().map(|vertex| vertex[axis]).sum::<f32>() / 4.0 - centre[axis]);
            let depth: f32 = (0..3).map(|axis| normal[axis] * face[axis]).sum::<f32>() / length;
            if depth.abs() < Self::MIN_DEPTH {
                continue;
            }
            let step = -depth.signum() * Self::PUSH / length;
            for vertex in quad.iter_mut() {
                for axis in 0..3 {
                    vertex[axis] += normal[axis] * step;
                }
            }
        }
    }

    /// Writes `alpha` into the high byte of every packed colour (ABGR, alpha highest).
    pub fn set_alpha(colors: &mut [u32], alpha: u8) {
        let value = u32::from(alpha) << 24;
        for color in colors {
            *color = (*color & 0x00FF_FFFF) | value;
        }
    }

    /// Whether a block of `shape` (`BlockGraphics::mBlockShape`) gets a back face.
    pub fn is_two_sided(shape: i32) -> bool {
        Self::TWO_SIDED_SHAPES.contains(&shape)
    }

    /// The back face copy of the per-vertex values `values`: each quad again with its vertices in reverse order, each
    /// value through `map`. `None` when the values are not whole quads.
    pub fn reversed<T: Copy>(values: &[T], map: impl Fn(T) -> T) -> Option<Vec<T>> {
        if !values.len().is_multiple_of(Self::QUAD) {
            return None;
        }
        Some(
            values
                .as_chunks::<{ Self::QUAD }>()
                .0
                .iter()
                .flat_map(|quad| quad.iter().rev().map(|value| map(*value)))
                .collect(),
        )
    }

    /// A normal or tangent of the back face: x, y and z negated, w kept.
    pub fn turn_normal(vector: [f32; 4]) -> [f32; 4] {
        [-vector[0], -vector[1], -vector[2], vector[3]]
    }

    /// Moves `positions` from world coordinates to coordinates relative to the block position `origin`.
    pub fn shift(positions: &mut [[f32; 3]], origin: [i32; 3]) {
        let origin = origin.map(|axis| axis as f32);
        for position in positions {
            for axis in 0..3 {
                position[axis] -= origin[axis];
            }
        }
    }

    /// The box of `positions`, lowest corner first; zero for none.
    pub fn fit_aabb(positions: &[[f32; 3]]) -> [[f32; 3]; 2] {
        let Some(first) = positions.first() else {
            return [[0.0; 3]; 2];
        };
        let (mut low, mut high) = (*first, *first);
        for vertex in positions {
            for axis in 0..3 {
                low[axis] = low[axis].min(vertex[axis]);
                high[axis] = high[axis].max(vertex[axis]);
            }
        }
        [low, high]
    }

    /// The render layers a block is tessellated in, as the chunk builder queues it: its main layer `main`
    /// (`Block::getRenderLayer`), then each bit of `extra` (`BlockType::getExtraRenderLayers`) below `LAYER_COUNT` that
    /// is not the main layer, rising.
    pub fn passes(main: u8, extra: u32) -> impl Iterator<Item = i32> {
        let real = (1u32 << Self::LAYER_COUNT) - 1;
        let own = 1u32.checked_shl(u32::from(main)).unwrap_or(0);
        let bits = extra & real & !own;
        std::iter::once(i32::from(main)).chain((0..Self::LAYER_COUNT as i32).filter(move |bit| bits & (1 << bit) != 0))
    }

    /// The cross product of the first two edges of `quad`.
    pub fn cross(quad: &[[f32; 3]]) -> [f32; 3] {
        let a: [f32; 3] = std::array::from_fn(|axis| quad[1][axis] - quad[0][axis]);
        let b: [f32; 3] = std::array::from_fn(|axis| quad[2][axis] - quad[0][axis]);
        [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
    }

    /// Multiplies the red, green and blue of the packed colours (ABGR, red lowest) of one block by `colour`, each channel
    /// clamped to 0 to 1, and keeps the alpha. The game's tessellator leaves a block whose tint method asks for a biome
    /// colour grey on this path, so a block with a colour already (a vertex whose channels differ) was tinted by the game
    /// and is left as it is. With `top_only` only the quads whose outward normal points up (within 60 degrees of +y, seen
    /// from `centre`, the block's cell centre) take the colour. Gives whether a colour changed: false for colours that do
    /// not cover `positions`, a colour of no red, green and blue (it would turn the block black), a block that has a
    /// colour, and `top_only` with vertices that are not whole quads.
    pub fn tint(
        positions: &[[f32; 3]],
        colors: &mut [u32],
        centre: [f32; 3],
        colour: [f32; 3],
        top_only: bool,
    ) -> bool {
        if positions.len() != colors.len() || colors.is_empty() {
            return false;
        }
        let multiply = colour.map(|channel| channel.clamp(0.0, 1.0));
        if multiply == [0.0; 3] || !colors.iter().all(|color| Self::is_grey(*color)) {
            return false;
        }
        if top_only && !positions.len().is_multiple_of(Self::QUAD) {
            return false;
        }
        for (quad, quad_colors) in colors.chunks_mut(Self::QUAD).enumerate() {
            if top_only {
                let corners = &positions[quad * Self::QUAD..(quad + 1) * Self::QUAD];
                if !Self::faces_up(corners, centre) {
                    continue;
                }
            }
            for color in quad_colors {
                let bytes = color.to_le_bytes();
                let mut out = bytes;
                for channel in 0..3 {
                    out[channel] = (f32::from(bytes[channel]) * multiply[channel]).round().min(255.0) as u8;
                }
                *color = u32::from_le_bytes(out);
            }
        }
        true
    }

    // Whether the red, green and blue of a packed colour are equal.
    fn is_grey(color: u32) -> bool {
        let [red, green, blue, _] = color.to_le_bytes();
        red == green && red == blue
    }

    // Whether the outward normal of `quad` (away from `centre`) is within 60 degrees of +y. A quad with no normal, or
    // one whose plane passes through the centre, does not face up.
    fn faces_up(quad: &[[f32; 3]], centre: [f32; 3]) -> bool {
        let normal = Self::cross(quad);
        let length = Self::length(normal);
        if length < Self::MIN_NORMAL {
            return false;
        }
        let face: [f32; 3] =
            std::array::from_fn(|axis| quad.iter().map(|vertex| vertex[axis]).sum::<f32>() / 4.0 - centre[axis]);
        let depth: f32 = (0..3).map(|axis| normal[axis] * face[axis]).sum();
        if depth.abs() < Self::MIN_DEPTH * length {
            return false;
        }
        normal[1] * depth.signum() >= Self::UP_COS * length
    }

    fn length(vector: [f32; 3]) -> f32 {
        (vector[0] * vector[0] + vector[1] * vector[1] + vector[2] * vector[2]).sqrt()
    }
}
