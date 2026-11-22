use std::ffi::c_void;
use std::marker::PhantomData;

use amml_abi::{
    AmmlAabb, AmmlActorTable, AmmlGameType, AmmlHitResult, AmmlInventoryTable, AmmlPermissionLevel, AmmlRotation,
    AmmlVec3, AmmlWorldTable,
};

use crate::{
    Aabb, ApiError, BlockSource, Dimension, GameMode, GameType, Hand, HitResult, Inventory, ItemView, PermissionLevel,
    Rotation, Vec3,
};

/// The local player of a `Client`, as an opaque handle. Two values are equal when they name the same
/// player. It lives no longer than the event that gave its client.
///
/// The reads go through the loader's actor table (area `actor.read`), on the game thread. The loader checks on
/// every call that the handle is still the local player of the primary client; a read it refuses, a loader
/// without the actor table and an area it does not serve give `None`.
pub struct LocalPlayer<'f> {
    handle: *const c_void,
    actor: Option<AmmlActorTable>,
    inventory: Option<AmmlInventoryTable>,
    world: Option<AmmlWorldTable>,
    module: &'static str,
    _event: PhantomData<&'f ()>,
}

impl<'f> LocalPlayer<'f> {
    pub(crate) fn new(
        handle: *const c_void,
        actor: Option<AmmlActorTable>,
        inventory: Option<AmmlInventoryTable>,
        world: Option<AmmlWorldTable>,
        module: &'static str,
    ) -> Self {
        Self { handle, actor, inventory, world, module, _event: PhantomData }
    }

    /// The stack in `hand`; `None` for an empty hand, a refused read, or a loader without the inventory table.
    /// Area `inventory.read` version 1 serves `Hand::Main`; version 2 (`Area::INVENTORY_READ.at(2)`) also
    /// `Hand::Off`.
    pub fn held_item(&self, hand: Hand) -> Option<ItemView<'f>> {
        let table = self.inventory?;
        let mut stack: *const c_void = std::ptr::null();
        // SAFETY: the loader checks the player handle before it reads game memory, and `stack` is a live local.
        #[allow(unsafe_code)]
        let status = unsafe { (table.held_item)(self.handle, hand.into(), &mut stack) };
        (status.is_ok() && !stack.is_null()).then(|| ItemView::new(table, self.handle, stack, self.module))
    }

    /// The inventory of the player; `None` without the inventory table. Area `inventory.read` version 1.
    pub fn inventory(&self) -> Option<Inventory<'f>> {
        self.inventory.map(|table| Inventory::new(table, self.handle, self.module))
    }

    /// Whether the player is alive: `Some(false)` from the moment it dies until it respawns. Area `actor.read`
    /// version 1.
    pub fn is_alive(&self) -> Option<bool> {
        let table = self.actor?;
        let mut out = 0u8;
        // SAFETY: the loader checks the handle before it reads game memory, and `out` is a live local.
        #[allow(unsafe_code)]
        let status = unsafe { (table.is_alive)(self.handle, &mut out) };
        status.is_ok().then_some(out != 0)
    }

    /// The position of the player's feet: the lowest point of its collision box, at the x and z of its position.
    /// Area `actor.read` version 2 (`Area::ACTOR_READ.at(2)`).
    pub fn position(&self) -> Option<Vec3> {
        let read = self.actor?.position?;
        let mut out = AmmlVec3::default();
        // SAFETY: the loader checks the handle before it reads game memory, and `out` is a live local.
        #[allow(unsafe_code)]
        let status = unsafe { read(self.handle, &mut out) };
        status.is_ok().then(|| Vec3::from(out))
    }

    /// The position of the player's eyes (the game's own position of the local player). Area `actor.read`
    /// version 2.
    pub fn eye_position(&self) -> Option<Vec3> {
        let read = self.actor?.eye_position?;
        let mut out = AmmlVec3::default();
        // SAFETY: the loader checks the handle before it reads game memory, and `out` is a live local.
        #[allow(unsafe_code)]
        let status = unsafe { read(self.handle, &mut out) };
        status.is_ok().then(|| Vec3::from(out))
    }

    /// The player's view direction, in degrees (`Rotation`). Area `actor.read` version 2.
    pub fn rotation(&self) -> Option<Rotation> {
        let read = self.actor?.rotation?;
        let mut out = AmmlRotation::default();
        // SAFETY: the loader checks the handle before it reads game memory, and `out` is a live local.
        #[allow(unsafe_code)]
        let status = unsafe { read(self.handle, &mut out) };
        status.is_ok().then(|| Rotation::from(out))
    }

    /// The dimension the player is in. Area `actor.read` version 2.
    pub fn dimension(&self) -> Option<Dimension<'f>> {
        let actor = self.actor?;
        let read = actor.dimension?;
        let mut out: *const c_void = std::ptr::null();
        // SAFETY: the loader checks the handle before it reads game memory, and `out` is a live local.
        #[allow(unsafe_code)]
        let status = unsafe { read(self.handle, &mut out) };
        (status.is_ok() && !out.is_null()).then(|| Dimension::new(out, actor, self.world))
    }

    /// The position of the player's head, as the game's own head attach point (the eyes of the model, not the camera).
    /// Area `actor.read` version 3 (`Area::ACTOR_READ.at(3)`).
    pub fn head_position(&self) -> Option<Vec3> {
        let read = self.actor?.head_position?;
        let mut out = AmmlVec3::default();
        // SAFETY: the loader checks the handle before it reads game memory, and `out` is a live local.
        #[allow(unsafe_code)]
        let status = unsafe { read(self.handle, &mut out) };
        status.is_ok().then(|| Vec3::from(out))
    }

    /// The block source of the dimension the player is in. Area `actor.read` version 3. The handle is valid in the
    /// game frame of the call: a later frame, another dimension or another world gives a refusal on every use.
    pub fn block_source(&self) -> Option<BlockSource<'f>> {
        let read = self.actor?.block_source?;
        let mut out: *const c_void = std::ptr::null();
        // SAFETY: the loader checks the handle before it reads game memory, and `out` is a live local.
        #[allow(unsafe_code)]
        let status = unsafe { read(self.handle, &mut out) };
        (status.is_ok() && !out.is_null()).then(|| BlockSource::new(out, self.world, self.module))
    }

    /// The direction the player looks, as a unit vector in world space (the game's `getViewVector` for the current
    /// frame). Area `actor.read` version 4 (`Area::ACTOR_READ.at(4)`).
    pub fn view_vector(&self) -> Option<Vec3> {
        let read = self.actor?.view_vector?;
        let mut out = AmmlVec3::default();
        // SAFETY: the loader checks the handle before it reads game memory, and `out` is a live local.
        #[allow(unsafe_code)]
        let status = unsafe { read(self.handle, &mut out) };
        status.is_ok().then(|| Vec3::from(out))
    }

    /// The collision box of the player in world space. Area `actor.read` version 4 (`Area::ACTOR_READ.at(4)`).
    pub fn aabb(&self) -> Option<Aabb> {
        let read = self.actor?.aabb?;
        let mut out = AmmlAabb::default();
        // SAFETY: the loader checks the handle before it reads game memory, and `out` is a live local.
        #[allow(unsafe_code)]
        let status = unsafe { read(self.handle, &mut out) };
        status.is_ok().then(|| Aabb::from(out))
    }

    /// The game mode of the player. A raw value the enum does not name gives `GameType::Other`. Area
    /// `actor.game_type` (`Area::ACTOR_GAME_TYPE`).
    pub fn game_type(&self) -> Option<GameType> {
        let read = self.actor?.game_type?;
        let mut out = AmmlGameType::default();
        // SAFETY: the loader checks the handle before it reads game memory, and `out` is a live local.
        #[allow(unsafe_code)]
        let status = unsafe { read(self.handle, &mut out) };
        status.is_ok().then(|| GameType::from(out))
    }

    /// The permission level of the player. Area `actor.permission` (`Area::ACTOR_PERMISSION`); a build whose game has
    /// no way the loader can read it does not offer the area.
    pub fn permission_level(&self) -> Option<PermissionLevel> {
        let read = self.actor?.permission_level?;
        let mut out = AmmlPermissionLevel::default();
        // SAFETY: the loader checks the handle before it reads game memory, and `out` is a live local.
        #[allow(unsafe_code)]
        let status = unsafe { read(self.handle, &mut out) };
        status.is_ok().then(|| PermissionLevel::from(out))
    }

    /// Turns the player to `rotation`, and its head to `head_yaw` when that is `Some`. The game does not interpolate
    /// from the old angle. Game thread only (`Refused` elsewhere); `Err(Loader(INVALID_ARGUMENT))` for a number that is
    /// not finite, with no change; `Err(Refused)` for a player of an earlier world; `Err(NotAvailable)` without the
    /// area. Area `actor.look_write` (`Area::ACTOR_LOOK_WRITE`).
    pub fn set_rotation(&self, rotation: Rotation, head_yaw: Option<f32>) -> Result<(), ApiError> {
        let write = self.actor.and_then(|table| table.set_rotation).ok_or(ApiError::NotAvailable)?;
        let rotation = AmmlRotation::from(rotation);
        // SAFETY: the loader checks the handle before it writes game memory, and `rotation` is a live local.
        #[allow(unsafe_code)]
        let status = unsafe { write(self.handle, &rotation, head_yaw.unwrap_or(0.0), u8::from(head_yaw.is_some())) };
        ApiError::check(status)
    }

    /// The game mode of the player: its own build, build delay and pick range (areas `player.build` and
    /// `player.pick_range`). `None` for a loader whose actor table has none of these calls (older than v5).
    pub fn game_mode(&self) -> Option<GameMode<'f>> {
        let actor = self.actor?;
        let any = actor.build_at.is_some() || actor.build_delay.is_some() || actor.max_pick_range.is_some();
        any.then(|| GameMode::new(self.handle, actor, self.module))
    }

    /// What the crosshair points at: the level's own hit result, the object the game's build and attack read (a
    /// block, an entity or nothing). Game thread. Area `player.hit_result` (`Area::PLAYER_HIT_RESULT`); `None` without
    /// it, on a loader before actor table v6, or when the loader refuses the read.
    pub fn hit_result(&self) -> Option<HitResult> {
        let read = self.actor?.hit_result?;
        let mut out = AmmlHitResult::default();
        // SAFETY: the loader checks the handle before it reads game memory, and `out` is a live local.
        #[allow(unsafe_code)]
        let status = unsafe { read(self.handle, &mut out) };
        status.is_ok().then(|| HitResult::from(out))
    }

    /// The health of the player, in half hearts as the game counts them. Area `actor.health`
    /// (`Area::ACTOR_HEALTH`).
    pub fn health(&self) -> Option<i32> {
        let read = self.actor?.health?;
        let mut out = 0i32;
        // SAFETY: the loader checks the handle before it reads game memory, and `out` is a live local.
        #[allow(unsafe_code)]
        let status = unsafe { read(self.handle, &mut out) };
        status.is_ok().then_some(out)
    }
}

impl PartialEq for LocalPlayer<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.handle == other.handle
    }
}
