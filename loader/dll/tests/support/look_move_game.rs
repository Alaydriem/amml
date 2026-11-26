use std::ffi::c_void;
use std::hint::black_box;
use std::sync::Mutex;

use amml_data::{EntryKind, Resolved};

use super::{ClientGame, Harness};

// Each stand-in call as a line.
static CALLS: Mutex<Vec<String>> = Mutex::new(Vec::new());
// What the stand-in `extractRawHIDInput` writes into the component, as the game's own keyboard fill.
static FILL: Mutex<[u8; 0x18]> = Mutex::new([0; 0x18]);

/// Stand-ins with the game signatures for the functions the look, move and gamepad adapters hook:
/// `LocalPlayer::_applyTurnDelta` (records `turn this=.. x=.. y=..`), `ClientInputUpdateSystem::
/// extractRawHIDInput` (records `extract ..` and fills the component with `set_fill`) and
/// `GameControllerHandler_GameCore::refresh` (records `refresh this=..`).
pub struct LookMoveGame;

impl LookMoveGame {
    /// The 26.63 values of the ten move layout entries.
    pub const LAYOUT: [(&'static str, u64); 10] = [
        ("RawMoveInputComponent.mInputState", 0x0),
        ("RawMoveInputComponent.mRawMove", 0x10),
        ("MoveInputState.mAnalogMoveVector", 0x4),
        ("MoveInputState.maskSneakTest", 0x40001),
        ("MoveInputState.maskJumpDown", 0x80),
        ("MoveInputState.bitSprintDown", 8),
        ("MoveInputState.bitUp", 13),
        ("MoveInputState.bitDown", 14),
        ("MoveInputState.bitLeft", 15),
        ("MoveInputState.bitRight", 16),
    ];
    /// The gamepad handler the tests pass as `this`.
    pub const HANDLER: usize = 0x9A_0000;

    /// The client entries, the three hooked functions pointing to the stand-ins, and the move layout
    /// with `layout` in place of the matching 26.63 values.
    pub fn entries(layout: &[(&'static str, u64)]) -> Vec<(&'static str, Resolved)> {
        let mut entries = ClientGame::entries(&[]);
        entries.extend([
            ("LocalPlayer.applyTurnDelta", Self::function(Self::apply_turn_delta as *const ())),
            ("ClientInputUpdateSystem.extractRawHIDInput", Self::function(Self::extract_raw_hid_input as *const ())),
            ("GameControllerHandler_GameCore.refresh", Self::function(Self::refresh as *const ())),
        ]);
        for (name, value) in Self::LAYOUT {
            let value = layout.iter().find(|(changed, _)| *changed == name).map_or(value, |(_, value)| *value);
            entries.push((name, Resolved { kind: EntryKind::Field, value, recipes_agreeing: 2 }));
        }
        entries
    }

    /// Calls (through its address) `_applyTurnDelta(player, delta)`.
    pub fn turn(player: usize, delta: &[f32; 2]) {
        let function = black_box(Self::apply_turn_delta as extern "C" fn(*mut c_void, *const [f32; 2]));
        function(black_box(player as *mut c_void), black_box(std::ptr::from_ref(delta)));
    }

    /// Calls (through its address) `extractRawHIDInput(1, 2, 3, raw, 5, 6)`.
    pub fn extract(raw: &mut [u8; 0x18]) {
        let function = black_box(Self::extract_raw_hid_input as extern "C" fn(u64, u64, u64, *mut u8, u64, u64));
        function(black_box(1), 2, 3, black_box(raw.as_mut_ptr()), 5, black_box(6));
    }

    /// Calls (through its address) `refresh(HANDLER)`.
    pub fn refresh_poll() {
        let function = black_box(Self::refresh as extern "C" fn(*mut c_void));
        function(black_box(Self::HANDLER as *mut c_void));
    }

    /// What the stand-in `extractRawHIDInput` writes into the component from now on.
    pub fn set_fill(fill: [u8; 0x18]) {
        *FILL.lock().unwrap() = fill;
    }

    /// The stand-in calls so far, in order.
    pub fn calls() -> Vec<String> {
        CALLS.lock().unwrap().clone()
    }

    fn function(address: *const ()) -> Resolved {
        Resolved { kind: EntryKind::Function, value: address as usize as u64 - Harness::BASE, recipes_agreeing: 2 }
    }

    fn record(line: String) {
        CALLS.lock().unwrap().push(line);
    }

    #[inline(never)]
    extern "C" fn apply_turn_delta(this: *mut c_void, delta: *const [f32; 2]) {
        // SAFETY: every caller passes a live `[f32; 2]`.
        let [x, y] = unsafe { black_box(delta).read() };
        Self::record(format!("turn this={:#x} x={x} y={y}", black_box(this) as usize));
    }

    #[inline(never)]
    extern "C" fn extract_raw_hid_input(
        abilities: u64,
        move_input: u64,
        actor_flags: u64,
        raw: *mut u8,
        sneaking: u64,
        in_water: u64,
    ) {
        let fill = *FILL.lock().unwrap();
        // SAFETY: every caller passes a live 18h-byte component.
        unsafe { std::ptr::copy_nonoverlapping(fill.as_ptr(), black_box(raw), fill.len()) };
        Self::record(format!("extract {abilities} {move_input} {actor_flags} {sneaking} {in_water}"));
    }

    #[inline(never)]
    extern "C" fn refresh(this: *mut c_void) {
        Self::record(format!("refresh this={:#x}", black_box(this) as usize));
    }
}
