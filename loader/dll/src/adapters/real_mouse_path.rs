use std::sync::atomic::{AtomicU8, AtomicU32, Ordering};

use amml_abi::{
    AmmlEventContext, AmmlInputBlocks, AmmlMouseAction, AmmlMouseButton, AmmlPhase, AmmlStatus, AmmlThread, MouseInput,
};

use crate::{EventHub, InputService, LoaderState};

/// What both mouse adapters do with one real `MouseDevice::feed` call: the `MouseInput` event, the
/// `MOUSE_BUTTONS` block of the active leases, the button table, the last pointer position, and the
/// thread rule of mouse injection.
///
/// The thread rule is `GAME_THREAD` from the first frame on: injection runs on the game thread, and the mouse
/// feed has been the game thread on every build measured. The first real feed then checks it: a real feed off the
/// game thread withdraws mouse injection for the rest of the process. No real mouse event is needed for injection
/// to be ready.
///
/// Only feeds on the `MouseDevice.instance` device go through the loader; the game's per-pointer devices
/// (touch and pen) pass unchanged. The thread rule: the first real feed that runs after the game thread is
/// known decides whether the mouse feed is the game thread. When it is not, mouse injection is not
/// available for the rest of the process and the `input.mouse_inject` area is withdrawn.
pub struct RealMousePath {
    state: &'static LoaderState,
    instance: usize,
    thread: AtomicU8,
    // The x of the last real feed in the low 16 bits, its y in the high 16 bits.
    position: AtomicU32,
    // Per button: no down seen, the last down reached the game, or the last down was cancelled.
    presses: [AtomicU8; 5],
}

impl RealMousePath {
    /// The `actionButtonId` of a pointer move.
    pub const MOVE: u8 = 0;
    /// The `actionButtonId` of the left button; right is 2 and middle 3.
    pub const LEFT: u8 = 1;
    pub const RIGHT: u8 = 2;
    pub const MIDDLE: u8 = 3;
    /// The `actionButtonId` of the wheel; `buttonData` is the signed wheel delta.
    pub const WHEEL: u8 = 4;
    /// The `actionButtonId` of the X1 button. The window procedure feeds `6 - (HIWORD(wParam) == XBUTTON1)`
    /// for `WM_XBUTTONDOWN` and `WM_XBUTTONUP` (26.63 at 0x98ad6), so X1 is 5 and X2 is 6.
    pub const X1: u8 = 5;
    pub const X2: u8 = 6;
    /// The `buttonData` of one wheel notch away from the user: the Windows `WHEEL_DELTA`.
    pub const WHEEL_NOTCH: i8 = 120;

    const MOUSE_INJECT: &'static str = "input.mouse_inject";
    const UNDECIDED: u8 = 0;
    const GAME_THREAD: u8 = 1;
    const OTHER_THREAD: u8 = 2;
    const NOT_SEEN: u8 = 0;
    const PASSED: u8 = 1;
    const CANCELLED: u8 = 2;

    /// The path of the device at `instance` (the `MouseDevice.instance` address).
    pub fn new(state: &'static LoaderState, instance: usize) -> Self {
        Self {
            state,
            instance,
            thread: AtomicU8::new(Self::UNDECIDED),
            position: AtomicU32::new(0),
            presses: std::array::from_fn(|_| AtomicU8::new(Self::NOT_SEEN)),
        }
    }

    /// The device injected input is fed to.
    pub fn instance(&self) -> usize {
        self.instance
    }

    /// The pointer position of the last real feed on the device that carries one (every feed but a raw
    /// input move, which carries a delta); 0, 0 before one.
    pub fn last_position(&self) -> (i16, i16) {
        let packed = self.position.load(Ordering::Acquire);
        (packed as u16 as i16, (packed >> 16) as u16 as i16)
    }

    /// The game's `actionButtonId` of `button`; `None` for a button the game does not have.
    pub fn action_of(button: AmmlMouseButton) -> Option<u8> {
        match button {
            AmmlMouseButton::LEFT => Some(Self::LEFT),
            AmmlMouseButton::RIGHT => Some(Self::RIGHT),
            AmmlMouseButton::MIDDLE => Some(Self::MIDDLE),
            AmmlMouseButton::X1 => Some(Self::X1),
            AmmlMouseButton::X2 => Some(Self::X2),
            _ => None,
        }
    }

    /// The button of the game's `actionButtonId`; `None` for a move, the wheel or an unknown id.
    pub fn button_of(action: u8) -> Option<AmmlMouseButton> {
        match action {
            Self::LEFT => Some(AmmlMouseButton::LEFT),
            Self::RIGHT => Some(AmmlMouseButton::RIGHT),
            Self::MIDDLE => Some(AmmlMouseButton::MIDDLE),
            Self::X1 => Some(AmmlMouseButton::X1),
            Self::X2 => Some(AmmlMouseButton::X2),
            _ => None,
        }
    }

    /// The whole notches in `notches`, either way, at most `InputService::MAX_WHEEL_NOTCHES`.
    pub fn whole_notches(notches: f32) -> u32 {
        let whole = notches.abs().trunc();
        if whole.is_finite() { (whole as u32).min(InputService::MAX_WHEEL_NOTCHES) } else { 0 }
    }

    /// Runs one real feed of `this` through the loader. Gives the pointer position the game gets, or
    /// `None` when the game must not get the feed.
    ///
    /// A move, a button and the wheel raise `MouseInput` (`x` and `y` writable). A button that an active
    /// `MOUSE_BUTTONS` lease blocks, or whose window message a listener cancelled, is raised already
    /// cancelled; moves and the wheel are never blocked.
    /// An up whose down was cancelled is raised cancelled and held back. An up whose down reached the game
    /// reaches it too, whatever the listeners say, and an up whose down the loader never saw reaches the
    /// game with no event, so the game never keeps a button down. An unknown action passes with no event.
    #[allow(clippy::too_many_arguments)]
    pub fn feed(&self, this: usize, action: u8, data: i8, x: i16, y: i16, dx: i16, dy: i16) -> Option<(i16, i16)> {
        if this != self.instance {
            return Some((x, y));
        }
        let on_game_thread = self.state.events().gate().on_game_thread();
        self.decide(on_game_thread);
        // A raw input move carries only a delta, with x and y 0; it does not move the kept position.
        if action != Self::MOVE || (dx == 0 && dy == 0) {
            self.position.store(u32::from(x as u16) | (u32::from(y as u16) << 16), Ordering::Release);
        }
        let mut event = MouseInput {
            header: EventHub::header::<MouseInput>(AmmlPhase::BEFORE),
            action: AmmlMouseAction::MOVE,
            button: AmmlMouseButton::LEFT,
            down: 0,
            _reserved: 0,
            x: i32::from(x),
            y: i32::from(y),
            dx: i32::from(dx),
            dy: i32::from(dy),
            notches: 0.0,
        };
        let mut press = None;
        match action {
            Self::MOVE => {}
            Self::WHEEL => {
                event.action = AmmlMouseAction::WHEEL;
                event.notches = f32::from(data) / f32::from(Self::WHEEL_NOTCH);
            }
            _ => {
                let Some((button, slot)) = Self::button_of(action)
                    .and_then(|button| self.presses.get(usize::from(button.0)).map(|slot| (button, slot)))
                else {
                    return Some((x, y));
                };
                let down = data != 0;
                event.action = AmmlMouseAction::BUTTON;
                event.button = button;
                event.down = u8::from(down);
                let earlier = if down { Self::NOT_SEEN } else { slot.swap(Self::NOT_SEEN, Ordering::AcqRel) };
                // A down whose window message a listener cancelled is cancelled here too; an up only ends the pair.
                let window_passed = self.state.window().pairs().feed_button(button, down);
                if !down && earlier == Self::NOT_SEEN {
                    self.state.input().real_button(button, false);
                    return Some((x, y));
                }
                let blocked = self.state.input().book().blocks_now().0 & AmmlInputBlocks::MOUSE_BUTTONS.0 != 0;
                event.header.cancelled = u8::from(blocked || earlier == Self::CANCELLED || (down && !window_passed));
                press = Some((button, down, earlier, slot));
            }
        }
        let hub = self.state.events();
        let thread = if on_game_thread { AmmlThread::GAME } else { AmmlThread::INPUT };
        let context = AmmlEventContext { thread, ..hub.loader_context() };
        hub.raise(&mut event, &context);
        let position = (Self::clamp(event.x), Self::clamp(event.y));
        let cancelled = event.header.cancelled != 0;
        let Some((button, down, earlier, slot)) = press else {
            return if cancelled { None } else { Some(position) };
        };
        let reaches = if down {
            slot.store(if cancelled { Self::CANCELLED } else { Self::PASSED }, Ordering::Release);
            !cancelled
        } else {
            earlier == Self::PASSED
        };
        if !reaches {
            return None;
        }
        self.state.input().real_button(button, down);
        Some(position)
    }

    /// `NOT_AVAILABLE` when a real feed ran off the game thread, `Ok` otherwise.
    pub fn thread_allows(&self) -> Result<(), AmmlStatus> {
        if self.thread.load(Ordering::Acquire) == Self::OTHER_THREAD { Err(AmmlStatus::NOT_AVAILABLE) } else { Ok(()) }
    }

    /// `Ok` when a button or wheel may be injected: the thread rule has not withdrawn the area, and the first
    /// frame has run. The first call after the gate opens takes `GAME_THREAD` as the rule when no real feed has
    /// decided it yet. `NOT_READY` before the first frame, `NOT_AVAILABLE` after a real feed off the game thread.
    pub fn ready(&self) -> Result<(), AmmlStatus> {
        self.thread_allows()?;
        if self.decided() {
            return Ok(());
        }
        if !self.state.events().gate().is_open() {
            return Err(AmmlStatus::NOT_READY);
        }
        if self.thread.compare_exchange(Self::UNDECIDED, Self::GAME_THREAD, Ordering::AcqRel, Ordering::Acquire).is_ok()
        {
            curia::info!("mouse feed thread is taken as the game thread");
        }
        self.thread_allows()
    }

    /// Whether a real feed has decided the thread rule.
    pub fn decided(&self) -> bool {
        self.thread.load(Ordering::Acquire) != Self::UNDECIDED
    }

    fn clamp(value: i32) -> i16 {
        value.clamp(i32::from(i16::MIN), i32::from(i16::MAX)) as i16
    }

    // Decides the thread rule at the first feed once the game thread is known (the gate opens at the first
    // frame); before that, nothing is decided.
    // A real feed off the game thread after the rule said "game thread" withdraws mouse injection as well.
    fn decide(&self, on_game_thread: bool) {
        if !self.state.events().gate().is_open() {
            return;
        }
        if !on_game_thread
            && self
                .thread
                .compare_exchange(Self::GAME_THREAD, Self::OTHER_THREAD, Ordering::AcqRel, Ordering::Acquire)
                .is_ok()
        {
            curia::warn!("mouse injection not available: the mouse feed is not the game thread");
            self.state.areas().withdraw(Self::MOUSE_INJECT);
            return;
        }
        if self.decided() {
            return;
        }
        let value = if on_game_thread { Self::GAME_THREAD } else { Self::OTHER_THREAD };
        if self.thread.compare_exchange(Self::UNDECIDED, value, Ordering::AcqRel, Ordering::Acquire).is_err() {
            return;
        }
        if on_game_thread {
            curia::info!("mouse feed thread is the game thread");
        } else {
            curia::warn!("mouse injection not available: the mouse feed is not the game thread");
            self.state.areas().withdraw(Self::MOUSE_INJECT);
        }
    }
}
