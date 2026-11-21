use std::sync::{Mutex, PoisonError};

use amml_api::amml_abi::{
    ClientTick, FramePresent, FramePresented, FrameUpdate, GamepadPoll, InjectedInputReleased, KeyInput, LookDelta,
    ModsLoaded, MouseInput, PlayerMoveInput, ProviderReady, RenderDeviceReset, ScreenshotTaken, ScreenshotWritten,
    Shutdown, SurfaceLost, SwapChainResized, WindowChar, WindowFocus, WindowPointer, WindowSize, WorldJoined,
    WorldLeft,
};
use amml_api::{
    AmmlMod, ApiError, Area, Ctx, DisableCtx, Event, EventCtx, Events, HookOrder, Input, Listener, Logger, Provided,
    Registry, Uses,
};

use crate::{ECHO, InputChecks, OverlayChecks, OverlayTools, Progress, ProvidedEcho, Step, WindowChecks};

// The listeners of the enabled mod; `disable` drops them, which removes them from the loader.
static LISTENERS: Mutex<Vec<Listener>> = Mutex::new(Vec::new());
static PROGRESS: Progress = Progress::new();
static CHECKS: InputChecks = InputChecks::new(&PROGRESS);
static WINDOWS: WindowChecks = WindowChecks::new(&PROGRESS);
static OVERLAY: OverlayChecks = OverlayChecks::new(&PROGRESS, &WINDOWS);

/// The end-to-end test mod of the stable API. It listens to the loader, session, input, window and
/// overlay events, provides `apitest.echo` and requires it back, reads the client on the title screen,
/// draws a magenta square on every frame, and in the first joined world runs the client and input
/// checks (`InputChecks`), the overlay checks (`OverlayChecks`) and the window checks
/// (`WindowChecks`). It logs `apitest ok` once every step was seen and at least `Progress::TICKS`
/// client ticks ran.
pub struct ApiTestMod;

impl AmmlMod for ApiTestMod {
    const NAMESPACE: &'static str = "amml";
    const NAME: &'static str = "apitest";
    const USES: Uses = amml_api::amml_uses! {
        feature "core" {
            required ModsLoaded;
            required ProviderReady;
            required FrameUpdate;
            required ClientTick;
            optional Shutdown;
        }
        feature "world" {
            optional WorldJoined;
            optional WorldLeft;
        }
        feature "client" {
            optional area Area::CLIENT_READ;
            optional area Area::CLIENT_TOP_SCREEN;
            optional area Area::CLIENT_HUD;
            optional area Area::CLIENT_MOUSE;
            optional area Area::CLIENT_PAUSE;
        }
        feature "input" {
            optional KeyInput;
            optional MouseInput;
            optional LookDelta;
            optional PlayerMoveInput;
            optional GamepadPoll;
            optional InjectedInputReleased;
            optional area Area::INPUT_LEASE;
            optional area Area::KEY_INJECT;
            optional area Area::MOUSE_INJECT;
            optional area Area::TURN;
            optional area Area::MOVE;
        }
        feature "window" {
            optional WindowPointer;
            optional WindowChar;
            optional WindowSize;
            optional WindowFocus;
            optional area Area::WINDOW_READ;
            optional area Area::WINDOW_REQUEST_SIZE;
        }
        feature "overlay" {
            optional FramePresent;
            optional FramePresented;
            optional SwapChainResized;
            optional SurfaceLost;
            optional RenderDeviceReset;
            optional ScreenshotTaken;
            optional ScreenshotWritten;
            optional area Area::OVERLAY_SURFACE;
            optional area Area::OVERLAY_SCREENSHOT;
            optional area Area::OVERLAY_CAPTURE;
        }
    };

    fn enable(ctx: &Ctx) -> Result<(), ApiError> {
        let events = ctx.events();
        let log = ctx.log();
        let registry = ctx.registry();
        let input = ctx.input();
        let tools = OverlayTools { events, overlay: ctx.overlay(), window: ctx.window(), dirs: ctx.dirs() };
        let mut listeners = vec![
            events.listen::<ModsLoaded>(HookOrder::NORMAL, move |_, _| {
                if PROGRESS.set(Step::ModsLoaded) {
                    log.info("mods loaded", &[]);
                }
            })?,
            events.listen::<ProviderReady>(HookOrder::NORMAL, move |_, _| Self::provider_ready(log, registry))?,
            events.listen::<FrameUpdate>(HookOrder::READ_AFTER_WRITERS, move |_, at| {
                if at.game_alive && PROGRESS.set(Step::FrameGate) {
                    log.info("frame gate open", &[]);
                }
            })?,
            events.listen::<ClientTick>(HookOrder::READ_AFTER_WRITERS, move |_, at| {
                Self::client_tick(log, input, tools, at);
            })?,
        ];
        listeners.extend(Self::optional::<WorldJoined>(events, log, move |event, _| {
            if PROGRESS.set(Step::WorldJoined) {
                log.info("world joined", &[("epoch", &event.epoch.to_string())]);
                CHECKS.joined();
                OVERLAY.joined();
            }
        })?);
        listeners.extend(Self::optional::<WorldLeft>(events, log, move |event, _| {
            log.info("world left", &[("epoch", &event.epoch.to_string())]);
        })?);
        listeners.extend(Self::optional::<KeyInput>(events, log, move |event, _| CHECKS.key_input(log, event))?);
        listeners.extend(Self::optional::<MouseInput>(events, log, move |event, _| CHECKS.mouse_input(log, event))?);
        listeners.extend(Self::optional::<LookDelta>(events, log, move |event, _| CHECKS.look_delta(log, event))?);
        listeners
            .extend(Self::optional::<PlayerMoveInput>(events, log, move |event, _| CHECKS.player_move(log, event))?);
        listeners.extend(Self::optional::<GamepadPoll>(events, log, move |event, _| CHECKS.gamepad_poll(event))?);
        listeners.extend(Self::optional::<InjectedInputReleased>(events, log, move |event, _| {
            CHECKS.released(log, input, event);
        })?);
        listeners.extend(Self::optional::<Shutdown>(events, log, move |_, at| WINDOWS.shutdown(log, at))?);
        listeners.extend(Self::optional::<WindowPointer>(events, log, move |event, _| WINDOWS.pointer(log, event))?);
        listeners.extend(Self::optional::<WindowChar>(events, log, move |event, _| WINDOWS.char(log, event))?);
        listeners.extend(Self::optional::<WindowSize>(events, log, move |event, _| WINDOWS.size(log, event))?);
        listeners.extend(Self::optional::<WindowFocus>(events, log, move |event, _| WINDOWS.focus(log, event))?);
        listeners.extend(Self::optional::<SwapChainResized>(events, log, move |event, _| WINDOWS.resized(log, event))?);
        listeners.extend(Self::optional::<FramePresented>(events, log, move |event, _| {
            OVERLAY.frame_presented(log, event);
        })?);
        listeners.extend(Self::optional::<SurfaceLost>(events, log, move |event, _| OVERLAY.surface_lost(log, event))?);
        listeners.extend(Self::optional::<RenderDeviceReset>(events, log, move |event, _| {
            OVERLAY.render_device_reset(log, event);
        })?);
        listeners.extend(Self::optional::<ScreenshotTaken>(events, log, move |event, _| {
            OVERLAY.screenshot_taken(log, event);
        })?);
        listeners.extend(Self::optional::<ScreenshotWritten>(events, log, move |event, _| {
            OVERLAY.screenshot_written(log, event);
        })?);
        // The square is drawn in the `NORMAL` band, so the `WithOverlays` copy after that band holds it; the
        // overlay checks remove this listener for the frame times without one
        let draw = Self::optional_at::<FramePresent>(events, log, HookOrder::NORMAL, move |event, _| {
            OVERLAY.frame_present(log, event);
        })?;
        for area in Self::AREAS {
            if !ctx.supports(area) {
                log.warn("area not available", &[("area", area.id)]);
            }
        }
        registry.provide(&ECHO)?;
        OVERLAY.set_draw(draw);
        *LISTENERS.lock().unwrap_or_else(PoisonError::into_inner) = listeners;
        PROGRESS.set(Step::Listening);
        log.info("listening", &[]);
        Ok(())
    }

    fn disable(ctx: &Ctx, how: &DisableCtx) {
        let listeners = std::mem::take(&mut *LISTENERS.lock().unwrap_or_else(PoisonError::into_inner));
        drop(listeners);
        // The leases and holds end here, outside the run lock
        drop(CHECKS.take());
        // The drawing listener and the capture stream end here, outside the overlay run lock
        drop(OVERLAY.take());
        WINDOWS.reset();
        // No listener runs after the drop, so a later enable starts from no steps seen
        PROGRESS.reset();
        ctx.log().info("disabled", &[("game_alive", if how.game_alive { "true" } else { "false" })]);
    }
}

impl ApiTestMod {
    /// The areas of `USES`; the mod logs `area not available` at enable for each one the loader does
    /// not serve.
    pub const AREAS: [Area; 15] = [
        Area::CLIENT_READ,
        Area::CLIENT_TOP_SCREEN,
        Area::CLIENT_HUD,
        Area::CLIENT_MOUSE,
        Area::CLIENT_PAUSE,
        Area::INPUT_LEASE,
        Area::KEY_INJECT,
        Area::MOUSE_INJECT,
        Area::TURN,
        Area::MOVE,
        Area::WINDOW_READ,
        Area::WINDOW_REQUEST_SIZE,
        Area::OVERLAY_SURFACE,
        Area::OVERLAY_SCREENSHOT,
        Area::OVERLAY_CAPTURE,
    ];

    // A listen the mod can do without, in the `READ_AFTER_WRITERS` band.
    fn optional<E: Event>(
        events: Events,
        log: Logger,
        f: impl FnMut(&mut E, &EventCtx) + Send + 'static,
    ) -> Result<Option<Listener>, ApiError> {
        Self::optional_at(events, log, HookOrder::READ_AFTER_WRITERS, f)
    }

    // A listen the mod can do without: `None`, with a log line, when the loader cannot serve `E`.
    fn optional_at<E: Event>(
        events: Events,
        log: Logger,
        order: HookOrder,
        f: impl FnMut(&mut E, &EventCtx) + Send + 'static,
    ) -> Result<Option<Listener>, ApiError> {
        match events.listen::<E>(order, f) {
            Ok(listener) => Ok(Some(listener)),
            Err(ApiError::NotAvailable) => {
                log.warn("event not available", &[("event", E::ID)]);
                Ok(None)
            }
            Err(error) => Err(error),
        }
    }

    // Takes the echo table back from the registry at a `ProviderReady` and checks its answer. The event's
    // `name` is a raw ABI view, so the mod asks the registry instead of reading it: the first
    // `ProviderReady` after the mod's own `provide` finds the table.
    fn provider_ready(log: Logger, registry: Registry) {
        let Some(table) = registry.require::<ProvidedEcho>() else {
            return;
        };
        let answer = (table.echo)(ProvidedEcho::PROBE);
        if answer != ProvidedEcho::PROBE.wrapping_add(1) {
            log.error("provider gave a wrong answer", &[("name", ProvidedEcho::NAME), ("answer", &answer.to_string())]);
            return;
        }
        if PROGRESS.set(Step::ProviderReady) {
            log.info("provider ready", &[("name", ProvidedEcho::NAME)]);
        }
    }

    fn client_tick(log: Logger, input: Input, tools: OverlayTools, at: &EventCtx) {
        CHECKS.client_tick(log, input, at.client());
        OVERLAY.client_tick(log, tools);
        let ticks = PROGRESS.tick();
        if ticks.is_multiple_of(Self::WAITING_EVERY) {
            let missing = PROGRESS.missing();
            if !missing.is_empty() {
                log.info("apitest waiting", &[("ticks", &ticks.to_string()), ("missing", &missing.join(","))]);
            }
        }
        if PROGRESS.try_pass() {
            log.info("apitest ok", &[("ticks", &ticks.to_string())]);
        }
    }

    // Client ticks between two `apitest waiting` lines.
    const WAITING_EVERY: u32 = 600;
}
