//! A mod built on `amml-abi` only. It registers listeners through the raw v3 tables and appends one
//! line per call to `<mods>/events.log`, which the loader tests read.

use std::ffi::c_void;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicPtr, Ordering};

use amml_abi::{
    AmmlApi, AmmlBand, AmmlEventContext, AmmlEventHeader, AmmlModInfo, AmmlServices, AmmlStatus, AmmlStr, AmmlUseRow,
    AmmlUses, ProviderGone, ProviderReady,
};

const NAMESPACE: &str = "events";
#[cfg(feature = "panic-listener")]
const NAME: &str = "events-panic";
#[cfg(all(feature = "provider", not(feature = "panic-listener")))]
const NAME: &str = "events-provider";
#[cfg(all(feature = "provider-fails", not(any(feature = "panic-listener", feature = "provider"))))]
const NAME: &str = "events-provider-fails";
#[cfg(all(
    feature = "require-tick",
    not(any(feature = "panic-listener", feature = "provider", feature = "provider-fails"))
))]
const NAME: &str = "events-required";
#[cfg(all(
    feature = "require-in-load",
    not(any(feature = "panic-listener", feature = "provider", feature = "provider-fails", feature = "require-tick"))
))]
const NAME: &str = "events-load-required";
#[cfg(all(
    feature = "require-area",
    not(any(
        feature = "panic-listener",
        feature = "provider",
        feature = "provider-fails",
        feature = "require-tick",
        feature = "require-in-load"
    ))
))]
const NAME: &str = "events-area";
#[cfg(all(
    feature = "optional-area",
    not(any(
        feature = "panic-listener",
        feature = "provider",
        feature = "provider-fails",
        feature = "require-tick",
        feature = "require-in-load",
        feature = "require-area"
    ))
))]
const NAME: &str = "events-optional-area";
#[cfg(not(any(
    feature = "panic-listener",
    feature = "provider",
    feature = "provider-fails",
    feature = "require-tick",
    feature = "require-in-load",
    feature = "require-area",
    feature = "optional-area"
)))]
const NAME: &str = "events";

/// The events this mod listens to; a listener's `user` value is its index here.
const EVENTS: [&str; 5] =
    ["client.tick", "mods.loaded", "registry.provider_ready", "registry.provider_gone", "loader.shutdown"];

/// The table this mod provides as `test.table` 1.0.
#[cfg(any(feature = "provider", feature = "provider-fails"))]
#[repr(C)]
struct TestTable {
    size: u32,
    version: u32,
}

#[cfg(any(feature = "provider", feature = "provider-fails"))]
static TABLE: TestTable = TestTable { size: 8, version: 1 };
static API: AtomicPtr<AmmlApi> = AtomicPtr::new(std::ptr::null_mut());
static RECORD: OnceLock<PathBuf> = OnceLock::new();

struct Fixture;

impl Fixture {
    fn id() -> String {
        format!("@{NAMESPACE}/{NAME}")
    }

    fn api() -> Option<&'static AmmlApi> {
        let api = API.load(Ordering::Acquire);
        // SAFETY: the loader's table lives for the process and is a full v3 table (checked in load).
        unsafe { api.cast_const().as_ref() }
    }

    fn services() -> Option<&'static AmmlServices> {
        let api = Self::api()?;
        // SAFETY: `api` is the loader's live table.
        let services = unsafe { AmmlApi::services(api) }?;
        // SAFETY: the loader keeps its service tables alive for the process.
        unsafe { services.as_ref() }
    }

    fn record(line: &str) {
        let Some(path) = RECORD.get() else {
            return;
        };
        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
            let _ = writeln!(file, "{NAME} {line}");
        }
    }

    // `<mods>/@events/<name>` is the mod folder; the record file sits in `<mods>`.
    fn find_record_file(api: &AmmlApi) -> Option<PathBuf> {
        let id = Self::id();
        let mut out = AmmlStr::EMPTY;
        // SAFETY: `id` lives for the call and `out` is writable.
        let status = unsafe { (api.mod_dir)(AmmlStr::from_str(&id), 0, &mut out) };
        if !status.is_ok() {
            return None;
        }
        // SAFETY: the loader keeps the folder bytes valid for the process.
        let dir = PathBuf::from(unsafe { out.as_str() }.ok()?);
        Some(dir.parent()?.parent()?.join("events.log"))
    }

    fn listen(event: usize, required: bool) -> AmmlStatus {
        let Some(services) = Self::services() else {
            return AmmlStatus::FAILED;
        };
        let mut handle = 0u64;
        // SAFETY: the events table is live; `on_event` has the callback signature.
        unsafe {
            ((*services.events).listen)(
                AmmlStr::from_str(EVENTS[event]),
                1,
                AmmlBand::NORMAL,
                u8::from(required),
                Some(Self::on_event),
                event as *mut c_void,
                &mut handle,
            )
        }
    }

    // 1 while the game runs, from the core table; 9 when the table cannot be reached.
    fn game_alive() -> u8 {
        let Some(services) = Self::services() else {
            return 9;
        };
        // SAFETY: the core table is live.
        unsafe { ((*services.core).game_alive)() }
    }

    #[cfg(any(feature = "provider", feature = "provider-fails"))]
    fn provide() -> AmmlStatus {
        let Some(services) = Self::services() else {
            return AmmlStatus::FAILED;
        };
        // SAFETY: the registry table is live; `TABLE` is static.
        unsafe {
            ((*services.registry).provide)(
                AmmlStr::from_str("test.table"),
                1,
                0,
                std::ptr::from_ref(&TABLE).cast::<c_void>(),
            )
        }
    }

    unsafe extern "C" fn on_event(
        event: *mut AmmlEventHeader,
        context: *const AmmlEventContext,
        user: *mut c_void,
    ) -> AmmlStatus {
        let index = user as usize;
        let Some(id) = EVENTS.get(index) else {
            return AmmlStatus::INVALID_ARGUMENT;
        };
        // SAFETY: the loader passes a live context for the call.
        let Some(context) = (unsafe { context.as_ref() }) else {
            return AmmlStatus::INVALID_ARGUMENT;
        };
        let mut line = format!("{id} thread={} epoch={} alive={}", context.thread.0, context.epoch, context.game_alive);
        match *id {
            "registry.provider_ready" => {
                // SAFETY: the loader raises `ProviderReady` for this id.
                let event = unsafe { &*event.cast::<ProviderReady>() };
                // SAFETY: the name is valid for the call.
                let name = unsafe { event.name.as_str() }.unwrap_or("?");
                line.push_str(&format!(" name={name} major={} minor={}", event.major, event.minor));
            }
            "registry.provider_gone" => {
                // SAFETY: the loader raises `ProviderGone` for this id.
                let event = unsafe { &*event.cast::<ProviderGone>() };
                // SAFETY: the name is valid for the call.
                let name = unsafe { event.name.as_str() }.unwrap_or("?");
                line.push_str(&format!(" name={name} major={}", event.major));
            }
            _ => {}
        }
        Self::record(&line);
        if cfg!(feature = "panic-listener") && index == 0 {
            return AmmlStatus::PANICKED;
        }
        AmmlStatus::OK
    }
}

/// # Safety
/// Called by the AMML loader with its API table and a writable info block of `SIZE_V3` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn amml_mod_load(api: *const AmmlApi, info: *mut AmmlModInfo) -> AmmlStatus {
    // SAFETY: the loader passes a live table.
    match unsafe { AmmlApi::header(api) } {
        Some((size, _)) if size >= AmmlApi::SIZE_V3 => {}
        _ => return AmmlStatus::VERSION_MISMATCH,
    }
    API.store(api.cast_mut(), Ordering::Release);
    let required = u8::from(cfg!(feature = "require-tick"));
    let mut rows = vec![AmmlUseRow {
        feature: AmmlStr::from_str("ticker"),
        kind: 0,
        name: AmmlStr::from_str("client.tick"),
        version: 1,
        required,
    }];
    if cfg!(any(feature = "require-area", feature = "optional-area")) {
        rows.push(AmmlUseRow {
            feature: AmmlStr::from_str("keys"),
            kind: AmmlUseRow::KIND_AREA,
            name: AmmlStr::from_str("input.key_inject"),
            version: 1,
            required: u8::from(cfg!(feature = "require-area")),
        });
    }
    if cfg!(feature = "optional-area") {
        // A version no loader serves yet, for the `loader too old` line.
        rows.push(AmmlUseRow {
            feature: AmmlStr::from_str("future"),
            kind: AmmlUseRow::KIND_AREA,
            name: AmmlStr::from_str("actor.read"),
            version: 9,
            required: 0,
        });
    }
    if cfg!(feature = "free-camera") {
        rows.push(AmmlUseRow {
            feature: AmmlStr::from_str("freecam"),
            kind: AmmlUseRow::KIND_AREA,
            name: AmmlStr::from_str("camera.free"),
            version: 1,
            required: 0,
        });
    }
    if cfg!(feature = "world-shapes") {
        rows.push(AmmlUseRow {
            feature: AmmlStr::from_str("markers"),
            kind: AmmlUseRow::KIND_AREA,
            name: AmmlStr::from_str("world.shapes"),
            version: 1,
            required: 0,
        });
    }
    let rows: &'static [AmmlUseRow] = Box::leak(rows.into_boxed_slice());
    let uses: &'static AmmlUses =
        Box::leak(Box::new(AmmlUses { size: AmmlUses::SIZE_V1, version: 1, rows: rows.as_ptr(), count: rows.len() }));
    let block = AmmlModInfo {
        size: AmmlModInfo::SIZE_V3,
        api_version: 3,
        name: AmmlStr::from_str(NAME),
        namespace: AmmlStr::from_str(NAMESPACE),
        uses,
    };
    // SAFETY: the loader passes a writable v3 info block.
    unsafe { info.write(block) };
    if cfg!(feature = "require-in-load") {
        let _ = Fixture::listen(0, true);
    }
    #[cfg(feature = "provider-fails")]
    {
        let _ = Fixture::provide();
    }
    AmmlStatus::OK
}

#[unsafe(no_mangle)]
pub extern "C" fn amml_mod_enable() -> AmmlStatus {
    if let Some(api) = Fixture::api()
        && let Some(path) = Fixture::find_record_file(api)
    {
        let _ = RECORD.set(path);
    }
    Fixture::record("enable");
    let status = Fixture::listen(0, cfg!(feature = "require-tick"));
    Fixture::record(&format!("listen client.tick status={}", status.0));
    for (event, id) in EVENTS.iter().enumerate().skip(1) {
        let status = Fixture::listen(event, false);
        if !status.is_ok() {
            Fixture::record(&format!("listen {id} status={}", status.0));
        }
    }
    #[cfg(feature = "provider")]
    {
        let status = Fixture::provide();
        Fixture::record(&format!("provide status={}", status.0));
    }
    if cfg!(feature = "provider-fails") {
        return AmmlStatus::FAILED;
    }
    AmmlStatus::OK
}

#[unsafe(no_mangle)]
pub extern "C" fn amml_mod_disable() -> AmmlStatus {
    let status = Fixture::listen(1, false);
    Fixture::record(&format!("disable listen status={} game_alive={}", status.0, Fixture::game_alive()));
    AmmlStatus::OK
}

#[unsafe(no_mangle)]
pub extern "C" fn amml_mod_unload() -> AmmlStatus {
    Fixture::record("unload");
    AmmlStatus::OK
}
