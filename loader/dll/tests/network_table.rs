use std::ffi::c_void;
use std::sync::{Arc, Mutex};

use amml::{ModListener, NetworkAdapter};
use amml_abi::{
    AmmlApi, AmmlBand, AmmlEventContext, AmmlEventHeader, AmmlImageView, AmmlNetworkTable, AmmlPlayerListEntry,
    AmmlServices, AmmlStatus, AmmlStr, AmmlThread, NETWORK_TEXT_NAME, NETWORK_TEXT_XUID, PlayerListUpdate,
    PlayerSkinChanged,
};

use crate::support::{Harness, HeapProbe, NetworkGame, SkinSpec};

const UUID_A: [u8; 16] = [0xa1; 16];
const UUID_B: [u8; 16] = [0xb2; 16];
const LIST: &str = "network.player_list";
const SKIN: &str = "network.player_skin";

type Visit = Box<dyn FnMut(&AmmlNetworkTable, &AmmlEventContext, *mut AmmlEventHeader)>;

fn start() -> Harness {
    start_with(&NetworkGame::entries())
}

fn start_with(entries: &[(&str, amml_data::Resolved)]) -> Harness {
    let harness = Harness::start_with_entries(entries);
    NetworkAdapter::install(harness.state);
    harness
}

// The network table, through the services table as a mod reaches it.
fn table(harness: &Harness) -> AmmlNetworkTable {
    // SAFETY: the loader's tables live for the process.
    let services = unsafe { AmmlApi::services(harness.state.api()) }.unwrap();
    // SAFETY: as above.
    let table = unsafe { AmmlServices::network(services) }.unwrap();
    // SAFETY: as above.
    unsafe { AmmlNetworkTable::read(table) }.unwrap()
}

unsafe extern "C" fn trampoline(
    event: *mut AmmlEventHeader,
    context: *const AmmlEventContext,
    user: *mut c_void,
) -> AmmlStatus {
    // SAFETY: `user` is the leaked `Visit` of `listen`; the loader passes a live context.
    let (visit, context) = unsafe { (&mut *user.cast::<Visit>(), &*context) };
    let services = amml::ServicesTable::build();
    // SAFETY: the loader's tables live for the process.
    let table = unsafe { AmmlNetworkTable::read(AmmlServices::network(services).unwrap()) }.unwrap();
    visit(&table, context, event);
    AmmlStatus::OK
}

// Calls `visit` for every raise of `event`, with the network table and the context.
fn listen(
    harness: &Harness,
    event: &str,
    visit: impl FnMut(&AmmlNetworkTable, &AmmlEventContext, *mut AmmlEventHeader) + 'static,
) {
    let visit: &'static mut Visit = Box::leak(Box::new(Box::new(visit)));
    let user = std::ptr::from_mut(visit).expose_provenance();
    let listener = ModListener { owner: 0, callback: trampoline, user };
    harness.state.events().listen(event, 1, AmmlBand::NORMAL, listener).unwrap();
}

fn entry(table: &AmmlNetworkTable, index: u32) -> Result<AmmlPlayerListEntry, AmmlStatus> {
    let mut out = AmmlPlayerListEntry { uuid: [0; 16], has_actor: 9, _reserved: [0; 7], actor_id: 0, skin: 0 };
    // SAFETY: the table is live; `out` is writable.
    let status = unsafe { (table.player_list_entry)(index, &mut out) };
    if status == AmmlStatus::OK { Ok(out) } else { Err(status) }
}

fn text(table: &AmmlNetworkTable, index: u32, field: u8) -> Result<String, AmmlStatus> {
    let mut out = AmmlStr::EMPTY;
    // SAFETY: the table is live; `out` is writable.
    let status = unsafe { (table.player_list_text)(index, field, &mut out) };
    if status != AmmlStatus::OK {
        return Err(status);
    }
    // SAFETY: the loader's view is valid until the event ends, and the test reads it inside the event.
    Ok(unsafe { out.as_str() }.unwrap().to_string())
}

fn string_of(read: unsafe extern "C" fn(u64, *mut AmmlStr) -> AmmlStatus, skin: u64) -> Result<String, AmmlStatus> {
    let mut out = AmmlStr::EMPTY;
    // SAFETY: the table function is live; `out` is writable.
    let status = unsafe { read(skin, &mut out) };
    if status != AmmlStatus::OK {
        return Err(status);
    }
    // SAFETY: the loader's view is valid until the event ends, and the test reads it inside the event.
    Ok(unsafe { out.as_str() }.unwrap().to_string())
}

fn image(table: &AmmlNetworkTable, skin: u64) -> Result<(u32, u32, Vec<u8>), AmmlStatus> {
    let mut out = AmmlImageView::EMPTY;
    // SAFETY: the table is live; `out` is writable.
    let status = unsafe { (table.skin_image)(skin, &mut out) };
    if status != AmmlStatus::OK {
        return Err(status);
    }
    // SAFETY: the view points to `len` bytes of the stand-in skin, which the test leaked.
    let pixels = unsafe { std::slice::from_raw_parts(out.pixels, out.len) }.to_vec();
    Ok((out.width, out.height, pixels))
}

fn flag(read: unsafe extern "C" fn(u64, *mut u8) -> AmmlStatus, skin: u64) -> Result<bool, AmmlStatus> {
    let mut out = 9u8;
    // SAFETY: the table function is live; `out` is writable.
    let status = unsafe { read(skin, &mut out) };
    if status == AmmlStatus::OK { Ok(out != 0) } else { Err(status) }
}

fn player_skin(table: &AmmlNetworkTable) -> Result<([u8; 16], u64), AmmlStatus> {
    let (mut uuid, mut skin) = ([0u8; 16], 0u64);
    // SAFETY: the table is live; both outputs are writable.
    let status = unsafe { (table.player_skin)(&mut uuid, &mut skin) };
    if status == AmmlStatus::OK { Ok((uuid, skin)) } else { Err(status) }
}

fn shared<T: Default>() -> Arc<Mutex<T>> {
    Arc::new(Mutex::new(T::default()))
}

#[test]
fn the_events_and_the_areas_are_offered_when_every_entry_resolves() {
    let harness = start();

    for event in [LIST, SKIN] {
        assert!(harness.state.events().is_available(event, 1), "{event}");
    }
    for area in ["network.skin", "network.skin_animated", "network.skin_geometry", "network.skin_details"] {
        assert!(harness.state.areas().is_available(area, 1), "{area}");
    }
}

#[test]
fn a_build_without_the_skin_entries_offers_the_events_and_no_skin_area() {
    let entries: Vec<_> =
        NetworkGame::entries().into_iter().filter(|(name, _)| !name.starts_with("SerializedSkinImpl.")).collect();
    let harness = start_with(&entries);

    assert!(harness.state.events().is_available(LIST, 1));
    assert!(harness.state.events().is_available(SKIN, 1));
    assert!(!harness.state.areas().is_available("network.skin", 1));
    assert!(!harness.state.areas().is_available("network.skin_details", 1));
}

#[test]
fn skin_read_outside_the_event_is_refused() {
    let harness = start();
    let handles = shared::<Vec<u64>>();
    let inside = handles.clone();
    listen(&harness, SKIN, move |table, _, _| {
        let (_, skin) = player_skin(table).unwrap();
        assert_ne!(skin, 0);
        // A handle of an earlier event is no handle of this one.
        for old in inside.lock().unwrap().iter() {
            assert_eq!(image(table, *old), Err(AmmlStatus::REFUSED));
        }
        assert!(image(table, skin).is_ok());
        inside.lock().unwrap().push(skin);
    });
    let skin = Some(NetworkGame::skin_ref(&SkinSpec::default()));
    NetworkGame::handle(NetworkGame::SKIN, NetworkGame::skin_packet(UUID_A, skin));
    NetworkGame::handle(NetworkGame::SKIN, NetworkGame::skin_packet(UUID_B, skin));

    let table = table(&harness);
    let handles = handles.lock().unwrap().clone();
    assert_eq!(handles.len(), 2);
    for skin in handles {
        assert_eq!(image(&table, skin), Err(AmmlStatus::REFUSED));
        assert_eq!(flag(table.skin_is_persona, skin), Err(AmmlStatus::REFUSED));
        assert_eq!(string_of(table.skin_geometry_json, skin), Err(AmmlStatus::REFUSED));
    }
    assert_eq!(player_skin(&table), Err(AmmlStatus::REFUSED));
    assert_eq!(entry(&table, 0), Err(AmmlStatus::REFUSED));
    assert_eq!(text(&table, 0, NETWORK_TEXT_NAME), Err(AmmlStatus::REFUSED));
}

#[test]
fn the_list_event_gives_entries_names_and_skins() {
    let harness = start();
    let seen = shared::<Vec<String>>();
    let inside = seen.clone();
    listen(&harness, LIST, move |table, _, event| {
        // SAFETY: the loader raises `network.player_list` with a `PlayerListUpdate`.
        let event = unsafe { &*event.cast::<PlayerListUpdate>() };
        let mut lines = inside.lock().unwrap();
        lines.push(format!("action {} count {}", event.action, event.count));
        for index in 0..event.count {
            let entry = entry(table, index).unwrap();
            lines.push(format!(
                "{:02x} actor {} {} {} {}",
                entry.uuid[0],
                entry.has_actor,
                entry.actor_id,
                text(table, index, NETWORK_TEXT_NAME).unwrap(),
                text(table, index, NETWORK_TEXT_XUID).unwrap(),
            ));
            let (width, height, pixels) = image(table, entry.skin).unwrap();
            lines.push(format!("image {width}x{height} first {} len {}", pixels[0], pixels.len()));
        }
    });
    let long_name = b"a name that does not fit in the small string buffer";
    let first = NetworkGame::add_entry(
        UUID_A,
        7,
        b"Alex",
        b"2535400000000001",
        Some(NetworkGame::skin_ref(&SkinSpec::default())),
    );
    let second = NetworkGame::add_entry(
        UUID_B,
        -9,
        long_name,
        b"2535400000000002",
        Some(NetworkGame::skin_ref(&SkinSpec { size: (4, 2), ..SkinSpec::default() })),
    );
    NetworkGame::handle(NetworkGame::LIST, NetworkGame::list_packet(vec![first, second]));

    assert_eq!(
        *seen.lock().unwrap(),
        [
            "action 1 count 2".to_string(),
            "a1 actor 1 7 Alex 2535400000000001".to_string(),
            "image 2x2 first 1 len 16".to_string(),
            format!("b2 actor 1 -9 {} 2535400000000002", String::from_utf8_lossy(long_name)),
            "image 4x2 first 1 len 32".to_string(),
        ]
    );
    // The original handler ran before the event.
    assert_eq!(NetworkGame::handled(), ["list"]);
}

#[test]
fn remove_entries_give_only_the_uuid() {
    let harness = start();
    let seen = shared::<Vec<String>>();
    let inside = seen.clone();
    listen(&harness, LIST, move |table, _, event| {
        // SAFETY: the loader raises `network.player_list` with a `PlayerListUpdate`.
        let event = unsafe { &*event.cast::<PlayerListUpdate>() };
        let entry = entry(table, 0).unwrap();
        let mut lines = inside.lock().unwrap();
        lines.push(format!(
            "action {} count {} actor {} skin {}",
            event.action, event.count, entry.has_actor, entry.skin
        ));
        lines.push(format!("uuid {:02x}", entry.uuid[0]));
        lines.push(format!("name {:?}", text(table, 0, NETWORK_TEXT_NAME)));
    });
    NetworkGame::handle(NetworkGame::LIST, NetworkGame::list_packet(vec![NetworkGame::remove_entry(UUID_B)]));

    assert_eq!(
        *seen.lock().unwrap(),
        [
            "action 2 count 1 actor 0 skin 0".to_string(),
            "uuid b2".to_string(),
            format!("name {:?}", Err::<String, _>(AmmlStatus::NOT_AVAILABLE)),
        ]
    );
}

#[test]
fn more_than_4096_entries_are_cut() {
    let harness = start();
    let seen = shared::<Vec<String>>();
    let inside = seen.clone();
    listen(&harness, LIST, move |table, _, event| {
        // SAFETY: the loader raises `network.player_list` with a `PlayerListUpdate`.
        let event = unsafe { &*event.cast::<PlayerListUpdate>() };
        let mut lines = inside.lock().unwrap();
        lines.push(format!("count {}", event.count));
        lines.push(format!("last {}", entry(table, 4095).is_ok()));
        lines.push(format!("past {:?}", entry(table, 4096).map(|_| ())));
    });
    let entries = (0..5000).map(|index| NetworkGame::add_entry([index as u8; 16], index, b"p", b"1", None)).collect();
    NetworkGame::handle(NetworkGame::LIST, NetworkGame::list_packet(entries));

    assert_eq!(
        *seen.lock().unwrap(),
        [
            "count 4096".to_string(),
            "last true".to_string(),
            format!("past {:?}", Err::<(), _>(AmmlStatus::INVALID_ARGUMENT)),
        ]
    );
}

#[test]
fn blob_size_mismatch_gives_no_image() {
    let harness = start();
    let seen = shared::<Vec<String>>();
    let inside = seen.clone();
    listen(&harness, SKIN, move |table, _, _| {
        let (_, skin) = player_skin(table).unwrap();
        let mut lines = inside.lock().unwrap();
        lines.push(format!("image {:?}", image(table, skin).map(|(width, _, _)| width)));
        // The other reads of the same skin still work.
        lines.push(format!("persona {:?}", flag(table.skin_is_persona, skin)));
        lines.push(format!("patch {:?}", string_of(table.skin_resource_patch, skin).map(|text| text.len())));
    });
    let short = SkinSpec { size: (2, 2), blob_len: Some(15), persona: true, ..SkinSpec::default() };
    NetworkGame::handle(NetworkGame::SKIN, NetworkGame::skin_packet(UUID_A, Some(NetworkGame::skin_ref(&short))));

    assert_eq!(
        *seen.lock().unwrap(),
        [
            format!("image {:?}", Err::<u32, _>(AmmlStatus::NOT_AVAILABLE)),
            format!("persona {:?}", Ok::<bool, AmmlStatus>(true)),
            format!("patch {:?}", Ok::<usize, AmmlStatus>(SkinSpec::default().resource_patch.len())),
        ]
    );
}

#[test]
fn non_utf8_text_reads_none() {
    let harness = start();
    let seen = shared::<Vec<String>>();
    let inside = seen.clone();
    listen(&harness, LIST, move |table, _, event| {
        // SAFETY: the loader raises `network.player_list` with a `PlayerListUpdate`.
        let event = unsafe { &*event.cast::<PlayerListUpdate>() };
        let mut lines = inside.lock().unwrap();
        for index in 0..event.count {
            let skin = entry(table, index).unwrap().skin;
            lines.push(format!("name {:?}", text(table, index, NETWORK_TEXT_NAME)));
            lines.push(format!("xuid {:?}", text(table, index, NETWORK_TEXT_XUID)));
            lines.push(format!("geometry name {:?}", string_of(table.skin_geometry_name, skin)));
            lines.push(format!("patch {:?}", string_of(table.skin_resource_patch, skin).map(|text| text.len())));
            lines.push(format!("persona {:?}", flag(table.skin_is_persona, skin)));
        }
    });
    let bad = SkinSpec { geometry_name: vec![0xff, 0xfe, b'x'], ..SkinSpec::default() };
    // A long text on the heap that is not UTF-8, and a short inline one.
    let long_bad = [vec![b'a'; 40], vec![0xc3, 0x28]].concat();
    let first = NetworkGame::add_entry(UUID_A, 1, &long_bad, b"12", Some(NetworkGame::skin_ref(&bad)));
    NetworkGame::handle(NetworkGame::LIST, NetworkGame::list_packet(vec![first]));

    let none = Err::<String, _>(AmmlStatus::NOT_AVAILABLE);
    assert_eq!(
        *seen.lock().unwrap(),
        [
            format!("name {none:?}"),
            format!("xuid {:?}", Ok::<String, AmmlStatus>("12".to_string())),
            format!("geometry name {none:?}"),
            format!("patch {:?}", Ok::<usize, AmmlStatus>(SkinSpec::default().resource_patch.len())),
            format!("persona {:?}", Ok::<bool, AmmlStatus>(false)),
        ]
    );
}

#[test]
fn geometry_json_is_built_once_for_each_skin_and_freed_when_the_event_ends() {
    let harness = start();
    NetworkGame::set_geometry(b"{\n   \"minecraft:geometry\" : []\n}\n");
    let seen = shared::<Vec<String>>();
    let inside = seen.clone();
    listen(&harness, LIST, move |table, _, event| {
        // SAFETY: the loader raises `network.player_list` with a `PlayerListUpdate`.
        let event = unsafe { &*event.cast::<PlayerListUpdate>() };
        let mut lines = inside.lock().unwrap();
        for index in 0..event.count {
            let skin = entry(table, index).unwrap().skin;
            let first = string_of(table.skin_geometry_json, skin).unwrap();
            let again = string_of(table.skin_geometry_json, skin).unwrap();
            lines.push(format!("{} {}", first == again, first.len()));
            lines.push(format!("name {}", string_of(table.skin_geometry_name, skin).unwrap()));
        }
        lines.push(format!("built {}", NetworkGame::styled_calls()));
    });
    let first = NetworkGame::add_entry(UUID_A, 1, b"a", b"1", Some(NetworkGame::skin_ref(&SkinSpec::default())));
    let second = NetworkGame::add_entry(UUID_B, 2, b"b", b"2", Some(NetworkGame::skin_ref(&SkinSpec::default())));
    let before = HeapProbe::counts();
    NetworkGame::handle(NetworkGame::LIST, NetworkGame::list_packet(vec![first, second]));
    let after = HeapProbe::counts();

    let name = "name geometry.humanoid.custom".to_string();
    assert_eq!(
        *seen.lock().unwrap(),
        ["true 33".to_string(), name.clone(), "true 33".to_string(), name, "built 2".to_string()]
    );
    // The two strings the game built are freed on the game heap once each, after the listener.
    assert_eq!((after.0 - before.0, after.1 - before.1), (2, 2));
}

#[test]
fn geometry_over_1_mb_gives_none() {
    let harness = start();
    NetworkGame::set_geometry(&vec![b'a'; 1024 * 1024 + 1]);
    let seen = shared::<Vec<String>>();
    let inside = seen.clone();
    listen(&harness, SKIN, move |table, _, _| {
        let (_, skin) = player_skin(table).unwrap();
        let mut lines = inside.lock().unwrap();
        lines.push(format!("json {:?}", string_of(table.skin_geometry_json, skin).map(|text| text.len())));
        lines.push(format!("again {:?}", string_of(table.skin_geometry_json, skin).map(|text| text.len())));
        lines.push(format!("image {:?}", image(table, skin).map(|(width, _, _)| width)));
        lines.push(format!("built {}", NetworkGame::styled_calls()));
    });
    let before = HeapProbe::counts();
    NetworkGame::handle(
        NetworkGame::SKIN,
        NetworkGame::skin_packet(UUID_A, Some(NetworkGame::skin_ref(&SkinSpec::default()))),
    );
    let after = HeapProbe::counts();

    let none = Err::<usize, _>(AmmlStatus::NOT_AVAILABLE);
    assert_eq!(
        *seen.lock().unwrap(),
        [
            format!("json {none:?}"),
            format!("again {none:?}"),
            format!("image {:?}", Ok::<u32, AmmlStatus>(2)),
            "built 1".to_string(),
        ]
    );
    // The text above 1 MB is freed at once, exactly once.
    assert_eq!((after.0 - before.0, after.1 - before.1), (1, 1));
}

#[test]
fn animated_images_and_flags_are_read_from_the_skin() {
    let harness = start();
    let seen = shared::<Vec<String>>();
    let inside = seen.clone();
    listen(&harness, SKIN, move |table, _, event| {
        // SAFETY: the loader raises `network.player_skin` with a `PlayerSkinChanged`.
        let _ = unsafe { &*event.cast::<PlayerSkinChanged>() };
        let (uuid, skin) = player_skin(table).unwrap();
        let mut lines = inside.lock().unwrap();
        lines.push(format!("uuid {:02x}", uuid[0]));
        let mut count = 0u32;
        // SAFETY: the table is live; `count` is writable.
        assert_eq!(unsafe { (table.skin_animated_count)(skin, &mut count) }, AmmlStatus::OK);
        lines.push(format!("animated {count}"));
        for index in 0..count + 1 {
            let (mut kind, mut view) = (0u32, AmmlImageView::EMPTY);
            // SAFETY: the table is live; both outputs are writable.
            let status = unsafe { (table.skin_animated)(skin, index, &mut kind, &mut view) };
            lines.push(format!("{index} {} kind {kind} {}x{}", status.0, view.width, view.height));
        }
        lines.push(format!(
            "persona {:?} premium {:?}",
            flag(table.skin_is_persona, skin),
            flag(table.skin_is_premium, skin)
        ));
    });
    let spec = SkinSpec { animated: vec![(1, 8, 8), (2, 32, 32)], persona: true, premium: true, ..SkinSpec::default() };
    NetworkGame::handle(NetworkGame::SKIN, NetworkGame::skin_packet(UUID_B, Some(NetworkGame::skin_ref(&spec))));

    assert_eq!(
        *seen.lock().unwrap(),
        [
            "uuid b2".to_string(),
            "animated 2".to_string(),
            "0 0 kind 1 8x8".to_string(),
            "1 0 kind 2 32x32".to_string(),
            format!("2 {} kind 0 0x0", AmmlStatus::INVALID_ARGUMENT.0),
            format!("persona {:?} premium {:?}", Ok::<bool, AmmlStatus>(true), Ok::<bool, AmmlStatus>(true)),
        ]
    );
}

#[test]
fn a_skin_event_without_a_skin_gives_the_handle_zero() {
    let harness = start();
    let seen = shared::<Vec<u64>>();
    let inside = seen.clone();
    listen(&harness, SKIN, move |table, _, _| {
        let (_, skin) = player_skin(table).unwrap();
        inside.lock().unwrap().push(skin);
        assert_eq!(image(table, skin), Err(AmmlStatus::REFUSED));
    });
    NetworkGame::handle(NetworkGame::SKIN, NetworkGame::skin_packet(UUID_A, None));

    assert_eq!(*seen.lock().unwrap(), [0]);
}

#[test]
fn the_original_handler_always_runs_and_an_idle_detour_raises_nothing() {
    let harness = start();
    NetworkGame::handle(NetworkGame::LIST, NetworkGame::list_packet(vec![NetworkGame::remove_entry(UUID_A)]));
    NetworkGame::handle(NetworkGame::SKIN, NetworkGame::skin_packet(UUID_A, None));

    assert_eq!(NetworkGame::handled(), ["list", "skin"]);
    assert_eq!(NetworkGame::styled_calls(), 0);
    drop(harness);
}

#[test]
fn the_event_runs_on_the_thread_of_the_handler() {
    let harness = start();
    let seen = shared::<Vec<(AmmlThread, bool)>>();
    let inside = seen.clone();
    listen(&harness, SKIN, move |_, context, _| {
        inside.lock().unwrap().push((context.thread, context.client.is_null()));
    });
    let packet = NetworkGame::skin_packet(UUID_A, None);
    std::thread::spawn(move || NetworkGame::handle(NetworkGame::SKIN, packet)).join().unwrap();
    harness.state.events().gate().note_game_thread();
    NetworkGame::handle(NetworkGame::SKIN, packet);

    let seen = seen.lock().unwrap().clone();
    assert_eq!(seen[0], (AmmlThread::NETWORK, true));
    assert_eq!(seen[1].0, AmmlThread::GAME);
}
