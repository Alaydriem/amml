use std::ffi::c_void;
use std::sync::Mutex;

use amml::SurfaceShape;
use amml_data::{EntryKind, Resolved};

use super::{ChunkGame, ChunkKind, Harness};

/// A `canProvideFullSupport` call the stand-in game saw: the entry, the block, the face and the flag (`None` for the
/// two-argument form).
pub type SupportCall = (&'static str, usize, u8, Option<u8>);
/// A `getCollisionShape` call: the block type, the block, the region, the position and the optional reference.
pub type CollisionCall = (usize, usize, usize, [i32; 3], usize);
/// A `getBrightnessPair` call: the block source and the position.
pub type LightCall = (usize, [i32; 3]);

static SUPPORT_CALLS: Mutex<Vec<SupportCall>> = Mutex::new(Vec::new());
static COLLISION_CALLS: Mutex<Vec<CollisionCall>> = Mutex::new(Vec::new());
static LIGHT_CALLS: Mutex<Vec<LightCall>> = Mutex::new(Vec::new());

/// Stand-ins for the calls of `world.scan_surfaces`, over the stand-in chunk of `ChunkGame`: stone and dirt give full
/// support on their top face, a slab and a flower do not; a flower has an empty collision box and every other block
/// a full box; the light at a block y is the sky light `y & 15` and the block light `15 - (y & 15)`.
pub struct SurfaceGame;

impl SurfaceGame {
    /// The slot of `getCollisionShape` in the stand-in block type vtable.
    pub const COLLISION_SLOT: usize = 4;

    /// The surface entries of `shape` for the 26.63 two-argument support call or the 70.x flag call; `without` names
    /// entries to leave out. Calls `setup`, so it comes after `ChunkGame::entries`.
    pub fn entries(shape: SurfaceShape, without: &[&str]) -> Vec<(&'static str, Resolved)> {
        Self::setup();
        let function = |address: usize| (EntryKind::Function, address as u64 - Harness::BASE);
        let support = match shape {
            SurfaceShape::Support => ("Block.canProvideFullSupport", function(Self::support as *const () as usize)),
            SurfaceShape::SupportFlag => {
                ("Block.canProvideFullSupportWithFlag", function(Self::support_flag as *const () as usize))
            }
        };
        [
            support,
            ("BlockType.getCollisionShape", (EntryKind::VSlot, Self::COLLISION_SLOT as u64)),
            ("BlockSource.getBrightnessPair", function(Self::brightness as *const () as usize)),
        ]
        .into_iter()
        .filter(|(name, _)| !without.contains(name))
        .map(|(name, (kind, value))| (name, Resolved { kind, value, recipes_agreeing: 2 }))
        .collect()
    }

    /// Puts the collision stand-in in the block type vtable and clears the calls seen.
    pub fn setup() {
        ChunkGame::set_type_slot(Self::COLLISION_SLOT, Self::collision_shape as *const () as usize);
        SUPPORT_CALLS.lock().unwrap().clear();
        COLLISION_CALLS.lock().unwrap().clear();
        LIGHT_CALLS.lock().unwrap().clear();
    }

    /// The support calls seen, in order.
    pub fn support_calls() -> Vec<SupportCall> {
        SUPPORT_CALLS.lock().unwrap().clone()
    }

    /// The collision shape calls seen, in order.
    pub fn collision_calls() -> Vec<CollisionCall> {
        COLLISION_CALLS.lock().unwrap().clone()
    }

    /// The light calls seen, in order.
    pub fn light_calls() -> Vec<LightCall> {
        LIGHT_CALLS.lock().unwrap().clone()
    }

    // Whether the stand-in block gives full support on its top face.
    fn gives_support(block: usize) -> bool {
        block == ChunkGame::block(ChunkKind::Stone) || block == ChunkGame::block(ChunkKind::Dirt)
    }

    // `bool Block::canProvideFullSupport(uchar) const`.
    extern "C" fn support(block: *mut c_void, face: u8) -> u8 {
        SUPPORT_CALLS.lock().unwrap().push(("Block.canProvideFullSupport", block.addr(), face, None));
        u8::from(Self::gives_support(block.addr()))
    }

    // `bool Block::canProvideFullSupport(uchar, bool) const`.
    extern "C" fn support_flag(block: *mut c_void, face: u8, flag: u8) -> u8 {
        SUPPORT_CALLS.lock().unwrap().push(("Block.canProvideFullSupportWithFlag", block.addr(), face, Some(flag)));
        u8::from(Self::gives_support(block.addr()))
    }

    // `AABB BlockType::getCollisionShape(Block const&, IConstBlockSource const&, BlockPos const&,
    // optional_ref<GetCollisionShapeInterface const>) const`.
    extern "C" fn collision_shape(
        block_type: *mut c_void,
        out: *mut [f32; 6],
        block: *mut c_void,
        region: *mut c_void,
        pos: *const [i32; 3],
        optional: *const c_void,
    ) -> *mut [f32; 6] {
        let shape =
            if block.addr() == ChunkGame::block(ChunkKind::Flower) { [0.0; 6] } else { [0.0, 0.0, 0.0, 1.0, 1.0, 1.0] };
        // SAFETY: the adapter passes a live `BlockPos` of three ints and a writable box of six floats.
        unsafe {
            out.write(shape);
            COLLISION_CALLS.lock().unwrap().push((
                block_type.addr(),
                block.addr(),
                region.addr(),
                pos.read(),
                optional.addr(),
            ));
        }
        out
    }

    // `BrightnessPair BlockSource::getBrightnessPair(BlockPos const&) const`.
    extern "C" fn brightness(source: *mut c_void, out: *mut [u8; 2], pos: *const [i32; 3]) -> *mut [u8; 2] {
        // SAFETY: the adapter passes a live `BlockPos` of three ints and a writable pair of bytes.
        unsafe {
            let pos = pos.read();
            let level = (pos[1] & 15) as u8;
            out.write([level, 15 - level]);
            LIGHT_CALLS.lock().unwrap().push((source.addr(), pos));
        }
        out
    }
}
