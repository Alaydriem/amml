use std::sync::{Arc, Mutex};

use amml_abi::{FrameEnd, LatencyBarrier, TimeStepApplied};
use amml_api::testing::FakeGame;
use amml_api::{ApiError, Area, FrameEndExt, HoldRequest, HookOrder, LatencyBarrierExt, TimeStepAppliedExt};

fn offered() -> FakeGame {
    let fake = FakeGame::new();
    fake.offer_area(Area::TIME_CONTROL);
    fake.open_gate();
    fake.set_epoch(1);
    fake
}

fn request() -> HoldRequest {
    HoldRequest { virtual_offset_ms: 86_400_000, watchdog_ms: 2000 }
}

#[test]
fn drop_releases_in_the_fake() {
    let fake = offered();
    let time = fake.ctx().time();
    assert_eq!(time.is_held(), Some(false));

    let hold = time.hold(request()).unwrap();
    assert_eq!(time.is_held(), Some(true));
    assert_eq!(hold.step_us(16_667), Ok(()));
    assert_eq!(time.stepped_us(), Some(16_667));
    drop(hold);

    assert_eq!(time.is_held(), Some(false));
    assert_eq!(fake.time_writes(), ["hold offset=86400000", "step 16667", "release reason=release"]);
}

#[test]
fn second_hold_is_refused_in_the_fake() {
    let fake = offered();
    let time = fake.ctx().time();
    let _hold = time.hold(request()).unwrap();

    assert_eq!(time.hold(request()).err(), Some(ApiError::Refused));
    assert_eq!(fake.time_writes(), ["hold offset=86400000"]);
}

#[test]
fn a_negative_step_is_refused_in_the_fake() {
    let fake = offered();
    let hold = fake.ctx().time().hold(request()).unwrap();

    assert_eq!(hold.step_us(-1), Err(ApiError::Loader(amml_abi::AmmlStatus::INVALID_ARGUMENT)));
}

#[test]
fn time_needs_the_area_and_a_v13_loader() {
    let fake = offered();
    fake.withdraw_area(Area::TIME_CONTROL);
    assert_eq!(fake.ctx().time().hold(request()).err(), Some(ApiError::NotAvailable));
    drop(fake);

    let old = FakeGame::with_v12_services();
    old.offer_area(Area::TIME_CONTROL);
    old.open_gate();
    let time = old.ctx().time();
    assert_eq!(time.is_held(), None);
    assert_eq!(time.hold(request()).err(), Some(ApiError::NotAvailable));
    assert!(!old.ctx().supports(Area::TIME_CONTROL));
}

#[test]
fn step_applied_from_the_fake() {
    let fake = offered();
    fake.offer::<TimeStepApplied>();
    let seen = Arc::new(Mutex::new(Vec::new()));
    let record = seen.clone();
    let _listener = fake
        .ctx()
        .events()
        .listen::<TimeStepApplied>(HookOrder::NORMAL, move |event, _| {
            record.lock().unwrap().push((event.stepped_us(), event.sim_ticks(), event.sim_alpha()));
        })
        .unwrap();

    fake.raise_time_step_applied(50_000, 1, 0.5);

    assert_eq!(*seen.lock().unwrap(), [(50_000, 1, 0.5)]);
}

#[test]
fn frame_end_and_barrier_from_the_fake() {
    let fake = offered();
    fake.offer::<FrameEnd>();
    fake.offer::<LatencyBarrier>();
    let ends = Arc::new(Mutex::new(Vec::new()));
    let barriers = Arc::new(Mutex::new(Vec::new()));
    let (record_end, record_barrier) = (ends.clone(), barriers.clone());
    let _end = fake
        .ctx()
        .events()
        .listen::<FrameEnd>(HookOrder::NORMAL, move |event, _| record_end.lock().unwrap().push(event.count()))
        .unwrap();
    let _barrier = fake
        .ctx()
        .events()
        .listen::<LatencyBarrier>(HookOrder::NORMAL, move |event, _| {
            record_barrier.lock().unwrap().push(event.create_time());
        })
        .unwrap();

    fake.raise_frame_end(7);
    fake.raise_latency_barrier(1_234_567);

    assert_eq!(*ends.lock().unwrap(), [7]);
    assert_eq!(*barriers.lock().unwrap(), [1_234_567]);
}
