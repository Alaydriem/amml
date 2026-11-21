use std::time::{Duration, Instant};

use windows::Win32::Foundation::HWND;
use windows::Win32::System::Threading::GetCurrentProcessId;
use windows::Win32::UI::WindowsAndMessaging::{
    GetForegroundWindow, GetWindowThreadProcessId, IsWindow, SetForegroundWindow,
};

/// The loader's test switch that keeps a test game from taking the person's mouse and keyboard: when
/// `AMML_TEST_BACKGROUND` is `1`, the window that had the foreground when the loader started gets it back each time a
/// window of the game process takes it, during the first `WATCH` of the process. A foreground window takes the raw
/// mouse and the keys, so without this a test game started on another screen moves with the person's mouse. Windows
/// lets the foreground process give the foreground away, and the loader runs in the game process. Read by the
/// standalone host only; it is not offered to mods.
pub struct ForegroundHandback {
    enabled: bool,
    previous: usize,
    own_process: u32,
}

impl ForegroundHandback {
    pub const VARIABLE: &'static str = "AMML_TEST_BACKGROUND";
    /// How long after the start the game is watched: its window appears and the test world loads in this time.
    pub const WATCH: Duration = Duration::from_secs(180);
    const POLL: Duration = Duration::from_millis(100);

    /// `previous` is the window to give the foreground back to (0: none), `own_process` the game process id.
    pub fn new(enabled: bool, previous: usize, own_process: u32) -> Self {
        Self { enabled, previous, own_process }
    }

    /// From `AMML_TEST_BACKGROUND` (on only for `1`), the foreground window now and this process.
    pub fn from_env() -> Self {
        let enabled = std::env::var(Self::VARIABLE).is_ok_and(|value| value == "1");
        // SAFETY: both calls take no arguments and only read the window manager's and the process's state.
        let (previous, own_process) = unsafe { (GetForegroundWindow().0 as usize, GetCurrentProcessId()) };
        Self::new(enabled, previous, own_process)
    }

    /// The window to give the foreground to when `foreground`, owned by process `foreground_process`, has it: the
    /// saved window when the switch is on, a saved window exists and it is not the game's own; else `None`.
    pub fn hand_back_to(&self, foreground: usize, foreground_process: u32) -> Option<usize> {
        let ours = foreground != 0 && foreground_process == self.own_process && foreground != self.previous;
        (self.enabled && self.previous != 0 && ours).then_some(self.previous)
    }

    /// Starts the watch thread when the switch is on and a window to give back to exists; does nothing else.
    pub fn start(self) {
        if !self.enabled {
            return;
        }
        if self.previous == 0 {
            curia::warn!("test background start: no foreground window to give back to");
            return;
        }
        curia::info!("test background start: the game gives the foreground back", { window: format!("{:#x}", self.previous) });
        let started =
            std::thread::Builder::new().name("amml-foreground-handback".to_string()).spawn(move || self.watch());
        if let Err(error) = started {
            curia::warn!("test background start: the watch thread did not start", { error: error.to_string() });
        }
    }

    fn watch(self) {
        let until = Instant::now() + Self::WATCH;
        let mut handed = 0u32;
        while Instant::now() < until {
            // SAFETY: reads the foreground window and its owner; a window that is gone gives 0.
            let (foreground, process) = unsafe {
                let window = GetForegroundWindow();
                let mut process = 0u32;
                GetWindowThreadProcessId(window, Some(&raw mut process));
                (window.0 as usize, process)
            };
            if let Some(target) = self.hand_back_to(foreground, process) {
                let target = HWND(std::ptr::with_exposed_provenance_mut(target));
                // SAFETY: `target` is a window handle; IsWindow checks it still exists, and SetForegroundWindow
                // only asks the window manager to activate it.
                let given = unsafe { IsWindow(Some(target)).as_bool() && SetForegroundWindow(target).as_bool() };
                handed += 1;
                curia::info!("test background start: foreground given back", { given: given, count: handed });
                if !given {
                    return;
                }
            }
            std::thread::sleep(Self::POLL);
        }
    }
}
