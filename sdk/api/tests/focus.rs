use std::sync::{Arc, Mutex};

use amml_abi::{AppFocusGained, AppFocusLost, ClientTick};
use amml_api::testing::{FakeClient, FakeGame};
use amml_api::{ApiError, Area, Cancel, Event, HookOrder, Screen};

fn tick() -> ClientTick {
    ClientTick { header: FakeGame::header::<ClientTick>() }
}

fn in_world() -> FakeClient {
    FakeClient { local_player: true, world_active: true, top_screen: Screen::Other, ..FakeClient::default() }
}

// A fake with the focus events and the replay area offered, a client and an open gate.
fn game() -> FakeGame {
    let fake = FakeGame::new();
    fake.offer::<AppFocusLost>();
    fake.offer::<AppFocusGained>();
    fake.offer::<ClientTick>();
    fake.offer_area(Area::CLIENT_FOCUS_REPLAY);
    fake.set_client(in_world());
    fake.open_gate();
    fake
}

// Listens to `AppFocusLost`; with `cancel` the listener cancels it.
fn listen_loss(fake: &FakeGame, cancel: bool) -> amml_api::Listener {
    fake.ctx()
        .events()
        .listen::<AppFocusLost>(HookOrder::NORMAL, move |event, _| {
            if cancel {
                event.cancel();
            }
        })
        .unwrap()
}

// The results of `replay_app_focus_lost` from one `ClientTick`.
fn replay(fake: &FakeGame) -> Vec<Result<(), ApiError>> {
    let results = Arc::new(Mutex::new(Vec::new()));
    let record = results.clone();
    let _listener = fake
        .ctx()
        .events()
        .listen::<ClientTick>(HookOrder::NORMAL, move |_, at| {
            record.lock().unwrap().push(at.client().unwrap().replay_app_focus_lost());
        })
        .unwrap();
    fake.raise_with_client(&mut tick());
    results.lock().unwrap().clone()
}

#[test]
fn a_listener_cancels_the_loss() {
    let fake = game();
    let _listener = listen_loss(&fake, true);

    assert!(fake.raise_app_focus_lost());
}

#[test]
fn a_loss_no_listener_cancels_is_not_held() {
    let fake = game();
    let _listener = listen_loss(&fake, false);

    assert!(!fake.raise_app_focus_lost());
    assert_eq!(replay(&fake), [Ok(())]);
    assert_eq!(fake.focus_replays(), 0);
}

#[test]
fn replay_runs_the_game_handler_once_for_a_cancelled_loss() {
    let fake = game();
    let _listener = listen_loss(&fake, true);
    assert!(fake.raise_app_focus_lost());

    assert_eq!(replay(&fake), [Ok(())]);
    assert_eq!(fake.focus_replays(), 1);
    assert_eq!(replay(&fake), [Ok(())]);
    assert_eq!(fake.focus_replays(), 1);
}

#[test]
fn a_gain_forgets_the_held_loss() {
    let fake = game();
    let _listener = listen_loss(&fake, true);
    assert!(fake.raise_app_focus_lost());
    fake.raise_app_focus_gained();

    assert_eq!(replay(&fake), [Ok(())]);
    assert_eq!(fake.focus_replays(), 0);
}

#[test]
fn a_gain_listener_sees_the_gain_after_the_game() {
    let fake = game();
    let seen = Arc::new(Mutex::new(Vec::new()));
    let record = seen.clone();
    let _listener = fake
        .ctx()
        .events()
        .listen::<AppFocusGained>(HookOrder::NORMAL, move |event, _| {
            record.lock().unwrap().push((event.header().phase, event.is_cancelled()));
        })
        .unwrap();

    fake.raise_app_focus_gained();

    assert_eq!(*seen.lock().unwrap(), [(amml_abi::AmmlPhase::AFTER, false)]);
}

#[test]
fn replay_needs_its_area() {
    let fake = game();
    fake.withdraw_area(Area::CLIENT_FOCUS_REPLAY);

    assert_eq!(replay(&fake), [Err(ApiError::NotAvailable)]);
    assert_eq!(fake.focus_replays(), 0);
}
