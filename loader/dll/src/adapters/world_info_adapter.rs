use std::ffi::c_void;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, Ordering};

use amml_abi::{AmmlConnectionKind, AmmlStatus, AmmlThread, AmmlWorldInfo, AmmlWorldText};
use amml_data::{AdapterCatalog, Availability, EntryId, NeedKind};
use amml_msvc::{MemberCall, MsvcString};

use crate::{ClientAdapter, GameConnectionInfoLayout, LoaderState, SessionAdapter, StorageDetours, ThreadNote};

static ADAPTER: OnceLock<WorldInfoAdapter> = OnceLock::new();

/// `std::optional<Social::GameConnectionInfo> IClientInstance::getGameConnectionInfo() const`: rcx the client, rdx the
/// optional the game builds (a hidden pointer), the pointer in rax.
type ConnectionFn = unsafe extern "C" fn(*mut c_void, *mut u8) -> *mut u8;
/// `Social::GameConnectionInfo::~GameConnectionInfo()`: rcx the value; it does not free the value.
type DestroyFn = unsafe extern "C" fn(*mut u8);

// The connection read: the layout of the running build, the destructor and the getter slot.
struct ConnectionReader {
    layout: GameConnectionInfoLayout,
    destructor: usize,
    getter: usize,
}

/// The `client.world_info` area: the facts of the joined world, read on the game thread from the primary client.
///
/// A call needs the game gate open (`NOT_READY`), the area offered (`NOT_AVAILABLE`), the game thread, the primary
/// client and a joined world (`REFUSED`): the facts are valid from `WorldJoined` to `WorldLeft`. A fact the game does
/// not give, or gives as text that is not UTF-8, is left out of the answer; the others stay.
///
/// The storage folder comes from the live `DBStorage` that `StorageDetours` keeps. The path is copied inside
/// `StorageDetours::with_live`, so the storage cannot be destroyed under the copy. The connection is read from an
/// `std::optional<GameConnectionInfo>` that the loader builds on the game heap with the size of the running build; the
/// game's destructor runs on the value and the block is freed. A size with no known layout leaves the connection out.
pub struct WorldInfoAdapter {
    state: &'static LoaderState,
    // Slots of the ClientInstance vtable.
    get_level: Option<usize>,
    is_in_realm: Option<usize>,
    // Slots of the Level vtable (a `ClientLevel` is a `Level`).
    level_seed: Option<usize>,
    level_data: Option<usize>,
    // Offsets in LevelData and DBStorage.
    level_name_at: Option<usize>,
    commands_at: Option<usize>,
    full_path_at: Option<usize>,
    connection: Option<ConnectionReader>,
    thread: ThreadNote,
    unknown_size_logged: AtomicBool,
}

impl WorldInfoAdapter {
    pub const AREA: &'static str = "client.world_info";
    const AREA_VERSION: u32 = 1;
    // Loader hooks run before every mod hook on the same function.
    const PRIORITY: i32 = i32::MAX;
    // The byte after the value of a `std::optional` of it: the engaged flag, padded to the 8 of the value.
    const FLAG_PAD: usize = 8;
    // The most a read of the connection allocates; the sizes the loader knows are far below it.
    const MAX_VALUE: usize = 0x1000 - Self::FLAG_PAD;

    /// Hooks the two `DBStorage` functions and offers `client.world_info` when its catalog shape resolved and both
    /// hooks were installed; logs one `area adapter` or `area not available` line. Returns the row. Works once per
    /// process: a second call hooks nothing and gives the catalog row.
    pub fn install(state: &'static LoaderState) -> Vec<Availability> {
        let resolved = |name: &str| state.table().get(&EntryId(name.to_string())).is_some();
        let mut rows: Vec<Availability> = AdapterCatalog::availability(&resolved)
            .into_iter()
            .filter(|row| row.kind == NeedKind::Area && row.id == Self::AREA)
            .collect();
        let value = |name: &str| state.resolve(name).ok().and_then(|value| usize::try_from(value).ok());
        let adapter = Self {
            state,
            get_level: value("ClientInstance.getLevel"),
            is_in_realm: value("ClientInstance.isInRealm"),
            level_seed: value("Level.getLevelSeed64"),
            level_data: value("ClientLevel.getLevelData"),
            level_name_at: value("LevelData.mLevelName"),
            commands_at: value("LevelData.mCommandsEnabled"),
            full_path_at: value("DBStorage.mFullPath"),
            connection: Self::connection_reader(&value),
            thread: ThreadNote::new(),
            unknown_size_logged: AtomicBool::new(false),
        };
        if ADAPTER.set(adapter).is_err() {
            curia::warn!("the world info adapter is already installed");
            return rows;
        }
        if let Some(adapter) = ADAPTER.get() {
            adapter.note_connection_layout(&value);
        }
        for row in &mut rows {
            if row.shape.is_some() {
                let refused = StorageDetours::install(state, Self::PRIORITY);
                if !refused.is_empty() {
                    row.shape = None;
                    row.missing = refused;
                }
            }
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

    // The connection reader of the running build; `None` when an entry is missing or the size has no known layout.
    fn connection_reader(value: &impl Fn(&str) -> Option<usize>) -> Option<ConnectionReader> {
        let layout = GameConnectionInfoLayout::for_size(value("GameConnectionInfo.size")?)?;
        Some(ConnectionReader {
            layout,
            destructor: value("GameConnectionInfo.dtor").filter(|address| *address != 0)?,
            getter: value("ClientInstance.getGameConnectionInfo")?,
        })
    }

    // Logs once that the connection is read, or that it is left out because the size has no layout.
    fn note_connection_layout(&self, value: &impl Fn(&str) -> Option<usize>) {
        if self.connection.is_some() {
            return;
        }
        if value("GameConnectionInfo.size").is_some() && !self.unknown_size_logged.swap(true, Ordering::Relaxed) {
            curia::warn!("the connection of the world info is left out", {
                size: value("GameConnectionInfo.size").unwrap_or(0) as u64
            });
        }
    }

    /// The installed adapter; `None` before `install`.
    pub fn get() -> Option<&'static WorldInfoAdapter> {
        ADAPTER.get()
    }

    /// The facts of the joined world of `client`, which must be the primary client. `REFUSED` when no world is joined,
    /// off the game thread, or for another client.
    pub fn world_info(&self, client: *const c_void) -> Result<AmmlWorldInfo, AmmlStatus> {
        let client = self.enter(client)?;
        let mut info = AmmlWorldInfo::default();
        let level = self.level(client);
        if let Some(level) = level {
            if let Some(seed) = self.seed(level) {
                info.seed = seed;
                info.flags |= AmmlWorldInfo::HAS_SEED;
            }
            if let Some(data) = self.level_data_of(level) {
                if self.level_name(data).is_some() {
                    info.flags |= AmmlWorldInfo::HAS_LEVEL_NAME;
                }
                if let Some(enabled) = self.commands(data) {
                    info.flags |= AmmlWorldInfo::HAS_COMMANDS;
                    if enabled {
                        info.flags |= AmmlWorldInfo::COMMANDS;
                    }
                }
            }
        }
        if let Ok(realm) = ClientAdapter::call_bool(client, self.is_in_realm) {
            info.flags |= AmmlWorldInfo::HAS_REALM;
            if realm {
                info.flags |= AmmlWorldInfo::REALM;
            }
        }
        if self.storage_folder().is_some() {
            info.flags |= AmmlWorldInfo::HAS_STORAGE;
        }
        if let Some((kind, port, _)) = self.read_connection(client) {
            info.flags |= AmmlWorldInfo::HAS_CONNECTION;
            info.connection_kind = kind;
            info.port = port;
        }
        Ok(info)
    }

    /// The port and host of the connection info of the primary client, for a connection that starts before a world is
    /// joined. `None` off the game thread, with no primary client, or when `read_connection` gives nothing; it does not
    /// need the area offered or a joined world.
    pub(crate) fn primary_connection(&self) -> Option<(u16, String)> {
        if !self.state.events().gate().on_game_thread() {
            return None;
        }
        let client = SessionAdapter::primary_client()?;
        self.read_connection(client).map(|(_, port, host)| (port, host))
    }

    /// The text `field` of the joined world of `client`. `NOT_AVAILABLE` when the game gives no text for it or the text
    /// is not UTF-8; the other call errors are those of `world_info`.
    pub fn world_text(&self, client: *const c_void, field: AmmlWorldText) -> Result<String, AmmlStatus> {
        let client = self.enter(client)?;
        let text = match field {
            AmmlWorldText::STORAGE_FOLDER => self.storage_folder(),
            AmmlWorldText::LEVEL_NAME => {
                let data = self.level(client).and_then(|level| self.level_data_of(level));
                data.and_then(|data| self.level_name(data))
            }
            AmmlWorldText::HOST => self.read_connection(client).map(|(_, _, host)| host),
            _ => return Err(AmmlStatus::INVALID_ARGUMENT),
        };
        text.ok_or(AmmlStatus::NOT_AVAILABLE)
    }

    // The gate, the area, the thread, the primary client and a joined world of a call: `NOT_READY`, `NOT_AVAILABLE`,
    // `REFUSED`. The thread of the call is logged the first time and when it changes.
    fn enter(&self, client: *const c_void) -> Result<usize, AmmlStatus> {
        let gate = self.state.events().gate();
        gate.require_open()?;
        if !self.state.areas().is_available(Self::AREA, Self::AREA_VERSION) {
            return Err(AmmlStatus::NOT_AVAILABLE);
        }
        let on_game_thread = gate.on_game_thread();
        let thread = if on_game_thread { AmmlThread::GAME } else { AmmlThread::RENDER };
        if self.thread.changed(thread) {
            curia::info!("world thread", { hook: "ClientInstance.getLevel", game: on_game_thread });
        }
        if !on_game_thread {
            return Err(AmmlStatus::REFUSED);
        }
        let client = client.expose_provenance();
        match SessionAdapter::primary_client() {
            Some(primary) if primary == client && gate.epoch().0 != 0 => Ok(client),
            _ => Err(AmmlStatus::REFUSED),
        }
    }

    // The level of `client`; `None` for none or a missing slot.
    fn level(&self, client: usize) -> Option<usize> {
        ClientAdapter::call_pointer(client, self.get_level).ok().filter(|level| *level != 0)
    }

    // `Level::getLevelSeed64`, which gives its 8-byte value through a hidden pointer.
    fn seed(&self, level: usize) -> Option<u64> {
        let function = ClientAdapter::virtual_fn(level, self.level_seed)?;
        // SAFETY: the slot holds `LevelSeed64 ILevel::getLevelSeed64() const`, a member function that returns a struct
        // of one 64-bit value through a hidden pointer; `level` is the live level of the primary client, on the game
        // thread.
        Some(unsafe {
            MemberCall::call_sret0::<u64>(function as *const c_void, std::ptr::with_exposed_provenance_mut(level))
        })
    }

    // `Level::getLevelData`, which gives a reference.
    fn level_data_of(&self, level: usize) -> Option<usize> {
        ClientAdapter::call_pointer(level, self.level_data).ok().filter(|data| *data != 0)
    }

    // The level name of the `LevelData` at `data`; `None` when its text is not UTF-8.
    fn level_name(&self, data: usize) -> Option<String> {
        let address = data.checked_add(self.level_name_at?)?;
        // SAFETY: `data` is the live `LevelData` of the level (a non-null reference the game gave), and the entry is
        // the offset of its `std::string mLevelName`.
        let name = unsafe { &*std::ptr::with_exposed_provenance::<MsvcString>(address) };
        name.to_str().ok().map(str::to_string)
    }

    // The commands flag of the `LevelData` at `data`.
    fn commands(&self, data: usize) -> Option<bool> {
        let address = data.checked_add(self.commands_at?)?;
        // SAFETY: `data` is the live `LevelData` of the level, and the entry is the offset of its one-byte flag.
        Some(unsafe { std::ptr::with_exposed_provenance::<u8>(address).read() } != 0)
    }

    // The folder of the live `DBStorage`, copied inside the guard of the storage detours; `None` when there is no live
    // storage or its path is not UTF-8.
    fn storage_folder(&self) -> Option<String> {
        let at = self.full_path_at?;
        StorageDetours::with_live(|storage| {
            let address = storage.checked_add(at)?;
            // SAFETY: `storage` is the live `DBStorage` (the guard keeps its shutdown from going on), and the entry is
            // the offset of its `Core::PathBuffer<std::string> mFullPath`, which is an `std::string`.
            let path = unsafe { &*std::ptr::with_exposed_provenance::<MsvcString>(address) };
            path.to_str().ok().map(str::to_string)
        })
        .flatten()
    }

    // The connection of `client`: its kind, port and host. `None` when the build has no layout for it, the heap is not
    // available, the game holds no connection or the host text is not UTF-8.
    fn read_connection(&self, client: usize) -> Option<(AmmlConnectionKind, u16, String)> {
        let reader = self.connection.as_ref()?;
        let size = reader.layout.size();
        if size > Self::MAX_VALUE || !self.state.heap().is_available() {
            return None;
        }
        let bytes = size + Self::FLAG_PAD;
        let heap = self.state.heap();
        let buffer = heap.alloc(bytes);
        if buffer.is_null() {
            return None;
        }
        // SAFETY: the block is `bytes` writable bytes just allocated; zeroed, the optional is empty until the game
        // fills it.
        unsafe { std::ptr::write_bytes(buffer, 0, bytes) };
        // SAFETY: the slot holds `getGameConnectionInfo`, which fills the optional behind its hidden pointer (this in
        // rcx, the optional in rdx); the block holds the value of `size` bytes and the flag after it, as the getter
        // writes them (`GameConnectionInfo.size` is the offset of the flag in the game's own getter).
        let engaged = unsafe {
            let function = ClientAdapter::virtual_fn(client, Some(reader.getter));
            match function {
                Some(function) => {
                    let call = std::mem::transmute::<usize, ConnectionFn>(function);
                    call(std::ptr::with_exposed_provenance_mut(client), buffer);
                    buffer.add(size).read() != 0
                }
                None => false,
            }
        };
        let read = engaged.then(|| {
            // SAFETY: the optional is engaged, so the game constructed a `GameConnectionInfo` of this layout at the
            // start of the block.
            let read = unsafe { reader.layout.read(buffer.expose_provenance()) };
            // SAFETY: the entry is `GameConnectionInfo::~GameConnectionInfo()`, a non-deleting destructor with this in
            // rcx; it runs once on the value the game constructed and does not free the block.
            unsafe { std::mem::transmute::<usize, DestroyFn>(reader.destructor)(buffer) };
            read
        });
        // SAFETY: the block came from `heap.alloc(bytes)` and nothing reads it again.
        unsafe { heap.free(buffer, bytes) };
        let read = read?;
        let host = read.host?;
        let port = u16::try_from(read.port).unwrap_or(0);
        Some((Self::kind_of(read.connection_type, &host), port, host))
    }

    // The kind of a connection of the game's `Social::ConnectionType` value `connection_type` to `host`: 0 is a local
    // game, a value above 0 with a host is a connection to another machine, anything else is another kind.
    fn kind_of(connection_type: i16, host: &str) -> AmmlConnectionKind {
        match connection_type {
            0 => AmmlConnectionKind::LOCAL,
            1.. if !host.is_empty() => AmmlConnectionKind::REMOTE,
            _ => AmmlConnectionKind::OTHER,
        }
    }
}
