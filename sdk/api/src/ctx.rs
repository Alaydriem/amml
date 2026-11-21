use amml_abi::{
    AMML_API_VERSION, AmmlApi, AmmlCoreTable, AmmlEventsTable, AmmlRegistryTable, AmmlServices, AmmlStatus, AmmlStr,
    AvailableFn, DirFn, GameAliveFn, ListenFn, LogFieldsFn, ProvideFn, RequireFn, UnlistenFn,
};

use crate::area_tables::AreaTables;
use crate::client_tables::ClientTables;
use crate::lent_tables::LentTables;
use crate::{
    ApiError, Area, Biomes, Blocks, CameraControl, Chat, Dirs, DisableCtx, Event, Events, Input, Lighting, Logger,
    Overlay, Registry, Textures, Uses, Window, WorldShapes,
};

/// The checked v3 loader tables of one mod: a cheap `Copy` view that holds function pointers, the
/// mod id and the mod's `Uses`. The client and input tables are copied when the services table has
/// them (`size >= SIZE_V2`), the window and overlay tables when it is v3 (`size >= SIZE_V3`); without a
/// table its areas are not supported.
#[derive(Clone, Copy)]
pub struct Ctx {
    listen: ListenFn,
    unlisten: UnlistenFn,
    log: LogFieldsFn,
    dir: DirFn,
    available: AvailableFn,
    game_alive: GameAliveFn,
    provide: ProvideFn,
    require: RequireFn,
    areas: AreaTables,
    module: &'static str,
    uses: Uses,
}

impl Ctx {
    /// Checks and copies the table the loader passed to `amml_mod_load`. A table older than v3
    /// (`version < 3` or `size < SIZE_V3`), or one whose service tables are missing or shorter than
    /// their v1 size (`SIZE_V1`), gives `Loader(VERSION_MISMATCH)`; a null table gives
    /// `Loader(INVALID_ARGUMENT)`. A table longer than this crate knows is accepted.
    ///
    /// Every read goes through a raw pointer, so a short table is never read past its `size`.
    /// `module` is the mod id, `@<namespace>/<name>`.
    ///
    /// # Safety
    /// A non-null `api` must point to a table of at least `size` bytes whose service tables, when
    /// present, are each at least their own `size` bytes, as the loader promises.
    #[allow(unsafe_code)]
    pub unsafe fn from_raw(api: *const AmmlApi, module: &'static str, uses: Uses) -> Result<Self, ApiError> {
        const MISMATCH: ApiError = ApiError::Loader(AmmlStatus::VERSION_MISMATCH);
        // SAFETY: the caller guarantees a non-null `api` has its two leading fields readable.
        let Some((size, version)) = (unsafe { AmmlApi::header(api) }) else {
            return Err(ApiError::Loader(AmmlStatus::INVALID_ARGUMENT));
        };
        if version < AMML_API_VERSION || size < AmmlApi::SIZE_V3 {
            return Err(MISMATCH);
        }
        // SAFETY: the caller guarantees `size` bytes are readable; `services` checks `size` itself.
        let services = unsafe { AmmlApi::services(api) }.ok_or(MISMATCH)?;
        // SAFETY: a non-null services table holds at least its leading `size` field.
        if unsafe { Self::table_size(services) } < AmmlServices::SIZE_V1 {
            return Err(MISMATCH);
        }
        // SAFETY: `size` covers every v1 field of `AmmlServices`.
        let (events, core, registry) = unsafe {
            (
                core::ptr::addr_of!((*services).events).read(),
                core::ptr::addr_of!((*services).core).read(),
                core::ptr::addr_of!((*services).registry).read(),
            )
        };
        if events.is_null() || core.is_null() || registry.is_null() {
            return Err(MISMATCH);
        }
        // SAFETY: each table is non-null and holds at least its leading `size` field.
        let sizes = unsafe { (Self::table_size(events), Self::table_size(core), Self::table_size(registry)) };
        if sizes.0 < AmmlEventsTable::SIZE_V1
            || sizes.1 < AmmlCoreTable::SIZE_V1
            || sizes.2 < AmmlRegistryTable::SIZE_V1
        {
            return Err(MISMATCH);
        }
        // SAFETY: `services` is non-null and readable for its `size`; the loader promises the area tables
        // it covers are readable for their own `size`.
        let areas = unsafe { AreaTables::read(services) };
        // SAFETY: each table's `size` covers its v1 fields, which are the fields read here; no
        // reference to a table is formed.
        let ctx = unsafe {
            Self {
                listen: core::ptr::addr_of!((*events).listen).read(),
                unlisten: core::ptr::addr_of!((*events).unlisten).read(),
                log: core::ptr::addr_of!((*core).log).read(),
                dir: core::ptr::addr_of!((*core).dir).read(),
                available: core::ptr::addr_of!((*core).available).read(),
                game_alive: core::ptr::addr_of!((*core).game_alive).read(),
                provide: core::ptr::addr_of!((*registry).provide).read(),
                require: core::ptr::addr_of!((*registry).require).read(),
                areas,
                module,
                uses,
            }
        };
        Ok(ctx)
    }

    /// The leading `size` field of a table.
    ///
    /// # Safety
    /// `table` must be non-null and point to at least 4 readable bytes, suitably aligned.
    #[allow(unsafe_code)]
    unsafe fn table_size<T>(table: *const T) -> u32 {
        // SAFETY: the caller guarantees the leading u32 is readable.
        unsafe { table.cast::<u32>().read() }
    }

    /// The mod id, `@<namespace>/<name>`.
    pub fn module(&self) -> &'static str {
        self.module
    }

    pub fn events(&self) -> Events {
        let lent = LentTables {
            copy_back_buffer: self.areas.overlay.map(|table| table.copy_back_buffer),
            ui: self.areas.ui,
            inventory: self.areas.inventory,
            network: self.areas.network,
            render: self.areas.render,
            camera: self.areas.camera,
        };
        Events::new(self.listen, self.unlisten, self.uses, self.log(), self.client_tables(), lent, self.module)
    }

    pub fn log(&self) -> Logger {
        Logger::new(self.log, self.module)
    }

    pub fn dirs(&self) -> Dirs {
        Dirs::new(self.dir, self.module)
    }

    pub fn registry(&self) -> Registry {
        Registry::new(self.provide, self.require)
    }

    /// The loader's input service. Without an input table every call gives `None` or `NotAvailable`.
    pub fn input(&self) -> Input {
        Input::new(self.areas.input, self.module)
    }

    /// The game window. Without a window table every call gives `None` or `NotAvailable`.
    pub fn window(&self) -> Window {
        Window::new(self.areas.window, self.module)
    }

    /// Screenshots, capture streams and the present count. Without an overlay table every call gives
    /// `NotAvailable` or 0.
    pub fn overlay(&self) -> Overlay {
        Overlay::new(self.areas.overlay, self.module)
    }

    /// The loader-owned held light. Without a lighting table every call gives `NotAvailable`.
    pub fn lighting(&self) -> Lighting {
        Lighting::new(self.areas.lighting, self.module)
    }

    /// The textures of the loader. Without a ui table every call gives `NotAvailable`.
    pub fn textures(&self) -> Textures {
        Textures::new(self.areas.ui, self.module)
    }

    /// The chat of the game: inline icons. Without a ui table every call gives `NotAvailable`.
    pub fn chat(&self) -> Chat {
        Chat::new(self.areas.ui, self.module)
    }

    /// The client's free camera (`Area::CAMERA_FREE`). Without a camera table, or with a v1 one, every call gives
    /// `NotAvailable` or `None`.
    pub fn camera(&self) -> CameraControl {
        CameraControl::new(self.areas.camera, self.module)
    }

    /// Lines, boxes and arrows in the world (`Area::WORLD_SHAPES`). Without a world table, or with one older than v5,
    /// every call gives `NotAvailable`, `0` or `None`.
    pub fn world_shapes(&self) -> WorldShapes {
        WorldShapes::new(self.areas.world, self.module)
    }

    /// The block types of the game: names for the ids of `ColumnSample`. Without a world table every call gives `None`.
    pub fn blocks(&self) -> Blocks {
        Blocks::new(self.areas.world)
    }

    /// The biomes of the game: names for the ids of `ColumnSample`. Without a world table every call gives `None`.
    pub fn biomes(&self) -> Biomes {
        Biomes::new(self.areas.world)
    }

    /// Whether the loader serves `area` at `area.version` now: the table of the area is there and the
    /// loader's area book offers it. Asks the loader on every call, so it follows the book, also when
    /// the loader withdraws an area after a later check.
    pub fn supports(&self, area: Area) -> bool {
        if !self.areas.serves(area) {
            return false;
        }
        // SAFETY: the id lives for the call.
        #[allow(unsafe_code)]
        let status = unsafe { (self.available)(AmmlStr::from_str(area.id), area.version) };
        status.is_ok()
    }

    /// The `DisableCtx` of a disable, which can ask the loader for the current client.
    pub(crate) fn disable_ctx(&self, game_alive: bool) -> DisableCtx {
        DisableCtx::with_client(game_alive, self.client_tables(), self.module)
    }

    // The tables of `Client` and `LocalPlayer`; `None` without a client table.
    fn client_tables(&self) -> Option<ClientTables> {
        self.areas.client.map(|client| ClientTables {
            client,
            actor: self.areas.actor,
            inventory: self.areas.inventory,
            world: self.areas.world,
        })
    }

    /// Whether the loader can deliver `E` on this game build.
    pub fn available<E: Event>(&self) -> bool {
        // SAFETY: the id lives for the call.
        #[allow(unsafe_code)]
        let status = unsafe { (self.available)(AmmlStr::from_str(E::ID), E::VERSION) };
        status.is_ok()
    }

    /// Whether the game runs and game actions may be called: from the first frame until shutdown.
    pub fn game_alive(&self) -> bool {
        // SAFETY: the loader's function takes no arguments.
        #[allow(unsafe_code)]
        let alive = unsafe { (self.game_alive)() };
        alive != 0
    }
}
