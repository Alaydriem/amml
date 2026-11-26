use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use amml_abi::ClientTick;
use amml_api::testing::FakeGame;
use amml_api::{AmmlMod, ApiError, Ctx, HookOrder, Listener, Uses};

use crate::support::CancellableProbe;

fn tick() -> ClientTick {
    ClientTick { header: FakeGame::header::<ClientTick>() }
}

#[test]
fn a_listener_gets_the_event_and_drop_removes_it() {
    let fake = FakeGame::new();
    fake.offer::<ClientTick>();
    let ctx = fake.ctx();
    let count = Arc::new(AtomicU64::new(0));
    let seen = count.clone();
    let listener = ctx
        .events()
        .listen::<ClientTick>(HookOrder::NORMAL, move |_, _| {
            seen.fetch_add(1, Ordering::SeqCst);
        })
        .unwrap();

    fake.raise(&mut tick());
    assert_eq!(count.load(Ordering::SeqCst), 1);

    drop(listener);
    fake.raise(&mut tick());
    assert_eq!(count.load(Ordering::SeqCst), 1);
}

#[test]
fn a_panic_in_a_listener_is_caught_and_reported() {
    let fake = FakeGame::new();
    fake.offer::<ClientTick>();
    let ctx = fake.ctx();
    let _listener = ctx.events().listen::<ClientTick>(HookOrder::NORMAL, |_, _| panic!("boom")).unwrap();

    let dispatch = fake.raise(&mut tick());

    assert!(dispatch.panicked);
    let logs = fake.logs();
    assert!(logs.iter().any(|line| line.starts_with("error a listener panicked")), "{logs:?}");
}

#[test]
fn a_listener_can_cancel_and_sees_the_context() {
    let fake = FakeGame::new();
    fake.offer::<CancellableProbe>();
    fake.open_gate();
    fake.set_epoch(7);
    let ctx = fake.ctx();
    let epoch = Arc::new(AtomicU64::new(0));
    let seen = epoch.clone();
    let _listener = ctx
        .events()
        .listen::<CancellableProbe>(HookOrder::FIRST, move |event, context| {
            if context.game_alive {
                seen.store(context.epoch.0, Ordering::SeqCst);
            }
            event.header.cancelled = 1;
        })
        .unwrap();

    let dispatch = fake.raise(&mut CancellableProbe { header: FakeGame::header::<CancellableProbe>() });

    assert!(dispatch.cancelled);
    assert_eq!(epoch.load(Ordering::SeqCst), 7);
}

struct NeedsTicks;

impl AmmlMod for NeedsTicks {
    const NAMESPACE: &'static str = "acme";
    const NAME: &'static str = "ticks";
    const USES: Uses = amml_api::amml_uses! { feature "core" { required ClientTick; } };

    fn enable(_ctx: &Ctx) -> Result<(), ApiError> {
        Ok(())
    }
}

#[test]
fn a_required_event_that_is_missing_is_reported_to_the_loader() {
    let fake = FakeGame::new();
    let ctx = fake.ctx_for::<NeedsTicks>();

    let refused = ctx.events().listen::<ClientTick>(HookOrder::NORMAL, |_, _| {});

    assert!(matches!(refused, Err(ApiError::NotAvailable)));
    assert_eq!(fake.missing_required(), vec!["client.tick".to_string()]);
}

#[test]
fn a_listener_that_panicked_is_not_called_again() {
    let fake = FakeGame::new();
    fake.offer::<ClientTick>();
    let count = Arc::new(AtomicU64::new(0));
    let seen = count.clone();
    let _listener = fake
        .ctx()
        .events()
        .listen::<ClientTick>(HookOrder::NORMAL, move |_, _| {
            seen.fetch_add(1, Ordering::SeqCst);
            panic!("boom");
        })
        .unwrap();

    assert!(fake.raise(&mut tick()).panicked);
    assert!(!fake.raise(&mut tick()).panicked);
    assert_eq!(count.load(Ordering::SeqCst), 1);
}

#[test]
fn a_listener_that_drops_itself_in_its_closure_is_not_called_again() {
    let fake = FakeGame::new();
    fake.offer::<ClientTick>();
    let count = Arc::new(AtomicU64::new(0));
    let seen = count.clone();
    let slot: Arc<Mutex<Option<Listener>>> = Arc::new(Mutex::new(None));
    let own = slot.clone();
    let listener = fake
        .ctx()
        .events()
        .listen::<ClientTick>(HookOrder::NORMAL, move |_, _| {
            seen.fetch_add(1, Ordering::SeqCst);
            let taken = own.lock().unwrap().take();
            drop(taken);
        })
        .unwrap();
    *slot.lock().unwrap() = Some(listener);

    let first = fake.raise(&mut tick());
    let second = fake.raise(&mut tick());

    assert!(!first.panicked && !second.panicked);
    assert_eq!(count.load(Ordering::SeqCst), 1);
    assert!(slot.lock().unwrap().is_none());
}

#[test]
fn a_listener_is_not_run_again_by_a_nested_raise_on_its_thread() {
    let fake = FakeGame::new();
    fake.offer::<ClientTick>();
    let count = Arc::new(AtomicU64::new(0));
    let seen = count.clone();
    let address = std::ptr::from_ref(&fake).addr();
    let _listener = fake
        .ctx()
        .events()
        .listen::<ClientTick>(HookOrder::NORMAL, move |_, _| {
            seen.fetch_add(1, Ordering::SeqCst);
            // SAFETY: the closure runs inside `fake.raise` on the test thread, while `fake` lives.
            let fake = unsafe { &*std::ptr::with_exposed_provenance::<FakeGame>(address) };
            fake.raise(&mut tick());
        })
        .unwrap();

    let dispatch = fake.raise(&mut tick());

    assert!(!dispatch.panicked);
    assert_eq!(count.load(Ordering::SeqCst), 1);
}

#[test]
#[should_panic(expected = "already live")]
fn a_second_live_fake_game_on_one_thread_is_refused() {
    let _first = FakeGame::new();
    let _second = FakeGame::new();
}

#[test]
fn a_listener_dropped_where_another_fake_runs_keeps_its_closure() {
    let fake = FakeGame::new();
    fake.offer::<ClientTick>();
    let count = Arc::new(AtomicU64::new(0));
    let seen = count.clone();
    let listener = fake
        .ctx()
        .events()
        .listen::<ClientTick>(HookOrder::NORMAL, move |_, _| {
            seen.fetch_add(1, Ordering::SeqCst);
        })
        .unwrap();

    let other_count = std::thread::spawn(move || {
        let other = FakeGame::new();
        other.offer::<ClientTick>();
        let other_seen = Arc::new(AtomicU64::new(0));
        let mark = other_seen.clone();
        // The other fake's first listener gets the same bus id as `listener`.
        let _own = other
            .ctx()
            .events()
            .listen::<ClientTick>(HookOrder::NORMAL, move |_, _| {
                mark.fetch_add(1, Ordering::SeqCst);
            })
            .unwrap();
        drop(listener);
        other.raise(&mut tick());
        other_seen.load(Ordering::SeqCst)
    })
    .join()
    .unwrap();

    assert_eq!(other_count, 1);
    fake.raise(&mut tick());
    assert_eq!(count.load(Ordering::SeqCst), 1);
}
