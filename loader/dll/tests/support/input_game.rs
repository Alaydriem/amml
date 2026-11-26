use std::ffi::c_void;
use std::hint::black_box;
use std::sync::Mutex;
use std::thread::ThreadId;

use amml_data::{EntryKind, Resolved};

use super::{Harness, SessionGame};

// The stand-in `MouseDevice.instance`; only its address is used.
static MOUSE_DEVICE: [u64; 8] = [0; 8];

// Each stand-in call as a line, with the thread it ran on.
static CALLS: Mutex<Vec<(String, ThreadId)>> = Mutex::new(Vec::new());

/// Stand-ins with the game signatures for the functions the input adapter hooks: the three
/// `HIDControllerGameCoreDesktop` functions and the `HIDController` key reset of 26.6x and
/// `KeyboardEventProcessor::_feedKeyState` of 70.x, and the seven-argument (26.6x) and eight-argument (70.x)
/// `MouseDevice::feed`. Each records its call (`down this=.. key=.. origin=..`, `up this=.. key=..`,
/// `update this=..`, `reset this=..`, `state this=.. key=.. state=..`, `mouse this=.. action=.. data=.. ..`)
/// and the calling thread.
pub struct InputGame;

impl InputGame {
    /// The keyboard controller the tests pass as `this`.
    pub const CONTROLLER: usize = 0x7C_0000;

    /// The session entries, the three HID entries and `HIDController.resetKeys`, pointing to the stand-ins.
    pub fn hid_entries() -> Vec<(&'static str, Resolved)> {
        let mut entries = Self::hid_entries_without_reset();
        entries.push(("HIDController.resetKeys", Self::function(Self::reset_keys_stand_in as *const ())));
        entries
    }

    /// The session entries and the three HID entries, without the optional key reset.
    pub fn hid_entries_without_reset() -> Vec<(&'static str, Resolved)> {
        let mut entries = SessionGame::entries(&[]);
        entries.extend([
            ("HIDControllerGameCoreDesktop.onKeyDown", Self::function(Self::on_key_down as *const ())),
            ("HIDControllerGameCoreDesktop.onKeyUp", Self::function(Self::on_key_up as *const ())),
            ("HIDControllerGameCoreDesktop.update", Self::function(Self::hid_update as *const ())),
        ]);
        entries
    }

    /// Calls (through its address) the key reset `resetKeys(CONTROLLER)`, as the game's focus handler does.
    pub fn reset_keys() {
        let function = black_box(Self::reset_keys_stand_in as extern "C" fn(*mut c_void));
        function(Self::controller());
    }

    /// The session entries and `KeyboardEventProcessor.feedKeyState`, pointing to the stand-ins.
    pub fn processor_entries() -> Vec<(&'static str, Resolved)> {
        let mut entries = SessionGame::entries(&[]);
        entries.push(("KeyboardEventProcessor.feedKeyState", Self::function(Self::feed_key_state as *const ())));
        entries
    }

    /// Calls (through its address) `onKeyDown(CONTROLLER, key, origin)`.
    pub fn key_down(key: i32, origin: i32) {
        let function = black_box(Self::on_key_down as extern "C" fn(*mut c_void, i32, i32));
        function(Self::controller(), black_box(key), black_box(origin));
    }

    /// Calls (through its address) `onKeyUp(CONTROLLER, key)`.
    pub fn key_up(key: i32) {
        let function = black_box(Self::on_key_up as extern "C" fn(*mut c_void, i32));
        function(Self::controller(), black_box(key));
    }

    /// Calls (through its address) `update(CONTROLLER)`.
    pub fn update() {
        let function = black_box(Self::hid_update as extern "C" fn(*mut c_void));
        function(Self::controller());
    }

    /// Calls (through its address) `_feedKeyState(this, key, state)`.
    pub fn feed(this: usize, key: i32, state: i32) {
        let function = black_box(Self::feed_key_state as extern "C" fn(*mut c_void, i32, i32));
        function(black_box(this as *mut c_void), black_box(key), black_box(state));
    }

    /// The session entries, `MouseDevice.feed` and `MouseDevice.instance`, pointing to the stand-ins.
    pub fn seven_arg_mouse_entries() -> Vec<(&'static str, Resolved)> {
        let mut entries = SessionGame::entries(&[]);
        entries.extend([
            ("MouseDevice.feed", Self::function(Self::seven_arg_feed as *const ())),
            ("MouseDevice.instance", Self::global(Self::mouse_device())),
        ]);
        entries
    }

    /// The session entries, `MouseDevice.feedWithControllerId` and `MouseDevice.instance`, pointing to the
    /// stand-ins.
    pub fn eight_arg_mouse_entries() -> Vec<(&'static str, Resolved)> {
        let mut entries = SessionGame::entries(&[]);
        entries.extend([
            ("MouseDevice.feedWithControllerId", Self::function(Self::eight_arg_feed as *const ())),
            ("MouseDevice.instance", Self::global(Self::mouse_device())),
        ]);
        entries
    }

    /// The address of the stand-in `MouseDevice.instance`.
    pub fn mouse_device() -> usize {
        std::ptr::addr_of!(MOUSE_DEVICE) as usize
    }

    /// Calls (through its address) the seven-argument `feed(this, action, data, x, y, dx, dy, false)`.
    #[allow(clippy::too_many_arguments)]
    pub fn mouse(this: usize, action: u8, data: i8, x: i16, y: i16, dx: i16, dy: i16) {
        let function = black_box(Self::seven_arg_feed as extern "C" fn(*mut c_void, u8, i8, i16, i16, i16, i16, u8));
        function(black_box(this as *mut c_void), action, data, black_box(x), y, dx, dy, black_box(0));
    }

    /// Calls (through its address) the eight-argument `feed(this, action, data, x, y, dx, dy, controller,
    /// true)`.
    #[allow(clippy::too_many_arguments)]
    pub fn mouse_eight(this: usize, action: u8, data: i8, x: i16, y: i16, dx: i16, dy: i16, controller: i32) {
        let function =
            black_box(Self::eight_arg_feed as extern "C" fn(*mut c_void, u8, i8, i16, i16, i16, i16, i32, u8));
        function(black_box(this as *mut c_void), action, data, black_box(x), y, dx, dy, black_box(controller), 1);
    }

    /// The stand-in calls so far, in order.
    pub fn calls() -> Vec<String> {
        CALLS.lock().unwrap().iter().map(|(line, _)| line.clone()).collect()
    }

    /// The threads of the stand-in calls so far, in order.
    pub fn threads() -> Vec<ThreadId> {
        CALLS.lock().unwrap().iter().map(|(_, thread)| *thread).collect()
    }

    /// Forgets the calls so far.
    pub fn clear() {
        CALLS.lock().unwrap().clear();
    }

    fn controller() -> *mut c_void {
        black_box(Self::CONTROLLER as *mut c_void)
    }

    fn function(address: *const ()) -> Resolved {
        Resolved { kind: EntryKind::Function, value: address as usize as u64 - Harness::BASE, recipes_agreeing: 2 }
    }

    fn global(address: usize) -> Resolved {
        Resolved { kind: EntryKind::Global, value: address as u64 - Harness::BASE, recipes_agreeing: 2 }
    }

    fn record(line: String) {
        CALLS.lock().unwrap().push((line, std::thread::current().id()));
    }

    #[inline(never)]
    extern "C" fn on_key_down(this: *mut c_void, key: i32, origin: i32) {
        Self::record(format!("down this={:#x} key={key:#x} origin={origin}", black_box(this) as usize));
    }

    #[inline(never)]
    extern "C" fn on_key_up(this: *mut c_void, key: i32) {
        Self::record(format!("up this={:#x} key={key:#x}", black_box(this) as usize));
    }

    #[inline(never)]
    extern "C" fn hid_update(this: *mut c_void) {
        Self::record(format!("update this={:#x}", black_box(this) as usize));
    }

    #[inline(never)]
    extern "C" fn reset_keys_stand_in(this: *mut c_void) {
        Self::record(format!("reset this={:#x}", black_box(this) as usize));
    }

    #[inline(never)]
    extern "C" fn feed_key_state(this: *mut c_void, key: i32, state: i32) {
        Self::record(format!("state this={:#x} key={key:#x} state={state}", black_box(this) as usize));
    }

    #[inline(never)]
    #[allow(clippy::too_many_arguments)]
    extern "C" fn seven_arg_feed(
        this: *mut c_void,
        action: u8,
        data: i8,
        x: i16,
        y: i16,
        dx: i16,
        dy: i16,
        motionless: u8,
    ) {
        Self::record(format!(
            "mouse this={} action={action} data={data} x={x} y={y} dx={dx} dy={dy} motionless={motionless}",
            Self::device_name(black_box(this) as usize)
        ));
    }

    #[inline(never)]
    #[allow(clippy::too_many_arguments)]
    extern "C" fn eight_arg_feed(
        this: *mut c_void,
        action: u8,
        data: i8,
        x: i16,
        y: i16,
        dx: i16,
        dy: i16,
        controller: i32,
        motionless: u8,
    ) {
        Self::record(format!(
            "mouse this={} action={action} data={data} x={x} y={y} dx={dx} dy={dy} controller={controller} motionless={motionless}",
            Self::device_name(black_box(this) as usize)
        ));
    }

    // `device` for the stand-in `MouseDevice.instance`, else the address.
    fn device_name(address: usize) -> String {
        if address == Self::mouse_device() { "device".to_string() } else { format!("{address:#x}") }
    }
}
