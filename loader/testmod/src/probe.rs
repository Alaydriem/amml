use std::ffi::c_void;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::ptr::NonNull;
use std::sync::OnceLock;

use amml_game::{
    Actor, BlockPos, ChunkBlockPos, ClientInstance, EntryId, EntryNeed, GameEntries, MaterialLayout, RenderCamera,
    StopLatch,
};
use amml_sdk::{ModRuntime, PanicText};

use crate::{ChunkReading, LightReading, NameTagEntries, NameTagLayout, ProbeReport, Throttle, WorldLatch};

/// What the test mod reads from the game: the player pose, the dimension and the camera. It only reads.
pub struct Probe {
    entries: OnceLock<GameEntries>,
    latch: StopLatch,
    world: WorldLatch,
    ticks: Throttle,
    forward_frames: Throttle,
    view_frames: Throttle,
    tag_frames: Throttle,
}

impl Probe {
    /// Calls and frames between two log lines.
    const EVERY: u32 = 100;

    /// The height of the player's eyes above the feet; the pose position is the eye position.
    const EYE_HEIGHT: f32 = 1.62;

    /// The block under the feet for a pose position (eye position): feet y is `pos.y - EYE_HEIGHT`, and the small
    /// bias keeps a player standing exactly on a block edge from reading the block below.
    pub fn block_under_feet(pos: [f32; 3]) -> [i32; 3] {
        let feet_y = pos[1] - Self::EYE_HEIGHT + 1e-3;
        [pos[0].floor() as i32, feet_y.floor() as i32 - 1, pos[2].floor() as i32]
    }

    /// A probe with no entries; it reads nothing until `start`.
    pub const fn new() -> Self {
        Self {
            entries: OnceLock::new(),
            latch: StopLatch::new(),
            world: WorldLatch::new(),
            ticks: Throttle::new(Self::EVERY),
            forward_frames: Throttle::new(Self::EVERY),
            view_frames: Throttle::new(Self::EVERY),
            tag_frames: Throttle::new(Self::EVERY),
        }
    }

    /// Every entry the probe reads. `ClientInstance.update` is required; each other group is one feature that
    /// turns off alone.
    pub fn wanted() -> Vec<(EntryId, EntryNeed)> {
        let pose = EntryNeed::Optional { feature: "pose" };
        let dimension = EntryNeed::Optional { feature: "dimension" };
        let block = EntryNeed::Optional { feature: "block" };
        let camera = EntryNeed::Optional { feature: "camera" };
        let chunk = EntryNeed::Optional { feature: "chunk" };
        let light = EntryNeed::Optional { feature: "light" };
        let mut wanted = vec![
            (EntryId::CLIENT_INSTANCE_UPDATE, EntryNeed::Required),
            (EntryId::CLIENT_INSTANCE_GET_LOCAL_PLAYER, pose),
            (EntryId::ACTOR_STATE_VECTOR_COMPONENT, pose),
            (EntryId::ACTOR_ACTOR_ROTATION_COMPONENT, pose),
            (EntryId::STATE_VECTOR_COMPONENT_POS, pose),
            (EntryId::ACTOR_ROTATION_COMPONENT_ROT, pose),
            (EntryId::ACTOR_GET_DIMENSION, dimension),
            (EntryId::DIMENSION_GET_DIMENSION_ID, dimension),
            (EntryId::ACTOR_GET_DIMENSION_BLOCK_SOURCE, block),
            (EntryId::CLIENT_INSTANCE_GET_REGION, chunk),
            (EntryId::BLOCK_SOURCE_GET_CHUNK, chunk),
            (EntryId::LEVEL_CHUNK_SUB_CHUNKS, chunk),
            (EntryId::LEVEL_CHUNK_GET_BLOCK, chunk),
            (EntryId::SUB_CHUNK_SIZE, chunk),
            (EntryId::LEVEL_RENDERER_PLAYER_SETUP_CAMERA, camera),
            (EntryId::LEVEL_RENDERER_PLAYER_CREATE_VIEW_RENDER_OBJECT, camera),
            (EntryId::CAMERA_FORWARD, camera),
            (EntryId::VIEW_RENDER_OBJECT_VIEW_DATA, camera),
            (EntryId::VIEW_RENDER_DATA_CAMERA_POS, camera),
            (EntryId::BLOCK_SOURCE_GET_BRIGHTNESS_PAIR, light),
            (EntryId::BLOCK_CAN_PROVIDE_FULL_SUPPORT, light),
            (EntryId::BLOCK_TYPE_GET_COLLISION_SHAPE, light),
        ];
        wanted.extend(NameTagEntries::wanted());
        // the material layout entry, which selects the BlockType mirror
        wanted.extend(MaterialLayout::wanted());
        wanted
    }

    /// Keeps the resolved entries and lowers the latch. A second call keeps the first entries, which belong to
    /// the same game build.
    pub fn start(&self, entries: GameEntries) {
        let _ = self.entries.set(entries);
        self.latch.lower();
    }

    /// Raises the latch: no later tick or frame reads the game.
    pub fn stop(&self) {
        self.latch.raise();
    }

    /// The latch that stops the probe.
    pub fn latch(&self) -> &StopLatch {
        &self.latch
    }

    fn log(message: &str) {
        if let Some(runtime) = ModRuntime::get() {
            runtime.logger().info(message);
        }
    }

    // Runs `read`; a panic is logged once and raises the latch, so the probe stops and the game goes on.
    fn guarded(&self, read: impl FnOnce()) {
        if self.latch.raised() {
            return;
        }
        if let Err(payload) = catch_unwind(AssertUnwindSafe(read)) {
            self.latch.raise();
            if let Some(runtime) = ModRuntime::get() {
                runtime
                    .logger()
                    .error(&format!("probe panicked: {}; the probe stops reading", PanicText::of(payload.as_ref())));
            }
        }
    }

    /// One game tick. Logs "probe: not in world" when the player left the world, and a world line on the first
    /// tick in a world and then every 100th tick.
    #[allow(unsafe_code)]
    pub fn on_tick(&self, client: NonNull<c_void>) {
        self.guarded(|| {
            let Some(entries) = self.entries.get() else {
                return;
            };
            // SAFETY: `client` is the `this` of the running `ClientInstance::update`, live for this call.
            let Some(client) = (unsafe { ClientInstance::from_raw(client.as_ptr(), entries) }) else {
                return;
            };
            let player = client.local_player();
            let changed = self.world.set(player.is_some());
            let Some(player) = player else {
                if changed {
                    Self::log("probe: not in world");
                }
                return;
            };
            if !self.ticks.hit() {
                return;
            }
            let dimension = player.dimension().and_then(|dimension| dimension.id());
            let block_source = Some(player.block_source().is_some());
            let pose = player.pose();
            Self::log(&ProbeReport::world_line(pose, dimension, block_source));
            if let Some(pose) = pose {
                Self::log_column(&client, pose.pos);
                Self::log_light(&client, &player, pose.pos);
            }
        });
    }

    // Logs the block under the feet and the chunk of the column at `pos`. The block is at floor(feet y) - 1; the chunk
    // is (bx >> 4, bz >> 4); the block y of a ChunkBlockPos is relative to the chunk's min y; the heightmap index
    // is (z << 4) | x, and the heights in the heightmap are relative to the chunk's min y too.
    fn log_column(client: &ClientInstance<'_>, pos: [f32; 3]) {
        let block_pos = Self::block_under_feet(pos);
        let [bx, by, bz] = block_pos;
        let chunk = client.region().and_then(|region| region.chunk(bx >> 4, bz >> 4));
        let min_y = chunk.as_ref().and_then(|chunk| chunk.min_y());
        let local_x = (bx & 15) as u8;
        let local_z = (bz & 15) as u8;
        // mMax.y is inclusive (319 in the Overworld, 127 in the Nether), so the range is min_y <= by <= max_y.
        let in_range =
            chunk.as_ref().zip(min_y).is_some_and(|(chunk, min_y)| ChunkReading::contains_y(min_y, chunk.max_y(), by));
        if in_range {
            let block = chunk.as_ref().zip(min_y).and_then(|(chunk, min_y)| {
                let y = i16::try_from(by - min_y).ok()?;
                chunk.block(ChunkBlockPos { x: local_x, z: local_z, y })
            });
            let name = block.as_ref().and_then(|block| block.type_name()).and_then(|name| name.to_str().ok());
            Self::log(&ProbeReport::block_line(block_pos, name, block.as_ref().and_then(|block| block.is_solid())));
        } else {
            Self::log(&ProbeReport::block_out_of_range_line(block_pos));
        }
        if let Some(chunk) = chunk {
            let index = usize::from(local_z) * 16 + usize::from(local_x);
            Self::log(&ProbeReport::chunk_line(&ChunkReading {
                pos: chunk.position(),
                min_y,
                max_y: chunk.max_y(),
                loaded: chunk.loaded(),
                sub_chunks: chunk.sub_chunk_count(),
                index,
                height: chunk.height(index),
            }));
        } else {
            Self::log("probe: chunk n/a (no region or chunk not loaded)");
        }
    }

    // The block at `pos`, through the chunk of its column. `None` when the chunk is not loaded or `pos` is outside
    // the chunk's y range.
    fn block_at<'a>(client: &ClientInstance<'a>, pos: [i32; 3]) -> Option<amml_game::Block<'a>> {
        let [bx, by, bz] = pos;
        let chunk = client.region()?.chunk(bx >> 4, bz >> 4)?;
        let min_y = chunk.min_y()?;
        if !ChunkReading::contains_y(min_y, chunk.max_y(), by) {
            return None;
        }
        chunk.block(ChunkBlockPos { x: (bx & 15) as u8, z: (bz & 15) as u8, y: i16::try_from(by - min_y).ok()? })
    }

    // Logs the light of the cell the player stands in, whether the block under the feet gives full support on its
    // top, and the collision box of the block in the feet cell.
    fn log_light(client: &ClientInstance<'_>, player: &Actor<'_>, pos: [f32; 3]) {
        let under = Self::block_under_feet(pos);
        let feet = [under[0], under[1] + 1, under[2]];
        let feet_pos = BlockPos { x: feet[0], y: feet[1], z: feet[2] };
        let region = player.block_source();
        let under_block = Self::block_at(client, under);
        let feet_block = Self::block_at(client, feet);
        let reading = LightReading {
            feet,
            brightness: region.as_ref().and_then(|region| region.brightness(&feet_pos)),
            under_air: under_block.as_ref().and_then(|block| block.is_air()),
            under_full_support: under_block.as_ref().and_then(|block| block.can_provide_full_support(1)),
            feet_air: feet_block.as_ref().and_then(|block| block.is_air()),
            feet_shape: feet_block
                .as_ref()
                .zip(region.as_ref())
                .and_then(|(block, region)| block.collision_shape(region, &feet_pos)),
            sky_darken: player.dimension().and_then(|dimension| dimension.sky_darken()),
        };
        Self::log(&ProbeReport::light_line(&reading));
    }

    /// After `LevelRendererPlayer::extractNameTags`: logs the number of tags the game made and the result of the
    /// list layout check every 100th frame. `ret` is the hidden return buffer the game filled.
    #[allow(unsafe_code)]
    pub fn on_name_tags(&self, ret: usize) {
        self.guarded(|| {
            let Some(entries) = self.entries.get() else {
                return;
            };
            if !self.tag_frames.hit() {
                return;
            }
            let Some(size) = entries.get(NameTagEntries::SIZE) else {
                return;
            };
            // SAFETY: `ret` is the NameTagRenderObjectCollection buffer that `extractNameTags` just filled and
            // returned, which `NameTagCollectionLayout` mirrors.
            let layout = unsafe { NameTagLayout::read(ret as *const c_void, size) };
            Self::log(&ProbeReport::name_tags_line(layout.as_ref()));
        });
    }

    /// After `LevelRendererPlayer::setupCamera`: logs the camera forward vector every 100th frame.
    #[allow(unsafe_code)]
    pub fn on_setup_camera(&self, camera: usize) {
        self.guarded(|| {
            let Some(entries) = self.entries.get() else {
                return;
            };
            if !self.forward_frames.hit() {
                return;
            }
            // SAFETY: `camera` is the Camera argument of the `setupCamera` call that just returned, live for it.
            let forward = unsafe { RenderCamera::new(entries).forward(camera as *const c_void) };
            Self::log(&ProbeReport::camera_line("forward", forward));
        });
    }

    /// After `LevelRendererPlayer::createViewRenderObject`: logs the view position every 100th frame. `ret` is the
    /// hidden return buffer the game filled.
    #[allow(unsafe_code)]
    pub fn on_view_render_object(&self, ret: usize) {
        self.guarded(|| {
            let Some(entries) = self.entries.get() else {
                return;
            };
            if !self.view_frames.hit() {
                return;
            }
            // SAFETY: `ret` is the ViewRenderObject buffer that `createViewRenderObject` just filled and returned.
            let position = unsafe { RenderCamera::new(entries).view_position(ret as *const c_void) };
            Self::log(&ProbeReport::camera_line("view_position", position));
        });
    }
}

impl Default for Probe {
    fn default() -> Self {
        Self::new()
    }
}
