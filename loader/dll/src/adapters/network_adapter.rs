use std::ffi::c_void;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Mutex, MutexGuard, OnceLock};
use std::time::Instant;

use amml_abi::{
    AmmlEvent, AmmlEventContext, AmmlImageView, AmmlPhase, AmmlPlayerListEntry, AmmlStatus, AmmlStr, AmmlThread,
    NETWORK_TEXT_NAME, NETWORK_TEXT_XUID, PlayerListUpdate, PlayerSkinChanged,
};
use amml_data::{AdapterCatalog, Availability, NeedKind};
use amml_msvc::MsvcHeap;
use amml_runtime::ChunkCacheSwitch;

use crate::{
    ChunkCacheDetours, ClientAdapter, EventHub, LoaderState, NetworkEvent, NetworkEventKind, NetworkListLayout,
    NetworkScope, ObjectRead, PacketDetours, SessionAdapter, SkinAnimatedLayout, SkinDetailsLayout, SkinGeometryLayout,
    SkinPacketLayout, SkinReader, ThreadNote, WireDetours,
};

static ADAPTER: OnceLock<NetworkAdapter> = OnceLock::new();
static STARTED: AtomicBool = AtomicBool::new(false);
static CUT_LOGGED: AtomicBool = AtomicBool::new(false);
static SKIPPED_LOGGED: AtomicBool = AtomicBool::new(false);
static NAME_LOGGED: AtomicBool = AtomicBool::new(false);
static XUID_LOGGED: AtomicBool = AtomicBool::new(false);
static CACHE_LOGGED: AtomicBool = AtomicBool::new(false);

/// The `network.*` events and areas: the player list and the player skin packets the game handles, and the reads of the
/// players and skins in them.
///
/// The packet detours (`PacketDetours`) call the original handler, then raise the event on the thread of the handler,
/// inside a `NetworkScope`: the event and the lent skin handles are valid only until the last listener returns, because
/// the game owns the packet and frees it after the handler. With no listener the detour does one relaxed load and nothing
/// else. The `network` table functions (`NetworkTable`) answer only on that thread and inside that scope.
///
/// The areas `network.protocol` (the derived protocol number of the build, any thread) and `network.chunk_cache` (a
/// per-mod wish for the `enabled` byte of the `ClientCacheStatusPacket` the client sends; `ChunkCacheSwitch` holds the
/// wishes). While any mod wishes it off, `ChunkCacheDetours` writes 0 into `mEnabled` of each such packet, found by its
/// vtable, before the game serializes it.
pub struct NetworkAdapter {
    state: &'static LoaderState,
    list: Option<NetworkListLayout>,
    packet: Option<SkinPacketLayout>,
    reader: SkinReader,
    list_thread: ThreadNote,
    skin_thread: ThreadNote,
    list_events: AtomicU64,
    slowest_nanos: AtomicU64,
    // The vtable of `ClientCacheStatusPacket` and the offset of its `mEnabled` byte; `Some` once the send is hooked.
    cache: Option<(usize, usize)>,
    protocol: Option<u32>,
    wishes: Mutex<ChunkCacheSwitch>,
}

impl NetworkAdapter {
    pub const PLAYER_LIST: &'static str = PlayerListUpdate::ID;
    pub const PLAYER_SKIN: &'static str = PlayerSkinChanged::ID;
    pub const SKIN: &'static str = "network.skin";
    pub const SKIN_ANIMATED: &'static str = "network.skin_animated";
    pub const SKIN_GEOMETRY: &'static str = "network.skin_geometry";
    pub const SKIN_DETAILS: &'static str = "network.skin_details";
    pub const CHUNK_CACHE: &'static str = "network.chunk_cache";
    pub const PROTOCOL: &'static str = "network.protocol";
    const AREA_VERSION: u32 = 1;
    // Loader hooks run before every mod hook on the same function.
    const PRIORITY: i32 = i32::MAX;
    // An event slower than this is logged when it is the slowest so far.
    const SLOW_NANOS: u64 = 1_000_000;

    /// Hooks the two packet handlers whose catalog rows resolved, offers the events and the areas, and logs one line per
    /// row. Works once per process: a second call hooks nothing and gives the catalog rows.
    pub fn install(state: &'static LoaderState) -> Vec<Availability> {
        let resolved = |name: &str| state.table().get(&amml_data::EntryId(name.to_string())).is_some();
        let mut rows: Vec<Availability> =
            AdapterCatalog::availability(&resolved).into_iter().filter(Self::is_own_row).collect();
        if STARTED.swap(true, Ordering::AcqRel) {
            curia::warn!("the network adapter is already installed");
            return rows;
        }
        let shaped = |id: &str| rows.iter().any(|row| row.id == id && row.shape.is_some());
        let field = |name: &str| state.resolve(name).ok().and_then(|value| usize::try_from(value).ok());
        let list = shaped(Self::PLAYER_LIST)
            .then(|| NetworkListLayout::from_state(state))
            .flatten()
            .filter(|_| PacketDetours::list_hook().install(state, Self::PRIORITY));
        let packet = shaped(Self::PLAYER_SKIN)
            .then(|| SkinPacketLayout::from_state(state))
            .flatten()
            .filter(|_| PacketDetours::skin_hook().install(state, Self::PRIORITY));
        let image = shaped(Self::SKIN).then(|| field("SerializedSkinImpl.mSkinImage")).flatten();
        let animated = shaped(Self::SKIN_ANIMATED)
            .then(|| {
                SkinAnimatedLayout::new(
                    field("SerializedSkinImpl.mSkinAnimatedImages")?,
                    field("AnimatedImageData.size")?,
                    field("AnimatedImageData.mType")?,
                    field("AnimatedImageData.mImage")?,
                )
            })
            .flatten();
        let geometry = shaped(Self::SKIN_GEOMETRY)
            .then(|| {
                Some(SkinGeometryLayout {
                    data: field("SerializedSkinImpl.mGeometryData")?,
                    name: field("SerializedSkinImpl.mDefaultGeometryName")?,
                    to_styled_string: field("JsonValue.toStyledString")?,
                })
            })
            .flatten();
        let details = shaped(Self::SKIN_DETAILS)
            .then(|| {
                Some(SkinDetailsLayout {
                    persona: field("SerializedSkinImpl.mIsPersona")?,
                    premium: field("SerializedSkinImpl.mIsPremium")?,
                    resource_patch: field("SerializedSkinImpl.mResourcePatch")?,
                })
            })
            .flatten();
        let reader = SkinReader::new(image, animated, geometry, details);
        let cache = shaped(Self::CHUNK_CACHE)
            .then(|| Some((field("ClientCacheStatusPacket.vtable")?, field("ClientCacheStatusPacket.mEnabled")?)))
            .flatten()
            .filter(|_| ChunkCacheDetours::send_hook().install(state, Self::PRIORITY));
        let protocol = shaped(Self::PROTOCOL)
            .then(|| field("SharedConstants.NetworkProtocolVersion").and_then(|value| u32::try_from(value).ok()))
            .flatten();
        let offered = |row: &Availability| match row.id {
            Self::PLAYER_LIST => list.is_some(),
            Self::PLAYER_SKIN => packet.is_some(),
            Self::SKIN => image.is_some(),
            Self::SKIN_ANIMATED => reader.has_animated(),
            Self::SKIN_GEOMETRY => reader.has_geometry(),
            Self::SKIN_DETAILS => reader.has_details(),
            Self::CHUNK_CACHE => cache.is_some(),
            Self::PROTOCOL => protocol.is_some(),
            _ => false,
        };
        for row in &mut rows {
            if row.shape.is_some() && !offered(row) {
                row.shape = None;
                row.missing = vec!["a hook or a layout"];
            }
        }
        let adapter = Self {
            state,
            list,
            packet,
            reader,
            list_thread: ThreadNote::new(),
            skin_thread: ThreadNote::new(),
            list_events: AtomicU64::new(0),
            slowest_nanos: AtomicU64::new(0),
            cache,
            protocol,
            wishes: Mutex::new(ChunkCacheSwitch::new()),
        };
        if ADAPTER.set(adapter).is_err() {
            curia::warn!("the network adapter is already installed");
            return rows;
        }
        for row in &rows {
            match (row.shape, row.kind) {
                (Some(shape), NeedKind::Area) => {
                    state.areas().offer(row.id, row.version);
                    curia::info!("area adapter", { area: row.id, shape: shape });
                }
                (Some(shape), NeedKind::Event) => {
                    state.events().offer(row.id, row.version);
                    curia::info!("event offered", { event: row.id, shape: shape });
                }
                (None, NeedKind::Area) => {
                    curia::info!("area not available", { area: row.id, missing: row.missing.join(", ") });
                }
                (None, NeedKind::Event) => {
                    curia::info!("event not available", { event: row.id, missing: row.missing.join(", ") });
                }
            }
        }
        rows
    }

    /// The installed adapter; `None` before `install`.
    pub fn get() -> Option<&'static NetworkAdapter> {
        ADAPTER.get()
    }

    /// Runs `body` on the installed adapter; `fallback` before `install`.
    pub(crate) fn with<R>(fallback: R, body: impl FnOnce(&NetworkAdapter) -> R) -> R {
        match ADAPTER.get() {
            Some(adapter) => body(adapter),
            None => fallback,
        }
    }

    /// The network protocol number of the build, from the derived `SharedConstants.NetworkProtocolVersion`. Any thread.
    /// `NOT_AVAILABLE` when the area is not offered.
    pub fn protocol_version(&self) -> Result<u32, AmmlStatus> {
        if !self.state.areas().is_available(Self::PROTOCOL, Self::AREA_VERSION) {
            return Err(AmmlStatus::NOT_AVAILABLE);
        }
        self.protocol.ok_or(AmmlStatus::NOT_AVAILABLE)
    }

    /// The mod at `owner` wishes the client's chunk cache `enabled` or off. `NOT_READY` before the first frame,
    /// `NOT_AVAILABLE` when the area is not offered, `REFUSED` off the game thread.
    pub fn set_client_chunk_cache(&self, owner: usize, enabled: bool) -> Result<(), AmmlStatus> {
        let gate = self.state.events().gate();
        gate.require_open()?;
        if !self.state.areas().is_available(Self::CHUNK_CACHE, Self::AREA_VERSION) {
            return Err(AmmlStatus::NOT_AVAILABLE);
        }
        if !gate.on_game_thread() {
            return Err(AmmlStatus::REFUSED);
        }
        let mut wishes = self.wishes();
        let was = wishes.disabled();
        wishes.set(owner, enabled);
        let now = wishes.disabled();
        WireDetours::set(WireDetours::CHUNK_CACHE, now);
        drop(wishes);
        if was != now {
            curia::info!("client chunk cache", { enabled: !now });
        }
        Ok(())
    }

    /// The mod at `owner` is disabled: its chunk cache wish ends.
    pub fn disable_owner(&self, owner: usize) {
        let mut wishes = self.wishes();
        if !wishes.clear(owner) {
            return;
        }
        let now = wishes.disabled();
        WireDetours::set(WireDetours::CHUNK_CACHE, now);
        drop(wishes);
        curia::info!("client chunk cache wish ended", { enabled: !now });
    }

    /// Before the game sends the packet at `packet`: when it is a `ClientCacheStatusPacket` (its first qword is the
    /// derived vtable), its `mEnabled` byte becomes 0.
    pub(crate) fn edit_cache_status(&self, packet: usize) {
        let Some((vtable, enabled_at)) = self.cache else { return };
        if packet == 0 {
            return;
        }
        // SAFETY: `packet` is the `Packet const&` the game passed to `NetworkSystem::send`, live for the call; every
        // packet starts with its vtable pointer.
        let head = unsafe { std::ptr::with_exposed_provenance::<usize>(packet).read_unaligned() };
        if head != vtable {
            return;
        }
        // SAFETY: the vtable is the one of `ClientCacheStatusPacket`, whose `mEnabled` byte lies at `enabled_at` (the
        // derived field entry) inside the 38h-byte packet. The game builds it on the sender's stack and reads it again
        // only to serialize it.
        unsafe { std::ptr::with_exposed_provenance_mut::<u8>(packet + enabled_at).write(0) };
        if !CACHE_LOGGED.swap(true, Ordering::AcqRel) {
            curia::info!("client chunk cache switched off for a status packet");
        }
    }

    fn wishes(&self) -> MutexGuard<'_, ChunkCacheSwitch> {
        self.wishes.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    fn is_own_row(row: &Availability) -> bool {
        [
            Self::PLAYER_LIST,
            Self::PLAYER_SKIN,
            Self::SKIN,
            Self::SKIN_ANIMATED,
            Self::SKIN_GEOMETRY,
            Self::SKIN_DETAILS,
            Self::CHUNK_CACHE,
            Self::PROTOCOL,
        ]
        .contains(&row.id)
    }

    fn heap() -> MsvcHeap {
        MsvcHeap::new(ClientAdapter::heap_new, ClientAdapter::heap_delete)
    }

    // The context of an event raised in a handler: the thread the handler runs on, and the primary client on the game
    // thread. Returns whether it is the game thread.
    fn context(&self) -> (AmmlEventContext, bool) {
        let hub = self.state.events();
        let game = hub.gate().on_game_thread();
        let (thread, client) = if game {
            (AmmlThread::GAME, SessionAdapter::primary_client().unwrap_or(0))
        } else {
            (AmmlThread::NETWORK, 0)
        };
        let context =
            AmmlEventContext { client: std::ptr::with_exposed_provenance::<c_void>(client), ..hub.context_on(thread) };
        (context, game)
    }

    /// The game handled the `PlayerListPacket` at `packet`: raises `PlayerListUpdate` on this thread while a mod listens
    /// to it.
    pub(crate) fn player_list(&self, packet: usize) {
        let Some(layout) = &self.list else { return };
        let hub = self.state.events();
        if !hub.has_listeners(PlayerListUpdate::ID) {
            return;
        }
        let started = Instant::now();
        // SAFETY: `packet` is the live `PlayerListPacket` the game passed to its handler, which has not returned.
        let Some((action, elements, cut)) = (unsafe { layout.elements(packet) }) else { return };
        if cut > 0 && !CUT_LOGGED.swap(true, Ordering::AcqRel) {
            curia::warn!("a player list is cut", { kept: elements.len(), left_out: cut });
        }
        if elements.is_empty() {
            if !SKIPPED_LOGGED.swap(true, Ordering::AcqRel) {
                curia::warn!("a player list has no entry of its first kind");
            }
            return;
        }
        let skins = elements
            .iter()
            .map(|element| {
                if action != PlayerListUpdate::ADD {
                    return 0;
                }
                let skin_ref = element + layout.skin;
                // SAFETY: an Add entry of the live packet holds a `SerializedSkinRef` at `skin`.
                unsafe { SkinReader::element(skin_ref) }.map_or(0, |_| skin_ref)
            })
            .collect();
        let count = elements.len() as u32;
        let (context, game) = self.context();
        if self.list_thread.changed(if game { AmmlThread::GAME } else { AmmlThread::NETWORK }) {
            curia::info!("player list thread", { game: game });
        }
        let kind = NetworkEventKind::List { elements };
        let scope = NetworkScope::enter(NetworkEvent::new(NetworkScope::next_serial(), Self::heap(), kind, skins));
        let mut event =
            PlayerListUpdate { header: EventHub::header::<PlayerListUpdate>(AmmlPhase::AFTER), action, count };
        hub.raise(&mut event, &context);
        drop(scope);
        self.note_time(started.elapsed().as_nanos(), count);
    }

    /// The game handled the `PlayerSkinPacket` at `packet`: raises `PlayerSkinChanged` on this thread while a mod listens
    /// to it.
    pub(crate) fn player_skin(&self, packet: usize) {
        let Some(layout) = &self.packet else { return };
        let hub = self.state.events();
        if !hub.has_listeners(PlayerSkinChanged::ID) {
            return;
        }
        // SAFETY: `packet` is the live `PlayerSkinPacket` the game passed to its handler, which has not returned.
        let Ok(uuid) = (unsafe { layout.uuid(packet) }) else { return };
        let skin = layout
            .skin_ref(packet)
            // SAFETY: the packet holds a `SerializedSkinRef` at `skin`.
            .filter(|skin_ref| unsafe { SkinReader::element(*skin_ref) }.is_some())
            .unwrap_or(0);
        let (context, game) = self.context();
        if self.skin_thread.changed(if game { AmmlThread::GAME } else { AmmlThread::NETWORK }) {
            curia::info!("player skin thread", { game: game });
        }
        let kind = NetworkEventKind::Skin { uuid };
        let scope = NetworkScope::enter(NetworkEvent::new(NetworkScope::next_serial(), Self::heap(), kind, vec![skin]));
        let mut event = PlayerSkinChanged { header: EventHub::header::<PlayerSkinChanged>(AmmlPhase::AFTER) };
        hub.raise(&mut event, &context);
        drop(scope);
    }

    // Logs the first player list event and each later one that is the slowest so far above `SLOW_NANOS`.
    fn note_time(&self, nanos: u128, entries: u32) {
        let nanos = u64::try_from(nanos).unwrap_or(u64::MAX);
        let previous = self.slowest_nanos.fetch_max(nanos, Ordering::Relaxed);
        let first = self.list_events.fetch_add(1, Ordering::Relaxed) == 0;
        if first || (nanos > previous && nanos >= Self::SLOW_NANOS) {
            curia::info!("player list event time", { nanos: nanos, entries: entries });
        }
    }

    /// Entry `index` of the running `PlayerListUpdate`.
    pub(crate) fn entry(&self, index: u32) -> Result<AmmlPlayerListEntry, AmmlStatus> {
        let layout = self.list.as_ref().ok_or(AmmlStatus::NOT_AVAILABLE)?;
        NetworkScope::with(|event| {
            let NetworkEventKind::List { elements } = event.kind() else { return Err(AmmlStatus::REFUSED) };
            let element = *elements.get(index as usize).ok_or(AmmlStatus::INVALID_ARGUMENT)?;
            // SAFETY: the element lies inside the vector of the live packet the running event holds.
            let uuid =
                unsafe { ObjectRead::value::<[u8; 16]>(element, layout.uuid) }.ok_or(AmmlStatus::NOT_AVAILABLE)?;
            // SAFETY: as above.
            let add = unsafe { ObjectRead::value::<u8>(element, layout.variant) } == Some(NetworkListLayout::ADD);
            let actor_id = if add {
                // SAFETY: an Add entry holds its actor id at `actor`.
                unsafe { ObjectRead::value::<i64>(element, layout.actor) }.ok_or(AmmlStatus::NOT_AVAILABLE)?
            } else {
                0
            };
            Ok(AmmlPlayerListEntry {
                uuid,
                has_actor: u8::from(add),
                _reserved: [0; 7],
                actor_id,
                skin: event.handle(index as usize),
            })
        })
        .unwrap_or(Err(AmmlStatus::REFUSED))
    }

    /// The text `field` of entry `index` of the running `PlayerListUpdate`.
    pub(crate) fn entry_text(&self, index: u32, field: u8) -> Result<AmmlStr, AmmlStatus> {
        let layout = self.list.as_ref().ok_or(AmmlStatus::NOT_AVAILABLE)?;
        let (at, logged) = match field {
            NETWORK_TEXT_NAME => (layout.name, &NAME_LOGGED),
            NETWORK_TEXT_XUID => (layout.xuid, &XUID_LOGGED),
            _ => return Err(AmmlStatus::INVALID_ARGUMENT),
        };
        NetworkScope::with(|event| {
            let NetworkEventKind::List { elements } = event.kind() else { return Err(AmmlStatus::REFUSED) };
            let element = *elements.get(index as usize).ok_or(AmmlStatus::INVALID_ARGUMENT)?;
            // SAFETY: the element lies inside the vector of the live packet the running event holds.
            if unsafe { ObjectRead::value::<u8>(element, layout.variant) } != Some(NetworkListLayout::ADD) {
                return Err(AmmlStatus::NOT_AVAILABLE);
            }
            // SAFETY: an Add entry holds a `std::string` at `at`.
            let text = unsafe { SkinReader::text(element, at) }?;
            if !logged.swap(true, Ordering::AcqRel) {
                // SAFETY: the view is of the live string just read.
                let digits = text.len > 0 && unsafe { std::slice::from_raw_parts(text.ptr, text.len) }.iter().all(u8::is_ascii_digit);
                curia::info!("first player list text read", { field: field, offset: at, length: text.len, digits: digits });
            }
            Ok(text)
        })
        .unwrap_or(Err(AmmlStatus::REFUSED))
    }

    /// The uuid and the skin handle of the running `PlayerSkinChanged`.
    pub(crate) fn skin_event(&self) -> Result<([u8; 16], u64), AmmlStatus> {
        NetworkScope::with(|event| match event.kind() {
            NetworkEventKind::Skin { uuid } => Ok((*uuid, event.handle(0))),
            NetworkEventKind::List { .. } => Err(AmmlStatus::REFUSED),
        })
        .unwrap_or(Err(AmmlStatus::REFUSED))
    }

    /// The main image of the skin `handle` names.
    pub(crate) fn skin_image(&self, handle: u64) -> Result<AmmlImageView, AmmlStatus> {
        // SAFETY: `with_skin` gives the live skin object of the running event.
        Self::with_skin(handle, |_, _, element| unsafe { self.reader.image(element) })
    }

    /// The number of animated images of the skin `handle` names.
    pub(crate) fn skin_animated_count(&self, handle: u64) -> Result<u32, AmmlStatus> {
        // SAFETY: `with_skin` gives the live skin object of the running event.
        Self::with_skin(handle, |_, _, element| unsafe { self.reader.animated_count(element) })
    }

    /// Animated image `index` of the skin `handle` names.
    pub(crate) fn skin_animated(&self, handle: u64, index: u32) -> Result<(u32, AmmlImageView), AmmlStatus> {
        // SAFETY: `with_skin` gives the live skin object of the running event.
        Self::with_skin(handle, |_, _, element| unsafe { self.reader.animated(element, index) })
    }

    /// Whether the skin `handle` names is a persona skin.
    pub(crate) fn skin_is_persona(&self, handle: u64) -> Result<bool, AmmlStatus> {
        // SAFETY: `with_skin` gives the live skin object of the running event.
        Self::with_skin(handle, |_, _, element| unsafe { self.reader.is_persona(element) })
    }

    /// Whether the skin `handle` names is a premium skin.
    pub(crate) fn skin_is_premium(&self, handle: u64) -> Result<bool, AmmlStatus> {
        // SAFETY: `with_skin` gives the live skin object of the running event.
        Self::with_skin(handle, |_, _, element| unsafe { self.reader.is_premium(element) })
    }

    /// The default geometry name of the skin `handle` names.
    pub(crate) fn skin_geometry_name(&self, handle: u64) -> Result<AmmlStr, AmmlStatus> {
        // SAFETY: `with_skin` gives the live skin object of the running event.
        Self::with_skin(handle, |_, _, element| unsafe { self.reader.geometry_name(element) })
    }

    /// The resource patch text of the skin `handle` names.
    pub(crate) fn skin_resource_patch(&self, handle: u64) -> Result<AmmlStr, AmmlStatus> {
        // SAFETY: `with_skin` gives the live skin object of the running event.
        Self::with_skin(handle, |_, _, element| unsafe { self.reader.resource_patch(element) })
    }

    /// The geometry JSON of the skin `handle` names. The loader builds the text once for each skin in an event, keeps it
    /// until the event ends and frees it on the game heap.
    pub(crate) fn skin_geometry_json(&self, handle: u64) -> Result<AmmlStr, AmmlStatus> {
        Self::with_skin(handle, |event, index, element| {
            if event.geometry(index).is_none() {
                let heap = event.heap();
                // SAFETY: `with_skin` gives the live skin object of the running event; `heap` is the game heap.
                let text = unsafe { self.reader.geometry_json(element, &heap) }.ok();
                event.keep_geometry(index, text);
            }
            let text = event.geometry(index).flatten().ok_or(AmmlStatus::NOT_AVAILABLE)?;
            AmmlStr::from_bytes(text.as_bytes()).map_err(|_| AmmlStatus::NOT_AVAILABLE)
        })
    }

    // Runs `body` with the running event, the position of the skin `handle` names and its live skin object. `REFUSED`
    // for a handle no running event of this thread gave, `NOT_AVAILABLE` for a skin whose object is null.
    fn with_skin<R>(
        handle: u64,
        body: impl FnOnce(&mut NetworkEvent, usize, usize) -> Result<R, AmmlStatus>,
    ) -> Result<R, AmmlStatus> {
        NetworkScope::with(|event| {
            let (index, skin_ref) = event.skin_of(handle).ok_or(AmmlStatus::REFUSED)?;
            // SAFETY: `skin_ref` is the address of a `SerializedSkinRef` inside the live packet of the running event.
            let element = unsafe { SkinReader::element(skin_ref) }.ok_or(AmmlStatus::NOT_AVAILABLE)?;
            body(event, index, element)
        })
        .unwrap_or(Err(AmmlStatus::REFUSED))
    }
}
