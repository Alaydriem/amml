use std::cell::RefCell;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use amml_abi::{ClientTick, JoinStarted, ServerJoinStarting, WorldStorageClosed, WorldStorageOpened};
use amml_api::testing::{FakeClient, FakeGame};
use amml_api::{Area, HookOrder, JoinStartedExt, ServerJoinStartingExt, WorldStorageOpenedExt};

thread_local! {
    // The copies the listeners keep past their call; the fake calls them on the test thread.
    static KEPT_JOIN: RefCell<Option<JoinStarted>> = const { RefCell::new(None) };
    static KEPT_STORAGE: RefCell<Option<WorldStorageOpened>> = const { RefCell::new(None) };
}

#[test]
fn join_started_reads_the_local_flag_and_names_in_the_call() {
    let fake = FakeGame::new();
    fake.offer::<JoinStarted>();
    let seen = Arc::new(Mutex::new(Vec::new()));
    let record = seen.clone();
    let _listener = fake
        .ctx()
        .events()
        .listen::<JoinStarted>(HookOrder::NORMAL, move |event, _| {
            let copy = *event;
            record.lock().unwrap().push((
                event.local(),
                event.server_name().map(str::to_string),
                event.world_name().map(str::to_string),
                copy.server_name().is_some(),
            ));
            KEPT_JOIN.with(|kept| *kept.borrow_mut() = Some(copy));
        })
        .unwrap();

    fake.raise_join_started(false, "play.example.net", "Survival");
    fake.raise_join_started(true, "", "My World");

    assert_eq!(
        *seen.lock().unwrap(),
        vec![
            (false, Some("play.example.net".to_string()), Some("Survival".to_string()), false),
            (true, None, Some("My World".to_string()), false),
        ]
    );
    let kept = KEPT_JOIN.with(|kept| kept.borrow_mut().take()).unwrap();
    assert_eq!((kept.server_name(), kept.world_name()), (None, None));
}

#[test]
fn server_join_starting_reads_its_fields_in_the_call() {
    let fake = FakeGame::new();
    fake.offer::<ServerJoinStarting>();
    let seen = Arc::new(Mutex::new(Vec::new()));
    let record = seen.clone();
    let _listener = fake
        .ctx()
        .events()
        .listen::<ServerJoinStarting>(HookOrder::NORMAL, move |event, _| {
            record.lock().unwrap().push((
                event.host().map(str::to_string),
                event.url().map(str::to_string),
                event.name().map(str::to_string),
                event.port(),
                event.transfer(),
            ));
        })
        .unwrap();

    fake.raise_server_join_starting("79.127.172.199", "play.example.net", "Example", 19132, true);
    fake.raise_server_join_starting("127.0.0.1", "", "", 0, false);

    assert_eq!(
        *seen.lock().unwrap(),
        vec![
            (
                Some("79.127.172.199".to_string()),
                Some("play.example.net".to_string()),
                Some("Example".to_string()),
                19132,
                true
            ),
            (Some("127.0.0.1".to_string()), None, None, 0, false),
        ]
    );
}

#[test]
fn storage_opened_reads_the_path_in_the_call_and_closed_has_no_fields() {
    let fake = FakeGame::new();
    fake.offer::<WorldStorageOpened>();
    fake.offer::<WorldStorageClosed>();
    let seen = Arc::new(Mutex::new(Vec::new()));
    let record = seen.clone();
    let _opened = fake
        .ctx()
        .events()
        .listen::<WorldStorageOpened>(HookOrder::NORMAL, move |event, _| {
            record.lock().unwrap().push(format!("opened {:?}", event.path()));
            KEPT_STORAGE.with(|kept| *kept.borrow_mut() = Some(*event));
        })
        .unwrap();
    let record = seen.clone();
    let _closed = fake
        .ctx()
        .events()
        .listen::<WorldStorageClosed>(HookOrder::NORMAL, move |_, _| {
            record.lock().unwrap().push("closed".to_string());
        })
        .unwrap();

    fake.raise_world_storage_opened("C:/worlds/My World");
    fake.raise_world_storage_closed();

    assert_eq!(
        *seen.lock().unwrap(),
        vec![format!("opened {:?}", Some(PathBuf::from("C:/worlds/My World").as_path())), "closed".to_string()]
    );
    let kept = KEPT_STORAGE.with(|kept| kept.borrow_mut().take()).unwrap();
    assert_eq!(kept.path(), None);
    assert_eq!(kept.path.len, "C:/worlds/My World".len());
}

fn tick() -> ClientTick {
    ClientTick { header: FakeGame::header::<ClientTick>() }
}

// `Client::in_gameplay` read inside a `ClientTick` listener, as a mod reads it.
fn read_in_gameplay(fake: &FakeGame) -> Option<bool> {
    let seen = Arc::new(Mutex::new(None));
    let record = seen.clone();
    let _listener = fake
        .ctx()
        .events()
        .listen::<ClientTick>(HookOrder::NORMAL, move |_, at| {
            *record.lock().unwrap() = Some(at.client().unwrap().in_gameplay());
        })
        .unwrap();
    fake.raise_with_client(&mut tick());
    let taken = seen.lock().unwrap().take();
    taken.unwrap()
}

#[test]
fn in_gameplay_from_the_fake() {
    let fake = FakeGame::new();
    fake.offer::<ClientTick>();
    fake.offer_area(Area::CLIENT_IN_GAMEPLAY);
    fake.set_client(FakeClient { in_gameplay: true, ..FakeClient::default() });
    fake.open_gate();
    assert_eq!(read_in_gameplay(&fake), Some(true));

    fake.set_client(FakeClient { in_gameplay: false, ..FakeClient::default() });
    assert_eq!(read_in_gameplay(&fake), Some(false));

    fake.withdraw_area(Area::CLIENT_IN_GAMEPLAY);
    assert_eq!(read_in_gameplay(&fake), None);
}
