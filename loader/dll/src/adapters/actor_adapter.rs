use std::ffi::c_void;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU64, Ordering};

use amml_abi::{AmmlAabb, AmmlGameType, AmmlPermissionLevel, AmmlRotation, AmmlStatus, AmmlVec3};
use amml_data::{AdapterCatalog, Availability, EntryId, NeedKind};

use amml_msvc::MemberCall;

use crate::{AabbShapeLayout, ClientAdapter, HandleStamp, LoaderState, PermissionShape, RotationComponentLayout};

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
/// `Vec3 Actor::getViewVector(float) const`: rcx the actor, the result buffer in rdx, the alpha in xmm2; rax gives the
/// buffer.
type ViewVectorFn = unsafe extern "C" fn(*mut c_void, *mut AmmlVec3, f32) -> *mut AmmlVec3;
/// `GameType Player::getPlayerGameType() const`: rcx the player, the type in eax.
type GameTypeFn = unsafe extern "C" fn(*mut c_void) -> i32;
/// `PlayerPermissionLevel Player::getPlayerPermissionLevel() const`: rcx the player, the level in al.
type PermissionFn = unsafe extern "C" fn(*mut c_void) -> u8;
/// `not_null<AbilitiesComponent*>* Player::getAbilitiesComponent(not_null<AbilitiesComponent*>*) const`: rcx the
/// player, rdx an 8-byte slot that receives the component pointer; rax gives the slot.
type AbilitiesFn = unsafe extern "C" fn(*mut c_void, *mut usize) -> *mut usize;
/// `void Actor::setYHeadRotations(float, float)`: rcx the actor, the head yaw in xmm1, the previous head yaw in xmm2.
type HeadRotationsFn = unsafe extern "C" fn(*mut c_void, f32, f32);

/// The `actor.read`, `actor.health`, `actor.game_type`, `actor.permission` and `actor.look_write` areas: reads and
/// the look write of the local player of the primary client through its Actor vtable and component pointers.
///
/// Every call needs the game gate open (`NOT_READY`), the area offered (`NOT_AVAILABLE`), the game thread and a
/// player handle that is the current local player of the primary client (`REFUSED`). The handle is checked again
/// on each call, so a handle kept from an earlier world or another client is refused, never read.
///
/// Positions come from the player's `StateVectorComponent.mPos` (the eyes: the local player's position is at eye
/// height) and its `AABBShapeComponent` box (the feet: the box's lowest y). The loader logs `actor position measured`
/// once per world with both heights. The rotation is `ActorRotationComponent.mRot` (pitch, then yaw), in degrees, with
/// the signs of `AmmlRotation`. The write stores the pitch and yaw to `mRot` and to the previous rotation 8 bytes after
/// it, then calls `setYHeadRotations(head, head)` for a head yaw.
///
/// The permission level has two shapes, picked once by which catalog shape resolved: the getter (26.63), or the byte
/// of the abilities component (70.x). The loader logs `actor permission read` once per world with the shape and value.
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
    // `Actor::getViewVector`, `Player::getPlayerGameType` and `Actor::setYHeadRotations`.
    get_view_vector: Option<usize>,
    get_game_type: Option<usize>,
    set_head_rotations: Option<usize>,
    // The permission shape that resolved, with its getter and the byte offset in the abilities component.
    permission: Option<PermissionShape>,
    get_permission: Option<usize>,
    get_abilities: Option<usize>,
    permission_at: Option<usize>,
    // The world epoch whose position was measured last, and whose permission was logged last.
    measured: AtomicU64,
    permission_logged: AtomicU64,
}

impl ActorAdapter {
    pub const READ: &'static str = "actor.read";
    pub const HEALTH: &'static str = "actor.health";
    pub const GAME_TYPE: &'static str = "actor.game_type";
    pub const PERMISSION: &'static str = "actor.permission";
    pub const LOOK_WRITE: &'static str = "actor.look_write";
    const AREAS: [&'static str; 5] = [Self::READ, Self::HEALTH, Self::GAME_TYPE, Self::PERMISSION, Self::LOOK_WRITE];
    const AREA_VERSION: u32 = 1;
    // The reads appended in version 2 of the area.
    const READS_VERSION: u32 = 2;
    // The head position and the block source, appended in version 3 of the area.
    const EXTRA_VERSION: u32 = 3;
    // The view vector, appended in version 4 of the area.
    const VIEW_VERSION: u32 = 4;
    // The alpha the view vector is asked for: the current frame, not an interpolation.
    const VIEW_ALPHA: f32 = 1.0;
    // `SharedTypes::Legacy::ActorLocation::Head`.
    const HEAD_LOCATION: i32 = 3;
    // `AABB::min.y`, `AABB::max.y`: the box is two Vec3 (min, then max) at the start of AABBShapeComponent.
    const AABB_MIN_Y: usize = 4;
    const AABB_MAX_Y: usize = 16;

    /// Offers the actor areas when their catalog shapes resolved and logs one `area adapter` or `area not available`
    /// line each. Returns the rows. Works once per process: a second call offers nothing and
    /// gives the catalog rows.
    pub fn install(state: &'static LoaderState) -> Vec<Availability> {
        let resolved = |name: &str| state.table().get(&EntryId(name.to_string())).is_some();
        let rows: Vec<Availability> = AdapterCatalog::availability(&resolved)
            .into_iter()
            .filter(|row| row.kind == NeedKind::Area && Self::AREAS.contains(&row.id))
            .collect();
        let permission = rows
            .iter()
            .find(|row| row.id == Self::PERMISSION)
            .and_then(|row| row.shape)
            .and_then(PermissionShape::from_catalog);
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
            get_view_vector: slot("Actor.getViewVector").filter(|address| *address != 0),
            get_game_type: slot("Player.getPlayerGameType").filter(|address| *address != 0),
            set_head_rotations: slot("Actor.setYHeadRotations").filter(|address| *address != 0),
            permission,
            get_permission: slot("Player.getPlayerPermissionLevel").filter(|address| *address != 0),
            get_abilities: slot("Player.getAbilitiesComponent").filter(|address| *address != 0),
            permission_at: slot("AbilitiesComponent.mPlayerPermissions"),
            measured: AtomicU64::new(0),
            permission_logged: AtomicU64::new(0),
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

    /// The view direction of `player` as a unit vector: `getViewVector(1.0)`.
    pub fn view_vector(&self, player: *const c_void) -> Result<AmmlVec3, AmmlStatus> {
        let player = self.enter_at(player, Self::VIEW_VERSION)?;
        let function = self.get_view_vector.ok_or(AmmlStatus::NOT_AVAILABLE)?;
        // SAFETY: the entry is `Vec3 Actor::getViewVector(float) const`, a member function that returns a class through
        // a hidden pointer.
        let function = unsafe { std::mem::transmute::<usize, ViewVectorFn>(function) };
        let mut out = AmmlVec3 { x: 0.0, y: 0.0, z: 0.0 };
        // SAFETY: called on the live local player, on the game thread; `out` is a Vec3 buffer the callee fills.
        unsafe { function(std::ptr::with_exposed_provenance_mut(player), &raw mut out, Self::VIEW_ALPHA) };
        Ok(out)
    }

    /// The collision box of `player`, from its `AABBShapeComponent`.
    pub fn aabb(&self, player: *const c_void) -> Result<AmmlAabb, AmmlStatus> {
        let player = self.enter_at(player, Self::VIEW_VERSION)?;
        let shape = Self::component(player, self.aabb_at)?;
        // SAFETY: a live `AABBShapeComponent`, laid out as the mirror.
        let [min_x, min_y, min_z, max_x, max_y, max_z] =
            unsafe { std::ptr::with_exposed_provenance::<AabbShapeLayout>(shape).read_unaligned() }.aabb;
        Ok(AmmlAabb { min: AmmlVec3 { x: min_x, y: min_y, z: min_z }, max: AmmlVec3 { x: max_x, y: max_y, z: max_z } })
    }

    /// The game mode of `player`: the raw value of `getPlayerGameType`.
    pub fn game_type(&self, player: *const c_void) -> Result<AmmlGameType, AmmlStatus> {
        let player = self.enter_in(player, Self::GAME_TYPE, 1)?;
        let function = self.get_game_type.ok_or(AmmlStatus::NOT_AVAILABLE)?;
        // SAFETY: the entry is `GameType Player::getPlayerGameType() const`, a member function that takes only `this`.
        let function = unsafe { std::mem::transmute::<usize, GameTypeFn>(function) };
        // SAFETY: called on the live local player, on the game thread.
        Ok(AmmlGameType(unsafe { function(std::ptr::with_exposed_provenance_mut(player)) }))
    }

    /// The player permission level of `player`, read as its build's shape says. The loader logs the shape and value
    /// once per world.
    pub fn permission_level(&self, player: *const c_void) -> Result<AmmlPermissionLevel, AmmlStatus> {
        let player = self.enter_in(player, Self::PERMISSION, 1)?;
        let shape = self.permission.ok_or(AmmlStatus::NOT_AVAILABLE)?;
        let value = match shape {
            PermissionShape::Getter => {
                let function = self.get_permission.ok_or(AmmlStatus::NOT_AVAILABLE)?;
                // SAFETY: the entry is `PlayerPermissionLevel Player::getPlayerPermissionLevel() const`, a member
                // function that takes only `this` and returns the level in `al`.
                let function = unsafe { std::mem::transmute::<usize, PermissionFn>(function) };
                // SAFETY: called on the live local player of a loaded world, on the game thread.
                unsafe { function(std::ptr::with_exposed_provenance_mut(player)) }
            }
            PermissionShape::Abilities => {
                let function = self.get_abilities.ok_or(AmmlStatus::NOT_AVAILABLE)?;
                let at = self.permission_at.ok_or(AmmlStatus::NOT_AVAILABLE)?;
                // SAFETY: the entry is `not_null<AbilitiesComponent*>* Player::getAbilitiesComponent(
                // not_null<AbilitiesComponent*>*) const`: `this`, then an 8-byte slot for the component pointer.
                let function = unsafe { std::mem::transmute::<usize, AbilitiesFn>(function) };
                let mut component = 0usize;
                // SAFETY: called on the live local player of a loaded world, on the game thread; `component` is the
                // slot the callee fills.
                unsafe { function(std::ptr::with_exposed_provenance_mut(player), &raw mut component) };
                if component == 0 {
                    return Err(AmmlStatus::NOT_AVAILABLE);
                }
                let address = component.checked_add(at).ok_or(AmmlStatus::FAILED)?;
                // SAFETY: `component` is the live abilities component the game gave, and `at` the derived offset of the
                // player permission byte in it.
                unsafe { std::ptr::with_exposed_provenance::<u8>(address).read() }
            }
        };
        self.note_permission(shape, value);
        Ok(AmmlPermissionLevel(value))
    }

    /// Turns `player` to `rotation` (pitch and yaw in degrees) and, for `Some(head_yaw)`, its head too. The
    /// rotation goes to `mRot` and to the previous rotation, so the game does not interpolate from the old angle. The
    /// arguments must be finite (`INVALID_ARGUMENT`).
    pub fn set_rotation(
        &self,
        player: *const c_void,
        rotation: AmmlRotation,
        head_yaw: Option<f32>,
    ) -> Result<(), AmmlStatus> {
        if !rotation.pitch.is_finite() || !rotation.yaw.is_finite() || head_yaw.is_some_and(|head| !head.is_finite()) {
            return Err(AmmlStatus::INVALID_ARGUMENT);
        }
        let player = self.enter_in(player, Self::LOOK_WRITE, 1)?;
        let component = Self::component(player, self.rotation_at)?;
        let at = self.rot_at.ok_or(AmmlStatus::NOT_AVAILABLE)?;
        let head = match head_yaw {
            Some(head) => {
                let function = self.set_head_rotations.ok_or(AmmlStatus::NOT_AVAILABLE)?;
                // SAFETY: the entry is `void Actor::setYHeadRotations(float, float)`, a member function.
                Some((head, unsafe { std::mem::transmute::<usize, HeadRotationsFn>(function) }))
            }
            None => None,
        };
        let layout =
            RotationComponentLayout { rot: [rotation.pitch, rotation.yaw], previous: [rotation.pitch, rotation.yaw] };
        // SAFETY: `component` is the live rotation component of the local player and `at` the derived offset of
        // `mRot`, followed by the previous rotation (16 bytes); written on the game thread.
        unsafe {
            std::ptr::with_exposed_provenance_mut::<RotationComponentLayout>(component + at).write_unaligned(layout)
        };
        if let Some((head, function)) = head {
            // SAFETY: called on the live local player, on the game thread.
            unsafe { function(std::ptr::with_exposed_provenance_mut(player), head, head) };
        }
        Ok(())
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

    // Logs the permission shape and value once per world.
    fn note_permission(&self, shape: PermissionShape, value: u8) {
        let epoch = self.state.events().gate().epoch().0;
        if self.permission_logged.swap(epoch, Ordering::AcqRel) != epoch {
            curia::info!("actor permission read", { shape: shape.name(), value: value });
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
