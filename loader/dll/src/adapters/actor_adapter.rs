use std::ffi::c_void;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU64, Ordering};

use amml_abi::{AmmlRotation, AmmlStatus, AmmlVec3};
use amml_data::{AdapterCatalog, Availability, EntryId, NeedKind};

use amml_msvc::MemberCall;

use crate::{ClientAdapter, HandleStamp, LoaderState};

static ADAPTER: OnceLock<ActorAdapter> = OnceLock::new();

/// `bool Actor::isAlive() const`: rcx the actor, the bool in al.
type BoolFn = unsafe extern "C" fn(*mut c_void) -> u8;
/// `Dimension& Actor::getDimension() const`: rcx the actor, the dimension in rax.
type PointerFn = unsafe extern "C" fn(*mut c_void) -> *mut c_void;
/// `Vec3 Actor::getAttachPos(ActorLocation, float) const`: rcx the actor, the result buffer in rdx, the location in r8d,
/// the alpha in xmm3; rax gives the buffer.
type AttachPosFn = unsafe extern "C" fn(*mut c_void, *mut AmmlVec3, i32, f32) -> *mut AmmlVec3;
/// `int ActorAttribute::getHealth(EntityContext const&)`: rcx the context, the health in eax.
type HealthFn = unsafe extern "C" fn(*const c_void) -> i32;

/// The `actor.read` and `actor.health` areas: reads of the local player of the primary client through its Actor vtable.
///
/// Every call needs the game gate open (`NOT_READY`), the area offered (`NOT_AVAILABLE`), the game thread and a
/// player handle that is the current local player of the primary client (`REFUSED`). The handle is checked again
/// on each call, so a handle kept from an earlier world or another client is refused, never read.
///
/// Positions come from the player's `StateVectorComponent.mPos` (the eyes: the local player's position is at eye
/// height) and its `AABBShapeComponent` box (the feet: the box's lowest y). The loader logs `actor position measured`
/// once per world with both heights. The rotation is `ActorRotationComponent.mRot` (pitch, then yaw), in degrees, with
/// the signs of `AmmlRotation`.
pub struct ActorAdapter {
    state: &'static LoaderState,
    // Slot of the Actor vtable.
    is_alive: Option<usize>,
    // Offsets of the component pointers in Actor, and of the members in each component.
    state_vector_at: Option<usize>,
    pos_at: Option<usize>,
    aabb_at: Option<usize>,
    rotation_at: Option<usize>,
    rot_at: Option<usize>,
    // `Actor::getDimension`, and the slots of `getDimensionId` and `getBlockSourceFromMainChunkSource` in the
    // Dimension vtable.
    get_dimension: Option<usize>,
    dimension_id_slot: Option<usize>,
    block_source_slot: Option<usize>,
    // `Actor::getAttachPos`, `ActorAttribute::getHealth` and the offset of the EntityContext in Actor.
    get_attach_pos: Option<usize>,
    get_health: Option<usize>,
    entity_context_at: Option<usize>,
    // The world epoch whose position was measured last.
    measured: AtomicU64,
}

impl ActorAdapter {
    pub const READ: &'static str = "actor.read";
    pub const HEALTH: &'static str = "actor.health";
    const AREA_VERSION: u32 = 1;
    // The reads appended in version 2 of the area.
    const READS_VERSION: u32 = 2;
    // The head position and the block source, appended in version 3 of the area.
    const EXTRA_VERSION: u32 = 3;
    // `SharedTypes::Legacy::ActorLocation::Head`.
    const HEAD_LOCATION: i32 = 3;
    // `AABB::min.y`, `AABB::max.y`: the box is two Vec3 (min, then max) at the start of AABBShapeComponent.
    const AABB_MIN_Y: usize = 4;
    const AABB_MAX_Y: usize = 16;

    /// Offers `actor.read` and `actor.health` when their catalog shapes resolved and logs one `area adapter` or
    /// `area not available` line each. Returns the rows. Works once per process: a second call offers nothing and
    /// gives the catalog rows.
    pub fn install(state: &'static LoaderState) -> Vec<Availability> {
        let resolved = |name: &str| state.table().get(&EntryId(name.to_string())).is_some();
        let rows: Vec<Availability> = AdapterCatalog::availability(&resolved)
            .into_iter()
            .filter(|row| row.kind == NeedKind::Area && (row.id == Self::READ || row.id == Self::HEALTH))
            .collect();
        let slot = |name: &str| state.resolve(name).ok().and_then(|value| usize::try_from(value).ok());
        let adapter = Self {
            state,
            is_alive: slot("Actor.isAlive"),
            state_vector_at: slot("Actor.stateVectorComponent"),
            pos_at: slot("StateVectorComponent.mPos"),
            aabb_at: slot("Actor.aabbShapeComponent"),
            rotation_at: slot("Actor.actorRotationComponent"),
            rot_at: slot("ActorRotationComponent.mRot"),
            get_dimension: slot("Actor.getDimension").filter(|address| *address != 0),
            dimension_id_slot: slot("Dimension.getDimensionId"),
            block_source_slot: slot("Dimension.getBlockSourceFromMainChunkSource"),
            get_attach_pos: slot("Actor.getAttachPos").filter(|address| *address != 0),
            get_health: slot("ActorAttribute.getHealth").filter(|address| *address != 0),
            entity_context_at: slot("Actor.mEntityContext"),
            measured: AtomicU64::new(0),
        };
        if ADAPTER.set(adapter).is_err() {
            curia::warn!("the actor adapter is already installed");
            return rows;
        }
        for row in &rows {
            match row.shape {
                Some(shape) => {
                    state.areas().offer(row.id, row.version);
                    curia::info!("area adapter", { area: row.id, shape: shape });
                }
                None => curia::info!("area not available", { area: row.id, missing: row.missing.join(", ") }),
            }
        }
        rows
    }

    /// The installed adapter; `None` before `install`.
    pub fn get() -> Option<&'static ActorAdapter> {
        ADAPTER.get()
    }

    /// Whether `player`, the local player of the primary client, is alive.
    pub fn is_alive(&self, player: *const c_void) -> Result<bool, AmmlStatus> {
        let player = self.enter(player)?;
        let function = Self::virtual_fn(player, self.is_alive).ok_or(AmmlStatus::NOT_AVAILABLE)?;
        // SAFETY: the slot holds `bool Actor::isAlive() const`, a member function that takes only `this`.
        let function = unsafe { std::mem::transmute::<usize, BoolFn>(function) };
        // SAFETY: called on its own object, the live local player, on the game thread.
        Ok(unsafe { function(std::ptr::with_exposed_provenance_mut(player)) } != 0)
    }

    /// The feet of `player`: x and z of `mPos`, y the lowest y of its collision box.
    pub fn position(&self, player: *const c_void) -> Result<AmmlVec3, AmmlStatus> {
        let player = self.enter_at(player, Self::READS_VERSION)?;
        let eye = self.read_eye(player)?;
        let aabb = Self::component(player, self.aabb_at)?;
        let min_y = Self::read_f32(aabb, Self::AABB_MIN_Y);
        self.note_measure(eye.y, min_y, Self::read_f32(aabb, Self::AABB_MAX_Y));
        Ok(AmmlVec3 { x: eye.x, y: min_y, z: eye.z })
    }

    /// The eyes of `player`: its `mPos`.
    pub fn eye_position(&self, player: *const c_void) -> Result<AmmlVec3, AmmlStatus> {
        let player = self.enter_at(player, Self::READS_VERSION)?;
        let eye = self.read_eye(player)?;
        if let Ok(aabb) = Self::component(player, self.aabb_at) {
            self.note_measure(eye.y, Self::read_f32(aabb, Self::AABB_MIN_Y), Self::read_f32(aabb, Self::AABB_MAX_Y));
        }
        Ok(eye)
    }

    /// The view direction of `player`: `mRot` holds the pitch, then the yaw, in degrees.
    pub fn rotation(&self, player: *const c_void) -> Result<AmmlRotation, AmmlStatus> {
        let player = self.enter_at(player, Self::READS_VERSION)?;
        let component = Self::component(player, self.rotation_at)?;
        let at = self.rot_at.ok_or(AmmlStatus::NOT_AVAILABLE)?;
        Ok(AmmlRotation { pitch: Self::read_f32(component, at), yaw: Self::read_f32(component, at + 4) })
    }

    /// The dimension of `player`, as a handle for `dimension_id`; null from the game is `NOT_AVAILABLE`.
    pub fn dimension(&self, player: *const c_void) -> Result<usize, AmmlStatus> {
        let player = self.enter_at(player, Self::READS_VERSION)?;
        self.dimension_of(player)
    }

    /// The id of `dimension`, which must be the dimension of the local player of the primary client now
    /// (`REFUSED` otherwise).
    pub fn dimension_id(&self, dimension: *const c_void) -> Result<i32, AmmlStatus> {
        let gate = self.state.events().gate();
        gate.require_open()?;
        if !self.state.areas().is_available(Self::READ, Self::READS_VERSION) {
            return Err(AmmlStatus::NOT_AVAILABLE);
        }
        if !gate.on_game_thread() {
            return Err(AmmlStatus::REFUSED);
        }
        let player = ClientAdapter::get().and_then(ClientAdapter::primary_local_player).ok_or(AmmlStatus::REFUSED)?;
        let dimension = dimension.expose_provenance();
        if dimension == 0 || self.dimension_of(player)? != dimension {
            return Err(AmmlStatus::REFUSED);
        }
        let function = Self::virtual_fn(dimension, self.dimension_id_slot).ok_or(AmmlStatus::NOT_AVAILABLE)?;
        // SAFETY: the slot holds `DimensionType Dimension::getDimensionId() const`; `DimensionType` is a struct of one
        // `int`, returned through a hidden pointer as every struct a member function returns.
        Ok(unsafe {
            MemberCall::call_sret0::<i32>(function as *const c_void, std::ptr::with_exposed_provenance_mut(dimension))
        })
    }

    /// The head of `player`: `getAttachPos(ActorLocation::Head, 0)`.
    pub fn head_position(&self, player: *const c_void) -> Result<AmmlVec3, AmmlStatus> {
        let player = self.enter_at(player, Self::EXTRA_VERSION)?;
        let function = self.get_attach_pos.ok_or(AmmlStatus::NOT_AVAILABLE)?;
        // SAFETY: the entry is `Vec3 Actor::getAttachPos(ActorLocation, float) const`, a member function that returns
        // a class through a hidden pointer.
        let function = unsafe { std::mem::transmute::<usize, AttachPosFn>(function) };
        let mut out = AmmlVec3 { x: 0.0, y: 0.0, z: 0.0 };
        // SAFETY: called on the live local player, on the game thread; `out` is a Vec3 buffer the callee fills.
        unsafe { function(std::ptr::with_exposed_provenance_mut(player), &raw mut out, Self::HEAD_LOCATION, 0.0) };
        Ok(out)
    }

    /// The block source of the dimension of `player`, as a handle for the world calls: its address with the epoch of
    /// the world in its top bits (`HandleStamp`), so a handle from an earlier world is refused. Null from the game is
    /// `NOT_AVAILABLE`.
    pub fn block_source(&self, player: *const c_void) -> Result<usize, AmmlStatus> {
        let player = self.enter_at(player, Self::EXTRA_VERSION)?;
        let source = self.block_source_of(player)?;
        let epoch = self.state.events().gate().epoch().0;
        HandleStamp::seal(source, epoch).ok_or(AmmlStatus::FAILED)
    }

    /// The health of `player`: `ActorAttribute::getHealth` of the entity context of the actor.
    pub fn health(&self, player: *const c_void) -> Result<i32, AmmlStatus> {
        let player = self.enter_in(player, Self::HEALTH, 1)?;
        let function = self.get_health.ok_or(AmmlStatus::NOT_AVAILABLE)?;
        let context_at = self.entity_context_at.ok_or(AmmlStatus::NOT_AVAILABLE)?;
        let context = player.checked_add(context_at).ok_or(AmmlStatus::FAILED)?;
        // SAFETY: the entry is `int ActorAttribute::getHealth(EntityContext const&)`.
        let function = unsafe { std::mem::transmute::<usize, HealthFn>(function) };
        // SAFETY: the actor is the live local player and holds its EntityContext by value at the entry's offset; called
        // on the game thread.
        Ok(unsafe { function(std::ptr::with_exposed_provenance(context)) })
    }

    /// The dimension of the local player of the primary client now. `REFUSED` when there is none. The world calls
    /// compare their handle with it.
    pub fn primary_dimension(&self) -> Result<usize, AmmlStatus> {
        let player = ClientAdapter::get().and_then(ClientAdapter::primary_local_player).ok_or(AmmlStatus::REFUSED)?;
        self.dimension_of(player)
    }

    /// The block source of the dimension of the local player of the primary client now. `REFUSED` when there is no
    /// local player.
    pub fn primary_block_source(&self) -> Result<usize, AmmlStatus> {
        let player = ClientAdapter::get().and_then(ClientAdapter::primary_local_player).ok_or(AmmlStatus::REFUSED)?;
        self.block_source_of(player)
    }

    // The block source of the dimension of the live local player `player`.
    fn block_source_of(&self, player: usize) -> Result<usize, AmmlStatus> {
        let dimension = self.dimension_of(player)?;
        let function = Self::virtual_fn(dimension, self.block_source_slot).ok_or(AmmlStatus::NOT_AVAILABLE)?;
        // SAFETY: the slot holds `BlockSource& Dimension::getBlockSourceFromMainChunkSource() const`, a member function
        // that takes only `this`.
        let function = unsafe { std::mem::transmute::<usize, PointerFn>(function) };
        // SAFETY: called on the live dimension of the local player, on the game thread.
        let source = unsafe { function(std::ptr::with_exposed_provenance_mut(dimension)) }.expose_provenance();
        if source == 0 { Err(AmmlStatus::NOT_AVAILABLE) } else { Ok(source) }
    }

    // The dimension of the live local player `player`.
    fn dimension_of(&self, player: usize) -> Result<usize, AmmlStatus> {
        let function = self.get_dimension.ok_or(AmmlStatus::NOT_AVAILABLE)?;
        // SAFETY: the entry is `Dimension& Actor::getDimension() const`, a member function that takes only `this`.
        let function = unsafe { std::mem::transmute::<usize, PointerFn>(function) };
        // SAFETY: called on the live local player, on the game thread.
        let dimension = unsafe { function(std::ptr::with_exposed_provenance_mut(player)) }.expose_provenance();
        if dimension == 0 { Err(AmmlStatus::NOT_AVAILABLE) } else { Ok(dimension) }
    }

    // `mPos` of the live local player `player`.
    fn read_eye(&self, player: usize) -> Result<AmmlVec3, AmmlStatus> {
        let component = Self::component(player, self.state_vector_at)?;
        let at = self.pos_at.ok_or(AmmlStatus::NOT_AVAILABLE)?;
        Ok(AmmlVec3 {
            x: Self::read_f32(component, at),
            y: Self::read_f32(component, at + 4),
            z: Self::read_f32(component, at + 8),
        })
    }

    // Logs both heights once per world, for the position measurement.
    fn note_measure(&self, eye_y: f32, min_y: f32, max_y: f32) {
        let epoch = self.state.events().gate().epoch().0;
        if self.measured.swap(epoch, Ordering::AcqRel) != epoch {
            curia::info!("actor position measured", {
                mpos_y: eye_y,
                aabb_min_y: min_y,
                aabb_max_y: max_y,
                eye_height: eye_y - min_y
            });
        }
    }

    // The non-null component pointer at `offset` in the live local player `player`.
    fn component(player: usize, offset: Option<usize>) -> Result<usize, AmmlStatus> {
        let address = player.checked_add(offset.ok_or(AmmlStatus::NOT_AVAILABLE)?).ok_or(AmmlStatus::FAILED)?;
        // SAFETY: `player` is the live local player and `offset` the derived offset of a component pointer in its
        // built-in component block, so the qword is readable and 8-aligned.
        let component = unsafe { std::ptr::with_exposed_provenance::<usize>(address).read() };
        if component == 0 { Err(AmmlStatus::NOT_AVAILABLE) } else { Ok(component) }
    }

    // The f32 at `offset` in the live component `component`.
    fn read_f32(component: usize, offset: usize) -> f32 {
        // SAFETY: `component` is a live component of the local player and `offset` lies inside its derived member
        // (a Vec2, a Vec3 or the AABB), 4-aligned.
        unsafe { std::ptr::with_exposed_provenance::<f32>(component + offset).read() }
    }

    // `enter` for a call of `actor.read` version `version`.
    fn enter_at(&self, player: *const c_void, version: u32) -> Result<usize, AmmlStatus> {
        self.enter_in(player, Self::READ, version)
    }

    // The address of `player` when a call of `actor.read` version 1 may use it.
    fn enter(&self, player: *const c_void) -> Result<usize, AmmlStatus> {
        self.enter_in(player, Self::READ, Self::AREA_VERSION)
    }

    // The address of `player` when a call of `area` at `version` may use it.
    fn enter_in(&self, player: *const c_void, area: &str, version: u32) -> Result<usize, AmmlStatus> {
        let gate = self.state.events().gate();
        gate.require_open()?;
        if !self.state.areas().is_available(area, version) {
            return Err(AmmlStatus::NOT_AVAILABLE);
        }
        if !gate.on_game_thread() {
            return Err(AmmlStatus::REFUSED);
        }
        let player = player.expose_provenance();
        let current = ClientAdapter::get().and_then(ClientAdapter::primary_local_player);
        match current {
            Some(current) if current == player => Ok(player),
            _ => Err(AmmlStatus::REFUSED),
        }
    }

    // The function in `slot` of the vtable at the start of `object`; `None` for an unknown slot, or a null vtable
    // or function.
    fn virtual_fn(object: usize, slot: Option<usize>) -> Option<usize> {
        let slot = slot?;
        // SAFETY: `object` is the live, non-null local player (checked by `enter`); its first qword is its vtable
        // pointer.
        let vtable = unsafe { std::ptr::with_exposed_provenance::<*const usize>(object).read() };
        if vtable.is_null() {
            return None;
        }
        // SAFETY: `vtable` is the non-null vtable of the player; the slot entry's recipe range check keeps `slot`
        // inside it.
        let function = unsafe { vtable.add(slot).read() };
        (function != 0).then_some(function)
    }
}
