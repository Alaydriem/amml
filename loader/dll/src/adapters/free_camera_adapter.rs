use std::cell::Cell;
use std::ffi::c_void;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, MutexGuard, OnceLock};

use amml_abi::{AmmlEvent, AmmlPhase, AmmlPose, AmmlStatus, AmmlThread, AmmlUseRow, CameraInstructionReceived};
use amml_data::{AdapterCatalog, Availability, EntryId, NeedKind};
use amml_msvc::SharedPtr;
use amml_runtime::{CapturedHandler, FreeCameraRefusal, HandlerCapture, ProtocolValues};

use crate::{
    CameraInstructionLayout, CameraPresetsLayout, ClientAdapter, EventHub, FreeCameraCalls, FreeCameraHeld,
    HandlerDetours, LoaderState, ModPhase, NetworkIdentifierBuffer, PacketFactory, SessionAdapter,
    SetInstructionLayout, ShapesAdapter, ThreadNote,
};

static ADAPTER: OnceLock<FreeCameraAdapter> = OnceLock::new();

thread_local! {
    // The mod whose instruction the loader hands to the handler on this thread now; `None` outside a delivery.
    static DELIVERING: Cell<Option<usize>> = const { Cell::new(None) };
}

/// `void ClientNetworkHandler::handle(NetworkIdentifier const&, CameraInstructionPacket const&)`: rcx the handler, rdx
/// the identifier, r8 the packet.
type HandleFn = unsafe extern "C" fn(*mut c_void, *mut c_void, *mut c_void);

/// The `camera.free` area and the `camera.instruction_received` event.
///
/// The loader keeps the client network handler of the joined world (`HandlerCapture`): every hooked handler slot gives
/// its handler and `NetworkIdentifier` while an enabled mod declares `camera.free` or `world.shapes`. `set_free` makes
/// one `CameraInstructionPacket` per world through `MinecraftPackets::createPacket` (id 300), writes the set
/// instruction of the `minecraft:free` preset (found by name in `Level::getCameraPresets` of the local player's level
/// at each call) at the pose, and calls the captured handler's own `CameraInstructionPacket` slot with it, as if the
/// server sent it.
/// `release_free` writes and hands over the clear instruction. The packet is given back at the world leave and at the
/// holder's disable. One mod holds the free camera (`FreeCameraBook`); the holder's disable sends the clear while the
/// game runs and the disable runs on the game thread. A server instruction that sets or clears the camera ends the
/// hold. Each delivery passes an invalid `NetworkIdentifier` of the loader's own (`NetworkIdentifierBuffer`) as the
/// sender.
///
/// The camera handler detour raises `CameraInstructionReceived` Before the game's handler: origin `GAME` for the
/// server's packet, and for the loader's own packet `THIS_MOD` / `OTHER_MOD` (injected) by the mod whose call made it.
/// Every call runs on the game thread and on the thread of the captured call; any other thread is `REFUSED`. A call
/// from inside a delivery (a listener of the instruction of a free camera call) is `REFUSED`.
pub struct FreeCameraAdapter {
    state: &'static LoaderState,
    calls: Option<FreeCameraCalls>,
    instruction_at: Option<usize>,
    capture: HandlerCapture,
    hooks_in: bool,
    held: Mutex<FreeCameraHeld>,
    thread: ThreadNote,
    create_failed_logged: AtomicBool,
}

impl FreeCameraAdapter {
    pub const AREA: &'static str = "camera.free";
    /// The preset `set_free` asks the game to use.
    pub const FREE_PRESET: &'static str = "minecraft:free";
    /// The areas whose mods need the client network handler captured.
    pub const CAPTURE_AREAS: [&'static str; 2] = ["camera.free", "world.shapes"];
    const VERSION: u32 = 1;
    // Loader hooks run before every mod hook on the same function.
    const PRIORITY: i32 = i32::MAX;

    /// Hooks the `CameraInstructionPacket` handler slot when one of its rows or `world.shapes` resolved, and the
    /// `NetworkChunkPublisherUpdatePacket` handler slot when `camera.free` or `world.shapes` resolved; offers
    /// `camera.instruction_received` when the camera hook is in and the instruction entries fit their mirror, and
    /// `camera.free` when both hooks are in and every entry fits. Logs one `event offered` / `area adapter` or `... not
    /// available` line per row, and `camera layout mismatch` with the entry for a mirror that does not fit. Works once
    /// per process.
    pub fn install(state: &'static LoaderState) -> Vec<Availability> {
        let resolved = |name: &str| state.table().get(&EntryId(name.to_string())).is_some();
        let lookup = |name: &str| state.resolve(name).ok();
        let all = AdapterCatalog::availability(&resolved);
        let shapes_resolved =
            all.iter().any(|row| row.kind == NeedKind::Area && row.id == ShapesAdapter::AREA && row.shape.is_some());
        let mut rows: Vec<Availability> = all.into_iter().filter(Self::is_own_row).collect();
        let wants = |id: &str| rows.iter().any(|row| row.id == id && row.shape.is_some());
        let (event_resolved, area_resolved) = (wants(CameraInstructionReceived::ID), wants(Self::AREA));
        let [camera_hook, chunk_hook] = HandlerDetours::hooks();
        let camera_in =
            (event_resolved || area_resolved || shapes_resolved) && camera_hook.install(state, Self::PRIORITY);
        let chunk_in = (area_resolved || shapes_resolved) && camera_in && chunk_hook.install(state, Self::PRIORITY);
        let instruction_at = FreeCameraCalls::instruction_at(&lookup);
        let calls = FreeCameraCalls::from_lookup(&lookup);
        for row in &mut rows {
            if row.shape.is_none() {
                continue;
            }
            let (hooked, checked) = if row.id == Self::AREA {
                (camera_in && chunk_in, calls.map(|_| ()))
            } else {
                (camera_in, instruction_at.map(|_| ()))
            };
            if !hooked {
                row.shape = None;
                row.missing = vec![if camera_in { HandlerDetours::CHUNK } else { HandlerDetours::CAMERA }];
            } else if let Err(entry) = checked {
                curia::warn!("camera layout mismatch", { area: row.id, entry: entry });
                row.shape = None;
                row.missing = vec![entry];
            }
        }
        let offered = |id: &str| rows.iter().any(|row| row.id == id && row.shape.is_some());
        let adapter = Self {
            state,
            calls: calls.ok().filter(|_| offered(Self::AREA)),
            instruction_at: instruction_at.ok().filter(|_| offered(CameraInstructionReceived::ID)),
            capture: HandlerCapture::new(),
            hooks_in: camera_in && chunk_in,
            held: Mutex::new(FreeCameraHeld::new()),
            thread: ThreadNote::new(),
            create_failed_logged: AtomicBool::new(false),
        };
        if ADAPTER.set(adapter).is_err() {
            curia::warn!("the free camera adapter is already installed");
            return rows;
        }
        for row in &rows {
            match (row.kind, row.shape) {
                (NeedKind::Event, Some(shape)) => {
                    state.events().offer(row.id, row.version);
                    curia::info!("event offered", { event: row.id, shape: shape });
                }
                (NeedKind::Area, Some(shape)) => {
                    state.areas().offer(row.id, row.version);
                    curia::info!("area adapter", { area: row.id, shape: shape });
                }
                (NeedKind::Event, None) => {
                    curia::info!("event not available", { event: row.id, missing: row.missing.join(", ") });
                }
                (NeedKind::Area, None) => {
                    curia::info!("area not available", { area: row.id, missing: row.missing.join(", ") });
                }
            }
        }
        Self::refresh_capture(state);
        rows
    }

    fn is_own_row(row: &Availability) -> bool {
        match row.kind {
            NeedKind::Event => row.id == CameraInstructionReceived::ID,
            NeedKind::Area => row.id == Self::AREA,
        }
    }

    /// The installed adapter; `None` before `install`.
    pub fn get() -> Option<&'static FreeCameraAdapter> {
        ADAPTER.get()
    }

    /// Runs `body` on the installed adapter; `fallback` before `install`.
    pub(crate) fn with<R>(fallback: R, body: impl FnOnce(&FreeCameraAdapter) -> R) -> R {
        match ADAPTER.get() {
            Some(adapter) => body(adapter),
            None => fallback,
        }
    }

    /// Whether both handler slots that capture the client network handler are hooked.
    pub fn capture_hooks_in(&self) -> bool {
        self.hooks_in
    }

    /// The client network handler held now, from any thread; `None` before the first capture of the world.
    pub fn captured(&self) -> Option<CapturedHandler> {
        self.capture.captured()
    }

    /// The captured handler of this world for a game call on this thread: `NOT_READY` before the first capture,
    /// `REFUSED` on another thread than the captured call or inside a free camera delivery.
    pub fn captured_here(&self) -> Result<CapturedHandler, AmmlStatus> {
        self.handler()
    }

    /// Sets the capture flag of the handler detours while an enabled mod of `state` declares one of `CAPTURE_AREAS`
    /// in its `uses` block; clears it otherwise. Runs after each enable and each disable.
    pub fn refresh_capture(state: &LoaderState) {
        let wanted = state.lock_mods().iter().any(|record| {
            record.phase == ModPhase::Enabled
                && record
                    .uses
                    .iter()
                    .any(|row| row.kind == AmmlUseRow::KIND_AREA && Self::CAPTURE_AREAS.contains(&row.name.as_str()))
        });
        HandlerDetours::set(HandlerDetours::CAPTURE, wanted);
    }

    // The state holds plain data and a packet use, so a lock poisoned by a panic is recovered.
    fn lock(&self) -> MutexGuard<'_, FreeCameraHeld> {
        self.held.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    // The `HELD` flag of the handler detours follows whether a mod holds the free camera.
    fn note_owner(held: &FreeCameraHeld) {
        HandlerDetours::set(HandlerDetours::HELD, held.book.owner().is_some());
    }

    fn hub(&self) -> &EventHub {
        self.state.events()
    }

    /// A hooked handler slot (`entry`) ran on `handler` with the identifier at `id`, while the capture flag is set.
    /// Logs the thread of the handler calls once and when it changes, and each new capture.
    pub(crate) fn capture(&self, entry: &'static str, handler: usize, id: usize) {
        let game = self.hub().gate().on_game_thread();
        if self.thread.changed(if game { AmmlThread::GAME } else { AmmlThread::NETWORK }) {
            curia::info!("world thread", { hook: entry, game: game });
        }
        if self.capture.capture(handler, id)
            && let Some(captured) = self.capture.captured()
        {
            curia::info!("captured client packet handler", { generation: captured.generation, game: game });
        }
    }

    /// The camera handler detour ran with `wanted` flags on `handler`, `id` and `packet`: captures the handler with
    /// `CAPTURE`, raises `CameraInstructionReceived` with `LISTENER`, and with `HELD` ends the hold of the free camera
    /// when the instruction is the server's and sets or clears the camera.
    pub(crate) fn instruction_seen(&self, wanted: u32, handler: usize, id: usize, packet: usize) {
        if wanted & HandlerDetours::CAPTURE != 0 {
            self.capture(HandlerDetours::CAMERA, handler, id);
        }
        if wanted & HandlerDetours::LISTENER != 0 {
            self.raise(packet);
        }
        if wanted & HandlerDetours::HELD != 0 && DELIVERING.with(Cell::get).is_none() {
            self.server_seen(packet);
        }
    }

    // A server instruction in `packet`: when it sets or clears the camera, the server took the camera and no mod holds
    // the free camera any more.
    fn server_seen(&self, packet: usize) {
        let Some(at) = self.instruction_at.or(self.calls.map(|calls| calls.instruction_at)) else { return };
        let Some(address) = packet.checked_add(at).filter(|_| packet != 0) else { return };
        // SAFETY: `packet` is the live `CameraInstructionPacket` the game's handler is called with, and its instruction
        // at `at` fits `CameraInstructionLayout` (checked at install).
        if !unsafe { CameraInstructionLayout::takes_camera(std::ptr::with_exposed_provenance(address)) } {
            return;
        }
        let mut held = self.lock();
        if let Some(owner) = held.book.owner() {
            held.book.server_instruction();
            curia::info!("the server took the free camera", { owner: owner });
        }
        Self::note_owner(&held);
    }

    // Raises `CameraInstructionReceived` for the instruction of `packet`: injected by the mod of the running delivery
    // on this thread, else from the game.
    fn raise(&self, packet: usize) {
        let Some(at) = self.instruction_at else { return };
        if packet == 0 {
            return;
        }
        let Some(address) = packet.checked_add(at) else { return };
        let hub = self.hub();
        // SAFETY: `packet` is the live `CameraInstructionPacket` the game's handler is called with, and its instruction
        // at `at` fits `CameraInstructionLayout` (checked at install).
        let kind = unsafe { CameraInstructionLayout::kind(std::ptr::with_exposed_provenance(address)) };
        let mut event = CameraInstructionReceived {
            header: EventHub::header::<CameraInstructionReceived>(AmmlPhase::BEFORE),
            kind,
            _reserved: [0; 7],
        };
        let game = hub.gate().on_game_thread();
        let mut context = hub.context_on(if game { AmmlThread::GAME } else { AmmlThread::NETWORK });
        if game && let Some(client) = SessionAdapter::primary_client() {
            context.client = std::ptr::with_exposed_provenance(client);
        }
        let keep = |event: &mut CameraInstructionReceived| {
            event.kind = kind;
            event._reserved = [0; 7];
        };
        match DELIVERING.with(Cell::get) {
            Some(owner) => hub.raise_injected_keeping(&mut event, &context, owner, &keep),
            None => hub.raise_keeping(&mut event, &context, &keep),
        };
    }

    // The area, the game gate, the game thread and a joined world of a call: `NOT_AVAILABLE`, `NOT_READY`, `REFUSED`.
    fn enter(&self) -> Result<&FreeCameraCalls, AmmlStatus> {
        if !self.state.areas().is_available(Self::AREA, Self::VERSION) {
            return Err(AmmlStatus::NOT_AVAILABLE);
        }
        let calls = self.calls.as_ref().ok_or(AmmlStatus::NOT_AVAILABLE)?;
        let gate = self.hub().gate();
        gate.require_open()?;
        if !gate.on_game_thread() {
            return Err(AmmlStatus::REFUSED);
        }
        if gate.epoch().0 == 0 {
            return Err(AmmlStatus::NOT_READY);
        }
        Ok(calls)
    }

    // The captured handler of this world, on its own thread, outside a delivery: `NOT_READY` before the first capture,
    // `REFUSED` on another thread or inside a delivery.
    fn handler(&self) -> Result<CapturedHandler, AmmlStatus> {
        if DELIVERING.with(Cell::get).is_some() {
            return Err(AmmlStatus::REFUSED);
        }
        let captured = self.capture.captured().ok_or(AmmlStatus::NOT_READY)?;
        if captured.thread != std::thread::current().id() {
            return Err(AmmlStatus::REFUSED);
        }
        Ok(captured)
    }

    // The index of the free preset in the level of the local player: `Ok(None)` when the list has no free preset,
    // `NOT_READY` when there is no local player or level.
    fn free_index(&self, calls: &FreeCameraCalls) -> Result<Option<u32>, AmmlStatus> {
        let player = ClientAdapter::with(None, ClientAdapter::primary_local_player).ok_or(AmmlStatus::NOT_READY)?;
        let level_at = player.checked_add(calls.level_at).ok_or(AmmlStatus::NOT_READY)?;
        // SAFETY: `player` is the live local player of the primary client, on the game thread; `Actor.mLevel` is the
        // offset of its `ILevel*`.
        let level = unsafe { std::ptr::with_exposed_provenance::<usize>(level_at).read() };
        if level == 0 {
            return Err(AmmlStatus::NOT_READY);
        }
        let presets =
            ClientAdapter::call_pointer(level, Some(calls.presets_slot)).map_err(|_| AmmlStatus::NOT_READY)?;
        if presets == 0 {
            return Err(AmmlStatus::NOT_READY);
        }
        // SAFETY: `getCameraPresets` gives a reference to the level's live `CameraPresets`, laid out as the mirror's
        // head; its presets fit `CameraPresetLayout` (`CameraPreset.size`, checked at install). Read on the game thread.
        let presets = unsafe { &*std::ptr::with_exposed_provenance::<CameraPresetsLayout>(presets) };
        let index =
            presets.presets.as_slice().iter().position(|preset| preset.name.as_bytes() == Self::FREE_PRESET.as_bytes());
        Ok(index.and_then(|index| u32::try_from(index).ok()))
    }

    /// Whether the level of the joined world has the `minecraft:free` preset. `NOT_AVAILABLE` without the area,
    /// `NOT_READY` before a world is joined or without a local player, `REFUSED` off the game thread.
    pub fn free_available(&self) -> Result<bool, AmmlStatus> {
        let calls = self.enter()?;
        Ok(self.free_index(calls)?.is_some())
    }

    /// The mod at `owner` sets the free camera at `pose`. The errors are those of `AmmlCameraTable::set_free`.
    pub fn set_free(&self, owner: usize, pose: AmmlPose) -> Result<(), AmmlStatus> {
        let calls = self.enter()?;
        if !pose.is_finite() {
            return Err(AmmlStatus::INVALID_ARGUMENT);
        }
        let captured = self.handler()?;
        let index = self.free_index(calls)?.ok_or(AmmlStatus::NOT_AVAILABLE)?;
        let mut held = self.lock();
        if held.book.owner().is_some_and(|holder| holder != owner) {
            return Err(AmmlStatus::REFUSED);
        }
        let position = [pose.position.x, pose.position.y, pose.position.z];
        let set = SetInstructionLayout::new(index, position, pose.rotation.pitch, pose.rotation.yaw);
        // SAFETY: the instruction of the packet the factory made for these entries, which fit the mirror.
        self.deliver(calls, &mut held, &captured, owner, |instruction| unsafe {
            CameraInstructionLayout::write_set(instruction, set);
        })?;
        let taken = held.book.set(owner).map_err(|FreeCameraRefusal::Taken| AmmlStatus::REFUSED);
        Self::note_owner(&held);
        taken
    }

    /// The mod at `owner` releases the free camera. The errors are those of `AmmlCameraTable::release_free`.
    pub fn release_free(&self, owner: usize) -> Result<(), AmmlStatus> {
        let calls = self.enter()?;
        let captured = self.handler()?;
        let mut held = self.lock();
        match held.book.owner() {
            None => return Ok(()),
            Some(holder) if holder != owner => return Err(AmmlStatus::REFUSED),
            Some(_) => {}
        }
        // SAFETY: as in `set_free`.
        self.deliver(calls, &mut held, &captured, owner, |instruction| unsafe {
            CameraInstructionLayout::write_clear(instruction);
        })?;
        let released = held.book.release(owner).map(|_| ()).map_err(|FreeCameraRefusal::Taken| AmmlStatus::REFUSED);
        Self::note_owner(&held);
        released
    }

    // Makes the packet of this world when none is kept, writes its instruction with `write` and calls the captured
    // handler's camera slot with it, marked as the delivery of `owner`. `FAILED` when the game makes no packet or the
    // handler has no slot. Runs under the lock, so the packet cannot be given back during the call.
    fn deliver(
        &self,
        calls: &FreeCameraCalls,
        held: &mut FreeCameraHeld,
        captured: &CapturedHandler,
        owner: usize,
        write: impl FnOnce(*mut CameraInstructionLayout),
    ) -> Result<(), AmmlStatus> {
        if held.packet.is_none() {
            held.packet = self.make_packet(calls);
        }
        let packet = held
            .packet
            .as_ref()
            .and_then(|packet| packet.get())
            .map(|packet| std::ptr::from_ref(packet).expose_provenance())
            .ok_or(AmmlStatus::FAILED)?;
        let handle = ClientAdapter::virtual_fn(captured.handler, Some(calls.camera_slot)).ok_or(AmmlStatus::FAILED)?;
        let instruction = packet.checked_add(calls.instruction_at).ok_or(AmmlStatus::FAILED)?;
        write(std::ptr::with_exposed_provenance_mut(instruction));
        // SAFETY: the slot holds the handler's `handle(NetworkIdentifier const&, CameraInstructionPacket const&)`.
        let handle = unsafe { std::mem::transmute::<usize, HandleFn>(handle) };
        let mut identifier = NetworkIdentifierBuffer::new();
        let previous = DELIVERING.with(|cell| cell.replace(Some(owner)));
        // SAFETY: the handler is the one of the last handler call of this world, on its thread (the game thread); the
        // identifier is the loader's invalid identifier of the identifier's size; the packet is live under the lock. The
        // handler takes the packet by reference and does not keep it.
        unsafe {
            handle(
                std::ptr::with_exposed_provenance_mut(captured.handler),
                identifier.as_mut_ptr(),
                std::ptr::with_exposed_provenance_mut(packet),
            );
        }
        DELIVERING.with(|cell| cell.set(previous));
        Ok(())
    }

    // A `CameraInstructionPacket` from the game's factory, held with one use; `None` when the factory makes none.
    fn make_packet(&self, calls: &FreeCameraCalls) -> Option<SharedPtr<c_void>> {
        let id = ProtocolValues::CAMERA_INSTRUCTION_ID;
        // SAFETY: on the game thread (checked by every caller), with the resolved factory and a packet id the game knows.
        let made = unsafe { PacketFactory::new(calls.create_packet).make(id) };
        if made.is_none() && !self.create_failed_logged.swap(true, Ordering::AcqRel) {
            curia::warn!("the game made no camera instruction packet", { id: id });
        }
        made
    }

    /// The mod at `owner` is disabled: when it held the free camera, the clear goes to the captured handler (only
    /// while the game runs, on the game thread, in a joined world) and the packet goes back to the game. Once the game
    /// stopped (a shutdown disable) the packet is left to the ending process: no game code runs for it. Then the
    /// capture flag follows the enabled mods.
    pub fn disable_owner(&self, owner: usize) {
        let gate = self.hub().gate();
        let live = gate.is_open() && gate.on_game_thread() && gate.epoch().0 != 0;
        let mut held = self.lock();
        let release = held.book.disable(owner, live);
        Self::note_owner(&held);
        if release.clear {
            match (self.calls.as_ref(), self.handler()) {
                (Some(calls), Ok(captured)) => {
                    // SAFETY: as in `set_free`.
                    let sent = self.deliver(calls, &mut held, &captured, owner, |instruction| unsafe {
                        CameraInstructionLayout::write_clear(instruction);
                    });
                    if sent.is_err() {
                        curia::warn!("the clear of a disabled mod's free camera was not delivered");
                    }
                }
                (_, Err(status)) => {
                    curia::warn!(
                        "the clear of a disabled mod's free camera could not reach the client packet handler",
                        { status: status.0 }
                    );
                }
                (None, Ok(_)) => {}
            }
        }
        if release.drop_packet && gate.is_open() {
            held.drop_packet();
        } else if release.drop_packet {
            held.forget_packet();
        }
        drop(held);
        Self::refresh_capture(self.state);
    }

    /// Before `WorldLeft`: the holder and the handler of the left world are forgotten and the packet goes back to the
    /// game. Nothing is sent: the handler belongs to the world being left.
    pub(crate) fn world_left(&self) {
        let mut held = self.lock();
        held.book.world_left();
        Self::note_owner(&held);
        held.drop_packet();
        drop(held);
        self.capture.world_left();
    }

    /// A join starts: the handler of an earlier world is forgotten, and the handler slots capture again.
    pub(crate) fn join_started(&self) {
        self.capture.join_started();
    }
}
