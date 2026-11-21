use std::ffi::c_void;
use std::sync::{Arc, Mutex};

use amml::{EventHub, InputService, KeyFeed, ModListener, MouseFeed, TurnFeed};
use amml_abi::{
    AmmlBand, AmmlEventContext, AmmlEventHeader, AmmlMouseButton, AmmlStatus, InjectedInputReleased, InputLeaseResumed,
    InputLeaseSuspended, KeyInput, LookDelta, MouseInput,
};

/// Stand-in key, mouse and turn feeds that record each call, and mod listeners that record each
/// event they get, in one log in the order things happened.
pub struct RecordingFeeds {
    log: Mutex<Vec<String>>,
    // The yaw and pitch a `LookDelta` listener writes, when set.
    look_override: Mutex<Option<(f32, f32)>>,
}

impl RecordingFeeds {
    pub fn new() -> Arc<Self> {
        Arc::new(Self { log: Mutex::new(Vec::new()), look_override: Mutex::new(None) })
    }

    /// Binds these feeds as the key, mouse and turn feeds of `input`.
    pub fn bind(self: &Arc<Self>, input: &InputService) {
        input.bind_keys(self.clone());
        input.bind_mouse(self.clone());
        input.bind_turn(self.clone());
    }

    /// Adds a listener of the mod at `owner` for `event` (offered first); it records
    /// `event <kind> owner=<owner> ...`.
    pub fn listen(self: &Arc<Self>, hub: &EventHub, event: &str, owner: usize) {
        hub.offer(event, 1);
        let callback: amml_abi::EventCallbackFn = match event {
            "input.key" => Self::on_key,
            "input.mouse" => Self::on_mouse,
            "input.look_delta" => Self::on_look,
            "input.injected_released" => Self::on_released,
            "input.lease_suspended" => Self::on_suspended,
            "input.lease_resumed" => Self::on_resumed,
            _ => panic!("no recorder for {event}"),
        };
        // Leaked: a listener may run until the end of the test process.
        let user = Box::into_raw(Box::new((self.clone(), owner))) as usize;
        hub.listen(event, 1, AmmlBand::NORMAL, ModListener { owner, callback, user }).unwrap();
    }

    /// The feed calls and events since the start, in order.
    pub fn log(&self) -> Vec<String> {
        self.log.lock().unwrap().clone()
    }

    /// The yaw and pitch every `LookDelta` listener writes from now on.
    pub fn override_look(&self, yaw: f32, pitch: f32) {
        *self.look_override.lock().unwrap() = Some((yaw, pitch));
    }

    fn push(&self, line: String) {
        self.log.lock().unwrap().push(line);
    }

    // The feeds and the owner of the listener that `user` belongs to, and the line prefix.
    fn recorder<'a>(user: *mut c_void, kind: &str) -> (&'a RecordingFeeds, String) {
        // SAFETY: `user` is the leaked `(feeds, owner)` box `listen` registered with this callback.
        let (feeds, owner) = unsafe { &*user.cast::<(Arc<RecordingFeeds>, usize)>() };
        (feeds, format!("event {kind} owner={owner}"))
    }

    fn origin(event: *mut AmmlEventHeader) -> String {
        // SAFETY: the loader passes a live header.
        let header = unsafe { &*event };
        format!("injected={} origin={}", header.injected, header.origin.0)
    }

    unsafe extern "C" fn on_key(
        event: *mut AmmlEventHeader,
        _: *const AmmlEventContext,
        user: *mut c_void,
    ) -> AmmlStatus {
        // SAFETY: the loader raises `input.key` with a `KeyInput`.
        let key = unsafe { &*event.cast::<KeyInput>() };
        let (feeds, head) = Self::recorder(user, "key");
        feeds.push(format!("{head} key={:#x} down={} {}", key.key, key.down, Self::origin(event)));
        AmmlStatus::OK
    }

    unsafe extern "C" fn on_mouse(
        event: *mut AmmlEventHeader,
        _: *const AmmlEventContext,
        user: *mut c_void,
    ) -> AmmlStatus {
        // SAFETY: the loader raises `input.mouse` with a `MouseInput`.
        let mouse = unsafe { &*event.cast::<MouseInput>() };
        let (feeds, head) = Self::recorder(user, "mouse");
        let line = format!(
            "{head} action={} button={} down={} notches={} {}",
            mouse.action.0,
            mouse.button.0,
            mouse.down,
            mouse.notches,
            Self::origin(event)
        );
        feeds.push(line);
        AmmlStatus::OK
    }

    unsafe extern "C" fn on_look(
        event: *mut AmmlEventHeader,
        _: *const AmmlEventContext,
        user: *mut c_void,
    ) -> AmmlStatus {
        let (feeds, head) = Self::recorder(user, "look");
        // SAFETY: the loader raises `input.look_delta` with a `LookDelta`.
        let look = unsafe { &mut *event.cast::<LookDelta>() };
        feeds.push(format!("{head} yaw={} pitch={} {}", look.yaw, look.pitch, Self::origin(event)));
        if let Some((yaw, pitch)) = *feeds.look_override.lock().unwrap() {
            look.yaw = yaw;
            look.pitch = pitch;
        }
        AmmlStatus::OK
    }

    unsafe extern "C" fn on_released(
        event: *mut AmmlEventHeader,
        _: *const AmmlEventContext,
        user: *mut c_void,
    ) -> AmmlStatus {
        // SAFETY: the loader raises `input.injected_released` with an `InjectedInputReleased`.
        let released = unsafe { &*event.cast::<InjectedInputReleased>() };
        let (feeds, head) = Self::recorder(user, "released");
        let one = if released.input == InjectedInputReleased::ALL {
            String::new()
        } else {
            format!(" input={} code={:#x}", released.input, released.code)
        };
        feeds.push(format!("{head} lease={} reason={}{one}", released.lease, released.reason.0));
        AmmlStatus::OK
    }

    unsafe extern "C" fn on_suspended(
        event: *mut AmmlEventHeader,
        _: *const AmmlEventContext,
        user: *mut c_void,
    ) -> AmmlStatus {
        // SAFETY: the loader raises `input.lease_suspended` with an `InputLeaseSuspended`.
        let suspended = unsafe { &*event.cast::<InputLeaseSuspended>() };
        let (feeds, head) = Self::recorder(user, "suspended");
        feeds.push(format!("{head} lease={}", suspended.lease));
        AmmlStatus::OK
    }

    unsafe extern "C" fn on_resumed(
        event: *mut AmmlEventHeader,
        _: *const AmmlEventContext,
        user: *mut c_void,
    ) -> AmmlStatus {
        // SAFETY: the loader raises `input.lease_resumed` with an `InputLeaseResumed`.
        let resumed = unsafe { &*event.cast::<InputLeaseResumed>() };
        let (feeds, head) = Self::recorder(user, "resumed");
        feeds.push(format!("{head} lease={}", resumed.lease));
        AmmlStatus::OK
    }
}

impl KeyFeed for RecordingFeeds {
    fn shape(&self) -> &'static str {
        "recording"
    }

    fn ready(&self) -> Result<(), AmmlStatus> {
        Ok(())
    }

    fn send(&self, key: u16, down: bool) {
        self.push(format!("feed key {key:#x} {}", if down { "down" } else { "up" }));
    }
}

impl MouseFeed for RecordingFeeds {
    fn shape(&self) -> &'static str {
        "recording"
    }

    fn ready(&self) -> Result<(), AmmlStatus> {
        Ok(())
    }

    fn send_button(&self, button: AmmlMouseButton, down: bool) {
        self.push(format!("feed button {} {}", button.0, if down { "down" } else { "up" }));
    }

    fn send_wheel(&self, notches: f32) {
        self.push(format!("feed wheel {notches}"));
    }
}

impl TurnFeed for RecordingFeeds {
    fn send_turn(&self, yaw: f32, pitch: f32) -> bool {
        self.push(format!("feed turn {yaw} {pitch}"));
        true
    }
}
