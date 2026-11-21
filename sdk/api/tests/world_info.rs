use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use amml_abi::ClientTick;
use amml_api::testing::{FakeClient, FakeGame};
use amml_api::{Area, Connection, ConnectionKind, HookOrder, Screen, WorldInfo};

fn tick() -> ClientTick {
    ClientTick { header: FakeGame::header::<ClientTick>() }
}

fn in_world(info: Option<WorldInfo>) -> FakeClient {
    FakeClient {
        local_player: true,
        world_active: true,
        top_screen: Screen::Other,
        world_info: info,
        ..FakeClient::default()
    }
}

// What `Client::world_info` gives in one `ClientTick`, after offering `areas`.
fn read(fake: &FakeGame, areas: &[Area]) -> Option<WorldInfo> {
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
            *record.lock().unwrap() = Some(at.client().unwrap().world_info());
        })
        .unwrap();
    fake.raise_with_client(&mut tick());
    let taken = seen.lock().unwrap().take();
    taken.unwrap()
}

fn full_info() -> WorldInfo {
    WorldInfo {
        storage_folder: Some(PathBuf::from("C:/worlds/My World")),
        level_name: Some("My World".to_string()),
        seed: Some(0xFEED_0000_0000_1234),
        realm: Some(false),
        commands_enabled: Some(true),
        connection: Some(Connection {
            kind: ConnectionKind::Remote,
            host: "play.example.net".to_string(),
            port: 19132,
        }),
    }
}

#[test]
fn world_info_fields_come_from_the_fake() {
    let fake = FakeGame::new();
    fake.set_client(in_world(Some(full_info())));

    let info = read(&fake, &[Area::CLIENT_WORLD_INFO]);

    assert_eq!(info, Some(full_info()));
}

#[test]
fn a_fact_the_game_does_not_give_is_none_and_the_others_stay() {
    let fake = FakeGame::new();
    let partial = WorldInfo { level_name: None, connection: None, seed: Some(7), ..WorldInfo::default() };
    fake.set_client(in_world(Some(partial.clone())));

    assert_eq!(read(&fake, &[Area::CLIENT_WORLD_INFO]), Some(partial));
}

#[test]
fn a_host_longer_than_the_first_buffer_is_read_whole() {
    let fake = FakeGame::new();
    let host = "h".repeat(700);
    let info = WorldInfo {
        connection: Some(Connection { kind: ConnectionKind::Remote, host: host.clone(), port: 1 }),
        ..WorldInfo::default()
    };
    fake.set_client(in_world(Some(info)));

    let read = read(&fake, &[Area::CLIENT_WORLD_INFO]).unwrap();

    assert_eq!(read.connection.unwrap().host, host);
}

#[test]
fn world_info_is_none_outside_a_world() {
    let fake = FakeGame::new();
    fake.set_client(in_world(None));

    assert_eq!(read(&fake, &[Area::CLIENT_WORLD_INFO]), None);
}

#[test]
fn world_info_is_none_without_the_area() {
    let fake = FakeGame::new();
    fake.set_client(in_world(Some(full_info())));

    assert_eq!(read(&fake, &[]), None);
}
