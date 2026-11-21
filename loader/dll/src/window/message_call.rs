use amml_abi::{
    AmmlCursorShape, AmmlEventContext, AmmlMouseButton, AmmlPhase, AmmlPoint, AmmlPointerKind, AmmlThread,
    AmmlWindowChange, WindowChanged, WindowChar, WindowCursor, WindowFocus, WindowPointer, WindowSize,
};
use amml_runtime::FocusMessage;
use windows::Win32::Foundation::POINT;
use windows::Win32::UI::WindowsAndMessaging::{
    SIZE_MINIMIZED, SIZE_RESTORED, WA_INACTIVE, WHEEL_DELTA, WM_ACTIVATE, WM_KILLFOCUS, WM_SETFOCUS,
};

use super::{CursorShapes, ProcOutcome, SizeMessage, SurrogateJoin, WindowService};
use crate::{EventHub, LoaderState};

/// One message in the loader's window procedure: the events it raises and what the game's procedure gets.
pub struct MessageCall<'a> {
    state: &'a LoaderState,
    service: &'a WindowService,
    hwnd: usize,
    wparam: usize,
    lparam: isize,
}

impl<'a> MessageCall<'a> {
    // The high word of a `WM_XBUTTON*` `wParam` for the first X button.
    const XBUTTON1: usize = 1;
    const CANCELLED: ProcOutcome = ProcOutcome::Answer(0);

    pub fn new(state: &'a LoaderState, hwnd: usize, wparam: usize, lparam: isize) -> Self {
        Self { state, service: state.window(), hwnd, wparam, lparam }
    }

    fn hub(&self) -> &EventHub {
        self.state.events()
    }

    // Thread `GAME` when the window thread is the game thread, `WINDOW` otherwise.
    fn context(&self) -> AmmlEventContext {
        let hub = self.hub();
        let thread = if hub.gate().on_game_thread() { AmmlThread::GAME } else { AmmlThread::WINDOW };
        hub.context_on(thread)
    }

    // The client point of a mouse message; the low and high words are signed.
    pub fn client_point(&self) -> AmmlPoint {
        AmmlPoint { x: f32::from(self.lparam as i16), y: f32::from((self.lparam >> 16) as i16) }
    }

    pub fn x_button(&self) -> AmmlMouseButton {
        if (self.wparam >> 16) & 0xFFFF == Self::XBUTTON1 { AmmlMouseButton::X1 } else { AmmlMouseButton::X2 }
    }

    fn pointer_event(&self, kind: AmmlPointerKind, client: AmmlPoint, notches: f32) -> WindowPointer {
        WindowPointer {
            header: EventHub::header::<WindowPointer>(AmmlPhase::BEFORE),
            kind,
            button: AmmlMouseButton::LEFT,
            down: 0,
            _reserved: 0,
            modifiers: self.state.input().keys().modifiers(),
            _reserved2: 0,
            client,
            buffer: self.service.buffer_point(self.hwnd, client),
            notches,
        }
    }

    // A move, the wheel or a leave: a cancel keeps the message from the game.
    pub fn pointer(&self, kind: AmmlPointerKind, client: AmmlPoint, notches: f32) -> ProcOutcome {
        let mut event = self.pointer_event(kind, client, notches);
        self.hub().raise(&mut event, &self.context());
        if event.header.cancelled != 0 { Self::CANCELLED } else { ProcOutcome::Pass }
    }

    // `WM_MOUSEWHEEL` carries a screen point.
    pub fn wheel(&self) -> ProcOutcome {
        let screen = POINT { x: i32::from(self.lparam as i16), y: i32::from((self.lparam >> 16) as i16) };
        let client = WindowService::screen_to_client(self.hwnd, screen).unwrap_or(AmmlPoint { x: 0.0, y: 0.0 });
        let notches = f32::from((self.wparam >> 16) as u16 as i16) / WHEEL_DELTA as f32;
        self.pointer(AmmlPointerKind::WHEEL, client, notches)
    }

    // A down follows its cancel. An up whose down was cancelled is raised cancelled and held back; any other up
    // reaches the game whatever the listeners say, so the game never keeps a button down.
    pub fn button(&self, button: AmmlMouseButton, down: bool) -> ProcOutcome {
        let mut event = self.pointer_event(AmmlPointerKind::BUTTON, self.client_point(), 0.0);
        event.button = button;
        event.down = u8::from(down);
        let pairs = self.service.pairs();
        if down {
            self.hub().raise(&mut event, &self.context());
            let reaches = pairs.window_button(button, true, event.header.cancelled != 0);
            return if reaches { ProcOutcome::Pass } else { Self::CANCELLED };
        }
        if pairs.window_button(button, false, false) {
            self.hub().raise_fixed(&mut event, &self.context());
            ProcOutcome::Pass
        } else {
            event.header.cancelled = 1;
            self.hub().raise(&mut event, &self.context());
            Self::CANCELLED
        }
    }

    // A high surrogate waits for its low half and reaches the game with it; a low half with no high half passes
    // with no event.
    pub fn char(&self) -> ProcOutcome {
        let unit = (self.wparam & 0xFFFF) as u32;
        let surrogates = self.service.router().surrogates();
        let Some(ch) = surrogates.feed(unit) else {
            return if SurrogateJoin::is_high(unit) { Self::CANCELLED } else { ProcOutcome::Pass };
        };
        let mut event = WindowChar { header: EventHub::header::<WindowChar>(AmmlPhase::BEFORE), ch };
        self.hub().raise(&mut event, &self.context());
        if event.header.cancelled != 0 {
            return Self::CANCELLED;
        }
        match SurrogateJoin::high_of(ch) {
            Some(high) => ProcOutcome::PassPair { high: high as usize },
            None => ProcOutcome::Pass,
        }
    }

    // One event per message. A gain forgets held losses and ignores a cancel; a loss the loader replays is raised
    // with `replayed = 1` and passes; any other loss a listener cancels is held for `replay_focus_loss`. A loss that
    // passes releases the real keys the loader holds down, because the game resets its keys on it.
    pub fn focus(&self, message: u32) -> ProcOutcome {
        let gained = match message {
            WM_SETFOCUS => true,
            WM_KILLFOCUS => false,
            WM_ACTIVATE => (self.wparam & 0xFFFF) as u32 != WA_INACTIVE,
            _ => self.wparam != 0,
        };
        let mut event = WindowFocus {
            header: EventHub::header::<WindowFocus>(AmmlPhase::BEFORE),
            gained: u8::from(gained),
            replayed: 0,
            _reserved: [0; 2],
        };
        let focus = self.service.focus();
        if gained {
            focus.gain();
            self.hub().raise_fixed(&mut event, &self.context());
            return ProcOutcome::Pass;
        }
        let held = FocusMessage { msg: message, wparam: self.wparam, lparam: self.lparam };
        if focus.is_replay(&held) {
            event.replayed = 1;
            self.hub().raise_fixed(&mut event, &self.context());
            self.state.input().release_real_keys();
            return ProcOutcome::Pass;
        }
        self.service.router().surrogates().reset();
        self.hub().raise(&mut event, &self.context());
        if !focus.loss(held, event.header.cancelled != 0) {
            return Self::CANCELLED;
        }
        self.state.input().release_real_keys();
        ProcOutcome::Pass
    }

    // A size message the loader posted is raised injected for the mod it was posted for. The game gets the size
    // the listeners leave.
    pub fn size(&self) -> ProcOutcome {
        self.service.refresh_client(self.hwnd);
        let (width, height) = SizeMessage::size_of(self.lparam);
        let mut event = WindowSize {
            header: EventHub::header::<WindowSize>(AmmlPhase::BEFORE),
            minimised: u8::from(self.wparam == SIZE_MINIMIZED as usize),
            _reserved: [0; 3],
            width,
            height,
        };
        let posted =
            if self.wparam == SIZE_RESTORED as usize { self.service.sizes().take_posted(self.lparam) } else { None };
        match posted {
            Some(owner) => self.hub().raise_injected(&mut event, &self.context(), owner),
            None => self.hub().raise(&mut event, &self.context()),
        };
        let written = SizeMessage::size_param(event.width, event.height);
        if written == self.lparam { ProcOutcome::Pass } else { ProcOutcome::PassWith { lparam: written } }
    }

    pub fn changed(&self, kind: AmmlWindowChange) -> ProcOutcome {
        let mut event =
            WindowChanged { header: EventHub::header::<WindowChanged>(AmmlPhase::BEFORE), kind, _reserved: [0; 3] };
        self.hub().raise(&mut event, &self.context());
        ProcOutcome::Pass
    }

    // A shape a listener set is applied and answers the message (1); the game then does not set its own.
    pub fn cursor(&self) -> ProcOutcome {
        let mut event = WindowCursor {
            header: EventHub::header::<WindowCursor>(AmmlPhase::BEFORE),
            shape: AmmlCursorShape::NONE,
            _reserved: [0; 3],
        };
        self.hub().raise(&mut event, &self.context());
        if event.shape != AmmlCursorShape::NONE && CursorShapes::apply(event.shape) {
            ProcOutcome::Answer(1)
        } else {
            ProcOutcome::Pass
        }
    }
}
