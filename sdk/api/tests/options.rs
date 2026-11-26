use std::sync::{Arc, Mutex};

use amml_abi::ClientTick;
use amml_api::testing::{FakeClient, FakeGame, FakeOptions};
use amml_api::{ApiError, Area, HookOrder, OptionKey, Persistence};

fn offered() -> FakeGame {
    let fake = FakeGame::new();
    fake.offer_area(Area::OPTIONS_VIEW_DISTANCE);
    fake.offer_area(Area::OPTIONS_INT);
    fake.open_gate();
    fake.set_client(FakeClient::default());
    fake.set_options(FakeOptions {
        view_distance: 18,
        ints: [(OptionKey::VSYNC, 1), (OptionKey::MAX_FRAMERATE, 60)].into_iter().collect(),
    });
    fake
}

// The view distance each `ClientTick` listener read for its client.
fn read_view_distance(fake: &FakeGame) -> Vec<Option<u32>> {
    let seen = Arc::new(Mutex::new(Vec::new()));
    let record = seen.clone();
    let ctx = fake.ctx();
    let options = ctx.options();
    let _listener = ctx
        .events()
        .listen::<ClientTick>(HookOrder::NORMAL, move |_, at| {
            let client = at.client().unwrap();
            record.lock().unwrap().push(options.view_distance_chunks(&client));
        })
        .unwrap();
    fake.raise_with_client(&mut ClientTick { header: FakeGame::header::<ClientTick>() });
    std::mem::take(&mut *seen.lock().unwrap())
}

#[test]
fn view_distance_from_the_fake() {
    let fake = offered();
    fake.offer::<ClientTick>();

    assert_eq!(read_view_distance(&fake), [Some(18)]);
    assert_eq!(fake.ctx().options().get_int(OptionKey::MAX_FRAMERATE), Some(60));
}

#[test]
fn view_distance_needs_the_area() {
    let fake = offered();
    fake.offer::<ClientTick>();
    fake.withdraw_area(Area::OPTIONS_VIEW_DISTANCE);

    assert_eq!(read_view_distance(&fake), [None]);
}

#[test]
fn transient_set_then_release_in_the_fake() {
    let fake = offered();
    let options = fake.ctx().options();

    assert_eq!(options.set_int(OptionKey::VSYNC, 0, Persistence::Transient), Ok(()));
    assert_eq!(options.get_int(OptionKey::VSYNC), Some(0));
    assert_eq!(options.release_transient(OptionKey::VSYNC), Ok(()));

    assert_eq!(options.get_int(OptionKey::VSYNC), Some(1));
    assert_eq!(fake.option_writes(), ["set_int vsync=0 transient", "release vsync restored=1"]);
}

#[test]
fn a_value_the_user_set_stays_after_the_release() {
    let fake = offered();
    let options = fake.ctx().options();
    options.set_int(OptionKey::VSYNC, 0, Persistence::Transient).unwrap();

    fake.set_options(FakeOptions { view_distance: 18, ints: [(OptionKey::VSYNC, 2)].into_iter().collect() });

    assert_eq!(options.release_transient(OptionKey::VSYNC), Ok(()));
    assert_eq!(options.get_int(OptionKey::VSYNC), Some(2));
    assert_eq!(fake.option_writes(), ["set_int vsync=0 transient", "release vsync kept=2"]);
}

#[test]
fn the_calls_check_the_gate_the_area_and_the_key() {
    let fake = offered();
    let options = fake.ctx().options();
    fake.set_options(FakeOptions { view_distance: 12, ints: [(OptionKey::VSYNC, 1)].into_iter().collect() });

    // The fake has no frame-rate option.
    assert_eq!(options.get_int(OptionKey::MAX_FRAMERATE), None);
    assert_eq!(options.set_int(OptionKey::MAX_FRAMERATE, 0, Persistence::Transient), Err(ApiError::NotAvailable));
    fake.withdraw_area(Area::OPTIONS_INT);
    assert_eq!(options.set_int(OptionKey::VSYNC, 0, Persistence::Transient), Err(ApiError::NotAvailable));
    assert!(fake.option_writes().is_empty());
}

#[test]
fn a_loader_before_services_v12_has_no_options() {
    let fake = FakeGame::with_v11_services();
    fake.open_gate();
    let options = fake.ctx().options();

    assert_eq!(options.get_int(OptionKey::VSYNC), None);
    assert_eq!(options.set_int(OptionKey::VSYNC, 0, Persistence::Transient), Err(ApiError::NotAvailable));
    assert_eq!(options.release_transient(OptionKey::VSYNC), Err(ApiError::NotAvailable));
    assert!(!fake.ctx().supports(Area::OPTIONS_INT));
}
