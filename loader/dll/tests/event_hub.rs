use std::collections::BTreeSet;
use std::ffi::c_void;
use std::sync::Mutex;

use amml::{EventHub, ModListener};
use amml_abi::{
    AmmlBand, AmmlEvent, AmmlEventContext, AmmlEventHeader, AmmlOrigin, AmmlPhase, AmmlStatus, AppFocusGained,
    AppFocusLost, CameraFov, CameraInstructionReceived, CameraSetup, ClientTick, DimensionChanging, FramePresent,
    FramePresented, FrameUpdate, GameTick, GamepadPoll, InjectedInputReleased, InputLeaseLost, InputLeaseResumed,
    InputLeaseSuspended, JoinServerResult, JoinStarted, KeyInput, LookDelta, ModsLoaded, MouseGrab, MouseInput,
    PlayerMoveInput, ProviderGone, ProviderReady, RenderDeviceReset, ScreenshotTaken, ScreenshotWritten, Shutdown,
    SurfaceLost, SwapChainResize, SwapChainResized, WindowChanged, WindowChar, WindowCursor, WindowFocus,
    WindowPointer, WindowSize, WorldJoined, WorldLeft,
};
use amml_data::{AdapterCatalog, NeedKind};

// (size, version, phase, origin, cancelled, injected) of a header.
type HeaderFields = (u32, u32, u8, u8, u8, u8);

// The headers a recording listener saw.
static SEEN: Mutex<Vec<HeaderFields>> = Mutex::new(Vec::new());

fn fields(header: &AmmlEventHeader) -> HeaderFields {
    (header.size, header.version, header.phase.0, header.origin.0, header.cancelled, header.injected)
}

// Cancels the event and rewrites every other header field.
unsafe extern "C" fn cancel_and_rewrite(
    event: *mut AmmlEventHeader,
    _context: *const AmmlEventContext,
    _user: *mut c_void,
) -> AmmlStatus {
    // SAFETY: the hub passes a live event header for the call.
    let header = unsafe { &mut *event };
    header.cancelled = 1;
    header.size = 4;
    header.version = 99;
    header.phase = AmmlPhase(7);
    header.origin = AmmlOrigin::OTHER_MOD;
    header.injected = 1;
    AmmlStatus::OK
}

// Records the header it sees, then tries to undo a cancel.
unsafe extern "C" fn record_and_uncancel(
    event: *mut AmmlEventHeader,
    _context: *const AmmlEventContext,
    _user: *mut c_void,
) -> AmmlStatus {
    // SAFETY: the hub passes a live event header for the call.
    let header = unsafe { &mut *event };
    SEEN.lock().unwrap().push(fields(header));
    header.cancelled = 0;
    AmmlStatus::OK
}

fn listen(hub: &EventHub, event: &str, owner: usize, band: AmmlBand, callback: amml_abi::EventCallbackFn) {
    hub.listen(event, 1, band, ModListener { owner, callback, user: 0 }).unwrap();
}

/// A cancellable event for the hub tests; no game event can be cancelled yet.
#[repr(C)]
#[derive(Clone, Copy)]
struct CancellableProbe {
    header: AmmlEventHeader,
}

impl AmmlEvent for CancellableProbe {
    const ID: &'static str = "test.cancellable";
    const CANCELLABLE: bool = true;
}

#[test]
fn a_listener_cannot_cancel_an_event_that_cannot_be_cancelled_or_rewrite_its_header() {
    let hub = EventHub::new();
    hub.offer("client.tick", 1);
    listen(&hub, "client.tick", 1, AmmlBand::FIRST, cancel_and_rewrite);
    listen(&hub, "client.tick", 2, AmmlBand::NORMAL, record_and_uncancel);
    let loader_header = EventHub::header::<ClientTick>(AmmlPhase::AFTER);
    let mut tick = ClientTick { header: loader_header };

    let report = hub.raise(&mut tick, &hub.loader_context());

    let expected = (16, 1, AmmlPhase::AFTER.0, AmmlOrigin::GAME.0, 0, 0);
    assert_eq!(*SEEN.lock().unwrap(), [expected]);
    assert!(!report.cancelled);
    assert_eq!(fields(&tick.header), expected);
}

#[test]
fn a_cancel_reaches_later_listeners_and_cannot_be_undone() {
    let hub = EventHub::new();
    hub.offer(CancellableProbe::ID, 1);
    listen(&hub, CancellableProbe::ID, 1, AmmlBand::FIRST, cancel_and_rewrite);
    listen(&hub, CancellableProbe::ID, 2, AmmlBand::NORMAL, record_and_uncancel);
    listen(&hub, CancellableProbe::ID, 3, AmmlBand::LAST, record_and_uncancel);
    let mut probe = CancellableProbe { header: EventHub::header::<CancellableProbe>(AmmlPhase::BEFORE) };

    let report = hub.raise(&mut probe, &hub.loader_context());

    let expected = (16, 1, AmmlPhase::BEFORE.0, AmmlOrigin::GAME.0, 1, 0);
    assert_eq!(*SEEN.lock().unwrap(), [expected, expected]);
    assert!(report.cancelled);
    assert_eq!(fields(&probe.header), expected);
}

#[test]
fn a_fixed_raise_ignores_a_cancel() {
    let hub = EventHub::new();
    hub.offer(CancellableProbe::ID, 1);
    listen(&hub, CancellableProbe::ID, 1, AmmlBand::FIRST, cancel_and_rewrite);
    listen(&hub, CancellableProbe::ID, 2, AmmlBand::NORMAL, record_and_uncancel);
    let mut probe = CancellableProbe { header: EventHub::header::<CancellableProbe>(AmmlPhase::BEFORE) };

    let report = hub.raise_fixed(&mut probe, &hub.loader_context());

    let expected = (16, 1, AmmlPhase::BEFORE.0, AmmlOrigin::GAME.0, 0, 0);
    assert_eq!(*SEEN.lock().unwrap(), [expected]);
    assert!(!report.cancelled);
    assert_eq!(fields(&probe.header), expected);
}

fn abi_event<E: AmmlEvent>() -> (&'static str, u32) {
    (E::ID, E::VERSION)
}

#[test]
fn every_catalog_event_is_an_abi_event_and_the_other_way_round() {
    let abi: BTreeSet<(&str, u32)> = [
        abi_event::<ClientTick>(),
        abi_event::<FrameUpdate>(),
        abi_event::<GameTick>(),
        abi_event::<JoinStarted>(),
        abi_event::<ModsLoaded>(),
        abi_event::<ProviderGone>(),
        abi_event::<ProviderReady>(),
        abi_event::<Shutdown>(),
        abi_event::<WorldJoined>(),
        abi_event::<WorldLeft>(),
        abi_event::<GamepadPoll>(),
        abi_event::<InjectedInputReleased>(),
        abi_event::<InputLeaseLost>(),
        abi_event::<InputLeaseResumed>(),
        abi_event::<InputLeaseSuspended>(),
        abi_event::<KeyInput>(),
        abi_event::<LookDelta>(),
        abi_event::<MouseGrab>(),
        abi_event::<AppFocusLost>(),
        abi_event::<AppFocusGained>(),
        abi_event::<MouseInput>(),
        abi_event::<PlayerMoveInput>(),
        abi_event::<WindowPointer>(),
        abi_event::<WindowChar>(),
        abi_event::<WindowFocus>(),
        abi_event::<WindowSize>(),
        abi_event::<WindowChanged>(),
        abi_event::<WindowCursor>(),
        abi_event::<FramePresent>(),
        abi_event::<FramePresented>(),
        abi_event::<SwapChainResize>(),
        abi_event::<SwapChainResized>(),
        abi_event::<SurfaceLost>(),
        abi_event::<RenderDeviceReset>(),
        abi_event::<ScreenshotTaken>(),
        abi_event::<ScreenshotWritten>(),
        abi_event::<CameraFov>(),
        abi_event::<CameraSetup>(),
        abi_event::<CameraInstructionReceived>(),
        abi_event::<DimensionChanging>(),
        abi_event::<JoinServerResult>(),
        abi_event::<amml_abi::IdleScreenOpen>(),
        abi_event::<amml_abi::BlockBreakStart>(),
        abi_event::<amml_abi::BlockBreakContinue>(),
        abi_event::<amml_abi::BlockChanged>(),
        abi_event::<amml_abi::ItemUse>(),
        abi_event::<amml_abi::PlayerAttack>(),
        abi_event::<amml_abi::ItemUseOnBlock>(),
        abi_event::<amml_abi::UsePressed>(),
        abi_event::<amml_abi::BuildBlock>(),
        abi_event::<amml_abi::ScreenRender>(),
        abi_event::<amml_abi::ScreenClosing>(),
        abi_event::<amml_abi::ScreenChanged>(),
        abi_event::<amml_abi::ContainerSlotHover>(),
        abi_event::<amml_abi::ContainerSlotUnhover>(),
        abi_event::<amml_abi::UiPointer>(),
        abi_event::<amml_abi::UiScroll>(),
        abi_event::<amml_abi::ItemHovertext>(),
        abi_event::<amml_abi::InventoryRequestDone>(),
        abi_event::<amml_abi::PlayerListUpdate>(),
        abi_event::<amml_abi::PlayerSkinChanged>(),
        abi_event::<amml_abi::LocatorBarIconDraw>(),
        abi_event::<amml_abi::ChatMessageAdded>(),
        abi_event::<amml_abi::TexturesReloaded>(),
        abi_event::<amml_abi::WorldRendered>(),
        abi_event::<amml_abi::BlockEntityPass>(),
        abi_event::<amml_abi::PacketWire>(),
        abi_event::<amml_abi::ConnectionStarted>(),
        abi_event::<amml_abi::TimeStepApplied>(),
        abi_event::<amml_abi::FrameEnd>(),
        abi_event::<amml_abi::LatencyBarrier>(),
        abi_event::<amml_abi::PlayerMoved>(),
        abi_event::<amml_abi::ServerJoinStarting>(),
        abi_event::<amml_abi::WorldStorageOpened>(),
        abi_event::<amml_abi::WorldStorageClosed>(),
    ]
    .into_iter()
    .collect();
    let catalog: BTreeSet<(&str, u32)> = AdapterCatalog::needs()
        .iter()
        .filter(|need| need.kind == NeedKind::Event)
        .map(|need| (need.id, need.version))
        .collect();

    assert_eq!(catalog, abi);
}

#[test]
fn the_event_hub_offers_no_area() {
    let hub = EventHub::new();

    assert!(hub.is_available("input.lease_lost", 1));
    assert!(!hub.is_available("input.lease", 1));
}

// (owner, origin, injected) per call of `record_owner`; the listener's `user` is its owner.
static CALLED: Mutex<Vec<(usize, u8, u8)>> = Mutex::new(Vec::new());

unsafe extern "C" fn record_owner(
    event: *mut AmmlEventHeader,
    _context: *const AmmlEventContext,
    user: *mut c_void,
) -> AmmlStatus {
    // SAFETY: the hub passes a live event header for the call.
    let header = unsafe { &*event };
    CALLED.lock().unwrap().push((user as usize, header.origin.0, header.injected));
    AmmlStatus::OK
}

unsafe extern "C" fn cancel(
    event: *mut AmmlEventHeader,
    _context: *const AmmlEventContext,
    user: *mut c_void,
) -> AmmlStatus {
    // SAFETY: the hub passes a live event header for the call.
    unsafe { (*event).cancelled = 1 };
    CALLED.lock().unwrap().push((user as usize, 0, 0));
    AmmlStatus::OK
}

fn listen_as(hub: &EventHub, event: &str, owner: usize, callback: amml_abi::EventCallbackFn) {
    hub.listen(event, 1, AmmlBand::NORMAL, ModListener { owner, callback, user: owner }).unwrap();
}

fn key_input() -> KeyInput {
    KeyInput {
        header: EventHub::header::<KeyInput>(AmmlPhase::BEFORE),
        key: 0x57,
        down: 1,
        repeat: 0,
        modifiers: amml_abi::AmmlModifiers(0),
    }
}

#[test]
fn raise_only_reaches_one_mod() {
    let hub = EventHub::new();
    listen_as(&hub, "input.lease_resumed", 1, record_owner);
    listen_as(&hub, "input.lease_resumed", 2, record_owner);
    let mut event = InputLeaseResumed { header: EventHub::header::<InputLeaseResumed>(AmmlPhase::AFTER), lease: 5 };

    hub.raise_only(&mut event, &hub.loader_context(), 2);

    assert_eq!(*CALLED.lock().unwrap(), [(2, AmmlOrigin::GAME.0, 0)]);
}

#[test]
fn an_injected_raise_marks_origin_per_listener() {
    let hub = EventHub::new();
    hub.offer("input.key", 1);
    listen_as(&hub, "input.key", 1, record_owner);
    listen_as(&hub, "input.key", 2, record_owner);
    let mut event = key_input();

    hub.raise_injected(&mut event, &hub.loader_context(), 1);

    assert_eq!(*CALLED.lock().unwrap(), [(1, AmmlOrigin::THIS_MOD.0, 1), (2, AmmlOrigin::OTHER_MOD.0, 1)]);
}

#[test]
fn a_cancel_of_an_injected_event_is_ignored() {
    let hub = EventHub::new();
    hub.offer("input.key", 1);
    hub.listen("input.key", 1, AmmlBand::FIRST, ModListener { owner: 1, callback: cancel, user: 1 }).unwrap();
    listen_as(&hub, "input.key", 2, record_owner);
    let mut event = key_input();

    let report = hub.raise_injected(&mut event, &hub.loader_context(), 1);

    assert!(!report.cancelled);
    assert_eq!(*CALLED.lock().unwrap(), [(1, 0, 0), (2, AmmlOrigin::OTHER_MOD.0, 1)]);
    assert_eq!(event.header.cancelled, 0);
}

// The owner of each listener called and each `between` run (as 99), in order.
static ORDER: Mutex<Vec<usize>> = Mutex::new(Vec::new());

unsafe extern "C" fn note_owner(_: *mut AmmlEventHeader, _: *const AmmlEventContext, user: *mut c_void) -> AmmlStatus {
    ORDER.lock().unwrap().push(user as usize);
    AmmlStatus::OK
}

#[test]
fn a_split_raise_runs_between_after_its_band_once() {
    let hub = EventHub::new();
    hub.offer(CancellableProbe::ID, 1);
    for (owner, band) in [(3, AmmlBand::READ_AFTER_WRITERS), (1, AmmlBand::FIRST), (2, AmmlBand::NORMAL)] {
        hub.listen(CancellableProbe::ID, 1, band, ModListener { owner, callback: note_owner, user: owner }).unwrap();
    }
    let mut probe = CancellableProbe { header: EventHub::header::<CancellableProbe>(AmmlPhase::BEFORE) };

    hub.raise_split(&mut probe, &hub.loader_context(), AmmlBand::NORMAL, |_| ORDER.lock().unwrap().push(99));
    let empty = EventHub::new();
    empty.offer(CancellableProbe::ID, 1);
    empty.raise_split(&mut probe, &empty.loader_context(), AmmlBand::NORMAL, |_| ORDER.lock().unwrap().push(99));

    assert_eq!(*ORDER.lock().unwrap(), [1, 2, 99, 3, 99]);
}
