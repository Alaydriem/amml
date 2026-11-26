use std::ffi::c_void;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::{Mutex, OnceLock, PoisonError};

use amml_abi::{AmmlEvent, AmmlEventContext, AmmlPhase, AmmlRotation, AmmlVec3, PlayerMoved};
use amml_data::{AdapterCatalog, Availability, EntryId, NeedKind};

use crate::{ClientAdapter, EventHub, LoaderState, MovePlayerFields, PlayerMovedDetours, SessionAdapter};

static ADAPTER: OnceLock<PlayerMovedAdapter> = OnceLock::new();

/// `ActorRuntimeID Actor::getRuntimeID() const`: rcx the actor, rdx an 8-byte result buffer; rax gives the buffer.
type RuntimeIdFn = unsafe extern "C" fn(*mut c_void, *mut u64) -> *mut u64;

/// The `player.moved` event: raised After the game handled a `MovePlayerPacket` whose runtime id is the local
/// player's. The runtime id of the local player is read with `Actor::getRuntimeID` on the game thread (the packet's
/// id at +30h is the id the game's handler looks the actor up by: 26.63 0x6bd43a2 `mov rdx,[rsi+30h]` before the
/// level's runtime-entity lookup; 70.26 0x46b1c4a). A packet of any other actor and a packet handled off the game thread
/// raise nothing. A server sends the first `MovePlayerPacket` of the local player (Respawn) right after `StartGame`,
/// before the loader's world join: a local player packet that comes after `onStartJoinGame` and before `WorldJoined` is
/// kept (the latest one) and raised right after `WorldJoined`; a new `onStartJoinGame` drops it. The rotation is the
/// packet's pitch, then yaw, in degrees, the order and signs of `actor.read`. Each reason a packet raises nothing is
/// logged once (`player moved skipped`), and the first raise once (`player moved raised`).
pub struct PlayerMovedAdapter {
    state: &'static LoaderState,
    get_runtime_id: Option<usize>,
    // the local player packet seen before the world join, raised after it
    kept: Mutex<Option<MovePlayerFields>>,
    // the skip reasons already logged, one bit each
    skips_logged: AtomicU8,
    kept_logged: AtomicBool,
    raised_logged: AtomicBool,
}

impl PlayerMovedAdapter {
    pub const EVENT: &'static str = PlayerMoved::ID;
    // Loader hooks run before every mod hook on the same function.
    const PRIORITY: i32 = i32::MAX;
    // skip reasons, one bit each
    const SKIP_NO_PLAYER: u8 = 1;
    const SKIP_OTHER_ACTOR: u8 = 2;
    const SKIP_NO_ID: u8 = 4;

    /// Hooks the `MovePlayerPacket` handler slot when the row resolved, offers the event when the hook stands and logs
    /// one line. Works once per process: a second call hooks nothing and gives the catalog rows.
    pub fn install(state: &'static LoaderState) -> Vec<Availability> {
        let resolved = |name: &str| state.table().get(&EntryId(name.to_string())).is_some();
        let mut rows: Vec<Availability> = AdapterCatalog::availability(&resolved)
            .into_iter()
            .filter(|row| row.kind == NeedKind::Event && row.id == Self::EVENT)
            .collect();
        let adapter = Self {
            state,
            get_runtime_id: state
                .resolve("Actor.getRuntimeID")
                .ok()
                .and_then(|value| usize::try_from(value).ok())
                .filter(|address| *address != 0),
            kept: Mutex::new(None),
            skips_logged: AtomicU8::new(0),
            kept_logged: AtomicBool::new(false),
            raised_logged: AtomicBool::new(false),
        };
        if ADAPTER.set(adapter).is_err() {
            curia::warn!("the player moved adapter is already installed");
            return rows;
        }
        for row in &mut rows {
            if row.shape.is_some() && !PlayerMovedDetours::hook().install(state, Self::PRIORITY) {
                row.shape = None;
                row.missing = vec![PlayerMovedDetours::ENTRY];
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

    /// The installed adapter; `None` before `install`.
    pub fn get() -> Option<&'static PlayerMovedAdapter> {
        ADAPTER.get()
    }

    /// Runs `body` on the installed adapter; `fallback` before `install`.
    pub(crate) fn with<R>(fallback: R, body: impl FnOnce(&PlayerMovedAdapter) -> R) -> R {
        match ADAPTER.get() {
            Some(adapter) => body(adapter),
            None => fallback,
        }
    }

    /// After the game handled a `MovePlayerPacket` with `fields`: raises `PlayerMoved` when the packet is the local
    /// player's; keeps it for the world join when no world is joined yet.
    pub(crate) fn moved(&self, fields: MovePlayerFields) {
        let hub = self.state.events();
        if !hub.has_listeners(Self::EVENT) {
            return;
        }
        let Some(player) = ClientAdapter::get().and_then(ClientAdapter::primary_local_player_any_epoch) else {
            self.skipped(Self::SKIP_NO_PLAYER, "no local player on the game thread", fields.player_id, 0);
            return;
        };
        let Some(own) = self.runtime_id(player) else {
            self.skipped(Self::SKIP_NO_ID, "no runtime id read", fields.player_id, 0);
            return;
        };
        if own != fields.player_id {
            self.skipped(Self::SKIP_OTHER_ACTOR, "another actor", fields.player_id, own);
            return;
        }
        if hub.gate().epoch().0 == 0 {
            *self.kept.lock().unwrap_or_else(PoisonError::into_inner) = Some(fields);
            if !self.kept_logged.swap(true, Ordering::Relaxed) {
                curia::info!("player moved kept until the world join", { mode: fields.mode, runtime_id: own });
            }
            return;
        }
        self.raise(fields);
    }

    /// After `WorldJoined`: raises the local player packet kept before the join.
    pub(crate) fn world_joined(&self) {
        let kept = self.kept.lock().unwrap_or_else(PoisonError::into_inner).take();
        if let Some(fields) = kept {
            self.raise(fields);
        }
    }

    /// After `onStartJoinGame`: a packet kept for an earlier join is dropped, and the once-only lines log again for
    /// this join.
    pub(crate) fn join_started(&self) {
        *self.kept.lock().unwrap_or_else(PoisonError::into_inner) = None;
        self.skips_logged.store(0, Ordering::Relaxed);
        self.kept_logged.store(false, Ordering::Relaxed);
        self.raised_logged.store(false, Ordering::Relaxed);
    }

    /// When the player leaves, also before the world join: a kept packet is dropped, so a later join never raises it.
    pub(crate) fn leaving(&self) {
        *self.kept.lock().unwrap_or_else(PoisonError::into_inner) = None;
    }

    // Logs `player moved skipped` once for the reason `bit`.
    fn skipped(&self, bit: u8, reason: &'static str, packet_id: u64, local_id: u64) {
        if self.skips_logged.fetch_or(bit, Ordering::Relaxed) & bit == 0 {
            curia::info!("player moved skipped", { reason: reason, packet_id: packet_id, local_id: local_id });
        }
    }

    fn raise(&self, fields: MovePlayerFields) {
        let hub = self.state.events();
        if !self.raised_logged.swap(true, Ordering::Relaxed) {
            curia::info!("player moved raised", { mode: fields.mode, runtime_id: fields.player_id });
        }
        let [pitch, yaw] = fields.rot;
        let [x, y, z] = fields.pos;
        let mut event = PlayerMoved {
            header: EventHub::header::<PlayerMoved>(AmmlPhase::AFTER),
            pos: AmmlVec3 { x, y, z },
            rotation: AmmlRotation { yaw, pitch },
            head_yaw: fields.head_yaw,
            mode: fields.mode,
            _reserved: [0; 7],
        };
        // The listeners reach the local player through the context's client: the primary client, as for the player
        // action events.
        let client = SessionAdapter::primary_client().unwrap_or(0);
        let context = AmmlEventContext { client: std::ptr::with_exposed_provenance(client), ..hub.loader_context() };
        hub.raise_keeping(&mut event, &context, &|event: &mut PlayerMoved| {
            event.pos = AmmlVec3 { x, y, z };
            event.rotation = AmmlRotation { yaw, pitch };
            event.head_yaw = fields.head_yaw;
            event.mode = fields.mode;
            event._reserved = [0; 7];
        });
    }

    // The runtime id of the live actor `actor`, on the game thread.
    fn runtime_id(&self, actor: usize) -> Option<u64> {
        let function = self.get_runtime_id?;
        // SAFETY: the entry is `ActorRuntimeID Actor::getRuntimeID() const`, a member function that returns the
        // 8-byte class through a hidden pointer.
        let function = unsafe { std::mem::transmute::<usize, RuntimeIdFn>(function) };
        let mut id = 0u64;
        // SAFETY: called on the live local player, on the game thread; `id` is the buffer the callee fills.
        unsafe { function(std::ptr::with_exposed_provenance_mut(actor), &raw mut id) };
        Some(id)
    }
}
