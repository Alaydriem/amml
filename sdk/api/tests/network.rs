use std::sync::{Arc, Mutex};

use amml_abi::{PlayerListUpdate, PlayerSkinChanged};
use amml_api::testing::{FakeGame, FakePlayerEntry, FakePlayerList, FakeSkin};
use amml_api::{
    ActorUniqueId, AnimatedKind, Area, HookOrder, PixelFormat, PlayerListAction, PlayerListUpdateExt,
    PlayerSkinChangedExt, Uuid,
};

type Seen = Arc<Mutex<Vec<String>>>;

const ALEX: Uuid = Uuid([0xa1; 16]);
const BOB: Uuid = Uuid([0xb2; 16]);

fn serving(fake: &FakeGame) {
    fake.offer::<PlayerListUpdate>();
    fake.offer::<PlayerSkinChanged>();
    for area in
        [Area::NETWORK_SKIN, Area::NETWORK_SKIN_ANIMATED, Area::NETWORK_SKIN_GEOMETRY, Area::NETWORK_SKIN_DETAILS]
    {
        fake.offer_area(area);
    }
}

fn pixels(width: u32, height: u32) -> (u32, u32, Vec<u8>) {
    (width, height, (0..width * height * 4).map(|index| index as u8).collect())
}

fn entry(uuid: Uuid, actor_id: i64, name: &str, skin: Option<FakeSkin>) -> FakePlayerEntry {
    FakePlayerEntry { uuid, actor_id, name: name.to_string(), xuid: format!("xuid-{name}"), skin }
}

#[test]
fn entries_iterate_the_fake_list() {
    let fake = FakeGame::new();
    serving(&fake);
    let seen: Seen = Arc::new(Mutex::new(Vec::new()));
    let record = seen.clone();
    let _listener = fake
        .ctx()
        .events()
        .listen::<PlayerListUpdate>(HookOrder::NORMAL, move |event, _| {
            let mut lines = record.lock().unwrap();
            lines.push(format!("{:?}", event.action()));
            for entry in event.entries() {
                let skin = entry.skin();
                lines.push(format!(
                    "{:02x} {:?} {:?} {:?} image {:?} persona {:?} premium {:?} name {:?} patch {:?} json {:?}",
                    entry.uuid().0[0],
                    entry.actor_id(),
                    entry.name(),
                    entry.xuid(),
                    skin.as_ref().and_then(|skin| skin.image()).map(|view| (
                        view.width,
                        view.height,
                        view.format,
                        view.pixels.len()
                    )),
                    skin.as_ref().and_then(|skin| skin.is_persona()),
                    skin.as_ref().and_then(|skin| skin.is_premium()),
                    skin.as_ref().and_then(|skin| skin.default_geometry_name()),
                    skin.as_ref().and_then(|skin| skin.resource_patch()),
                    skin.as_ref().and_then(|skin| skin.geometry_json()),
                ));
            }
        })
        .unwrap();
    let skin = FakeSkin {
        image: Some(pixels(2, 2)),
        persona: true,
        geometry_name: Some("geometry.humanoid".to_string()),
        geometry_json: Some("{}".to_string()),
        resource_patch: Some("{\"geometry\":{}}".to_string()),
        ..FakeSkin::default()
    };

    let dispatch = fake.raise_player_list(&FakePlayerList {
        action: PlayerListAction::Add,
        entries: vec![entry(ALEX, 7, "Alex", Some(skin)), entry(BOB, -9, "Bob", None)],
    });

    assert!(!dispatch.panicked);
    assert_eq!(
        *seen.lock().unwrap(),
        [
            "Add".to_string(),
            format!(
                "a1 Some({:?}) Some(\"Alex\") Some(\"xuid-Alex\") image Some((2, 2, {:?}, 16)) persona Some(true) premium Some(false) name Some(\"geometry.humanoid\") patch Some(\"{{\\\"geometry\\\":{{}}}}\") json Some(\"{{}}\")",
                ActorUniqueId(7),
                PixelFormat::Rgba8
            ),
            format!(
                "b2 Some({:?}) Some(\"Bob\") Some(\"xuid-Bob\") image None persona None premium None name None patch None json None",
                ActorUniqueId(-9)
            ),
        ]
    );
}

#[test]
fn remove_entries_give_only_uuid() {
    let fake = FakeGame::new();
    serving(&fake);
    let seen: Seen = Arc::new(Mutex::new(Vec::new()));
    let record = seen.clone();
    let _listener = fake
        .ctx()
        .events()
        .listen::<PlayerListUpdate>(HookOrder::NORMAL, move |event, _| {
            let mut lines = record.lock().unwrap();
            lines.push(format!("{:?}", event.action()));
            for entry in event.entries() {
                lines.push(format!(
                    "{:02x} {:?} {:?} {:?} {}",
                    entry.uuid().0[0],
                    entry.actor_id(),
                    entry.name(),
                    entry.xuid(),
                    entry.skin().is_some()
                ));
            }
        })
        .unwrap();

    // The skin of a Remove entry is never read, even where the fake list holds one.
    let remove = entry(BOB, 5, "Bob", Some(FakeSkin { image: Some(pixels(1, 1)), ..FakeSkin::default() }));
    fake.raise_player_list(&FakePlayerList { action: PlayerListAction::Remove, entries: vec![remove] });

    assert_eq!(*seen.lock().unwrap(), ["Remove".to_string(), "b2 None None None false".to_string()]);
}

#[test]
fn animated_face_kind_is_face() {
    let fake = FakeGame::new();
    serving(&fake);
    let seen: Seen = Arc::new(Mutex::new(Vec::new()));
    let record = seen.clone();
    let _listener = fake
        .ctx()
        .events()
        .listen::<PlayerSkinChanged>(HookOrder::NORMAL, move |event, _| {
            let skin = event.skin().unwrap();
            let mut lines = record.lock().unwrap();
            for (kind, view) in skin.animated_images() {
                lines.push(format!("{kind:?} {}x{}", view.width, view.height));
            }
        })
        .unwrap();
    let skin = FakeSkin {
        animated: vec![
            (AnimatedKind::Face, pixels(8, 8)),
            (AnimatedKind::Body32x32, pixels(32, 32)),
            (AnimatedKind::Body128x128, pixels(2, 2)),
            (AnimatedKind::Other(9), pixels(1, 1)),
            // A blob that is not width * height * 4 gives no view and no entry.
            (AnimatedKind::Face, (4, 4, vec![0; 3])),
        ],
        ..FakeSkin::default()
    };

    fake.raise_skin(ALEX, Some(skin));

    assert_eq!(AnimatedKind::from(1), AnimatedKind::Face);
    assert_eq!(
        *seen.lock().unwrap(),
        [
            "Face 8x8".to_string(),
            "Body32x32 32x32".to_string(),
            "Body128x128 2x2".to_string(),
            "Other(9) 1x1".to_string()
        ]
    );
}

#[test]
fn the_skin_event_gives_the_uuid_and_the_skin() {
    let fake = FakeGame::new();
    serving(&fake);
    let seen: Seen = Arc::new(Mutex::new(Vec::new()));
    let record = seen.clone();
    let _listener = fake
        .ctx()
        .events()
        .listen::<PlayerSkinChanged>(HookOrder::NORMAL, move |event, _| {
            record.lock().unwrap().push(format!(
                "{:?} {:?}",
                event.uuid().map(|uuid| uuid.0[0]),
                event.skin().and_then(|skin| skin.image()).map(|view| (view.width, view.height))
            ));
        })
        .unwrap();

    fake.raise_skin(ALEX, Some(FakeSkin { image: Some(pixels(4, 2)), ..FakeSkin::default() }));
    fake.raise_skin(BOB, None);

    assert_eq!(*seen.lock().unwrap(), ["Some(161) Some((4, 2))".to_string(), "Some(178) None".to_string()]);
}

#[test]
fn a_blob_that_is_not_width_times_height_times_four_gives_no_image() {
    let fake = FakeGame::new();
    serving(&fake);
    let seen: Seen = Arc::new(Mutex::new(Vec::new()));
    let record = seen.clone();
    let _listener = fake
        .ctx()
        .events()
        .listen::<PlayerSkinChanged>(HookOrder::NORMAL, move |event, _| {
            let skin = event.skin().unwrap();
            record.lock().unwrap().push(format!("{} {:?}", skin.image().is_some(), skin.is_persona()));
        })
        .unwrap();

    fake.raise_skin(ALEX, Some(FakeSkin { image: Some((2, 2, vec![0; 15])), persona: true, ..FakeSkin::default() }));

    assert_eq!(*seen.lock().unwrap(), ["false Some(true)".to_string()]);
}

#[test]
fn a_skin_read_needs_the_area_of_its_group() {
    let fake = FakeGame::new();
    fake.offer::<PlayerSkinChanged>();
    fake.offer_area(Area::NETWORK_SKIN_DETAILS);
    let seen: Seen = Arc::new(Mutex::new(Vec::new()));
    let record = seen.clone();
    let _listener = fake
        .ctx()
        .events()
        .listen::<PlayerSkinChanged>(HookOrder::NORMAL, move |event, _| {
            let skin = event.skin().unwrap();
            record.lock().unwrap().push(format!(
                "{} {} {:?}",
                skin.image().is_some(),
                skin.geometry_json().is_some(),
                skin.is_premium()
            ));
        })
        .unwrap();

    fake.raise_skin(
        ALEX,
        Some(FakeSkin {
            image: Some(pixels(1, 1)),
            geometry_json: Some("{}".to_string()),
            premium: true,
            ..FakeSkin::default()
        }),
    );

    assert_eq!(*seen.lock().unwrap(), ["false false Some(true)".to_string()]);
}

#[test]
fn a_copy_of_the_event_reads_nothing_after_the_listener() {
    let fake = FakeGame::new();
    serving(&fake);
    let kept: Arc<Mutex<Option<(PlayerListUpdate, PlayerSkinChanged)>>> = Arc::new(Mutex::new(None));
    let list = kept.clone();
    let _list = fake
        .ctx()
        .events()
        .listen::<PlayerListUpdate>(HookOrder::NORMAL, move |event, _| {
            assert_eq!(event.entries().count(), 1);
            let skin = PlayerSkinChanged { header: FakeGame::header::<PlayerSkinChanged>() };
            *list.lock().unwrap() = Some((*event, skin));
        })
        .unwrap();

    fake.raise_player_list(&FakePlayerList {
        action: PlayerListAction::Add,
        entries: vec![entry(ALEX, 1, "Alex", None)],
    });

    let (list, skin) = kept.lock().unwrap().take().unwrap();
    assert_eq!(list.entries().count(), 0);
    assert_eq!(skin.uuid(), None);
    assert!(skin.skin().is_none());
}

#[test]
fn a_fake_list_is_cut_at_4096_entries() {
    let fake = FakeGame::new();
    serving(&fake);
    let seen: Seen = Arc::new(Mutex::new(Vec::new()));
    let record = seen.clone();
    let _listener = fake
        .ctx()
        .events()
        .listen::<PlayerListUpdate>(HookOrder::NORMAL, move |event, _| {
            record.lock().unwrap().push(format!("{} {}", event.entries().count(), event.count));
        })
        .unwrap();

    let entries = (0..5000).map(|index| entry(Uuid([index as u8; 16]), index, "p", None)).collect();
    fake.raise_player_list(&FakePlayerList { action: PlayerListAction::Add, entries });

    assert_eq!(*seen.lock().unwrap(), ["4096 4096".to_string()]);
}

#[test]
fn at_most_16_animated_images_are_given() {
    let fake = FakeGame::new();
    serving(&fake);
    let seen: Seen = Arc::new(Mutex::new(Vec::new()));
    let record = seen.clone();
    let _listener = fake
        .ctx()
        .events()
        .listen::<PlayerSkinChanged>(HookOrder::NORMAL, move |event, _| {
            let skin = event.skin().unwrap();
            record.lock().unwrap().push(skin.animated_images().count().to_string());
        })
        .unwrap();
    let skin =
        FakeSkin { animated: (0..20).map(|_| (AnimatedKind::Face, pixels(1, 1))).collect(), ..FakeSkin::default() };

    fake.raise_skin(ALEX, Some(skin));

    assert_eq!(*seen.lock().unwrap(), ["16".to_string()]);
}
