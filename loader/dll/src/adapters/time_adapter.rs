use std::sync::atomic::{AtomicI64, AtomicU64, Ordering};
use std::sync::{Mutex, MutexGuard, OnceLock};
use std::time::Instant;

use amml_abi::{
    AmmlEvent, AmmlHoldRequest, AmmlPhase, AmmlStatus, AmmlThread, AmmlTimeHold, AmmlTimeState, FrameEnd,
    LatencyBarrier, TimeStepApplied,
};
use amml_data::{AdapterCatalog, Availability, EntryId, NeedKind};
use amml_runtime::{TimeReleaseReason, VirtualClock};

use crate::{
    EventHub, HandlerDetours, LoaderState, NowRestore, ParticleEngineLayout, SessionAdapter, SessionHook, SteadyClock,
    SystemUniformsLayout, TimeDetours, TimeHeld, TimerLayout,
};

static ADAPTER: OnceLock<TimeAdapter> = OnceLock::new();

/// The `time.control` area and the `time.step_applied`, `time.frame_end` and `time.latency_barrier` events: one mod
/// holds the game clock.
///
/// A hold takes the four client timers of the joined world: `MinecraftGame.mSimTimer` and `mRealTimer` (pointers to the
/// timers) and `ClientInstance.mSimTimer` and `mRealTimer` (timers inside the client), and the game's `steady_clock`
/// count, plus the request's offset, as the origin of the virtual clock. While held, the `Timer::advanceTime` detour
/// writes into each of the four, after the game's advance, the ticks and the alpha of the stepped game time
/// (`VirtualClock`, integer microseconds from the tick boundary of the hold), the alpha as the passed time, and the
/// overflow time from before the advance, so no wall-clock tick reaches the game. The first of them to advance in a
/// frame takes the steps queued since the last frame. `TimeStepApplied` is raised After the frame in which a step
/// reached the timers. The particle tick and the predicted movement follow the same clock: Before the engine's tick the
/// engine's last tick time moves so that the step it measures is the game time stepped since the last tick (only for an
/// engine that has ticked before), and the `mNow` of the predicted movement's uniforms is the virtual clock for the
/// call. `FrameEnd` is raised Before `MinecraftGame::endFrame` whether held or not, and `LatencyBarrier` After the
/// game's handler of a `NetworkStackLatencyPacket`.
///
/// One hold for the process (`TimeHoldBook`): a second hold is `REFUSED`. The hold ends by its release, the world leave,
/// the holder's disable and the watchdog (no `step_us` for the hold's watchdog time, looked at each `FrameUpdate`
/// Before); each end logs `time hold released` with its reason, and the timers run on the wall clock from the next
/// advance. `hold` and `step_us` run on the game thread (`REFUSED` elsewhere); a release from another thread is queued
/// for the next `FrameUpdate` Before.
pub struct TimeAdapter {
    state: &'static LoaderState,
    game_sim_at: Option<usize>,
    game_real_at: Option<usize>,
    client_sim_at: Option<usize>,
    client_real_at: Option<usize>,
    held: Mutex<TimeHeld>,
    virtual_base_ns: AtomicI64,
    frames: AtomicU64,
}

impl TimeAdapter {
    pub const AREA: &'static str = "time.control";
    pub const EVENT: &'static str = TimeStepApplied::ID;
    pub const FRAME_END: &'static str = FrameEnd::ID;
    pub const LATENCY_BARRIER: &'static str = LatencyBarrier::ID;
    const VERSION: u32 = 1;
    // Loader hooks run before every mod hook on the same function.
    const PRIORITY: i32 = i32::MAX;
    const NANOS_PER_MILLI: i64 = 1_000_000;
    const NANOS_PER_MICRO: i64 = 1000;
    // The index of the game's sim timer among the four.
    const GAME_SIM: usize = 0;

    /// Hooks `Timer::advanceTime`, the particle tick and the predicted movement when a clock row resolved, `endFrame` for
    /// the `FrameEnd` row and the latency handler slot for the `LatencyBarrier` row, offers each row whose hooks stand,
    /// and logs one line per row. Works once per process: a second call hooks nothing and gives the catalog rows.
    pub fn install(state: &'static LoaderState) -> Vec<Availability> {
        let resolved = |name: &str| state.table().get(&EntryId(name.to_string())).is_some();
        let mut rows: Vec<Availability> =
            AdapterCatalog::availability(&resolved).into_iter().filter(Self::is_own_row).collect();
        let value = |name: &str| state.resolve(name).ok().and_then(|value| usize::try_from(value).ok());
        let adapter = Self {
            state,
            game_sim_at: value("MinecraftGame.mSimTimer"),
            game_real_at: value("MinecraftGame.mRealTimer"),
            client_sim_at: value("ClientInstance.mSimTimer"),
            client_real_at: value("ClientInstance.mRealTimer"),
            held: Mutex::new(TimeHeld::new()),
            virtual_base_ns: AtomicI64::new(0),
            frames: AtomicU64::new(0),
        };
        if ADAPTER.set(adapter).is_err() {
            curia::warn!("the time adapter is already installed");
            return rows;
        }
        let mismatched: Vec<&'static str> = [
            ("ParticleSystemEngine.mValid", ParticleEngineLayout::VALID_AT),
            ("ParticleSystemEngine.mLastTickTime", ParticleEngineLayout::LAST_TICK_AT),
            ("ParticleSystemEngine.mParticleCount", ParticleEngineLayout::COUNT_AT),
            ("SystemUniforms.mNow", SystemUniformsLayout::NOW_AT),
        ]
        .into_iter()
        .filter(|(name, at)| value(name).is_some_and(|derived| derived != *at))
        .map(|(name, _)| name)
        .collect();
        if !mismatched.is_empty() {
            curia::warn!("the time mirrors do not match the build", { entries: mismatched.join(", ") });
        }
        let has = |id: &str| rows.iter().any(|row| row.id == id && row.shape.is_some());
        let clock = has(Self::AREA) || has(Self::EVENT);
        // The clock rows share three hooks; a refused hook or a mirror that does not match withdraws both.
        let clock_refused: Vec<&'static str> = if !clock {
            Vec::new()
        } else if mismatched.is_empty() {
            Self::install_all(state, [TimeDetours::hook(), TimeDetours::particle_hook(), TimeDetours::movement_hook()])
        } else {
            mismatched
        };
        let end_refused =
            if has(Self::FRAME_END) { Self::install_all(state, [TimeDetours::end_frame_hook()]) } else { Vec::new() };
        let latency_refused = if has(Self::LATENCY_BARRIER) {
            Self::install_all(state, [HandlerDetours::latency_hook()])
        } else {
            Vec::new()
        };
        for row in &mut rows {
            let refused = match row.id {
                Self::FRAME_END => &end_refused,
                Self::LATENCY_BARRIER => &latency_refused,
                _ => &clock_refused,
            };
            if row.shape.is_some() && !refused.is_empty() {
                row.shape = None;
                row.missing = refused.clone();
            }
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
        rows
    }

    fn is_own_row(row: &Availability) -> bool {
        match row.kind {
            NeedKind::Event => [Self::EVENT, Self::FRAME_END, Self::LATENCY_BARRIER].contains(&row.id),
            NeedKind::Area => row.id == Self::AREA,
        }
    }

    // Installs each hook of `hooks`; gives the entries of those that were refused.
    fn install_all<const N: usize>(state: &'static LoaderState, hooks: [SessionHook; N]) -> Vec<&'static str> {
        hooks.iter().filter(|hook| !hook.install(state, Self::PRIORITY)).map(|hook| hook.entry).collect()
    }

    /// The installed adapter; `None` before `install`.
    pub fn get() -> Option<&'static TimeAdapter> {
        ADAPTER.get()
    }

    /// Runs `body` on the installed adapter; `fallback` before `install`.
    pub(crate) fn with<R>(fallback: R, body: impl FnOnce(&TimeAdapter) -> R) -> R {
        match ADAPTER.get() {
            Some(adapter) => body(adapter),
            None => fallback,
        }
    }

    // The state holds plain data, so a lock poisoned by a panic is recovered.
    fn lock(&self) -> MutexGuard<'_, TimeHeld> {
        self.held.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    fn hub(&self) -> &EventHub {
        self.state.events()
    }

    /// Whether the area is offered.
    pub fn is_offered(&self) -> bool {
        self.state.areas().is_available(Self::AREA, Self::VERSION)
    }

    // The area, the game gate and the game thread of a call: `NOT_AVAILABLE`, `NOT_READY`, `REFUSED`.
    fn enter(&self) -> Result<(), AmmlStatus> {
        if !self.is_offered() {
            return Err(AmmlStatus::NOT_AVAILABLE);
        }
        let gate = self.hub().gate();
        gate.require_open()?;
        if !gate.on_game_thread() {
            return Err(AmmlStatus::REFUSED);
        }
        Ok(())
    }

    // The four timers of the joined world: `NOT_READY` before a world is joined, without a game or a primary client, or
    // with a null game timer.
    fn timers(&self) -> Result<[usize; VirtualClock::TIMERS], AmmlStatus> {
        if self.hub().gate().epoch().0 == 0 {
            return Err(AmmlStatus::NOT_READY);
        }
        let game = SessionAdapter::minecraft_game().ok_or(AmmlStatus::NOT_READY)?;
        let client = SessionAdapter::primary_client().ok_or(AmmlStatus::NOT_READY)?;
        let pointer = |at: Option<usize>| -> Result<usize, AmmlStatus> {
            let address = game.checked_add(at.ok_or(AmmlStatus::NOT_AVAILABLE)?).ok_or(AmmlStatus::NOT_READY)?;
            // SAFETY: `game` is the live MinecraftGame of the last frame, on the game thread; the entry is the offset of
            // its `std::unique_ptr<Timer>` (survey section 8: `mov rcx,[rsi+12C8h]` / `[rsi+1308h]` before each advance).
            let timer = unsafe { std::ptr::with_exposed_provenance::<usize>(address).read() };
            if timer == 0 { Err(AmmlStatus::NOT_READY) } else { Ok(timer) }
        };
        let inline = |at: Option<usize>| -> Result<usize, AmmlStatus> {
            client.checked_add(at.ok_or(AmmlStatus::NOT_AVAILABLE)?).ok_or(AmmlStatus::NOT_READY)
        };
        Ok([
            pointer(self.game_sim_at)?,
            pointer(self.game_real_at)?,
            inline(self.client_sim_at)?,
            inline(self.client_real_at)?,
        ])
    }

    /// The mod at `owner` takes the game clock. The errors are those of `AmmlTimeTable::hold`.
    pub fn hold(&self, owner: usize, request: AmmlHoldRequest) -> Result<AmmlTimeHold, AmmlStatus> {
        self.enter()?;
        let timers = self.timers()?;
        let mut held = self.lock();
        let hold = held.book.hold(owner, request.watchdog_ms, Instant::now()).map_err(|_| AmmlStatus::REFUSED)?;
        held.reset();
        held.timers = timers;
        let offset = request.virtual_offset_ms.saturating_mul(Self::NANOS_PER_MILLI);
        self.virtual_base_ns.store(SteadyClock::now_ns().saturating_add(offset), Ordering::Release);
        TimeDetours::set(TimeDetours::HELD, true);
        curia::info!("time hold", {
            owner: owner,
            offset_ms: request.virtual_offset_ms,
            watchdog_ms: request.watchdog_ms
        });
        Ok(hold)
    }

    /// Queues `micros` of game time for the timers of `hold`. The errors are those of `AmmlTimeTable::step_us`.
    pub fn step_us(&self, hold: AmmlTimeHold, micros: i64) -> Result<(), AmmlStatus> {
        self.enter()?;
        if micros < 0 {
            return Err(AmmlStatus::INVALID_ARGUMENT);
        }
        let mut held = self.lock();
        held.book.step(hold, Instant::now()).map_err(|_| AmmlStatus::STALE)?;
        held.clock.step_us(micros);
        Ok(())
    }

    /// Ends `hold`; off the game thread the release is queued for the next frame. The errors are those of
    /// `AmmlTimeTable::release`.
    pub fn release(&self, hold: AmmlTimeHold) -> Result<(), AmmlStatus> {
        if !self.hub().gate().on_game_thread() {
            self.lock().queued.push(hold);
            return Ok(());
        }
        let mut held = self.lock();
        held.book.release(hold).map_err(|_| AmmlStatus::STALE)?;
        Self::end(&mut held, TimeReleaseReason::Release);
        Ok(())
    }

    /// The state of the game clock.
    pub fn state(&self) -> AmmlTimeState {
        let held = self.lock();
        AmmlTimeState {
            held: u8::from(held.book.holder().is_some()),
            _reserved: [0; 3],
            sim_ticks: held.sim_ticks,
            stepped_us: held.clock.stepped_us(),
            sim_alpha: held.sim_alpha,
            particle_count: held.particle_count,
        }
    }

    /// While held: the game's `steady_clock` count of the virtual clock, in nanoseconds (the origin of the hold plus the
    /// stepped game time). `None` while the clock runs free.
    pub fn virtual_now_ns(&self) -> Option<i64> {
        let held = self.lock();
        held.book.holder()?;
        let stepped = held.clock.stepped_us().saturating_mul(Self::NANOS_PER_MICRO);
        Some(self.virtual_base_ns.load(Ordering::Acquire).saturating_add(stepped))
    }

    // The hold of `held` ended for `reason` (the book already forgot it): the detour runs free from its next call.
    fn end(held: &mut TimeHeld, reason: TimeReleaseReason) {
        TimeDetours::set(TimeDetours::HELD, false);
        held.reset();
        curia::info!("time hold released", { reason: reason.name() });
    }

    /// After the game advanced the timer at `timer` while held, with `overflow` its overflow time from before: one of
    /// the four timers of the hold gets the held clock; any other timer is left alone.
    pub(crate) fn advanced(&self, timer: usize, overflow: f32) {
        let mut held = self.lock();
        if held.book.holder().is_none() {
            return;
        }
        let Some(index) = held.timers.iter().position(|held_timer| *held_timer == timer) else {
            return;
        };
        if held.clock.apply() {
            held.applied = true;
        }
        // SAFETY: `timer` is one of the four live client timers of the joined world, advanced by the game on this thread.
        let reading = held.clock.read(index, unsafe { TimerLayout::ticks_per_second(timer) });
        // SAFETY: as above; the game wrote it just before and no other thread writes it.
        unsafe { TimerLayout::write_held(timer, reading, overflow) };
        if index == Self::GAME_SIM {
            held.sim_ticks = reading.ticks;
            held.sim_alpha = reading.alpha;
        }
    }

    /// Before the particle engine at `engine` ticks while held: the engine's last tick time moves so that the step the
    /// game measures (its own clock minus the last tick time) is the game time stepped since the last particle tick.
    /// An engine that has not ticked before measures no step, and keeps its last tick time. The game's own clock read
    /// follows this one by well under a microsecond.
    pub(crate) fn particles_before(&self, engine: usize) {
        let delta = {
            let mut held = self.lock();
            if held.book.holder().is_none() {
                return;
            }
            let stepped = held.clock.stepped_us();
            let delta = stepped.saturating_sub(held.particle_stepped_us);
            held.particle_stepped_us = stepped;
            delta
        };
        // SAFETY: `engine` is the live engine the game ticks now, on this thread; the mirror covers the field.
        if !unsafe { ParticleEngineLayout::valid(engine) } {
            return;
        }
        let now = SteadyClock::now_ns();
        if now > 0 {
            let moved = now.saturating_sub(delta.saturating_mul(Self::NANOS_PER_MICRO));
            // SAFETY: as above.
            unsafe { ParticleEngineLayout::set_last_tick_time(engine, moved) };
        }
    }

    /// After the particle engine at `engine` ticked while held: its particle count is noted for `state`.
    pub(crate) fn particles_after(&self, engine: usize) {
        // SAFETY: `engine` is the live engine the game ticked on this thread; the mirror covers the field.
        let count = unsafe { ParticleEngineLayout::particle_count(engine) };
        let mut held = self.lock();
        if held.book.holder().is_some() {
            held.particle_count = u32::try_from(count).unwrap_or(u32::MAX);
        }
    }

    /// While held: the virtual clock goes into the `mNow` of the uniforms at `uniforms`; the guard puts the wall value
    /// back when it drops. `None` while the clock runs free.
    pub(crate) fn swapped_now(&self, uniforms: usize) -> Option<NowRestore> {
        let now = self.virtual_now_ns()?;
        // SAFETY: `uniforms` is the live `SystemUniforms` the game passes to the predicted movement on this thread; the
        // guard is dropped before the call returns to the game.
        Some(unsafe { NowRestore::replace(uniforms, now) })
    }

    /// Before `MinecraftGame::endFrame`, while a mod listens to `FrameEnd` or holds the clock: the call is counted, and
    /// `FrameEnd` is raised when a mod listens.
    pub(crate) fn frame_ending(&self) {
        let count = self.frames.fetch_add(1, Ordering::Relaxed).saturating_add(1);
        if !self.hub().has_listeners(Self::FRAME_END) {
            return;
        }
        let mut event = FrameEnd { header: EventHub::header::<FrameEnd>(AmmlPhase::BEFORE), count };
        self.hub().raise_keeping(&mut event, &self.hub().loader_context(), &|event: &mut FrameEnd| {
            event.count = count;
        });
    }

    /// After the game handled a `NetworkStackLatencyPacket` whose `createTime` was `create_time`.
    pub(crate) fn latency_barrier(&self, create_time: u64) {
        let hub = self.hub();
        let thread = if hub.gate().on_game_thread() { AmmlThread::GAME } else { AmmlThread::NETWORK };
        let mut event = LatencyBarrier { header: EventHub::header::<LatencyBarrier>(AmmlPhase::AFTER), create_time };
        hub.raise_keeping(&mut event, &hub.context_on(thread), &|event: &mut LatencyBarrier| {
            event.create_time = create_time;
        });
    }

    /// A `FrameUpdate` Before, on the game thread: the releases queued from other threads run, then the watchdog looks.
    pub(crate) fn frame_began(&self) {
        if TimeDetours::flags() & TimeDetours::HELD == 0 {
            return;
        }
        let mut held = self.lock();
        for hold in std::mem::take(&mut held.queued) {
            if held.book.release(hold).is_ok() {
                Self::end(&mut held, TimeReleaseReason::Release);
            }
        }
        if let Some(owner) = held.book.watchdog(Instant::now()) {
            curia::warn!("time hold watchdog", { owner: owner });
            Self::end(&mut held, TimeReleaseReason::Watchdog);
        }
    }

    /// A `FrameUpdate` After, on the game thread: `TimeStepApplied` when a step reached the timers in this frame.
    pub(crate) fn frame_ended(&self) {
        if TimeDetours::flags() & TimeDetours::HELD == 0 {
            return;
        }
        let mut event = {
            let mut held = self.lock();
            if !std::mem::take(&mut held.applied) {
                return;
            }
            TimeStepApplied {
                header: EventHub::header::<TimeStepApplied>(AmmlPhase::AFTER),
                stepped_us: held.clock.stepped_us(),
                sim_ticks: held.sim_ticks,
                sim_alpha: held.sim_alpha,
            }
        };
        let (stepped_us, sim_ticks, sim_alpha) = (event.stepped_us, event.sim_ticks, event.sim_alpha);
        self.hub().raise_keeping(&mut event, &self.hub().loader_context(), &|event: &mut TimeStepApplied| {
            event.stepped_us = stepped_us;
            event.sim_ticks = sim_ticks;
            event.sim_alpha = sim_alpha;
        });
    }

    /// Before `WorldLeft`: the hold ends.
    pub(crate) fn world_left(&self) {
        let mut held = self.lock();
        if held.book.world_left().is_some() {
            Self::end(&mut held, TimeReleaseReason::WorldLeft);
        }
    }

    /// The mod at `owner` is disabled: its hold ends. No game call is made, so this runs on any thread.
    pub fn disable_owner(&self, owner: usize) {
        let mut held = self.lock();
        if held.book.disable(owner) {
            Self::end(&mut held, TimeReleaseReason::Disabled);
        }
    }
}
