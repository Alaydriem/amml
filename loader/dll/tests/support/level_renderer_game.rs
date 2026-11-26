use std::ffi::c_void;
use std::hint::black_box;
use std::sync::Mutex;

use amml_data::{EntryKind, Resolved};

use super::{ClientGame, Harness};

// Each stand-in call as a line, and the lines the tests add.
static CALLS: Mutex<Vec<String>> = Mutex::new(Vec::new());

/// Stand-ins with the game signatures for the two `LevelRendererPlayer` functions the level renderer
/// adapter hooks: `getFov` (gives 70 with variable FOV on and 60 with it off, and records `fov variable=..`)
/// and `onWillChangeDimension` (records `game`).
pub struct LevelRendererGame;

impl LevelRendererGame {
    pub const GET_FOV: &'static str = "LevelRendererPlayer.getFov";
    pub const WILL_CHANGE_DIMENSION: &'static str = "LevelRendererPlayer.onWillChangeDimension";
    /// The `LevelRendererPlayer` and the player the tests pass.
    const RENDERER: usize = 0x7100;
    const PLAYER: usize = 0x7200;

    /// The client entries and the two hooked functions pointing to the stand-ins; `without` names entries
    /// to leave out.
    pub fn entries(without: &[&str]) -> Vec<(&'static str, Resolved)> {
        let mut entries = ClientGame::entries(&[]);
        entries.extend([
            (Self::GET_FOV, Self::function(Self::get_fov as *const ())),
            (Self::WILL_CHANGE_DIMENSION, Self::function(Self::will_change_dimension_raw as *const ())),
        ]);
        entries.into_iter().filter(|(id, _)| !without.contains(id)).collect()
    }

    /// Calls (through its address) `getFov(renderer, a, variable)`.
    pub fn fov(a: f32, variable: bool) -> f32 {
        let function = black_box(Self::get_fov as extern "C" fn(*mut c_void, f32, u8) -> f32);
        function(black_box(Self::RENDERER as *mut c_void), black_box(a), black_box(u8::from(variable)))
    }

    /// Calls (through its address) `onWillChangeDimension(renderer, player)`.
    pub fn will_change_dimension() {
        let function = black_box(Self::will_change_dimension_raw as extern "C" fn(*mut c_void, *mut c_void));
        function(black_box(Self::RENDERER as *mut c_void), black_box(Self::PLAYER as *mut c_void));
    }

    /// The `getFov` stand-in calls so far.
    pub fn fov_calls() -> usize {
        CALLS.lock().unwrap().iter().filter(|line| line.starts_with("fov")).count()
    }

    /// The stand-in calls and the lines the tests added, in order.
    pub fn calls() -> Vec<String> {
        CALLS.lock().unwrap().clone()
    }

    /// Adds `line` to the call list.
    pub fn record(line: &str) {
        CALLS.lock().unwrap().push(line.to_string());
    }

    fn function(address: *const ()) -> Resolved {
        Resolved { kind: EntryKind::Function, value: address as usize as u64 - Harness::BASE, recipes_agreeing: 2 }
    }

    #[inline(never)]
    extern "C" fn get_fov(this: *mut c_void, a: f32, variable: u8) -> f32 {
        let variable = black_box(variable) != 0;
        Self::record(&format!("fov this={:#x} a={} variable={variable}", black_box(this) as usize, black_box(a)));
        if variable { 70.0 } else { 60.0 }
    }

    #[inline(never)]
    extern "C" fn will_change_dimension_raw(this: *mut c_void, player: *mut c_void) {
        black_box((this, player));
        Self::record("game");
    }
}
