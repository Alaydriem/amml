use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};

use amml_abi::{AmmlEvent, AmmlPhase, AmmlStr, ServerJoinStarting, WorldStorageClosed, WorldStorageOpened};
use amml_data::{AdapterCatalog, Availability, EntryId, NeedKind};
use amml_msvc::MsvcString;

use crate::{
    DirectNote, EventHub, ExternalJoinDetours, GameConnectionInfoLayout, GameJoinDetours, JoinNames, LoaderState,
    PlayerJoinWorldContextLayout, StorageDetours, StorageNote,
};

static ADAPTER: OnceLock<JoinEvents> = OnceLock::new();

/// The session facts of a join that `SessionAdapter` does not raise: `world.server_join_starting`,
/// `world.storage_opened` and `world.storage_closed`.
///
/// `ServerJoinStarting` is raised once for every server join, Before the game's call, from two hooks:
/// `MinecraftGame::joinMultiplayerWithAddress`, which the game's server joins reach (a join with its local flag set is a
/// local world and raises nothing; a call off the game thread raises nothing and logs a warning, because the event is a
/// game thread event), and `ClientInstance::startExternalNetworkWorld` (the direct join, on the thread of the call).
/// The game builds the info of a direct join again and may pass it on to `joinMultiplayerWithAddress`: the address and
/// port that `startExternalNetworkWorld` raised are kept, and the next `joinMultiplayerWithAddress` call with the same
/// port and the same address (its host or its unresolved URL) takes them and raises nothing. A world join drops them.
/// The host and port come from the connection info through the layout its `GameConnectionInfo.size` selects; the
/// unresolved URL and the external server name from the derived member offsets. A build with no layout for the size
/// gives an empty host and port 0; the other fields stay.
///
/// The storage events are raised from the `DBStorage` detours of `StorageDetours`: `WorldStorageOpened` After the
/// constructor with the folder copied from `DBStorage.mFullPath`, `WorldStorageClosed` After `checkShutdownDone` first
/// reports done for a storage that was opened. When the detour runs on the game thread and no earlier event waits, the
/// event is raised at once; otherwise it is kept and raised after the listeners of the next `FrameUpdate` (After), in
/// the order it was seen.
pub struct JoinEvents {
    state: &'static LoaderState,
    layout: Option<GameConnectionInfoLayout>,
    url_at: Option<usize>,
    name_at: Option<usize>,
    pending: Mutex<Vec<StorageNote>>,
    // The direct join raised last, until a `joinMultiplayerWithAddress` call takes it, a join starts or it expires.
    direct: Mutex<Option<DirectNote>>,
    // Frames seen (`flush` runs once per frame).
    frames: AtomicU64,
    unknown_size_logged: AtomicBool,
    off_thread_logged: AtomicBool,
}

impl JoinEvents {
    pub const SERVER_JOIN: &'static str = ServerJoinStarting::ID;
    pub const STORAGE_OPENED: &'static str = WorldStorageOpened::ID;
    pub const STORAGE_CLOSED: &'static str = WorldStorageClosed::ID;
    /// How many frames a direct join's address is kept for the game's `joinMultiplayerWithAddress` (30 s at 60 fps).
    pub const DIRECT_FRAMES: u64 = 1800;
    // `Social::ConnectionType::UnknownIP`: the only type the game builds again and passes on.
    const UNKNOWN_IP: u16 = 8;
    // Loader hooks run before every mod hook on the same function.
    const PRIORITY: i32 = i32::MAX;
    // The most storage events kept for a frame; a storage opens and closes once per world.
    const MAX_PENDING: usize = 64;

    /// Hooks the join and storage functions whose rows resolved, offers each row whose hooks stand and logs one
    /// `event offered` or `event not available` line per row. The storage hooks are shared with `WorldInfoAdapter` and
    /// installed once. Returns the rows. Works once per process: a second call hooks nothing and gives the catalog
    /// rows.
    pub fn install(state: &'static LoaderState) -> Vec<Availability> {
        let resolved = |name: &str| state.table().get(&EntryId(name.to_string())).is_some();
        let mut rows: Vec<Availability> = AdapterCatalog::availability(&resolved)
            .into_iter()
            .filter(|row| {
                row.kind == NeedKind::Event
                    && [Self::SERVER_JOIN, Self::STORAGE_OPENED, Self::STORAGE_CLOSED].contains(&row.id)
            })
            .collect();
        let value = |name: &str| state.resolve(name).ok().and_then(|value| usize::try_from(value).ok());
        let adapter = Self {
            state,
            layout: value("GameConnectionInfo.size").and_then(GameConnectionInfoLayout::for_size),
            url_at: value("GameConnectionInfo.mUnresolvedUrl"),
            name_at: value("GameConnectionInfo.mExternalServerName"),
            pending: Mutex::new(Vec::new()),
            direct: Mutex::new(None),
            frames: AtomicU64::new(0),
            unknown_size_logged: AtomicBool::new(false),
            off_thread_logged: AtomicBool::new(false),
        };
        if ADAPTER.set(adapter).is_err() {
            curia::warn!("the join events adapter is already installed");
            return rows;
        }
        if let Some(adapter) = ADAPTER.get() {
            adapter.note_layout(value("GameConnectionInfo.size"));
        }
        let mut storage: Option<Vec<&'static str>> = None;
        for row in &mut rows {
            if row.shape.is_some() {
                let refused: Vec<&'static str> = if row.id == Self::SERVER_JOIN {
                    [ExternalJoinDetours::hook(), GameJoinDetours::hook()]
                        .into_iter()
                        .filter(|hook| !hook.install(state, Self::PRIORITY))
                        .map(|hook| hook.entry)
                        .collect()
                } else {
                    storage.get_or_insert_with(|| StorageDetours::install(state, Self::PRIORITY)).clone()
                };
                if !refused.is_empty() {
                    row.shape = None;
                    row.missing = refused;
                }
            }
            match row.shape {
                Some(shape) => {
                    state.events().offer(row.id, row.version);
                    curia::info!("event offered", { event: row.id, shape: shape });
                }
                None => curia::info!("event not available", { event: row.id, missing: row.missing.join(", ") }),
            }
        }
        rows
    }

    // Logs once that the host and port of a server join are left out because the size has no layout.
    fn note_layout(&self, size: Option<usize>) {
        if self.layout.is_none() && size.is_some() && !self.unknown_size_logged.swap(true, Ordering::Relaxed) {
            curia::warn!("the host of a server join is left out", { size: size.unwrap_or(0) as u64 });
        }
    }

    /// Runs `body` on the installed adapter; nothing before `install`.
    pub(crate) fn with(body: impl FnOnce(&JoinEvents)) {
        if let Some(adapter) = ADAPTER.get() {
            body(adapter);
        }
    }

    fn hub(&self) -> &EventHub {
        self.state.events()
    }

    // The text of the `std::string` member at `offset` in the object at `base`; empty when the offset is unknown or the
    // text unreadable.
    fn member_text(base: usize, offset: Option<usize>) -> String {
        let Some(address) = offset.and_then(|offset| base.checked_add(offset)) else {
            return String::new();
        };
        // SAFETY: `base` is the connection info the game passed (a live `GameConnectionInfo`), and the offset is the
        // derived offset of one of its `std::string` members.
        unsafe { JoinNames::text(address as u64) }
    }

    /// `ServerJoinStarting` Before the game's `startExternalNetworkWorld`: `info` the by-value connection info, `name`
    /// the server name argument, `context` the by-value join context (all addresses the game passed). The address and
    /// port are kept for `game_join_starting`.
    pub(crate) fn server_join_starting(&self, info: usize, name: usize, context: usize) {
        if info == 0 || !self.hub().has_listeners(Self::SERVER_JOIN) {
            return;
        }
        let (host, url, mut server, port) = self.read_info(info);
        if server.is_empty() {
            // SAFETY: `name` is the address of the live `std::string` the game passed, or 0.
            server = unsafe { JoinNames::text(name as u64) };
        }
        // SAFETY: `info` is a live connection info; its type is the u16 at +0 on every layout.
        let kind = unsafe { std::ptr::with_exposed_provenance::<u16>(info).read_unaligned() };
        if kind == Self::UNKNOWN_IP {
            let address = if host.is_empty() { url.clone() } else { host.clone() };
            let note = DirectNote::new(address, port, self.frames.load(Ordering::Relaxed));
            *self.direct.lock().unwrap_or_else(std::sync::PoisonError::into_inner) = Some(note);
        }
        curia::info!("server join starting", { through: "startExternalNetworkWorld", port: port });
        self.raise_server_join(&host, &url, &server, port, context);
    }

    /// `ServerJoinStarting` Before the game's `joinMultiplayerWithAddress`: `info` the by-value connection info, `local`
    /// the whole register of the local flag (its low byte is the flag), `context` the by-value join context. Nothing for
    /// a local join, off the game thread, or for the direct join `server_join_starting` raised.
    pub(crate) fn game_join_starting(&self, info: usize, local: usize, context: usize) {
        if info == 0 || local & 0xFF != 0 || !self.hub().has_listeners(Self::SERVER_JOIN) {
            return;
        }
        if !self.hub().gate().on_game_thread() {
            if !self.off_thread_logged.swap(true, Ordering::Relaxed) {
                curia::warn!("a server join off the game thread raised no event", {
                    through: "joinMultiplayerWithAddress"
                });
            }
            return;
        }
        let (host, url, server, port) = self.read_info(info);
        let mut direct = self.direct.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        let now = self.frames.load(Ordering::Relaxed);
        if direct.as_ref().is_some_and(|note| note.expired(now, Self::DIRECT_FRAMES)) {
            *direct = None;
        }
        let same = direct.as_ref().is_some_and(|note| note.matches(&host, &url, port));
        if same {
            *direct = None;
            drop(direct);
            curia::info!("server join already raised", { through: "joinMultiplayerWithAddress", port: port });
            return;
        }
        drop(direct);
        curia::info!("server join starting", { through: "joinMultiplayerWithAddress", port: port });
        self.raise_server_join(&host, &url, &server, port, context);
    }

    /// After a world is joined, and after `onStartJoinGame` (a join started): a kept direct join is dropped.
    pub(crate) fn forget_direct(&self) {
        *self.direct.lock().unwrap_or_else(std::sync::PoisonError::into_inner) = None;
    }

    // The host, unresolved URL, external server name and port of the connection info at `info`.
    fn read_info(&self, info: usize) -> (String, String, String, u16) {
        // SAFETY: `info` is the address of a live connection info of the layout its size selects.
        let read = self.layout.map(|layout| unsafe { layout.read(info) });
        let host = read.as_ref().and_then(|read| read.host.clone()).unwrap_or_default();
        let port = read.as_ref().map_or(0, |read| u16::try_from(read.port).unwrap_or(0));
        (host, Self::member_text(info, self.url_at), Self::member_text(info, self.name_at), port)
    }

    fn raise_server_join(&self, host: &str, url: &str, server: &str, port: u16, context: usize) {
        let mut event = ServerJoinStarting {
            header: EventHub::header::<ServerJoinStarting>(AmmlPhase::BEFORE),
            port,
            // SAFETY: `context` is the address of the live join context the game passed, or 0.
            transfer: u8::from(unsafe { PlayerJoinWorldContextLayout::is_server_transfer(context) }),
            _reserved: [0; 5],
            host: AmmlStr::from_str(host),
            url: AmmlStr::from_str(url),
            name: AmmlStr::from_str(server),
        };
        self.hub().raise(&mut event, &self.hub().loader_context());
    }

    /// `WorldStorageOpened` after the `DBStorage` constructor made the storage at `storage`; its folder is read at
    /// `path_at` inside it.
    pub(crate) fn storage_opened(&self, storage: usize, path_at: usize) {
        if !self.hub().has_listeners(Self::STORAGE_OPENED) {
            return;
        }
        let Some(address) = storage.checked_add(path_at) else {
            return;
        };
        // SAFETY: `storage` is the `DBStorage` the game just constructed on this thread, and `path_at` the derived
        // offset of its `std::string mFullPath`.
        let path = unsafe { &*std::ptr::with_exposed_provenance::<MsvcString>(address) };
        let Ok(path) = path.to_str() else {
            curia::warn!("the folder of an opened storage is not UTF-8");
            return;
        };
        self.note(StorageNote::Opened(path.to_string()));
    }

    /// `WorldStorageClosed` after `checkShutdownDone` first reported done for a storage that was opened.
    pub(crate) fn storage_closed(&self) {
        if self.hub().has_listeners(Self::STORAGE_CLOSED) {
            self.note(StorageNote::Closed);
        }
    }

    // Raises `note` at once when this is the game thread and nothing waits before it; keeps it for the next frame
    // otherwise.
    fn note(&self, note: StorageNote) {
        let mut pending = self.pending.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        if pending.is_empty() && self.hub().gate().on_game_thread() {
            drop(pending);
            self.raise(&note);
        } else if pending.len() < Self::MAX_PENDING {
            pending.push(note);
        }
    }

    /// After the listeners of `FrameUpdate` (After): raises the storage events kept since the last frame.
    pub(crate) fn flush(&self) {
        self.frames.fetch_add(1, Ordering::Relaxed);
        let kept = std::mem::take(&mut *self.pending.lock().unwrap_or_else(std::sync::PoisonError::into_inner));
        for note in &kept {
            self.raise(note);
        }
    }

    fn raise(&self, note: &StorageNote) {
        let context = self.hub().loader_context();
        match note {
            StorageNote::Opened(path) => {
                let mut event = WorldStorageOpened {
                    header: EventHub::header::<WorldStorageOpened>(AmmlPhase::AFTER),
                    path: AmmlStr::from_str(path),
                };
                self.hub().raise(&mut event, &context);
            }
            StorageNote::Closed => {
                let mut event = WorldStorageClosed { header: EventHub::header::<WorldStorageClosed>(AmmlPhase::AFTER) };
                self.hub().raise(&mut event, &context);
            }
        }
    }
}
