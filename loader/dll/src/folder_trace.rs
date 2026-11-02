use std::sync::Mutex;

use windows_sys::core::{GUID, PCWSTR};

use crate::FolderCall;

// Distinct (call, detail) pairs already logged.
static SEEN: Mutex<Vec<String>> = Mutex::new(Vec::new());

/// Logs each distinct folder request the game makes, to find out which functions a game build
/// reaches its data folder through. Compiled only with the `folder-trace` feature.
pub struct FolderTrace;

impl FolderTrace {
    // Most distinct requests kept; later ones are not logged.
    const LIMIT: usize = 128;
    // Longest name read from a pointer.
    const NAME_LIMIT: usize = 64;

    pub fn known_folder(id: &GUID) {
        let detail = format!(
            "{:08x}-{:04x}-{:04x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
            id.data1,
            id.data2,
            id.data3,
            id.data4[0],
            id.data4[1],
            id.data4[2],
            id.data4[3],
            id.data4[4],
            id.data4[5],
            id.data4[6],
            id.data4[7]
        );
        Self::note(FolderCall::KnownFolder, detail);
    }

    pub fn folder_path(csidl: i32) {
        Self::note(FolderCall::FolderPath, format!("csidl {csidl:#x}"));
    }

    /// A call that has no argument worth logging.
    pub fn plain(call: FolderCall) {
        Self::note(call, String::new());
    }

    /// A call with a NUL-terminated UTF-16 name argument.
    ///
    /// `name` is null or NUL-terminated.
    pub fn wide(call: FolderCall, name: PCWSTR) {
        if name.is_null() {
            return;
        }
        let mut units = Vec::new();
        for position in 0..Self::NAME_LIMIT {
            // SAFETY: `name` is NUL-terminated and the walk stops at the terminator.
            let unit = unsafe { *name.add(position) };
            if unit == 0 {
                break;
            }
            units.push(unit);
        }
        Self::note(call, String::from_utf16_lossy(&units));
    }

    /// A call with a NUL-terminated narrow name argument.
    ///
    /// `name` is null or NUL-terminated.
    pub fn narrow(call: FolderCall, name: *const u8) {
        if name.is_null() {
            return;
        }
        let mut bytes = Vec::new();
        for position in 0..Self::NAME_LIMIT {
            // SAFETY: `name` is NUL-terminated and the walk stops at the terminator.
            let byte = unsafe { *name.add(position) };
            if byte == 0 {
                break;
            }
            bytes.push(byte);
        }
        Self::note(call, String::from_utf8_lossy(&bytes).into_owned());
    }

    fn note(call: FolderCall, detail: String) {
        let key = format!("{}:{detail}", call.function());
        {
            let Ok(mut seen) = SEEN.lock() else {
                return;
            };
            if seen.len() >= Self::LIMIT || seen.contains(&key) {
                return;
            }
            seen.push(key);
        }
        curia::info!("folder requested", { call: call.function(), detail: detail });
    }
}
