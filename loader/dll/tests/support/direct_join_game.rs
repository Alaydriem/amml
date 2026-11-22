use std::hint::black_box;
use std::sync::Mutex;

use amml_data::{EntryKind, Resolved};
use amml_msvc::MsvcString;

use super::Harness;

// What the stand-in `connectByAddress` received: (this, ip text, port).
type Connected = (usize, String, i32);
static CONNECTED: Mutex<Vec<Connected>> = Mutex::new(Vec::new());

/// The stand-in for the direct server join: `ClientInstance.connectByAddress`, which records its arguments.
pub struct DirectJoinGame;

impl DirectJoinGame {
    /// The entries of `client.join_server_direct`, pointing to the stand-in, with the subobject offset `this_offset`.
    pub fn entries_at(this_offset: u64) -> Vec<(&'static str, Resolved)> {
        vec![
            (
                "ClientInstance.connectByAddress",
                Resolved {
                    kind: EntryKind::Function,
                    value: Self::connect as *const () as usize as u64 - Harness::BASE,
                    recipes_agreeing: 2,
                },
            ),
            (
                "ClientInstance.connectByAddressThis",
                Resolved { kind: EntryKind::Field, value: this_offset, recipes_agreeing: 1 },
            ),
        ]
    }

    /// `entries_at(0)` added to `base`.
    pub fn with_entries(base: Vec<(&'static str, Resolved)>) -> Vec<(&'static str, Resolved)> {
        let mut entries = base;
        entries.extend(Self::entries_at(0));
        entries
    }

    /// Every `connectByAddress` call: (this, ip, port).
    pub fn connected() -> Vec<Connected> {
        CONNECTED.lock().unwrap().clone()
    }

    /// `void ClientInstance.connectByAddress(const std::string& ip, int port)`: records its arguments.
    #[inline(never)]
    pub extern "C" fn connect(this: *mut u8, ip: *const MsvcString, port: i32) {
        black_box(this);
        // SAFETY: the loader passes a live MSVC string for the call.
        let text = unsafe { &*ip }.to_str().unwrap_or("<not utf-8>").to_string();
        CONNECTED.lock().unwrap().push((this as usize, text, port));
    }
}
