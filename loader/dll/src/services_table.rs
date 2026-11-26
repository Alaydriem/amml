use std::ffi::c_void;

use amml_abi::{
    AmmlBand, AmmlCoreTable, AmmlDirKind, AmmlEventsTable, AmmlLogLevel, AmmlRegistryTable, AmmlServices, AmmlStatus,
    AmmlStr, EventCallbackFn, KnownLogLevel,
};

use crate::actor_table::ACTOR;
use crate::camera_table::CAMERA;
use crate::client_table::CLIENT;
use crate::input_table::INPUT;
use crate::inventory_table::INVENTORY;
use crate::lighting_table::LIGHTING;
use crate::network_table::NETWORK;
use crate::options_table::OPTIONS;
use crate::overlay_table::OVERLAY;
use crate::render_table::RENDER;
use crate::time_table::TIME;
use crate::ui_table::UI;
use crate::window_table::WINDOW;
use crate::world_table::WORLD;
use crate::{Boundary, LoaderLog, LoaderState};

const TABLE_VERSION: u32 = 1;
// The services table is at v13: v12 plus the `time` table (`AmmlServices::SIZE_V13`).
const SERVICES_VERSION: u32 = 13;

static EVENTS: AmmlEventsTable = AmmlEventsTable {
    size: std::mem::size_of::<AmmlEventsTable>() as u32,
    version: TABLE_VERSION,
    listen: ServicesTable::listen,
    unlisten: ServicesTable::unlisten,
};

static CORE: AmmlCoreTable = AmmlCoreTable {
    size: std::mem::size_of::<AmmlCoreTable>() as u32,
    version: TABLE_VERSION,
    log: ServicesTable::log,
    dir: ServicesTable::dir,
    available: ServicesTable::available,
    game_alive: ServicesTable::game_alive,
};

static REGISTRY: AmmlRegistryTable = AmmlRegistryTable {
    size: std::mem::size_of::<AmmlRegistryTable>() as u32,
    version: TABLE_VERSION,
    provide: ServicesTable::provide,
    require: ServicesTable::require,
};

static SERVICES: ServicesTable = ServicesTable(AmmlServices {
    size: AmmlServices::SIZE_V13,
    version: SERVICES_VERSION,
    events: &EVENTS,
    core: &CORE,
    registry: &REGISTRY,
    client: &CLIENT,
    input: &INPUT,
    window: &WINDOW,
    overlay: &OVERLAY,
    actor: &ACTOR,
    inventory: &INVENTORY,
    lighting: &LIGHTING,
    ui: &UI,
    network: &NETWORK,
    world: &WORLD,
    render: &RENDER,
    camera: &CAMERA,
    options: &OPTIONS,
    time: &TIME,
});

/// The v13 service tables (events, core, registry, client, input, window, overlay, actor, inventory, lighting, ui, network, world, render, camera, options, time) behind `AmmlApi::services`, and their
/// `extern "C"` functions. Each function is a thin delegator into `LoaderState` inside
/// `Boundary::guard`, so a panic becomes `AmmlStatus::PANICKED`.
pub struct ServicesTable(AmmlServices);

// SAFETY: the tables are immutable statics; the pointers in `AmmlServices` point to other statics.
unsafe impl Sync for ServicesTable {}

impl ServicesTable {
    /// Most fields one `log` call may carry; more are left out.
    const MAX_FIELDS: usize = 64;

    /// The services table every `AmmlApi` points to. It lives for the process.
    pub fn build() -> *const AmmlServices {
        &SERVICES.0
    }

    unsafe extern "C" fn listen(
        event: AmmlStr,
        version: u32,
        band: AmmlBand,
        required: u8,
        callback: Option<EventCallbackFn>,
        user: *mut c_void,
        handle: *mut u64,
    ) -> AmmlStatus {
        Boundary::guard(|| {
            let Some(state) = LoaderState::get() else {
                return AmmlStatus::FAILED;
            };
            let Some(callback) = callback else {
                return AmmlStatus::INVALID_ARGUMENT;
            };
            if handle.is_null() {
                return AmmlStatus::INVALID_ARGUMENT;
            }
            // SAFETY: the caller promises `event` is valid for the call.
            let event = match unsafe { event.as_str() } {
                Ok(event) => event,
                Err(status) => return status,
            };
            match state.listen(event, version, band, required != 0, callback, user.expose_provenance()) {
                Ok(id) => {
                    // SAFETY: `handle` is non-null and the caller promises it is writable.
                    unsafe { handle.write(id) };
                    AmmlStatus::OK
                }
                Err(status) => status,
            }
        })
    }

    unsafe extern "C" fn unlisten(handle: u64) -> AmmlStatus {
        Boundary::guard(|| match LoaderState::get() {
            Some(state) => match state.unlisten(handle) {
                Ok(()) => AmmlStatus::OK,
                Err(status) => status,
            },
            None => AmmlStatus::FAILED,
        })
    }

    unsafe extern "C" fn log(
        module: AmmlStr,
        level: AmmlLogLevel,
        message: AmmlStr,
        keys: *const AmmlStr,
        values: *const AmmlStr,
        count: usize,
    ) {
        Boundary::run((), || {
            // SAFETY: the caller promises both views are valid for the call.
            let (module, message) = match unsafe { (module.as_str(), message.as_str()) } {
                (Ok(module), Ok(message)) => (module, message),
                _ => {
                    curia::warn!("a mod sent a log line that is not valid text");
                    return;
                }
            };
            let level = level.known().unwrap_or(KnownLogLevel::Info);
            // SAFETY: the caller promises `keys` and `values` hold `count` views valid for the call.
            let line = unsafe { Self::with_fields(message, keys, values, count) };
            LoaderLog::module_line(module, level, &line);
        });
    }

    /// `message` followed by ` key=value` for each field. Fields that are not valid text are left out.
    ///
    /// # Safety
    /// With `count > 0`, non-null `keys` and `values` each point to `count` views valid for the call.
    unsafe fn with_fields(message: &str, keys: *const AmmlStr, values: *const AmmlStr, count: usize) -> String {
        let mut line = message.to_string();
        if count == 0 || keys.is_null() || values.is_null() {
            return line;
        }
        let count = count.min(Self::MAX_FIELDS);
        // SAFETY: the caller guarantees both arrays hold at least `count` views.
        let (keys, values) =
            unsafe { (std::slice::from_raw_parts(keys, count), std::slice::from_raw_parts(values, count)) };
        for (key, value) in keys.iter().zip(values) {
            // SAFETY: the caller guarantees each view is valid for the call.
            if let (Ok(key), Ok(value)) = unsafe { (key.as_str(), value.as_str()) } {
                line.push(' ');
                line.push_str(key);
                line.push('=');
                line.push_str(value);
            }
        }
        line
    }

    unsafe extern "C" fn dir(module: AmmlStr, kind: AmmlDirKind, out: *mut AmmlStr) -> AmmlStatus {
        Boundary::guard(|| {
            let Some(state) = LoaderState::get() else {
                return AmmlStatus::FAILED;
            };
            if out.is_null() {
                return AmmlStatus::INVALID_ARGUMENT;
            }
            // SAFETY: the caller promises `module` is valid for the call.
            let module = match unsafe { module.as_str() } {
                Ok(module) => module,
                Err(status) => return status,
            };
            match state.dir(module, kind) {
                Ok(view) => {
                    // SAFETY: `out` is non-null and the caller promises it is writable.
                    unsafe { out.write(view) };
                    AmmlStatus::OK
                }
                Err(status) => status,
            }
        })
    }

    unsafe extern "C" fn available(event: AmmlStr, version: u32) -> AmmlStatus {
        Boundary::guard(|| {
            let Some(state) = LoaderState::get() else {
                return AmmlStatus::FAILED;
            };
            // SAFETY: the caller promises `event` is valid for the call.
            let event = match unsafe { event.as_str() } {
                Ok(event) => event,
                Err(status) => return status,
            };
            if state.events().is_available(event, version) || state.areas().is_available(event, version) {
                AmmlStatus::OK
            } else {
                AmmlStatus::NOT_AVAILABLE
            }
        })
    }

    unsafe extern "C" fn game_alive() -> u8 {
        Boundary::run(0, || LoaderState::get().map_or(0, |state| u8::from(state.events().gate().is_open())))
    }

    unsafe extern "C" fn provide(name: AmmlStr, major: u16, minor: u16, table: *const c_void) -> AmmlStatus {
        Boundary::guard(|| {
            let Some(state) = LoaderState::get() else {
                return AmmlStatus::FAILED;
            };
            if table.is_null() {
                return AmmlStatus::INVALID_ARGUMENT;
            }
            // SAFETY: the caller promises `name` is valid for the call.
            let name = match unsafe { name.as_str() } {
                Ok(name) => name,
                Err(status) => return status,
            };
            match state.provide(name, major, minor, table.expose_provenance()) {
                Ok(()) => AmmlStatus::OK,
                Err(status) => status,
            }
        })
    }

    unsafe extern "C" fn require(
        name: AmmlStr,
        major: u16,
        min_minor: u16,
        out_table: *mut *const c_void,
        out_minor: *mut u16,
    ) -> AmmlStatus {
        Boundary::guard(|| {
            let Some(state) = LoaderState::get() else {
                return AmmlStatus::FAILED;
            };
            if out_table.is_null() || out_minor.is_null() {
                return AmmlStatus::INVALID_ARGUMENT;
            }
            // SAFETY: the caller promises `name` is valid for the call.
            let name = match unsafe { name.as_str() } {
                Ok(name) => name,
                Err(status) => return status,
            };
            match state.require(name, major, min_minor) {
                Ok((table, minor)) => {
                    // SAFETY: both outputs are non-null and the caller promises they are writable.
                    unsafe {
                        out_table.write(std::ptr::with_exposed_provenance::<c_void>(table));
                        out_minor.write(minor);
                    }
                    AmmlStatus::OK
                }
                Err(status) => status,
            }
        })
    }
}
