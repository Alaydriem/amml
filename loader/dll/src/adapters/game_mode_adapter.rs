use std::ffi::c_void;
use std::panic::{AssertUnwindSafe, catch_unwind, resume_unwind};
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU64, Ordering};

use amml_abi::{AmmlBlockPos, AmmlFace, AmmlHand, AmmlHitResult, AmmlStatus, AmmlVec3};
use amml_data::{AdapterCatalog, Availability, EntryId, NeedKind};

use crate::{ClientAdapter, HitResultLayout, LoaderState, PickRangeShape, UseBuildDetours};

static ADAPTER: OnceLock<GameModeAdapter> = OnceLock::new();

/// `HitResult& Level::getHitResult()`: rcx the level, the reference in rax.
type HitResultFn = unsafe extern "C" fn(*mut c_void) -> *mut HitResultLayout;
/// `bool GameMode::buildBlock(BlockPos const&, uchar face, HandSlot, bool isSimTick)`: rcx the game mode, rdx the
/// position, r8b the face, r9b the hand slot, the isSimTick byte in the fifth argument slot; the bool in al.
type BuildFn = unsafe extern "C" fn(*mut c_void, *const AmmlBlockPos, u8, u8, u8) -> u8;
/// `float GameMode::getMaxPickRange()`: rcx the game mode, the range in xmm0.
type PickRangeFn = unsafe extern "C" fn(*mut c_void) -> f32;
/// `bool Player::isCreative() const`: rcx the player, the bool in al.
type IsCreativeFn = unsafe extern "C" fn(*mut c_void) -> u8;

/// The `player.build` and `player.pick_range` areas: the game mode of the local player of the primary client, its own
/// build, its build repeat delay and the pick range.
///
/// Every call needs the game gate open (`NOT_READY`), the area offered (`NOT_AVAILABLE`), the game thread and a player
/// handle that is the current local player of the primary client (`REFUSED`). The game mode is
/// `Player::mGameMode` (a `unique_ptr`); a null game mode, level or hit result is `NOT_AVAILABLE`.
///
/// `build_at` points the level's hit result (`Level::getHitResult`, the object the game's own build reads) at the
/// clicked block, calls the game mode's `buildBlock` through its vtable, as the game does, and puts the four members it
/// wrote back on every path. The call runs in an action scope of the calling mod (`UseBuildDetours::reentry`), so the
/// `BuildBlock` it raises has the origin `THIS_MOD` for that mod. `build_delay` reads `GameMode::buildingTickDelay`;
/// the loader logs `build delay read` once per world with the value. The pick range has two shapes, picked once by
/// which catalog shape resolved (`PickRangeShape`).
pub struct GameModeAdapter {
    state: &'static LoaderState,
    // `Player::mGameMode` and `Actor::mLevel` in the player, the slots of `Level::getHitResult` and
    // `GameMode::buildBlock`, and `GameMode::buildingTickDelay` in the game mode.
    game_mode_at: Option<usize>,
    level_at: Option<usize>,
    hit_result_slot: Option<usize>,
    build_slot: Option<usize>,
    delay_at: Option<usize>,
    pick: Option<PickRangeShape>,
    get_max_pick_range: Option<usize>,
    is_creative: Option<usize>,
    // The world epoch whose build delay was logged last.
    delay_logged: AtomicU64,
}

impl GameModeAdapter {
    pub const BUILD: &'static str = "player.build";
    pub const PICK_RANGE: &'static str = "player.pick_range";
    pub const HIT_RESULT: &'static str = "player.hit_result";
    // `HitResultType`: Tile 0, Entity 1, EntityOutOfRange 2, NoHit 3 (the game's consumers take 0 and 1 as a hit:
    // `add eax,-4; cmp eax,-3; ja`, 26.63 0x5391493, 70.26 0x69f1151).
    const HIT_ENTITY: i32 = 1;
    const AREA_VERSION: u32 = 1;
    /// `HandSlot::Mainhand`.
    const MAIN_HAND: AmmlHand = AmmlHand::MAIN;

    /// Offers the two areas when their catalog shapes resolved and logs one `area adapter` or `area not available` line
    /// each; with `player.build` offered, the build detour keeps the isSimTick of the local player's builds. Returns
    /// the rows. Works once per process: a second call offers nothing and gives the catalog rows.
    pub fn install(state: &'static LoaderState) -> Vec<Availability> {
        let resolved = |name: &str| state.table().get(&EntryId(name.to_string())).is_some();
        let rows: Vec<Availability> = AdapterCatalog::availability(&resolved)
            .into_iter()
            .filter(|row| {
                row.kind == NeedKind::Area && [Self::BUILD, Self::PICK_RANGE, Self::HIT_RESULT].contains(&row.id)
            })
            .collect();
        let pick = rows
            .iter()
            .find(|row| row.id == Self::PICK_RANGE)
            .and_then(|row| row.shape)
            .and_then(PickRangeShape::from_catalog);
        let value = |name: &str| state.resolve(name).ok().and_then(|value| usize::try_from(value).ok());
        let adapter = Self {
            state,
            game_mode_at: value("Player.mGameMode"),
            level_at: value("Actor.mLevel"),
            hit_result_slot: value("Level.getHitResult"),
            build_slot: value("GameMode.buildBlock"),
            delay_at: value("GameMode.buildingTickDelay"),
            pick,
            get_max_pick_range: value("GameMode.getMaxPickRange").filter(|address| *address != 0),
            is_creative: value("Player.isCreative").filter(|address| *address != 0),
            delay_logged: AtomicU64::new(0),
        };
        if ADAPTER.set(adapter).is_err() {
            curia::warn!("the game mode adapter is already installed");
            return rows;
        }
        for row in &rows {
            match row.shape {
                Some(shape) => {
                    state.areas().offer(row.id, row.version);
                    curia::info!("area adapter", { area: row.id, shape: shape });
                    if row.id == Self::BUILD {
                        UseBuildDetours::set(UseBuildDetours::SIM_TICK, true);
                    }
                }
                None => curia::info!("area not available", { area: row.id, missing: row.missing.join(", ") }),
            }
        }
        rows
    }

    /// The installed adapter; `None` before `install`.
    pub fn get() -> Option<&'static GameModeAdapter> {
        ADAPTER.get()
    }

    /// The pick range shape that resolved; `None` when `player.pick_range` is not offered.
    pub fn pick_shape(&self) -> Option<PickRangeShape> {
        self.pick
    }

    /// What the crosshair of `player`, the local player of the primary client, points at: the level's hit result read
    /// through the `HitResultLayout` mirror. A tile hit with a face above 5, an entity out of reach and no hit are
    /// `NOTHING`.
    pub fn hit_result(&self, player: *const c_void) -> Result<AmmlHitResult, AmmlStatus> {
        let player = self.enter(player, Self::HIT_RESULT)?;
        let level = Self::pointer_at(player, self.level_at)?;
        let get_hit = Self::virtual_fn(level, self.hit_result_slot).ok_or(AmmlStatus::NOT_AVAILABLE)?;
        // SAFETY: the slot holds `HitResult& Level::getHitResult()`, a member function that takes only `this` and
        // returns the reference as a pointer.
        let get_hit = unsafe { std::mem::transmute::<usize, HitResultFn>(get_hit) };
        // SAFETY: called on the live level of the local player, on the game thread.
        let hit = unsafe { get_hit(std::ptr::with_exposed_provenance_mut(level)) };
        if hit.is_null() {
            return Err(AmmlStatus::NOT_AVAILABLE);
        }
        // SAFETY: `hit` is the level's live `HitResult`, laid out as the mirror (88h bytes), read on its game thread.
        let hit = unsafe { hit.read_unaligned() };
        let [x, y, z] = hit.pos;
        let point = AmmlVec3 { x, y, z };
        Ok(match hit.kind {
            HitResultLayout::TILE if hit.facing <= AmmlFace::EAST.0 => {
                let [bx, by, bz] = hit.block;
                AmmlHitResult {
                    kind: AmmlHitResult::BLOCK,
                    face: AmmlFace(hit.facing),
                    _reserved: [0; 2],
                    pos: AmmlBlockPos { x: bx, y: by, z: bz },
                    point,
                }
            }
            Self::HIT_ENTITY => AmmlHitResult { kind: AmmlHitResult::ENTITY, point, ..AmmlHitResult::default() },
            _ => AmmlHitResult::default(),
        })
    }

    /// The mod at `owner` builds at the face `face` of the block `pos` of `player`, the local player of the primary
    /// client, with the click point `click` (world coordinates) and the main hand. True when the game built. A face
    /// above 5 or a click point that is not finite is `INVALID_ARGUMENT`, with no game call.
    pub fn build_at(
        &self,
        owner: usize,
        player: *const c_void,
        pos: AmmlBlockPos,
        face: AmmlFace,
        click: AmmlVec3,
    ) -> Result<bool, AmmlStatus> {
        if face.0 > AmmlFace::EAST.0 || !click.x.is_finite() || !click.y.is_finite() || !click.z.is_finite() {
            return Err(AmmlStatus::INVALID_ARGUMENT);
        }
        let player = self.enter(player, Self::BUILD)?;
        let game_mode = Self::pointer_at(player, self.game_mode_at)?;
        let level = Self::pointer_at(player, self.level_at)?;
        let get_hit = Self::virtual_fn(level, self.hit_result_slot).ok_or(AmmlStatus::NOT_AVAILABLE)?;
        let build = Self::virtual_fn(game_mode, self.build_slot).ok_or(AmmlStatus::NOT_AVAILABLE)?;
        // SAFETY: the slot holds `HitResult& Level::getHitResult()`, a member function that takes only `this` and
        // returns the reference as a pointer.
        let get_hit = unsafe { std::mem::transmute::<usize, HitResultFn>(get_hit) };
        // SAFETY: called on the live level of the local player, on the game thread.
        let hit = unsafe { get_hit(std::ptr::with_exposed_provenance_mut(level)) };
        if hit.is_null() {
            return Err(AmmlStatus::NOT_AVAILABLE);
        }
        // SAFETY: the slot holds `bool GameMode::buildBlock(BlockPos const&, uchar, HandSlot, bool)`.
        let build = unsafe { std::mem::transmute::<usize, BuildFn>(build) };
        // SAFETY: `hit` is the level's live `HitResult`, laid out as the mirror (88h bytes), read and written on the game
        // thread that owns it.
        let saved = unsafe { hit.read_unaligned() };
        // SAFETY: as above.
        unsafe {
            Self::write_hit(hit, HitResultLayout::TILE, face.0, [pos.x, pos.y, pos.z], [click.x, click.y, click.z])
        };
        let sim_tick = UseBuildDetours::last_sim_tick(self.state.events().gate().epoch().0);
        let scope = UseBuildDetours::reentry().enter(owner);
        let built = catch_unwind(AssertUnwindSafe(|| {
            // SAFETY: called on the live game mode of the local player, on the game thread; `pos` lives across the call.
            unsafe {
                build(
                    std::ptr::with_exposed_provenance_mut(game_mode),
                    &raw const pos,
                    face.0,
                    Self::MAIN_HAND.0,
                    sim_tick,
                )
            }
        }));
        drop(scope);
        // SAFETY: as above; the members written before the call get their saved values back.
        unsafe { Self::write_hit(hit, saved.kind, saved.facing, saved.block, saved.pos) };
        match built {
            Ok(built) => Ok(built != 0),
            Err(panic) => resume_unwind(panic),
        }
    }

    /// The build repeat delay of the game mode of `player`, in milliseconds. The loader logs the value once per world.
    pub fn build_delay(&self, player: *const c_void) -> Result<u32, AmmlStatus> {
        let player = self.enter(player, Self::BUILD)?;
        let game_mode = Self::pointer_at(player, self.game_mode_at)?;
        let at = self.delay_at.ok_or(AmmlStatus::NOT_AVAILABLE)?;
        let address = game_mode.checked_add(at).ok_or(AmmlStatus::FAILED)?;
        // SAFETY: `game_mode` is the live game mode of the local player and `at` the derived offset of its
        // `std::chrono::milliseconds buildingTickDelay`, a 64-bit count.
        let value = unsafe { std::ptr::with_exposed_provenance::<i64>(address).read_unaligned() };
        self.note_delay(value);
        u32::try_from(value).map_err(|_| AmmlStatus::FAILED)
    }

    /// The largest distance at which `player` can pick a block, in blocks, as its build's shape gives it.
    pub fn max_pick_range(&self, player: *const c_void) -> Result<f32, AmmlStatus> {
        let player = self.enter(player, Self::PICK_RANGE)?;
        match self.pick.ok_or(AmmlStatus::NOT_AVAILABLE)? {
            PickRangeShape::Getter => {
                let game_mode = Self::pointer_at(player, self.game_mode_at)?;
                let function = self.get_max_pick_range.ok_or(AmmlStatus::NOT_AVAILABLE)?;
                // SAFETY: the entry is `float GameMode::getMaxPickRange()`, a member function that takes only `this`.
                let function = unsafe { std::mem::transmute::<usize, PickRangeFn>(function) };
                // SAFETY: called on the live game mode of the local player, on the game thread.
                Ok(unsafe { function(std::ptr::with_exposed_provenance_mut(game_mode)) })
            }
            PickRangeShape::Creative => {
                let function = self.is_creative.ok_or(AmmlStatus::NOT_AVAILABLE)?;
                // SAFETY: the entry is `bool Player::isCreative() const`, a function that takes only the player.
                let function = unsafe { std::mem::transmute::<usize, IsCreativeFn>(function) };
                // SAFETY: called on the live local player, on the game thread.
                let creative = unsafe { function(std::ptr::with_exposed_provenance_mut(player)) } != 0;
                Ok(PickRangeShape::range_of(creative))
            }
        }
    }

    // Writes the four members a build reads into the hit result at `hit`.
    //
    // Safety: `hit` is a live, writable `HitResult`.
    unsafe fn write_hit(hit: *mut HitResultLayout, kind: i32, facing: u8, block: [i32; 3], pos: [f32; 3]) {
        // SAFETY: the caller guarantees a live, writable `HitResult`; each member is written in place.
        unsafe {
            (&raw mut (*hit).kind).write_unaligned(kind);
            (&raw mut (*hit).facing).write(facing);
            (&raw mut (*hit).block).write_unaligned(block);
            (&raw mut (*hit).pos).write_unaligned(pos);
        }
    }

    // Logs the build delay once per world.
    fn note_delay(&self, value: i64) {
        let epoch = self.state.events().gate().epoch().0;
        if self.delay_logged.swap(epoch, Ordering::AcqRel) != epoch {
            curia::info!("build delay read", { value: value });
        }
    }

    // The address of `player` when a call of `area` may use it.
    fn enter(&self, player: *const c_void, area: &str) -> Result<usize, AmmlStatus> {
        let gate = self.state.events().gate();
        gate.require_open()?;
        if !self.state.areas().is_available(area, Self::AREA_VERSION) {
            return Err(AmmlStatus::NOT_AVAILABLE);
        }
        if !gate.on_game_thread() {
            return Err(AmmlStatus::REFUSED);
        }
        let player = player.expose_provenance();
        match ClientAdapter::get().and_then(ClientAdapter::primary_local_player) {
            Some(current) if current == player => Ok(player),
            _ => Err(AmmlStatus::REFUSED),
        }
    }

    // The non-null pointer at `offset` in the live object `object`.
    fn pointer_at(object: usize, offset: Option<usize>) -> Result<usize, AmmlStatus> {
        let address = object.checked_add(offset.ok_or(AmmlStatus::NOT_AVAILABLE)?).ok_or(AmmlStatus::FAILED)?;
        // SAFETY: `object` is the live local player and `offset` the derived offset of a pointer member in it.
        let pointer = unsafe { std::ptr::with_exposed_provenance::<usize>(address).read_unaligned() };
        if pointer == 0 { Err(AmmlStatus::NOT_AVAILABLE) } else { Ok(pointer) }
    }

    // The function in `slot` of the vtable at the start of the live object `object`; `None` for an unknown slot, or a
    // null vtable or function.
    fn virtual_fn(object: usize, slot: Option<usize>) -> Option<usize> {
        let slot = slot?;
        // SAFETY: `object` is a live game object (the level or the game mode of the local player); its first qword is
        // its vtable pointer.
        let vtable = unsafe { std::ptr::with_exposed_provenance::<usize>(object).read_unaligned() };
        if vtable == 0 {
            return None;
        }
        let entry = vtable.checked_add(slot.checked_mul(size_of::<usize>())?)?;
        // SAFETY: the vtable of the object; the slot entry's recipe range check keeps `slot` inside it.
        let function = unsafe { std::ptr::with_exposed_provenance::<usize>(entry).read_unaligned() };
        (function != 0).then_some(function)
    }
}
