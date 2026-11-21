use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Mutex, PoisonError};
use std::time::{Duration, Instant};

use amml_api::amml_abi::{
    AmmlReleaseReason, GamepadPoll, InjectedInputReleased, KeyInput, LookDelta, MouseInput, PlayerMoveInput,
};
use amml_api::{
    ApiError, Client, Event, Input, InputBlocks, KeyCode, KeyInputExt, LeaseMode, LeaseRequest, Logger, Look,
    LookDeltaExt, MouseAction, MouseButton, MouseInputExt, MoveIntent, PlayerMoveInputExt, Screen,
};

use crate::{CheckPhase, CheckRun, Progress, Step};

/// The in-world checks of the API test mod: the client reads and writes, then one injection of each
/// kind, each seen back as an injected event, then the pause screen. They run one phase at a time on
/// the client ticks of the joined world (the event that carries the client); the input listeners only
/// mark steps, so they never wait for the run lock.
pub struct InputChecks {
    progress: &'static Progress,
    run: Mutex<CheckRun>,
    // True while the W hold of the key phase is down, for the `PlayerMoveInput` listener.
    key_held: AtomicBool,
    gamepad_skips: AtomicU32,
    // The injected yaw the `LookDelta` listener saw in the turn phase, as `f32` bits.
    turn_seen: AtomicU32,
}

impl InputChecks {
    /// The key the checks hold.
    pub const KEY: KeyCode = KeyCode::W;
    /// Client ticks after `WorldJoined` before the first check, so the world can settle.
    pub const SETTLE_TICKS: u32 = 100;
    /// Client ticks the W hold stays down.
    pub const HOLD_TICKS: u32 = 40;
    /// The longest time the W hold stays down while no player movement was seen.
    pub const HOLD_MAX: Duration = Duration::from_secs(3);
    /// Client ticks a phase waits for its event or for a feed to bind before it gives up.
    pub const WAIT_TICKS: u32 = 1800;
    /// Client ticks the `FULL` lease stays active.
    pub const GAMEPAD_TICKS: u32 = 20;
    /// The turn the checks inject, in degrees of yaw (positive turns right).
    pub const TURN_YAW: f32 = 90.0;
    /// The turn goes out as this many equal steps, so it can be seen in the game.
    pub const TURN_STEPS: u32 = 10;
    /// The time over which the steps are spread.
    pub const TURN_SPREAD: Duration = Duration::from_secs(1);
    /// The time the turned view stays before the next phase (and the pause screen).
    pub const TURN_HOLD: Duration = Duration::from_secs(2);

    pub const fn new(progress: &'static Progress) -> Self {
        Self {
            progress,
            run: Mutex::new(CheckRun::new()),
            key_held: AtomicBool::new(false),
            gamepad_skips: AtomicU32::new(0),
            turn_seen: AtomicU32::new(0),
        }
    }

    /// Ends the run: gives back its state, which the caller drops outside any lock it holds.
    pub fn take(&self) -> CheckRun {
        self.key_held.store(false, Ordering::Release);
        self.gamepad_skips.store(0, Ordering::Release);
        self.turn_seen.store(0, Ordering::Release);
        std::mem::take(&mut *self.run.lock().unwrap_or_else(PoisonError::into_inner))
    }

    /// `WorldJoined`: the first join starts the checks.
    pub fn joined(&self) {
        let mut run = self.run.lock().unwrap_or_else(PoisonError::into_inner);
        if run.phase == CheckPhase::Title {
            run.advance();
        }
    }

    /// One client tick of the primary client: the title screen read before a world, then one step of
    /// the current phase.
    pub fn client_tick(&self, log: Logger, input: Input, client: Option<Client<'_>>) {
        let Some(client) = client else {
            return;
        };
        let mut run = self.run.lock().unwrap_or_else(PoisonError::into_inner);
        run.ticks = run.ticks.saturating_add(1);
        match run.phase {
            CheckPhase::Title => self.title(&mut run, log, &client),
            CheckPhase::Settle => {
                if run.ticks >= Self::SETTLE_TICKS {
                    run.advance();
                }
            }
            CheckPhase::Reads => self.reads(&mut run, log, &client),
            CheckPhase::HudHide => self.hud(&mut run, log, &client, true),
            CheckPhase::HudShow => self.hud(&mut run, log, &client, false),
            CheckPhase::MouseRelease => self.mouse(&mut run, log, &client, false),
            CheckPhase::MouseCapture => self.mouse(&mut run, log, &client, true),
            CheckPhase::KeyHold => self.key_hold(&mut run, log, input),
            CheckPhase::Move => self.move_intent(&mut run, log),
            CheckPhase::Turn => self.turn(&mut run, log),
            CheckPhase::Wheel => {
                self.inject(&mut run, log, Step::Wheel, "wheel", |lease| lease.wheel(1.0));
            }
            CheckPhase::Click => self.click(&mut run, log, MouseButton::Left, Step::Click),
            CheckPhase::ClickX1 => self.click(&mut run, log, MouseButton::X1, Step::ClickX1),
            CheckPhase::ClickX2 => self.click(&mut run, log, MouseButton::X2, Step::ClickX2),
            CheckPhase::Gamepad => self.gamepad(&mut run, log, input),
            CheckPhase::Pause => self.pause(&mut run, log, &client),
            CheckPhase::Done => {}
        }
    }

    fn title(&self, run: &mut CheckRun, log: Logger, client: &Client<'_>) {
        let screen = client.top_screen();
        let text = Self::screen_text(screen);
        if run.screen_seen != Some(text) {
            run.screen_seen = Some(text);
            log.info("top screen before a world", &[("top_screen", text)]);
        }
        if screen == Some(Screen::Start) && self.progress.set(Step::TitleScreen) {
            log.info("client reads", &[("top_screen", "Start")]);
        }
    }

    fn reads(&self, run: &mut CheckRun, log: Logger, client: &Client<'_>) {
        let player = client.local_player().is_some();
        let world = client.world_active();
        let menu = client.showing_menu();
        let screen = client.top_screen();
        let expected = player && world == Some(true) && menu == Some(false) && screen == Some(Screen::Other);
        if !expected && run.ticks < Self::WAIT_TICKS {
            return;
        }
        let fields = [
            ("local_player", if player { "yes" } else { "no" }),
            ("world_active", Self::flag_text(world)),
            ("showing_menu", Self::flag_text(menu)),
            ("top_screen", Self::screen_text(screen)),
        ];
        if expected {
            self.progress.set(Step::ClientReads);
            log.info("client reads", &fields);
        } else {
            log.warn("client reads", &fields);
        }
        run.advance();
    }

    fn hud(&self, run: &mut CheckRun, log: Logger, client: &Client<'_>, hidden: bool) {
        if let Err(error) = client.set_hide_hud(hidden) {
            log.warn("hud write failed", &[("error", &error.to_string())]);
        }
        let read = client.hide_hud();
        log.info("hud", &[("hidden", Self::flag_text(read))]);
        if read == Some(hidden) {
            self.progress.set(if hidden { Step::HudHidden } else { Step::HudShown });
        }
        run.advance();
    }

    fn mouse(&self, run: &mut CheckRun, log: Logger, client: &Client<'_>, captured: bool) {
        if let Err(error) = client.set_mouse_captured(captured) {
            log.warn("mouse write failed", &[("error", &error.to_string())]);
        }
        let read = client.mouse_captured();
        log.info("mouse", &[("captured", Self::flag_text(read))]);
        if read == Some(captured) {
            self.progress.set(if captured { Step::MouseCaptured } else { Step::MouseReleased });
        }
        run.advance();
    }

    // Takes the shared lease once; false, with a log line, when the loader refuses it.
    fn lease(run: &mut CheckRun, log: Logger, input: Input) -> bool {
        if run.lease.is_some() {
            return true;
        }
        let request =
            LeaseRequest { mode: LeaseMode::Shared, blocks: InputBlocks::NONE, passthrough: &[], preempt: false };
        match input.acquire(request) {
            Ok(lease) => {
                log.info("lease taken", &[("lease", &lease.id().to_string()), ("blocks", "none")]);
                run.lease = Some(lease);
                true
            }
            Err(error) => {
                log.warn("lease refused", &[("error", &error.to_string())]);
                false
            }
        }
    }

    fn key_hold(&self, run: &mut CheckRun, log: Logger, input: Input) {
        if !Self::lease(run, log, input) {
            run.phase = CheckPhase::Pause;
            run.ticks = 0;
            return;
        }
        if run.key.is_some() {
            // The hold stays until the game moved the player (its movement read may come at a lower rate than the
            // client ticks) or `HOLD_MAX` passed
            let held = run.key_since.map_or(Duration::ZERO, |since| since.elapsed());
            if run.ticks >= Self::HOLD_TICKS && (self.progress.seen(Step::PlayerMove) || held >= Self::HOLD_MAX) {
                run.key = None;
                run.key_since = None;
                self.key_held.store(false, Ordering::Release);
                let frames = run.ticks.to_string();
                log.info("key held", &[("key", &Self::key_text(Self::KEY)), ("frames", &frames)]);
                log.info("key hold time", &[("ms", &held.as_millis().to_string())]);
                self.progress.set(Step::KeyHeld);
                run.advance();
            }
            return;
        }
        let Some(lease) = run.lease.as_ref() else {
            return;
        };
        match lease.key_down(Self::KEY) {
            Ok(hold) => {
                run.key = Some(hold);
                run.key_since = Some(Instant::now());
                run.ticks = 0;
                self.key_held.store(true, Ordering::Release);
            }
            Err(ApiError::NotReady) if run.ticks < Self::WAIT_TICKS => {}
            Err(ApiError::NotAvailable) => {
                log.warn("key injection not available", &[("waived", "key_input,key_held,player_move")]);
                for step in [Step::KeyInput, Step::KeyHeld, Step::PlayerMove] {
                    self.progress.set(step);
                }
                run.advance();
            }
            Err(error) => {
                log.warn("key injection failed", &[("error", &error.to_string())]);
                run.advance();
            }
        }
    }

    fn move_intent(&self, run: &mut CheckRun, log: Logger) {
        let intent = MoveIntent { forward: 1.0, ..MoveIntent::default() };
        let done = self.inject(run, log, Step::MoveIntent, "move", |lease| lease.set_move(Some(intent)));
        if done && let Some(lease) = run.lease.as_ref() {
            let _ = lease.set_move(None);
        }
    }

    // Sends one injection on the shared lease (again while it gives `NotReady`), then waits for `step`.
    // True when the phase ended on this tick.
    fn inject(
        &self,
        run: &mut CheckRun,
        log: Logger,
        step: Step,
        what: &str,
        send: impl FnOnce(&amml_api::InputLease) -> Result<(), ApiError>,
    ) -> bool {
        let Some(lease) = run.lease.as_ref() else {
            run.advance();
            return true;
        };
        if !run.sent {
            match send(lease) {
                Ok(()) => {
                    run.sent = true;
                    run.ticks = 0;
                }
                Err(ApiError::NotReady) if run.ticks < Self::WAIT_TICKS => return false,
                Err(ApiError::NotReady) if step == Step::Wheel => {
                    self.waive_mouse(log, step);
                    run.advance();
                    return true;
                }
                Err(error) => {
                    log.warn("injection failed", &[("input", what), ("error", &error.to_string())]);
                    run.advance();
                    return true;
                }
            }
            return false;
        }
        if self.progress.seen(step) || run.ticks >= Self::WAIT_TICKS {
            if !self.progress.seen(step) {
                log.warn("injected event not seen", &[("input", what)]);
            }
            run.advance();
            return true;
        }
        false
    }

    // Sends `TURN_STEPS` equal turns spread over `TURN_SPREAD`, then holds the turned view for
    // `TURN_HOLD` once the listener saw the whole turn.
    fn turn(&self, run: &mut CheckRun, log: Logger) {
        let Some(lease) = run.lease.as_ref() else {
            run.advance();
            return;
        };
        let now = Instant::now();
        if run.turn_steps < Self::TURN_STEPS {
            let started = *run.turn_started.get_or_insert(now);
            let due = Self::TURN_SPREAD * run.turn_steps / (Self::TURN_STEPS - 1);
            if now.duration_since(started) < due {
                return;
            }
            let step = Look { yaw: Self::TURN_YAW / Self::TURN_STEPS as f32, pitch: 0.0 };
            match lease.turn(step) {
                Ok(()) => {
                    run.turn_steps += 1;
                    run.ticks = 0;
                }
                Err(ApiError::NotReady) if run.turn_steps == 0 && run.ticks < Self::WAIT_TICKS => {
                    run.turn_started = None;
                }
                Err(error) => {
                    log.warn("injection failed", &[("input", "turn"), ("error", &error.to_string())]);
                    run.advance();
                }
            }
            if run.turn_steps == Self::TURN_STEPS {
                run.turn_ended = Some(now);
                let spread = now.duration_since(started).as_millis().to_string();
                log.info("turn sent", &[("steps", &Self::TURN_STEPS.to_string()), ("yaw", "90")]);
                log.info("turn spread", &[("ms", &spread)]);
            }
            return;
        }
        let ended = *run.turn_ended.get_or_insert(now);
        let held = now.duration_since(ended);
        if self.progress.seen(Step::Turn) && held >= Self::TURN_HOLD {
            log.info("turn held", &[("ms", &held.as_millis().to_string())]);
            run.advance();
        } else if !self.progress.seen(Step::Turn) && run.ticks >= Self::WAIT_TICKS {
            log.warn("injected event not seen", &[("input", "turn")]);
            run.advance();
        }
    }

    // Holds `button` down for one client tick, then waits for `step`, which the `MouseInput` listener marks.
    fn click(&self, run: &mut CheckRun, log: Logger, button: MouseButton, step: Step) {
        if run.button.is_some() {
            run.button = None;
            run.ticks = 0;
            return;
        }
        let what = step.name();
        if run.sent {
            if self.progress.seen(step) || run.ticks >= Self::WAIT_TICKS {
                if !self.progress.seen(step) {
                    log.warn("injected event not seen", &[("input", what)]);
                }
                run.advance();
            }
            return;
        }
        let Some(lease) = run.lease.as_ref() else {
            run.advance();
            return;
        };
        match lease.button_down(button) {
            Ok(hold) => {
                run.button = Some(hold);
                run.sent = true;
            }
            Err(ApiError::NotReady) if run.ticks < Self::WAIT_TICKS => {}
            Err(ApiError::NotReady) => {
                self.waive_mouse(log, step);
                run.advance();
            }
            Err(error) => {
                log.warn("injection failed", &[("input", what), ("error", &error.to_string())]);
                run.advance();
            }
        }
    }

    // Mouse injection binds at the first real mouse feed, which needs a human at the mouse: posted window
    // messages do not reach the game's mouse feed. Without one, the wheel and click steps are waived
    // with a warning, as the key steps are on a build whose key feed is not the game thread.
    fn waive_mouse(&self, log: Logger, step: Step) {
        log.warn("mouse injection not ready", &[("waived", step.name()), ("reason", "no real mouse feed")]);
        self.progress.set(step);
    }

    fn gamepad(&self, run: &mut CheckRun, log: Logger, input: Input) {
        if run.full.is_none() && !run.sent {
            let request =
                LeaseRequest { mode: LeaseMode::Shared, blocks: InputBlocks::FULL, passthrough: &[], preempt: false };
            match input.acquire(request) {
                Ok(lease) => {
                    self.gamepad_skips.store(0, Ordering::Release);
                    run.full = Some(lease);
                    run.sent = true;
                    run.ticks = 0;
                }
                Err(error) => {
                    log.warn("lease refused", &[("blocks", "full"), ("error", &error.to_string())]);
                    run.advance();
                }
            }
            return;
        }
        let skips = self.gamepad_skips.load(Ordering::Acquire);
        if (run.ticks >= Self::GAMEPAD_TICKS && skips > 0) || run.ticks >= Self::WAIT_TICKS {
            run.full = None;
            if skips > 0 {
                self.progress.set(Step::GamepadSkipped);
                log.info("gamepad polls skipped", &[("count", &skips.to_string())]);
            } else {
                log.warn("gamepad polls not skipped", &[]);
            }
            run.advance();
        }
    }

    fn pause(&self, run: &mut CheckRun, log: Logger, client: &Client<'_>) {
        if !run.sent {
            if let Err(error) = client.open_pause_screen() {
                log.warn("pause not opened", &[("error", &error.to_string())]);
                run.advance();
                return;
            }
            run.sent = true;
            run.ticks = 0;
            return;
        }
        // The game's focus-lost path ends with its leaf `releaseMouse`, which the loader sees
        if run.ticks == 1 {
            log.info("mouse after pause call", &[("captured", Self::flag_text(client.mouse_captured()))]);
        }
        let screen = client.top_screen();
        if screen != Some(Screen::Pause) {
            if run.ticks >= Self::WAIT_TICKS {
                log.warn("pause not seen", &[("top_screen", Self::screen_text(screen))]);
                run.advance();
            }
            return;
        }
        log.info("mouse after pause", &[("captured", Self::flag_text(client.mouse_captured()))]);
        // A key held across the window close, so the world leave must release it
        if let Some(lease) = run.lease.as_ref()
            && let Ok(hold) = lease.key_down(Self::KEY)
        {
            run.key = Some(hold);
            log.info("key held across close", &[("key", &Self::key_text(Self::KEY))]);
        }
        log.info("pause opened", &[("top_screen", "Pause")]);
        self.progress.set(Step::PauseOpened);
        run.advance();
    }

    /// `KeyInput`: an injected down of the held key.
    pub fn key_input(&self, log: Logger, event: &KeyInput) {
        if event.is_injected() && event.key_code() == Self::KEY && event.down != 0 && self.progress.set(Step::KeyInput)
        {
            log.info("key input seen", &[("key", &Self::key_text(Self::KEY)), ("injected", "true")]);
        }
    }

    /// `PlayerMoveInput`: forward movement from the W hold, then the injected move intent.
    pub fn player_move(&self, log: Logger, event: &PlayerMoveInput) {
        let intent = event.intent();
        if !event.is_injected()
            && self.key_held.load(Ordering::Acquire)
            && intent.forward > 0.0
            && self.progress.set(Step::PlayerMove)
        {
            log.info("player move", &[("forward", &intent.forward.to_string())]);
        }
        if event.is_injected() && intent.forward == 1.0 && self.progress.set(Step::MoveIntent) {
            log.info("move intent seen", &[("forward", "1"), ("injected", "true")]);
        }
    }

    /// `LookDelta`: the injected turn steps, summed until they make the whole turn.
    pub fn look_delta(&self, log: Logger, event: &LookDelta) {
        let delta = event.delta();
        if !event.is_injected() {
            return;
        }
        let mut total = 0.0_f32;
        let _ = self.turn_seen.fetch_update(Ordering::AcqRel, Ordering::Acquire, |bits| {
            total = f32::from_bits(bits) + delta.yaw;
            Some(total.to_bits())
        });
        if (total - Self::TURN_YAW).abs() < 0.01 && self.progress.set(Step::Turn) {
            let pitch = delta.pitch.to_string();
            log.info("turn seen", &[("yaw", "90"), ("pitch", &pitch), ("injected", "true")]);
        }
    }

    /// `MouseInput`: the injected wheel notch and the left, X1 and X2 clicks.
    pub fn mouse_input(&self, log: Logger, event: &MouseInput) {
        if !event.is_injected() {
            return;
        }
        match event.action() {
            MouseAction::Wheel { notches } if notches == 1.0 && self.progress.set(Step::Wheel) => {
                log.info("wheel seen", &[("notches", "1"), ("injected", "true")]);
            }
            MouseAction::Button { button: MouseButton::Left, down: true } if self.progress.set(Step::Click) => {
                log.info("click seen", &[("button", "Left"), ("injected", "true")]);
            }
            MouseAction::Button { button: MouseButton::X1, down: true } if self.progress.set(Step::ClickX1) => {
                log.info("click seen", &[("button", "X1"), ("injected", "true")]);
            }
            MouseAction::Button { button: MouseButton::X2, down: true } if self.progress.set(Step::ClickX2) => {
                log.info("click seen", &[("button", "X2"), ("injected", "true")]);
            }
            _ => {}
        }
    }

    /// `GamepadPoll`: counts the polls the `FULL` lease skipped.
    pub fn gamepad_poll(&self, event: &GamepadPoll) {
        if event.is_cancelled() {
            self.gamepad_skips.fetch_add(1, Ordering::AcqRel);
        }
    }

    /// `InjectedInputReleased`: the reason, and whether the user holds the key after it.
    pub fn released(&self, log: Logger, input: Input, event: &InjectedInputReleased) {
        let is_down = Self::flag_text(input.is_down(Self::KEY));
        let lease = event.lease.to_string();
        log.info(
            "injected released",
            &[("reason", Self::reason_text(event.reason)), ("lease", &lease), ("w_is_down", is_down)],
        );
    }

    fn flag_text(value: Option<bool>) -> &'static str {
        match value {
            Some(true) => "true",
            Some(false) => "false",
            None => "none",
        }
    }

    fn screen_text(screen: Option<Screen>) -> &'static str {
        match screen {
            Some(Screen::Start) => "Start",
            Some(Screen::Pause) => "Pause",
            Some(_) => "Other",
            None => "none",
        }
    }

    fn key_text(key: KeyCode) -> String {
        format!("0x{:02X}", key.0)
    }

    fn reason_text(reason: AmmlReleaseReason) -> &'static str {
        match reason {
            AmmlReleaseReason::FOCUS_LOST => "FocusLost",
            AmmlReleaseReason::WORLD_LEFT => "WorldLeft",
            AmmlReleaseReason::SUSPENDED => "Suspended",
            AmmlReleaseReason::LEASE_ENDED => "LeaseEnded",
            AmmlReleaseReason::REPLACED => "Replaced",
            AmmlReleaseReason::DISABLED => "Disabled",
            AmmlReleaseReason::RESET => "Reset",
            _ => "Other",
        }
    }
}
