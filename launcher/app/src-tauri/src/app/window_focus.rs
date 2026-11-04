use tauri::{AppHandle, Manager};

// Brings the one window to the front.
pub struct WindowFocus;

impl WindowFocus {
    pub fn main(app: &AppHandle) {
        match app.get_webview_window("main") {
            Some(window) => {
                let _ = window.unminimize();
                let _ = window.show();
                let _ = window.set_focus();
            }
            None => curia::warn!("a second start found no window to bring forward"),
        }
    }
}
