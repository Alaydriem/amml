use std::cell::Cell;
use std::ffi::c_void;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, MutexGuard, OnceLock};
use std::time::Instant;

use amml_abi::{AmmlApplyStats, AmmlShapeOp, AmmlStatus};
use amml_data::{AdapterCatalog, Availability, EntryId, NeedKind};
use amml_msvc::{MsvcHeap, MsvcVec};
use amml_runtime::{CapturedHandler, ProtocolValues, ShapeBook, ShapeKind, ShapeRefusal, ShapeRequest, ShapeSend};

use crate::{
    ClientAdapter, FreeCameraAdapter, HandlerDetours, LoaderState, NetworkIdentifierBuffer, PacketFactory,
    PrimitiveShapeLayout, ShapeCalls, ShapesHeld,
};

static ADAPTER: OnceLock<ShapesAdapter> = OnceLock::new();

thread_local! {
    // True while this thread hands a shapes packet to the game's handler.
    static DELIVERING: Cell<bool> = const { Cell::new(false) };
}

/// `void ClientNetworkHandler::handle(NetworkIdentifier const&, PrimitiveShapesPacket const&)`: rcx the handler, rdx
/// the identifier, r8 the packet.
type HandleFn = unsafe extern "C" fn(*mut c_void, *mut c_void, *mut c_void);

/// The `world.shapes` area: lines, boxes and arrows the game draws in the world, as a server's script shapes.
///
/// The loader makes one `PrimitiveShapesPacket` per world through `MinecraftPackets::createPacket` (id 328), and for
/// each batch fills its shape vector with one `PrimitiveShapeLayout` payload per change in a game-heap block that grows
/// to the largest batch, calls the slot of the client network handler that `FreeCameraAdapter` captured (with an invalid
/// identifier of the loader's own), and empties the vector again. The mods' shape ids map to game ids through
/// `ShapeBook`. At a mod's disable (the game runs, game thread, joined world) the loader sends the removes of that mod's
/// live shapes; at the world leave it forgets them and gives the packet back. Every call that reaches the game runs on
/// the game thread and on the thread of the captured handler call; any other thread is `REFUSED`.
pub struct ShapesAdapter {
    state: &'static LoaderState,
    calls: Option<ShapeCalls>,
    held: Mutex<ShapesHeld>,
    create_failed_logged: AtomicBool,
}

impl ShapesAdapter {
    pub const AREA: &'static str = "world.shapes";
    const VERSION: u32 = 1;

    /// Offers `world.shapes` when its entries resolved, `FreeCameraAdapter` hooked the handler slots that capture the
    /// client network handler, and the payload size fits `PrimitiveShapeLayout`. Logs one `area adapter` or `area not
    /// available` line, and `shape layout mismatch` with the entry for a payload that does not fit. Works once per
    /// process; install it after `FreeCameraAdapter`.
    pub fn install(state: &'static LoaderState) -> Vec<Availability> {
        let resolved = |name: &str| state.table().get(&EntryId(name.to_string())).is_some();
        let lookup = |name: &str| state.resolve(name).ok();
        let mut rows: Vec<Availability> = AdapterCatalog::availability(&resolved)
            .into_iter()
            .filter(|row| row.kind == NeedKind::Area && row.id == Self::AREA)
            .collect();
        let hooked = FreeCameraAdapter::get().is_some_and(FreeCameraAdapter::capture_hooks_in);
        let calls = ShapeCalls::from_lookup(&lookup);
        for row in &mut rows {
            if row.shape.is_none() {
                continue;
            }
            if !hooked {
                row.shape = None;
                row.missing = vec![HandlerDetours::CHUNK];
            } else if let Err(entry) = calls {
                curia::warn!("shape layout mismatch", { area: row.id, entry: entry });
                row.shape = None;
                row.missing = vec![entry];
            }
        }
        let offered = rows.iter().any(|row| row.shape.is_some());
        let adapter = Self {
            state,
            calls: calls.ok().filter(|_| offered),
            held: Mutex::new(ShapesHeld::new()),
            create_failed_logged: AtomicBool::new(false),
        };
        if ADAPTER.set(adapter).is_err() {
            curia::warn!("the shapes adapter is already installed");
            return rows;
        }
        for row in &rows {
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

    /// The installed adapter; `None` before `install`.
    pub fn get() -> Option<&'static ShapesAdapter> {
        ADAPTER.get()
    }

    /// Runs `body` on the installed adapter; `fallback` before `install`.
    pub(crate) fn with<R>(fallback: R, body: impl FnOnce(&ShapesAdapter) -> R) -> R {
        match ADAPTER.get() {
            Some(adapter) => body(adapter),
            None => fallback,
        }
    }

    /// The most ops one `apply` takes.
    pub fn max_batch() -> u32 {
        ShapeBook::MAX_BATCH as u32
    }

    // The holder data and a packet use are plain data, so a lock poisoned by a panic is recovered.
    fn lock(&self) -> MutexGuard<'_, ShapesHeld> {
        self.held.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn available(&self) -> Result<&ShapeCalls, AmmlStatus> {
        if !self.state.areas().is_available(Self::AREA, Self::VERSION) {
            return Err(AmmlStatus::NOT_AVAILABLE);
        }
        self.calls.as_ref().ok_or(AmmlStatus::NOT_AVAILABLE)
    }

    /// The generation of the captured client network handler. `NOT_AVAILABLE` without the area, `NOT_READY` before
    /// the handler is captured in this world. Allowed from any thread.
    pub fn target_epoch(&self) -> Result<u32, AmmlStatus> {
        self.available()?;
        let capture = FreeCameraAdapter::get().ok_or(AmmlStatus::NOT_AVAILABLE)?;
        capture.captured().map(|captured| captured.generation).ok_or(AmmlStatus::NOT_READY)
    }

    /// The mod at `owner` applies `ops` as shapes of `dimension`. The errors are those of `AmmlWorldTable::shapes_apply`.
    pub fn apply(&self, owner: usize, dimension: i32, ops: &[AmmlShapeOp]) -> Result<AmmlApplyStats, AmmlStatus> {
        let started = Instant::now();
        let calls = self.available()?;
        let gate = self.state.events().gate();
        gate.require_open()?;
        if !gate.on_game_thread() || DELIVERING.with(Cell::get) {
            return Err(AmmlStatus::REFUSED);
        }
        if gate.epoch().0 == 0 {
            return Err(AmmlStatus::NOT_READY);
        }
        if ops.len() > ShapeBook::MAX_BATCH {
            return Err(AmmlStatus::REFUSED);
        }
        let requests = ops.iter().map(Self::request).collect::<Result<Vec<ShapeRequest>, AmmlStatus>>()?;
        let captured = Self::handler()?;
        let mut held = self.lock();
        let sends = held
            .book
            .apply(captured.generation, owner, &requests)
            .map_err(|ShapeRefusal::TooMany { .. }| AmmlStatus::REFUSED)?;
        let delivered = if sends.is_empty() {
            0
        } else {
            self.deliver(calls, &mut held, &captured, &sends, |send, payload| {
                Self::write(payload, send, &ops[send.op], dimension);
            })?
        };
        drop(held);
        let elapsed_us = u32::try_from(started.elapsed().as_micros()).unwrap_or(u32::MAX);
        Ok(AmmlApplyStats { delivered, elapsed_us })
    }

    // The book's view of `op`, with its geometry checked: `INVALID_ARGUMENT` for an unknown kind or a number that is
    // not finite.
    fn request(op: &AmmlShapeOp) -> Result<ShapeRequest, AmmlStatus> {
        let kind = match op.kind {
            AmmlShapeOp::LINE => Some(ShapeKind::Line),
            AmmlShapeOp::BOX => Some(ShapeKind::Box),
            AmmlShapeOp::ARROW => Some(ShapeKind::Arrow),
            AmmlShapeOp::REMOVE => None,
            _ => return Err(AmmlStatus::INVALID_ARGUMENT),
        };
        let finite = [op.from.x, op.from.y, op.from.z, op.to.x, op.to.y, op.to.z].iter().all(|value| value.is_finite());
        if kind.is_some() && !finite {
            return Err(AmmlStatus::INVALID_ARGUMENT);
        }
        Ok(ShapeRequest { id: op.id, kind })
    }

    // Writes the payload of `send` from `op` into the zeroed payload at `payload`.
    fn write(payload: *mut PrimitiveShapeLayout, send: &ShapeSend, op: &AmmlShapeOp, dimension: i32) {
        let Some(kind) = send.kind else {
            // SAFETY: `deliver` passes a zeroed, writable payload of the block.
            unsafe { PrimitiveShapeLayout::write_remove(payload, send.network_id) };
            return;
        };
        let channel = |value: u8| f32::from(value) / 255.0;
        let color = [channel(op.color.r), channel(op.color.g), channel(op.color.b), channel(op.color.a)];
        let ends = [[op.from.x, op.from.y, op.from.z], [op.to.x, op.to.y, op.to.z]];
        // SAFETY: as above.
        unsafe { PrimitiveShapeLayout::write_draw(payload, send.network_id, kind, ends, color, dimension) };
    }

    // The captured handler of this world, on its own thread, outside a free camera delivery.
    fn handler() -> Result<CapturedHandler, AmmlStatus> {
        FreeCameraAdapter::get().ok_or(AmmlStatus::NOT_AVAILABLE)?.captured_here()
    }

    // Makes the packet of this world when none is kept, grows the block to `sends.len()` payloads, writes payload
    // `index` with `write(&sends[index], payload)` into the zeroed block, lets the packet's vector span them, calls the
    // captured handler's shapes slot with the packet, and empties the vector. Gives the number of payloads handed over.
    // `FAILED` when the game makes no packet, the game heap is not available or gives no block, or the handler has no
    // slot. Runs under the lock, so the packet and its block cannot be given back during the call.
    fn deliver(
        &self,
        calls: &ShapeCalls,
        held: &mut ShapesHeld,
        captured: &CapturedHandler,
        sends: &[ShapeSend],
        write: impl Fn(&ShapeSend, *mut PrimitiveShapeLayout),
    ) -> Result<u32, AmmlStatus> {
        let count = sends.len();
        let delivered = u32::try_from(count).map_err(|_| AmmlStatus::REFUSED)?;
        if !self.state.heap().is_available() {
            return Err(AmmlStatus::FAILED);
        }
        if held.packet.is_none() {
            held.packet = self.make_packet(calls);
        }
        let packet = held
            .packet
            .as_ref()
            .and_then(|packet| packet.get())
            .map(|packet| std::ptr::from_ref(packet).expose_provenance())
            .ok_or(AmmlStatus::FAILED)?;
        let handle = ClientAdapter::virtual_fn(captured.handler, Some(calls.shapes_slot)).ok_or(AmmlStatus::FAILED)?;
        let vector = packet.checked_add(calls.shapes_at).ok_or(AmmlStatus::FAILED)?;
        let vector = std::ptr::with_exposed_provenance_mut::<MsvcVec<PrimitiveShapeLayout>>(vector);
        let heap = MsvcHeap::new(ClientAdapter::heap_new, ClientAdapter::heap_delete);
        // SAFETY: the packet is the live `PrimitiveShapesPacket` of this world under the lock, whose shape vector of
        // B8h-byte payloads sits at `shapes_at` (`PrimitiveShapeDataPayload.size`, checked at install); the vector is
        // empty between deliveries and its block is `held.block` (null before the first delivery).
        unsafe { Self::grow(&heap, held, vector, count) }?;
        let size = size_of::<PrimitiveShapeLayout>();
        let first = held.block.cast::<PrimitiveShapeLayout>();
        // SAFETY: the block holds `capacity >= count` payloads; each is zeroed (every option disengaged, the extra data
        // the first alternative), then written. The vector then spans the written payloads.
        unsafe {
            std::ptr::write_bytes(held.block, 0, count * size);
            for (index, send) in sends.iter().enumerate() {
                write(send, held.block.add(index * size).cast());
            }
            (*vector).set_raw_parts(first, first.add(count), first.add(held.capacity));
        }
        // SAFETY: the slot holds the handler's `handle(NetworkIdentifier const&, PrimitiveShapesPacket const&)`.
        let handle = unsafe { std::mem::transmute::<usize, HandleFn>(handle) };
        let mut identifier = NetworkIdentifierBuffer::new();
        DELIVERING.with(|cell| cell.set(true));
        // SAFETY: the handler is the one of the last handler call of this world, on its thread (the game thread); the
        // identifier is the loader's invalid identifier of the identifier's size; the packet is live under the lock. The
        // handler takes the packet by reference: the shape system copies each payload into its shape objects and does
        // not keep the packet.
        unsafe {
            handle(
                std::ptr::with_exposed_provenance_mut(captured.handler),
                identifier.as_mut_ptr(),
                std::ptr::with_exposed_provenance_mut(packet),
            );
        }
        DELIVERING.with(|cell| cell.set(false));
        // SAFETY: the vector of the live packet; its payloads need no destructor, so the end moves back to the start.
        unsafe { (*vector).set_raw_parts(first, first, first.add(held.capacity)) };
        Ok(delivered)
    }

    // Makes the block hold `count` payloads: a new block from the game heap when it holds fewer, the old one freed.
    //
    // Safety: `vector` is the empty shape vector of the kept packet, whose block is `held.block` for `held.capacity`
    // payloads, from `heap` (or null).
    unsafe fn grow(
        heap: &MsvcHeap,
        held: &mut ShapesHeld,
        vector: *mut MsvcVec<PrimitiveShapeLayout>,
        count: usize,
    ) -> Result<(), AmmlStatus> {
        if held.capacity >= count {
            return Ok(());
        }
        let size = size_of::<PrimitiveShapeLayout>();
        let bytes = count.checked_mul(size).ok_or(AmmlStatus::FAILED)?;
        // SAFETY: `heap` is the game's allocator.
        let block = unsafe { heap.allocate(bytes) };
        if block.is_null() {
            return Err(AmmlStatus::FAILED);
        }
        let old = std::mem::replace(&mut held.block, block);
        let old_capacity = std::mem::replace(&mut held.capacity, count);
        let first = block.cast::<PrimitiveShapeLayout>();
        // SAFETY: the vector is empty; it takes the new block (empty, with room for `count`) before the old block goes.
        // The old block came from `heap` for `old_capacity` payloads and nothing points to it after this.
        unsafe {
            (*vector).set_raw_parts(first, first, first.add(count));
            heap.deallocate(old, old_capacity * size);
        }
        Ok(())
    }

    // A `PrimitiveShapesPacket` from the game's factory, held with one use; `None` when the factory makes none.
    fn make_packet(&self, calls: &ShapeCalls) -> Option<amml_msvc::SharedPtr<c_void>> {
        let id = ProtocolValues::PRIMITIVE_SHAPES_ID;
        // SAFETY: on the game thread (checked by every caller), with the resolved factory and a packet id the game knows.
        let made = unsafe { PacketFactory::new(calls.create_packet).make(id) };
        if made.is_none() && !self.create_failed_logged.swap(true, Ordering::AcqRel) {
            curia::warn!("the game made no primitive shapes packet", { id: id });
        }
        made
    }

    /// The mod at `owner` is disabled: while the game runs, on the game thread, in a joined world with the handler of
    /// its shapes still captured on this thread, the removes of its live shapes go to the handler (in batches of the
    /// maximum). A warning is logged when they cannot. Once the game stopped (a shutdown disable) the packet and its
    /// block are left to the ending process: no game code runs for them.
    pub fn disable_owner(&self, owner: usize) {
        let gate = self.state.events().gate();
        let live = gate.is_open() && gate.on_game_thread() && gate.epoch().0 != 0;
        let capture = FreeCameraAdapter::get();
        let generation = capture.and_then(FreeCameraAdapter::captured).map_or(0, |captured| captured.generation);
        let captured = Self::handler();
        let mut held = self.lock();
        let removes = held.book.disable(owner, generation);
        if !gate.is_open() {
            held.forget_packet();
        }
        if removes.is_empty() {
            return;
        }
        let sent = match (live, self.calls.as_ref(), captured) {
            (true, Some(calls), Ok(captured)) => removes.chunks(ShapeBook::MAX_BATCH).try_for_each(|chunk| {
                let sends: Vec<ShapeSend> =
                    chunk.iter().map(|&network_id| ShapeSend { op: 0, network_id, kind: None }).collect();
                self.deliver(calls, &mut held, &captured, &sends, |send, payload| {
                    // SAFETY: `deliver` passes a zeroed, writable payload of the block.
                    unsafe { PrimitiveShapeLayout::write_remove(payload, send.network_id) };
                })
                .map(|_| ())
            }),
            (true, _, Err(status)) => Err(status),
            _ => Ok(()),
        };
        if let Err(status) = sent {
            curia::warn!("the shapes of a disabled mod were not removed", { count: removes.len(), status: status.0 });
        }
    }

    /// Before `WorldLeft`: every shape is forgotten and the packet goes back to the game. Nothing is sent: the handler
    /// belongs to the world being left.
    pub(crate) fn world_left(&self) {
        let mut held = self.lock();
        held.book.world_left();
        held.drop_packet();
    }
}
