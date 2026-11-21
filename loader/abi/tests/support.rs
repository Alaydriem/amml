use core::ffi::c_void;

use amml_abi::{
    AMML_API_VERSION, AmmlApi, AmmlBand, AmmlCoreTable, AmmlDirKind, AmmlEventContext, AmmlEventHeader,
    AmmlEventsTable, AmmlLogLevel, AmmlRegistryTable, AmmlServices, AmmlStatus, AmmlStr,
};

/// A v3 `AmmlApi` with its service tables, all backed by dummy `extern "C"` functions.
pub struct TestTables {
    pub api: AmmlApi,
    pub services: Box<AmmlServices>,
    _events: Box<AmmlEventsTable>,
    _core: Box<AmmlCoreTable>,
    _registry: Box<AmmlRegistryTable>,
}

impl TestTables {
    unsafe extern "C" fn resolve(_: AmmlStr, _: *mut u64) -> AmmlStatus {
        AmmlStatus::OK
    }

    unsafe extern "C" fn hook(
        _: *const c_void,
        _: *const c_void,
        _: i32,
        _: *mut *const c_void,
        _: *mut u64,
    ) -> AmmlStatus {
        AmmlStatus::OK
    }

    unsafe extern "C" fn unhook(_: u64) -> AmmlStatus {
        AmmlStatus::OK
    }

    unsafe extern "C" fn log(_: AmmlStr, _: AmmlLogLevel, _: AmmlStr) {}

    unsafe extern "C" fn heap_new(_: usize) -> *mut u8 {
        core::ptr::null_mut()
    }

    unsafe extern "C" fn heap_delete(_: *mut u8, _: usize) {}

    unsafe extern "C" fn mod_dir(_: AmmlStr, _: u8, _: *mut AmmlStr) -> AmmlStatus {
        AmmlStatus::OK
    }

    unsafe extern "C" fn listen(
        _: AmmlStr,
        _: u32,
        _: AmmlBand,
        _: u8,
        _: Option<unsafe extern "C" fn(*mut AmmlEventHeader, *const AmmlEventContext, *mut c_void) -> AmmlStatus>,
        _: *mut c_void,
        _: *mut u64,
    ) -> AmmlStatus {
        AmmlStatus::OK
    }

    unsafe extern "C" fn unlisten(_: u64) -> AmmlStatus {
        AmmlStatus::OK
    }

    unsafe extern "C" fn log_fields(
        _: AmmlStr,
        _: AmmlLogLevel,
        _: AmmlStr,
        _: *const AmmlStr,
        _: *const AmmlStr,
        _: usize,
    ) {
    }

    unsafe extern "C" fn dir(_: AmmlStr, _: AmmlDirKind, _: *mut AmmlStr) -> AmmlStatus {
        AmmlStatus::OK
    }

    unsafe extern "C" fn available(_: AmmlStr, _: u32) -> AmmlStatus {
        AmmlStatus::OK
    }

    unsafe extern "C" fn game_alive() -> u8 {
        0
    }

    unsafe extern "C" fn provide(_: AmmlStr, _: u16, _: u16, _: *const c_void) -> AmmlStatus {
        AmmlStatus::OK
    }

    unsafe extern "C" fn require(_: AmmlStr, _: u16, _: u16, _: *mut *const c_void, _: *mut u16) -> AmmlStatus {
        AmmlStatus::NOT_AVAILABLE
    }

    pub fn v3() -> Self {
        let events = Box::new(AmmlEventsTable { size: 24, version: 1, listen: Self::listen, unlisten: Self::unlisten });
        let core = Box::new(AmmlCoreTable {
            size: AmmlCoreTable::SIZE_V1,
            version: 1,
            log: Self::log_fields,
            dir: Self::dir,
            available: Self::available,
            game_alive: Self::game_alive,
        });
        let registry =
            Box::new(AmmlRegistryTable { size: 24, version: 1, provide: Self::provide, require: Self::require });
        let services = Box::new(AmmlServices {
            size: AmmlServices::SIZE_V1,
            version: 1,
            events: &raw const *events,
            core: &raw const *core,
            registry: &raw const *registry,
            client: core::ptr::null(),
            input: core::ptr::null(),
            window: core::ptr::null(),
            overlay: core::ptr::null(),
            actor: core::ptr::null(),
            inventory: core::ptr::null(),
            lighting: core::ptr::null(),
            ui: core::ptr::null(),
            network: core::ptr::null(),
        });
        let api = AmmlApi {
            size: AmmlApi::SIZE_V3,
            version: AMML_API_VERSION,
            module_base: 0,
            resolve: Self::resolve,
            hook: Self::hook,
            unhook: Self::unhook,
            log: Self::log,
            heap_new: Self::heap_new,
            heap_delete: Self::heap_delete,
            mod_dir: Self::mod_dir,
            services: &raw const *services,
        };
        Self { api, services, _events: events, _core: core, _registry: registry }
    }

    pub fn services_ptr(&self) -> *const AmmlServices {
        &raw const *self.services
    }
}
