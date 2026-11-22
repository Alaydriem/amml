use amml_abi::{
    AmmlActorTable, AmmlCameraTable, AmmlClientTable, AmmlInputTable, AmmlInventoryTable, AmmlLightingTable,
    AmmlNetworkTable, AmmlOptionsTable, AmmlOverlayTable, AmmlRenderTable, AmmlServices, AmmlTimeTable, AmmlUiTable,
    AmmlWindowTable, AmmlWorldTable,
};

use crate::Area;

/// Copies of the loader's client, input, window, overlay, actor, inventory, lighting, ui, network, world, render and camera tables, taken once by `Ctx::from_raw`. A
/// table the services table does not cover, a null one, or one shorter than its v1 size is `None`, and
/// every call of its area gives `None` or `NotAvailable`.
#[derive(Clone, Copy)]
pub(crate) struct AreaTables {
    pub(crate) client: Option<AmmlClientTable>,
    pub(crate) input: Option<AmmlInputTable>,
    pub(crate) window: Option<AmmlWindowTable>,
    pub(crate) overlay: Option<AmmlOverlayTable>,
    pub(crate) actor: Option<AmmlActorTable>,
    pub(crate) inventory: Option<AmmlInventoryTable>,
    pub(crate) lighting: Option<AmmlLightingTable>,
    pub(crate) ui: Option<AmmlUiTable>,
    pub(crate) network: Option<AmmlNetworkTable>,
    pub(crate) world: Option<AmmlWorldTable>,
    pub(crate) render: Option<AmmlRenderTable>,
    pub(crate) camera: Option<AmmlCameraTable>,
    pub(crate) options: Option<AmmlOptionsTable>,
    pub(crate) time: Option<AmmlTimeTable>,
}

impl AreaTables {
    /// Copies the tables of `services`.
    ///
    /// # Safety
    /// `services` is non-null and readable for its `size` bytes; a non-null table it covers points to a
    /// table readable for its own `size` bytes, which stays valid while the loader runs.
    #[allow(unsafe_code)]
    pub(crate) unsafe fn read(services: *const AmmlServices) -> Self {
        // SAFETY: the caller's guarantee is the contract of the `AmmlServices` readers, which check
        // `size` and the table's own size before they give a table.
        let (
            client,
            input,
            window,
            overlay,
            actor,
            inventory,
            lighting,
            ui,
            network,
            world,
            render,
            camera,
            options,
            time,
        ) = unsafe {
            (
                AmmlServices::client(services),
                AmmlServices::input(services),
                AmmlServices::window(services),
                AmmlServices::overlay(services),
                AmmlServices::actor(services),
                AmmlServices::inventory(services),
                AmmlServices::lighting(services),
                AmmlServices::ui(services),
                AmmlServices::network(services),
                AmmlServices::world(services),
                AmmlServices::render(services),
                AmmlServices::camera(services),
                AmmlServices::options(services),
                AmmlServices::time(services),
            )
        };
        // SAFETY: each table given is at least its v1 size, so the whole v1 struct is readable; the input
        // table, which has grown, is read only as far as its own `size` covers.
        unsafe {
            Self {
                client: client.and_then(|table| AmmlClientTable::read(table)),
                input: input.and_then(|table| AmmlInputTable::read(table)),
                window: window.and_then(|table| AmmlWindowTable::read(table)),
                overlay: overlay.map(|table| table.read()),
                actor: actor.and_then(|table| AmmlActorTable::read(table)),
                inventory: inventory.and_then(|table| AmmlInventoryTable::read(table)),
                lighting: lighting.map(|table| table.read()),
                ui: ui.and_then(|table| AmmlUiTable::read(table)),
                network: network.and_then(|table| AmmlNetworkTable::read(table)),
                world: world.and_then(|table| AmmlWorldTable::read(table)),
                render: render.and_then(|table| AmmlRenderTable::read(table)),
                camera: camera.and_then(|table| AmmlCameraTable::read(table)),
                options: options.and_then(|table| AmmlOptionsTable::read(table)),
                time: time.and_then(|table| AmmlTimeTable::read(table)),
            }
        }
    }

    /// Whether the table that serves `area` is here. An area of no table needs none.
    pub(crate) fn serves(&self, area: Area) -> bool {
        if area.is_client() {
            self.client.is_some()
        } else if area.is_input() {
            self.input.is_some()
        } else if area.is_window() {
            self.window.is_some()
        } else if area.is_overlay() {
            self.overlay.is_some()
        } else if area.is_actor() {
            self.actor.is_some()
        } else if area.is_inventory() {
            self.inventory.is_some()
        } else if area.is_lighting() {
            self.lighting.is_some()
        } else if area.is_ui() {
            self.ui.is_some()
        } else if area.is_network() {
            match area.id {
                "network.chunk_cache" => self.network.is_some_and(|table| table.set_client_chunk_cache.is_some()),
                "network.protocol" => self.network.is_some_and(|table| table.protocol_version.is_some()),
                _ => self.network.is_some(),
            }
        } else if area.is_world() {
            self.world.is_some()
        } else if area.is_render() {
            self.render.is_some()
        } else if area.is_camera() {
            self.camera.is_some()
        } else if area.is_options() {
            self.options.is_some()
        } else if area.is_time() {
            self.time.is_some()
        } else {
            true
        }
    }
}
