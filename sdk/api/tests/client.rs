use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::{Arc, Mutex};

use amml_abi::ClientTick;
use amml_api::testing::{FakeClient, FakeGame};
use amml_api::{AmmlMod, ApiError, Area, Ctx, DisableCtx, HookOrder, Screen};

fn tick() -> ClientTick {
    ClientTick { header: FakeGame::header::<ClientTick>() }
}

fn in_world() -> FakeClient {
    FakeClient { local_player: true, world_active: true, top_screen: Screen::Other, ..FakeClient::default() }
}

#[test]
fn a_listener_reads_the_client_of_its_event() {
    let fake = FakeGame::new();
    fake.offer::<ClientTick>();
    fake.offer_area(Area::CLIENT_READ);
    fake.set_client(in_world());
    fake.open_gate();
    let seen = Arc::new(Mutex::new(Vec::new()));
    let record = seen.clone();
    let _listener = fake
        .ctx()
        .events()
        .listen::<ClientTick>(HookOrder::NORMAL, move |_, at| {
            let client = at.client().unwrap();
            record.lock().unwrap().push((client.world_active(), client.local_player().is_some(), client.top_screen()));
        })
        .unwrap();

    fake.raise_with_client(&mut tick());

    // `client.top_screen` is not offered, so that read gives `None`.
    assert_eq!(*seen.lock().unwrap(), [(Some(true), true, None)]);
}

#[test]
fn client_writes_go_through_the_table() {
    let fake = FakeGame::new();
    fake.offer::<ClientTick>();
    fake.offer_area(Area::CLIENT_HUD);
    fake.set_client(in_world());
    fake.open_gate();
    let seen = Arc::new(Mutex::new(Vec::new()));
    let record = seen.clone();
    let _listener = fake
        .ctx()
        .events()
        .listen::<ClientTick>(HookOrder::NORMAL, move |_, at| {
            let client = at.client().unwrap();
            let set = client.set_hide_hud(true);
            let pause = client.open_pause_screen();
            record.lock().unwrap().push((set, pause, client.hide_hud()));
        })
        .unwrap();

    fake.raise_with_client(&mut tick());

    assert_eq!(*seen.lock().unwrap(), [(Ok(()), Err(ApiError::NotAvailable), Some(true))]);
    assert_eq!(fake.client_writes(), ["hide_hud=true"]);
}

#[test]
fn open_pause_is_refused_when_the_option_is_off() {
    let fake = FakeGame::new();
    fake.offer::<ClientTick>();
    fake.offer_area(Area::CLIENT_PAUSE);
    fake.set_client(FakeClient { pause_on_focus_lost: false, ..in_world() });
    fake.open_gate();
    let seen = Arc::new(Mutex::new(Vec::new()));
    let record = seen.clone();
    let _listener = fake
        .ctx()
        .events()
        .listen::<ClientTick>(HookOrder::NORMAL, move |_, at| {
            let client = at.client().unwrap();
            record.lock().unwrap().push((client.open_pause_screen(), client.top_screen()));
        })
        .unwrap();

    fake.raise_with_client(&mut tick());

    assert_eq!(*seen.lock().unwrap(), [(Err(ApiError::Refused), None)]);
    assert!(fake.client_writes().is_empty());
}

#[test]
fn open_pause_opens_the_pause_screen_when_the_option_is_on() {
    let fake = FakeGame::new();
    fake.offer::<ClientTick>();
    fake.offer_area(Area::CLIENT_PAUSE);
    fake.offer_area(Area::CLIENT_TOP_SCREEN);
    fake.set_client(in_world());
    fake.open_gate();
    let seen = Arc::new(Mutex::new(Vec::new()));
    let record = seen.clone();
    let _listener = fake
        .ctx()
        .events()
        .listen::<ClientTick>(HookOrder::NORMAL, move |_, at| {
            let client = at.client().unwrap();
            record.lock().unwrap().push((client.open_pause_screen(), client.top_screen()));
        })
        .unwrap();

    fake.raise_with_client(&mut tick());

    assert_eq!(*seen.lock().unwrap(), [(Ok(()), Some(Screen::Pause))]);
    assert_eq!(fake.client_writes(), ["open_pause_screen"]);
}

#[test]
fn a_loader_without_a_client_table_gives_no_client() {
    let fake = FakeGame::with_v1_services();
    fake.offer::<ClientTick>();
    fake.offer_area(Area::CLIENT_READ);
    fake.set_client(in_world());
    fake.open_gate();
    let ctx = fake.ctx();
    let seen = Arc::new(Mutex::new(Vec::new()));
    let record = seen.clone();
    let _listener = ctx
        .events()
        .listen::<ClientTick>(HookOrder::NORMAL, move |_, at| record.lock().unwrap().push(at.client().is_none()))
        .unwrap();

    fake.raise_with_client(&mut tick());

    assert_eq!(*seen.lock().unwrap(), [true]);
    assert!(!ctx.supports(Area::CLIENT_READ));
}

#[test]
fn an_event_raised_without_a_client_gives_none() {
    let fake = FakeGame::new();
    fake.offer::<ClientTick>();
    fake.offer_area(Area::CLIENT_READ);
    fake.set_client(in_world());
    fake.open_gate();
    let seen = Arc::new(Mutex::new(Vec::new()));
    let record = seen.clone();
    let _listener = fake
        .ctx()
        .events()
        .listen::<ClientTick>(HookOrder::NORMAL, move |_, at| record.lock().unwrap().push(at.client().is_none()))
        .unwrap();

    fake.raise(&mut tick());

    assert_eq!(*seen.lock().unwrap(), [true]);
}

#[test]
fn supports_follows_the_area_book() {
    let fake = FakeGame::new();
    let ctx = fake.ctx();
    assert!(ctx.supports(Area::INPUT_LEASE));
    assert!(!ctx.supports(Area::KEY_INJECT));

    fake.offer_area(Area::KEY_INJECT);
    assert!(ctx.supports(Area::KEY_INJECT));

    fake.withdraw_area(Area::KEY_INJECT);
    assert!(!ctx.supports(Area::KEY_INJECT));
}

// 0 = no client, 1 = a client in a world, 9 = disable did not run.
static CLIENT_AT_DISABLE: AtomicU8 = AtomicU8::new(9);

struct ClientMod;

impl AmmlMod for ClientMod {
    const NAMESPACE: &'static str = "acme";
    const NAME: &'static str = "client";

    fn enable(_ctx: &Ctx) -> Result<(), ApiError> {
        Ok(())
    }

    fn disable(_ctx: &Ctx, how: &DisableCtx) {
        let in_world = how.client().and_then(|client| client.world_active()) == Some(true);
        CLIENT_AT_DISABLE.store(u8::from(in_world), Ordering::SeqCst);
    }
}

#[test]
fn a_disable_gets_the_current_client_only_while_the_game_runs() {
    let fake = FakeGame::new();
    fake.offer_area(Area::CLIENT_READ);
    fake.set_client(in_world());
    fake.enable::<ClientMod>().unwrap();

    fake.disable::<ClientMod>();
    assert_eq!(CLIENT_AT_DISABLE.load(Ordering::SeqCst), 0);

    fake.open_gate();
    fake.enable::<ClientMod>().unwrap();
    fake.disable::<ClientMod>();
    assert_eq!(CLIENT_AT_DISABLE.load(Ordering::SeqCst), 1);
}

// What `is_alive` gave in one `ClientTick` on `fake`, with `client` set.
fn alive_in(fake: &FakeGame, client: FakeClient) -> Option<bool> {
    fake.offer::<ClientTick>();
    fake.offer_area(Area::CLIENT_READ);
    fake.set_client(client);
    let seen = Arc::new(Mutex::new(Vec::new()));
    let record = seen.clone();
    let _listener = fake
        .ctx()
        .events()
        .listen::<ClientTick>(HookOrder::NORMAL, move |_, at| {
            let alive = at.client().and_then(|client| client.local_player()).and_then(|player| player.is_alive());
            record.lock().unwrap().push(alive);
        })
        .unwrap();
    fake.raise_with_client(&mut tick());
    let seen = seen.lock().unwrap();
    seen.first().copied().flatten()
}

#[test]
fn is_alive_reads_the_local_player() {
    let fake = FakeGame::new();
    fake.offer_area(Area::ACTOR_READ);
    fake.open_gate();
    assert_eq!(alive_in(&fake, FakeClient { player_alive: Some(true), ..in_world() }), Some(true));
    assert_eq!(alive_in(&fake, FakeClient { player_alive: Some(false), ..in_world() }), Some(false));
}

#[test]
fn is_alive_is_none_without_the_area() {
    let fake = FakeGame::new();
    fake.open_gate();
    assert_eq!(alive_in(&fake, FakeClient { player_alive: Some(true), ..in_world() }), None);
    assert!(!fake.ctx().supports(Area::ACTOR_READ));
}

#[test]
fn is_alive_is_none_before_the_gate_opens() {
    let fake = FakeGame::new();
    fake.offer_area(Area::ACTOR_READ);
    assert_eq!(alive_in(&fake, FakeClient { player_alive: Some(true), ..in_world() }), None);
}

#[test]
fn is_alive_is_none_on_a_loader_without_the_actor_table() {
    let fake = FakeGame::with_v3_services();
    fake.offer_area(Area::ACTOR_READ);
    fake.open_gate();
    assert_eq!(alive_in(&fake, FakeClient { player_alive: Some(true), ..in_world() }), None);
    assert!(!fake.ctx().supports(Area::ACTOR_READ));
}

#[test]
fn is_alive_is_none_when_the_loader_cannot_read_it() {
    let fake = FakeGame::new();
    fake.offer_area(Area::ACTOR_READ);
    fake.open_gate();
    assert_eq!(alive_in(&fake, in_world()), None);
}

// Runs `body` with the client of one `ClientTick` after offering `areas`; gives what `body` gave.
fn with_client<R: Send + 'static>(
    fake: &FakeGame,
    areas: &[Area],
    body: impl Fn(&amml_api::Client<'_>) -> R + Send + 'static,
) -> R {
    fake.offer::<ClientTick>();
    for area in areas {
        fake.offer_area(*area);
    }
    fake.open_gate();
    let seen = Arc::new(Mutex::new(None));
    let record = seen.clone();
    let _listener = fake
        .ctx()
        .events()
        .listen::<ClientTick>(HookOrder::NORMAL, move |_, at| {
            *record.lock().unwrap() = Some(body(&at.client().unwrap()));
        })
        .unwrap();
    fake.raise_with_client(&mut tick());
    let taken = seen.lock().unwrap().take();
    taken.unwrap()
}

#[test]
fn the_title_screen_actions_reach_the_game_and_the_world_actions_are_refused() {
    let fake = FakeGame::new();
    fake.set_client(FakeClient::default());
    let areas = [
        Area::CLIENT_CAN_JOIN,
        Area::CLIENT_START_LOCAL_WORLD,
        Area::CLIENT_JOIN_SERVER,
        Area::CLIENT_OPEN_MENU,
        Area::CLIENT_LEAVE_WORLD,
        Area::CLIENT_SEND_COMMAND,
    ];

    let results = with_client(&fake, &areas, |client| {
        (
            client.can_join(),
            client.start_local_world("abc="),
            client.join_server(amml_api::JoinTarget { host: "example.test", port: 19132, name: None }),
            client.open_menu(amml_api::Menu::Servers),
            client.leave_world(),
            client.send_command("/say hi"),
        )
    });

    assert_eq!(
        results,
        (
            Some(true),
            Ok(amml_api::StartWorldOutcome::Started),
            Ok(amml_api::JoinId(1)),
            Ok(()),
            Err(ApiError::Refused),
            Err(ApiError::Refused)
        )
    );
    assert_eq!(fake.client_writes(), ["start_local_world abc=", "join_server example.test:19132", "open_menu servers"]);
}

#[test]
fn the_world_actions_reach_the_game_and_the_title_screen_actions_are_refused() {
    let fake = FakeGame::new();
    fake.set_client(in_world());
    let areas = [
        Area::CLIENT_START_LOCAL_WORLD,
        Area::CLIENT_JOIN_SERVER,
        Area::CLIENT_SEND_COMMAND,
        Area::HUD_SELECT_HOTBAR,
        Area::CLIENT_LEAVE_WORLD,
    ];

    let results = with_client(&fake, &areas, |client| {
        (
            client.start_local_world("abc="),
            client.join_server(amml_api::JoinTarget { host: "example.test", port: 19132, name: None }),
            client.send_command("/say hi"),
            client.send_command(&"x".repeat(amml_api::Client::MAX_COMMAND + 1)),
            client.hud().select_hotbar_slot(2),
            client.hud().select_hotbar_slot(9),
            client.leave_world(),
        )
    });

    assert_eq!(
        results,
        (
            Err(ApiError::Refused),
            Err(ApiError::Refused),
            Ok(()),
            Err(ApiError::Refused),
            Ok(()),
            Err(ApiError::Refused),
            Ok(())
        )
    );
    assert_eq!(fake.client_writes(), ["send_command /say hi", "select_hotbar_slot 2", "leave_world"]);
}

#[test]
fn a_start_the_game_is_not_ready_for_gives_not_ready_and_an_area_not_offered_gives_not_available() {
    let fake = FakeGame::new();
    fake.set_client(FakeClient { start_world_answer: None, ..FakeClient::default() });

    let results = with_client(&fake, &[Area::CLIENT_START_LOCAL_WORLD], |client| {
        (client.start_local_world("abc="), client.open_menu(amml_api::Menu::Servers), client.can_join())
    });

    assert_eq!(results, (Err(ApiError::NotReady), Err(ApiError::NotAvailable), None));
    assert!(fake.client_writes().is_empty());
}

#[test]
fn the_player_reads_give_the_position_rotation_and_dimension_of_the_fake() {
    let fake = FakeGame::new();
    fake.set_client(FakeClient {
        position: Some(amml_api::Vec3 { x: 1.0, y: 64.0, z: 2.0 }),
        eye_position: Some(amml_api::Vec3 { x: 1.0, y: 65.62, z: 2.0 }),
        rotation: Some(amml_api::Rotation { yaw: 90.0, pitch: 30.0 }),
        dimension: Some(amml_api::Dimension::NETHER),
        ..in_world()
    });

    let reads = with_client(&fake, &[Area::CLIENT_READ, Area::ACTOR_READ.at(2)], |client| {
        let player = client.local_player().unwrap();
        (player.position(), player.eye_position(), player.rotation(), player.dimension().and_then(|d| d.id()))
    });

    assert_eq!(
        reads,
        (
            Some(amml_api::Vec3 { x: 1.0, y: 64.0, z: 2.0 }),
            Some(amml_api::Vec3 { x: 1.0, y: 65.62, z: 2.0 }),
            Some(amml_api::Rotation { yaw: 90.0, pitch: 30.0 }),
            Some(amml_api::Dimension::NETHER)
        )
    );
}

#[test]
fn the_player_reads_need_actor_read_version_2() {
    let fake = FakeGame::new();
    fake.set_client(FakeClient { position: Some(amml_api::Vec3::default()), ..in_world() });

    let read =
        with_client(&fake, &[Area::CLIENT_READ, Area::ACTOR_READ], |client| client.local_player().unwrap().position());

    assert_eq!(read, None);
}

// The results of `note_activity` from one `ClientTick` per `FakeActivity`, with area `client.activity` offered when
// `offered`.
fn activity_results(
    offered: bool,
    activities: &[amml_api::testing::FakeActivity],
) -> (Vec<Result<(), ApiError>>, Vec<String>) {
    let fake = FakeGame::new();
    fake.offer::<ClientTick>();
    if offered {
        fake.offer_area(Area::CLIENT_ACTIVITY);
    }
    fake.open_gate();
    let seen = Arc::new(Mutex::new(Vec::new()));
    let record = seen.clone();
    let _listener = fake
        .ctx()
        .events()
        .listen::<ClientTick>(HookOrder::NORMAL, move |_, at| {
            record.lock().unwrap().push(at.client().unwrap().note_activity());
        })
        .unwrap();
    for activity in activities {
        fake.set_client(FakeClient { activity: *activity, ..FakeClient::default() });
        fake.raise_with_client(&mut tick());
    }
    let results = seen.lock().unwrap().clone();
    (results, fake.client_writes())
}

#[test]
fn note_activity_gives_what_the_game_gives_and_records_only_a_call_that_ran() {
    use amml_api::testing::FakeActivity;
    let (results, writes) =
        activity_results(true, &[FakeActivity::Ok, FakeActivity::Refused, FakeActivity::NotAvailable]);

    assert_eq!(results, [Ok(()), Err(ApiError::Refused), Err(ApiError::NotAvailable)]);
    assert_eq!(writes, ["note_activity"]);
}

#[test]
fn note_activity_without_its_area_is_not_available() {
    let (results, writes) = activity_results(false, &[amml_api::testing::FakeActivity::Ok]);

    assert_eq!(results, [Err(ApiError::NotAvailable)]);
    assert!(writes.is_empty());
}

#[test]
fn a_direct_join_reaches_the_game_on_the_title_screen_and_is_refused_in_a_world_or_without_its_area() {
    let target = amml_api::JoinTarget { host: "127.0.0.1", port: 28231, name: Some("DejaVu replay") };
    let join = move |client: &amml_api::Client<'_>| client.join_server_direct(target);

    let title = FakeGame::new();
    title.set_client(FakeClient::default());
    let joined = with_client(&title, &[Area::CLIENT_JOIN_SERVER_DIRECT], join);
    assert_eq!(title.client_writes(), ["join_server_direct 127.0.0.1:28231 DejaVu replay"]);
    drop(title);
    let world = FakeGame::new();
    world.set_client(in_world());
    let in_world = with_client(&world, &[Area::CLIENT_JOIN_SERVER_DIRECT], join);
    assert!(world.client_writes().is_empty());
    drop(world);
    let without = FakeGame::new();
    without.set_client(FakeClient::default());
    let not_offered = with_client(&without, &[], join);
    drop(without);
    let named = FakeGame::new();
    named.set_client(FakeClient::default());
    let host_name = with_client(&named, &[Area::CLIENT_JOIN_SERVER_DIRECT], |client| {
        client.join_server_direct(amml_api::JoinTarget { host: "replay.example", port: 28231, name: None })
    });

    assert_eq!(
        (joined, in_world, not_offered, host_name),
        (
            Ok(amml_api::JoinId(1)),
            Err(ApiError::Refused),
            Err(ApiError::NotAvailable),
            Err(ApiError::Loader(amml_abi::AmmlStatus::INVALID_ARGUMENT))
        )
    );
}

#[test]
fn the_title_read_and_the_call_back_to_the_title_reach_the_fake() {
    let areas = [Area::CLIENT_TITLE_SCREEN, Area::CLIENT_GO_TO_TITLE];
    let title = FakeGame::new();
    title.set_client(FakeClient { title_screen: true, ..FakeClient::default() });
    let on_title = with_client(&title, &areas, |client| (client.on_title_screen(), client.go_to_title()));
    assert_eq!(title.client_writes(), ["go_to_title"]);
    drop(title);
    let world = FakeGame::new();
    world.set_client(FakeClient { title_screen: true, ..in_world() });
    let in_world = with_client(&world, &areas, |client| (client.on_title_screen(), client.go_to_title()));
    drop(world);
    let without = FakeGame::new();
    without.set_client(FakeClient::default());
    let not_offered = with_client(&without, &[], |client| (client.on_title_screen(), client.go_to_title()));

    assert_eq!(
        (on_title, in_world, not_offered),
        ((Some(true), Ok(())), (Some(false), Err(ApiError::Refused)), (None, Err(ApiError::NotAvailable)))
    );
}

#[test]
fn the_hit_result_comes_from_the_fake_local_player() {
    let fake = FakeGame::new();
    let hit = amml_api::BlockHit {
        pos: amml_api::BlockPos { x: 4, y: 70, z: -2 },
        point: amml_api::Vec3 { x: 4.5, y: 70.25, z: -1.0 },
        face: amml_api::Face::Up,
    };
    fake.set_client(FakeClient { hit_result: Some(amml_api::HitResult::Block(hit)), ..in_world() });

    let read = with_client(&fake, &[Area::CLIENT_READ, Area::PLAYER_HIT_RESULT], |client| {
        client.local_player().and_then(|player| player.hit_result())
    });

    assert_eq!(read, Some(amml_api::HitResult::Block(hit)));
}
