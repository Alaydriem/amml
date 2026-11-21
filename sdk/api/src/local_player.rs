use std::ffi::c_void;
use std::marker::PhantomData;

use amml_abi::{AmmlActorTable, AmmlInventoryTable, AmmlRotation, AmmlVec3, AmmlWorldTable};

use crate::{BlockSource, Dimension, Hand, Inventory, ItemView, Rotation, Vec3};

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
        (status.is_ok() && !out.is_null()).then(|| BlockSource::new(out, self.world))
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
