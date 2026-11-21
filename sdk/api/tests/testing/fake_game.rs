use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use amml_abi::{AmmlOrigin, AmmlStatus, ClientTick, GameTick, KeyInput};
use amml_api::testing::FakeGame;
use amml_api::{
    AmmlMod, ApiError, Area, Cancel, CaptureRequest, Ctx, DisableCtx, Event, HookOrder, InputBlocks, KeyCode,
    KeyInputExt, LeaseMode, LeaseRequest, Listener, ShotLayers, Uses,
};

use crate::support::CancellableProbe;

#[test]
fn an_event_that_is_not_offered_gives_not_available() {
    let fake = FakeGame::new();
    let refused = fake.ctx().events().listen::<GameTick>(HookOrder::NORMAL, |_, _| {});
    assert!(matches!(refused, Err(ApiError::NotAvailable)));
}

#[test]
fn a_refused_listen_gives_refused_while_the_event_stays_offered() {
    let fake = FakeGame::new();
    fake.offer::<GameTick>();
    fake.refuse_listen::<GameTick>();

    let refused = fake.ctx().events().listen::<GameTick>(HookOrder::NORMAL, |_, _| {});

    assert!(matches!(refused, Err(ApiError::Refused)));
    assert!(fake.missing_required().is_empty());
}

#[test]
fn logs_are_kept_with_their_fields() {
    let fake = FakeGame::new();
    fake.ctx().log().info("hello", &[("a", "1")]);
    assert_eq!(fake.logs(), vec!["info hello a=1".to_string()]);
}

#[test]
fn a_listener_cannot_cancel_an_event_that_cannot_be_cancelled_or_rewrite_its_header() {
    let fake = FakeGame::new();
    fake.offer::<GameTick>();
    let ctx = fake.ctx();
    let _rewrite = ctx
        .events()
        .listen::<GameTick>(HookOrder::FIRST, |event, _| {
            event.header.cancelled = 1;
            event.header.size = 4;
            event.header.injected = 1;
        })
        .unwrap();
    let seen = Arc::new(Mutex::new(Vec::new()));
    let record = seen.clone();
    let _read = ctx
        .events()
        .listen::<GameTick>(HookOrder::NORMAL, move |event, _| {
            record.lock().unwrap().push((event.header.size, event.header.cancelled, event.header.injected));
        })
        .unwrap();

    let dispatch = fake.raise(&mut GameTick { header: FakeGame::header::<GameTick>(), tick: 1 });

    assert!(!dispatch.cancelled);
    assert_eq!(*seen.lock().unwrap(), [(std::mem::size_of::<GameTick>() as u32, 0, 0)]);
}

#[test]
fn a_cancel_reaches_later_listeners_and_cannot_be_undone() {
    let fake = FakeGame::new();
    fake.offer::<CancellableProbe>();
    let ctx = fake.ctx();
    let _cancel =
        ctx.events().listen::<CancellableProbe>(HookOrder::FIRST, |event, _| event.header.cancelled = 1).unwrap();
    let seen = Arc::new(Mutex::new(Vec::new()));
    let record = seen.clone();
    let _undo = ctx
        .events()
        .listen::<CancellableProbe>(HookOrder::NORMAL, move |event, _| {
            record.lock().unwrap().push(event.header.cancelled);
            event.header.cancelled = 0;
        })
        .unwrap();
    let last = seen.clone();
    let _last = ctx
        .events()
        .listen::<CancellableProbe>(HookOrder::LAST, move |event, _| last.lock().unwrap().push(event.header.cancelled))
        .unwrap();
    let mut probe = CancellableProbe { header: FakeGame::header::<CancellableProbe>() };

    let dispatch = fake.raise(&mut probe);

    assert!(dispatch.cancelled);
    assert_eq!(*seen.lock().unwrap(), [1, 1]);
    assert_eq!(probe.header.cancelled, 1);
}

static TICKER_LISTENERS: Mutex<Vec<Listener>> = Mutex::new(Vec::new());
static TICKER_CALLS: AtomicU64 = AtomicU64::new(0);

struct Ticker;

impl AmmlMod for Ticker {
    const NAMESPACE: &'static str = "acme";
    const NAME: &'static str = "ticker";

    fn enable(ctx: &Ctx) -> Result<(), ApiError> {
        let listener = ctx.events().listen::<ClientTick>(HookOrder::NORMAL, |_, _| {
            TICKER_CALLS.fetch_add(1, Ordering::SeqCst);
        })?;
        TICKER_LISTENERS.lock().unwrap().push(listener);
        Ok(())
    }

    fn disable(_ctx: &Ctx, _how: &DisableCtx) {
        let taken = std::mem::take(&mut *TICKER_LISTENERS.lock().unwrap());
        drop(taken);
    }
}

fn client_tick() -> ClientTick {
    ClientTick { header: FakeGame::header::<ClientTick>() }
}

#[test]
fn the_driver_refuses_listens_outside_load_and_enable_and_a_mod_can_be_enabled_again() {
    let fake = FakeGame::new();
    fake.offer::<ClientTick>();
    fake.enable::<Ticker>().unwrap();

    let late = fake.ctx_for::<Ticker>().events().listen::<ClientTick>(HookOrder::NORMAL, |_, _| {});
    assert!(matches!(late, Err(ApiError::Loader(AmmlStatus::FAILED))));
    fake.raise(&mut client_tick());
    fake.disable::<Ticker>();
    fake.raise(&mut client_tick());
    assert_eq!(TICKER_CALLS.load(Ordering::SeqCst), 1);

    fake.enable::<Ticker>().unwrap();
    fake.raise(&mut client_tick());
    assert_eq!(TICKER_CALLS.load(Ordering::SeqCst), 2);
}

static DECLARED_ENABLED: AtomicBool = AtomicBool::new(false);

struct DeclaresTicks;

impl AmmlMod for DeclaresTicks {
    const NAMESPACE: &'static str = "acme";
    const NAME: &'static str = "declares";
    const USES: Uses = amml_api::amml_uses! { feature "core" { required ClientTick; } };

    fn enable(_ctx: &Ctx) -> Result<(), ApiError> {
        DECLARED_ENABLED.store(true, Ordering::SeqCst);
        Ok(())
    }
}

#[test]
fn a_required_event_that_is_not_offered_stops_the_enable_before_it_runs() {
    let fake = FakeGame::new();

    assert_eq!(fake.enable::<DeclaresTicks>(), Err(ApiError::NotAvailable));
    assert!(!DECLARED_ENABLED.load(Ordering::SeqCst));
}

#[test]
fn an_injected_key_raises_a_key_input_whose_cancel_is_ignored() {
    let fake = FakeGame::new();
    fake.offer::<KeyInput>();
    fake.offer_area(Area::KEY_INJECT);
    fake.open_gate();
    fake.bind_input();
    let ctx = fake.ctx();
    let _cancel = ctx.events().listen::<KeyInput>(HookOrder::FIRST, |event, _| event.cancel()).unwrap();
    let seen = Arc::new(Mutex::new(Vec::new()));
    let record = seen.clone();
    let _read = ctx
        .events()
        .listen::<KeyInput>(HookOrder::NORMAL, move |event, _| {
            let header = event.header();
            record.lock().unwrap().push((event.key_code(), event.is_injected(), event.is_cancelled(), header.origin));
        })
        .unwrap();
    let request = LeaseRequest { mode: LeaseMode::Shared, blocks: InputBlocks::NONE, passthrough: &[], preempt: false };
    let lease = ctx.input().acquire(request).unwrap();

    let _hold = lease.key_down(KeyCode::W).unwrap();
    fake.frame();

    assert_eq!(*seen.lock().unwrap(), [(KeyCode::W, true, false, AmmlOrigin::THIS_MOD)]);
    assert_eq!(fake.injected(), ["key 0x57 down"]);
}

static AREA_ENABLED: AtomicBool = AtomicBool::new(false);

struct DeclaresKeys;

impl AmmlMod for DeclaresKeys {
    const NAMESPACE: &'static str = "acme";
    const NAME: &'static str = "keys";
    const USES: Uses = amml_api::amml_uses! { feature "input" { required area Area::KEY_INJECT; } };

    fn enable(_ctx: &Ctx) -> Result<(), ApiError> {
        AREA_ENABLED.store(true, Ordering::SeqCst);
        Ok(())
    }
}

#[test]
fn a_required_area_that_is_not_offered_stops_the_enable_before_it_runs() {
    let fake = FakeGame::new();

    assert_eq!(fake.enable::<DeclaresKeys>(), Err(ApiError::NotAvailable));
    assert!(!AREA_ENABLED.load(Ordering::SeqCst));

    fake.offer_area(Area::KEY_INJECT);
    assert_eq!(fake.enable::<DeclaresKeys>(), Ok(()));
    assert!(AREA_ENABLED.load(Ordering::SeqCst));
}

#[test]
fn a_stream_of_another_fake_is_unknown_to_the_current_one() {
    let first = FakeGame::new();
    first.offer_area(Area::OVERLAY_CAPTURE);
    first.open_gate();
    let request = CaptureRequest { layers: ShotLayers::GameOnly, every_nth_present: 1, ring: 1, max_size: None };
    let old = first.ctx().overlay().open_capture_stream(request).unwrap();
    drop(first);
    let second = FakeGame::new();
    second.offer_area(Area::OVERLAY_CAPTURE);
    second.open_gate();
    let current = second.ctx().overlay().open_capture_stream(request).unwrap();

    drop(old);

    assert_eq!(second.open_capture_streams(), 1);
    assert_eq!(current.free_slots(), 1);
}

static DRAWS_ENABLED: AtomicBool = AtomicBool::new(false);

struct DeclaresWindow;

impl AmmlMod for DeclaresWindow {
    const NAMESPACE: &'static str = "acme";
    const NAME: &'static str = "window";
    const USES: Uses = amml_api::amml_uses! {
        feature "window" { required area Area::WINDOW_READ; optional area Area::OVERLAY_CAPTURE; }
    };

    fn enable(_ctx: &Ctx) -> Result<(), ApiError> {
        DRAWS_ENABLED.store(true, Ordering::SeqCst);
        Ok(())
    }
}

#[test]
fn a_required_window_area_is_checked_before_the_enable() {
    let fake = FakeGame::new();

    assert_eq!(fake.enable::<DeclaresWindow>(), Err(ApiError::NotAvailable));
    assert!(!DRAWS_ENABLED.load(Ordering::SeqCst));

    fake.offer_area(Area::WINDOW_READ);
    assert_eq!(fake.enable::<DeclaresWindow>(), Ok(()));
    assert!(DRAWS_ENABLED.load(Ordering::SeqCst));
}
