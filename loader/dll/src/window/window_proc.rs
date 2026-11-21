use std::panic::AssertUnwindSafe;
use std::time::{Duration, Instant};

use amml_abi::{AmmlMouseButton, AmmlPoint, AmmlPointerKind, AmmlWindowChange};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    DefWindowProcW, HTCLIENT, WM_ACTIVATE, WM_ACTIVATEAPP, WM_CHAR, WM_CLOSE, WM_DESTROY, WM_DISPLAYCHANGE,
    WM_DPICHANGED, WM_ENDSESSION, WM_KILLFOCUS, WM_LBUTTONDBLCLK, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MBUTTONDBLCLK,
    WM_MBUTTONDOWN, WM_MBUTTONUP, WM_MOUSEMOVE, WM_MOUSEWHEEL, WM_NCDESTROY, WM_RBUTTONDBLCLK, WM_RBUTTONDOWN,
    WM_RBUTTONUP, WM_SETCURSOR, WM_SETFOCUS, WM_SIZE, WM_WINDOWPOSCHANGED, WM_XBUTTONDBLCLK, WM_XBUTTONDOWN,
    WM_XBUTTONUP,
};

use super::{MessageCall, ProcOutcome, WindowService};
use crate::{LoaderState, ModLoader, ProcessorKeyFeed, SessionAdapter};

/// The loader's window procedure on the game window. It raises the `window.*` events before the game's own
/// procedure runs, and passes each message on unless a listener cancelled it (the caller then gets 0).
///
/// A panic in the loader's routing is caught: it is logged once, routing stays off for the rest of the process,
/// and the message goes to the game's procedure unchanged.
///
/// `WM_NCDESTROY` makes the window service forget the window (`WindowService::detach`), also while routing is off.
///
/// `WM_CLOSE` and `WM_ENDSESSION` (ending) run the live shutdown (`ModLoader::shutdown_live`) before the game's
/// procedure gets the message, also while routing is off. The game's own quit (the title screen and pause menu
/// buttons) destroys the window without a `WM_CLOSE`: `WM_DESTROY` of the attached window, and `WM_NCDESTROY` before
/// the window is forgotten, run the same shutdown when it has not run and the loader is not closing yet. Game actions run only on the game thread: when the window
/// thread is not the game thread, the shutdown is queued for the next `FrameUpdate` and the window thread waits for
/// it at most `SHUTDOWN_WAIT` before it passes the message on.
pub struct WindowProc;

impl WindowProc {
    // `WM_MOUSELEAVE`; the `windows` crate keeps it with the common controls.
    const WM_MOUSELEAVE: u32 = 0x02A3;
    /// How long the window thread waits for a shutdown it queued for the game thread.
    pub const SHUTDOWN_WAIT: Duration = Duration::from_millis(500);

    /// The address of the procedure, for `SetWindowLongPtrW`.
    pub fn address() -> usize {
        Self::window_proc as *const () as usize
    }

    unsafe extern "system" fn window_proc(hwnd: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        let Some(state) = LoaderState::get() else {
            // SAFETY: the arguments of a live message.
            return unsafe { DefWindowProcW(hwnd, message, wparam, lparam) };
        };
        let router = state.window().router();
        let window = hwnd.0.expose_provenance();
        let routed =
            std::panic::catch_unwind(AssertUnwindSafe(|| Self::route(state, window, message, wparam.0, lparam.0)));
        let outcome = routed.unwrap_or_else(|_| {
            if router.disable() {
                curia::error!("game window routing panicked, window events are off for this process");
            }
            ProcOutcome::Pass
        });
        match outcome {
            ProcOutcome::Pass => router.pass_on(window, message, wparam.0, lparam.0),
            ProcOutcome::PassWith { lparam } => router.pass_on(window, message, wparam.0, lparam),
            ProcOutcome::PassPair { high } => {
                router.pass_on(window, WM_CHAR, high, lparam.0);
                router.pass_on(window, message, wparam.0, lparam.0)
            }
            ProcOutcome::Answer(result) => LRESULT(result),
        }
    }

    fn route(state: &'static LoaderState, hwnd: usize, message: u32, wparam: usize, lparam: isize) -> ProcOutcome {
        let service = state.window();
        if service.router().first_call() {
            let game_thread = state.events().gate().on_game_thread();
            curia::info!("game window procedure runs", { game_thread: game_thread });
            if game_thread {
                ProcessorKeyFeed::with((), ProcessorKeyFeed::window_on_game_thread);
            }
        }
        if message == WM_CLOSE || (message == WM_ENDSESSION && wparam != 0) {
            Self::close(state);
            return ProcOutcome::Pass;
        }
        if message == WM_DESTROY {
            Self::destroyed(state, hwnd, "destroy");
            return ProcOutcome::Pass;
        }
        if message == WM_NCDESTROY {
            Self::destroyed(state, hwnd, "nc_destroy");
            service.detach(hwnd);
            return ProcOutcome::Pass;
        }
        if !service.router().active() {
            return ProcOutcome::Pass;
        }
        let call = MessageCall::new(state, hwnd, wparam, lparam);
        match message {
            WM_MOUSEMOVE => call.pointer(AmmlPointerKind::MOVE, call.client_point(), 0.0),
            WM_LBUTTONDOWN | WM_LBUTTONDBLCLK => call.button(AmmlMouseButton::LEFT, true),
            WM_LBUTTONUP => call.button(AmmlMouseButton::LEFT, false),
            WM_RBUTTONDOWN | WM_RBUTTONDBLCLK => call.button(AmmlMouseButton::RIGHT, true),
            WM_RBUTTONUP => call.button(AmmlMouseButton::RIGHT, false),
            WM_MBUTTONDOWN | WM_MBUTTONDBLCLK => call.button(AmmlMouseButton::MIDDLE, true),
            WM_MBUTTONUP => call.button(AmmlMouseButton::MIDDLE, false),
            WM_XBUTTONDOWN | WM_XBUTTONDBLCLK => call.button(call.x_button(), true),
            WM_XBUTTONUP => call.button(call.x_button(), false),
            WM_MOUSEWHEEL => call.wheel(),
            Self::WM_MOUSELEAVE => {
                let client = WindowService::cursor_in(hwnd).unwrap_or(AmmlPoint { x: 0.0, y: 0.0 });
                call.pointer(AmmlPointerKind::LEAVE, client, 0.0)
            }
            WM_CHAR => call.char(),
            WM_SETFOCUS | WM_KILLFOCUS | WM_ACTIVATE | WM_ACTIVATEAPP => call.focus(message),
            WM_SIZE => call.size(),
            WM_DPICHANGED => call.changed(AmmlWindowChange::DPI),
            WM_DISPLAYCHANGE => call.changed(AmmlWindowChange::DISPLAY),
            WM_WINDOWPOSCHANGED => call.changed(AmmlWindowChange::MOVED),
            WM_SETCURSOR if (lparam & 0xFFFF) as u32 == HTCLIENT => call.cursor(),
            _ => ProcOutcome::Pass,
        }
    }

    // The attached window is destroyed: the shutdown runs as for a close, unless it ran, runs, or the window thread
    // passed a close on already.
    fn destroyed(state: &'static LoaderState, hwnd: usize, message: &'static str) {
        if state.window().hwnd() != Some(hwnd) || state.shutdown().is_closing() {
            return;
        }
        let game_thread = state.events().gate().on_game_thread();
        curia::info!("game window destroyed without a close, the loader shuts down", {
            message: message,
            game_thread: game_thread
        });
        Self::close(state);
    }

    // On the game thread the shutdown runs now; on any other thread it is queued for the next frame and waited
    // for, within `SHUTDOWN_WAIT`.
    fn close(state: &'static LoaderState) {
        if state.events().gate().on_game_thread() {
            ModLoader::new(state).shutdown_live(SessionAdapter::primary_client().unwrap_or(0));
            return;
        }
        let latch = state.shutdown();
        if latch.wait_done(Duration::ZERO) {
            return;
        }
        if latch.queue() {
            curia::info!("shutdown queued: the window thread is not the game thread");
        }
        let started = Instant::now();
        let finished = latch.wait_done(Self::SHUTDOWN_WAIT);
        let waited_ms = started.elapsed().as_millis() as u64;
        if finished {
            curia::info!("queued shutdown finished", { waited_ms: waited_ms });
        } else {
            latch.pass_on();
            curia::warn!("queued shutdown did not finish in time, the window close goes on and the loader counts as closing", {
                waited_ms: waited_ms,
                limit_ms: Self::SHUTDOWN_WAIT.as_millis() as u64
            });
        }
    }
}
