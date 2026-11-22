use std::sync::{Arc, Mutex};

use amml_abi::{AmmlOrigin, AmmlThread, BuildBlock, ClientTick, UsePressed};
use amml_api::testing::{FakeClient, FakeGame};
use amml_api::{ApiError, Area, BlockPos, BuildBlockExt, Cancel, Event, Face, Hand, HookOrder, Screen, Vec3};

fn in_world() -> FakeClient {
    FakeClient {
        local_player: true,
        world_active: true,
        top_screen: Screen::Other,
        build_delay: Some(200),
        max_pick_range: Some(6.7),
        ..FakeClient::default()
    }
}

// A fake with both events and both areas offered, a client in a world and an open gate.
fn game() -> FakeGame {
    let fake = FakeGame::new();
    fake.offer::<UsePressed>();
    fake.offer::<BuildBlock>();
    fake.offer::<ClientTick>();
    fake.offer_area(Area::CLIENT_READ);
    fake.offer_area(Area::PLAYER_BUILD);
    fake.offer_area(Area::PLAYER_PICK_RANGE);
    fake.set_client(in_world());
    fake.open_gate();
    fake
}

const POS: BlockPos = BlockPos { x: 4, y: 70, z: -2 };
const CLICK: Vec3 = Vec3 { x: 4.5, y: 71.0, z: -1.75 };

// What `f` gives for the local player's game mode, from one `ClientTick` with the fake client.
fn with_game_mode<T: Send + 'static>(
    fake: &FakeGame,
    f: impl Fn(&amml_api::GameMode<'_>) -> T + Send + 'static,
) -> Option<T> {
    let out = Arc::new(Mutex::new(None));
    let record = out.clone();
    let _listener = fake
        .ctx()
        .events()
        .listen::<ClientTick>(HookOrder::NORMAL, move |_, at| {
            let player = at.client().and_then(|client| client.local_player());
            let value = player.and_then(|player| player.game_mode()).map(|mode| f(&mode));
            *record.lock().unwrap() = value;
        })
        .unwrap();
    fake.raise_with_client(&mut ClientTick { header: FakeGame::header::<ClientTick>() });
    out.lock().unwrap().take()
}

#[test]
fn game_mode_calls_from_the_fake() {
    let fake = game();

    assert_eq!(with_game_mode(&fake, |mode| mode.build_delay()), Some(Some(200)));
    assert_eq!(with_game_mode(&fake, |mode| mode.max_pick_range()), Some(Some(6.7)));
    assert_eq!(with_game_mode(&fake, |mode| mode.build_at(POS, Face::Up, CLICK)), Some(Ok(true)));
    assert_eq!(fake.builds(), ["build_at 4,70,-2 face=up"]);
}

#[test]
fn the_game_mode_calls_need_their_areas() {
    let fake = game();
    fake.withdraw_area(Area::PLAYER_BUILD);
    fake.withdraw_area(Area::PLAYER_PICK_RANGE);

    assert_eq!(with_game_mode(&fake, |mode| mode.build_delay()), Some(None));
    assert_eq!(with_game_mode(&fake, |mode| mode.max_pick_range()), Some(None));
    assert_eq!(with_game_mode(&fake, |mode| mode.build_at(POS, Face::Up, CLICK)), Some(Err(ApiError::NotAvailable)));
    assert!(fake.builds().is_empty());
}

#[test]
fn build_at_raises_build_block_with_origin_this_mod_and_a_cancel_stops_it() {
    let fake = game();
    let seen = Arc::new(Mutex::new(Vec::new()));
    let record = seen.clone();
    let _listener = fake
        .ctx()
        .events()
        .listen::<BuildBlock>(HookOrder::WRITE, move |event, at| {
            record.lock().unwrap().push((event.header().origin, at.thread, event.pos()));
            event.cancel();
        })
        .unwrap();

    let built = with_game_mode(&fake, |mode| mode.build_at(POS, Face::North, CLICK));

    assert_eq!(built, Some(Ok(false)));
    assert_eq!(*seen.lock().unwrap(), [(AmmlOrigin::THIS_MOD, AmmlThread::GAME, POS)]);
    assert!(fake.builds().is_empty());
}

#[test]
fn build_at_inside_a_use_pressed_listener_raises_build_block_on_the_same_stack() {
    let fake = game();
    let seen = Arc::new(Mutex::new(Vec::new()));
    let builds = seen.clone();
    let _build = fake
        .ctx()
        .events()
        .listen::<BuildBlock>(HookOrder::NORMAL, move |event, _| {
            builds.lock().unwrap().push(format!("build origin={}", event.header().origin.0));
        })
        .unwrap();
    let uses = seen.clone();
    let _use = fake
        .ctx()
        .events()
        .listen::<UsePressed>(HookOrder::WRITE, move |event, at| {
            let player = at.client().and_then(|client| client.local_player()).unwrap();
            let built = player.game_mode().unwrap().build_at(POS, Face::Up, CLICK);
            uses.lock().unwrap().push(format!("use built={built:?}"));
            event.cancel();
        })
        .unwrap();

    assert!(fake.raise_use_pressed());

    assert_eq!(
        *seen.lock().unwrap(),
        [format!("build origin={}", AmmlOrigin::THIS_MOD.0), "use built=Ok(true)".to_string()]
    );
}

#[test]
fn build_block_ext_reads_the_fields() {
    let fake = game();
    let seen = Arc::new(Mutex::new(Vec::new()));
    let record = seen.clone();
    let _listener = fake
        .ctx()
        .events()
        .listen::<BuildBlock>(HookOrder::NORMAL, move |event, at| {
            let client = at.client().is_some();
            record.lock().unwrap().push((
                event.pos(),
                event.face(),
                event.hand(),
                event.sim_tick(),
                event.is_local_player(),
                at.thread,
                client,
                event.header().origin,
            ));
        })
        .unwrap();

    assert!(!fake.raise_build_block(AmmlThread::SERVER, AmmlOrigin::GAME, POS, Face::West, Hand::Off, true, false));
    assert!(!fake.raise_build_block(AmmlThread::GAME, AmmlOrigin::GAME, POS, Face::Down, Hand::Main, false, true));

    assert_eq!(
        *seen.lock().unwrap(),
        [
            (POS, Face::West, Hand::Off, true, false, AmmlThread::SERVER, false, AmmlOrigin::GAME),
            (POS, Face::Down, Hand::Main, false, true, AmmlThread::GAME, true, AmmlOrigin::GAME),
        ]
    );
}

#[test]
fn a_cancelled_use_press_is_reported() {
    let fake = game();
    assert!(!fake.raise_use_pressed());
    let _listener = fake.ctx().events().listen::<UsePressed>(HookOrder::WRITE, |event, _| event.cancel()).unwrap();

    assert!(fake.raise_use_pressed());
}
