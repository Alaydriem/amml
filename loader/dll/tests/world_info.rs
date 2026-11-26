use std::ffi::c_void;

use amml::{SessionAdapter, WorldInfoAdapter};
use amml_abi::{AmmlApi, AmmlClientTable, AmmlConnectionKind, AmmlServices, AmmlStatus, AmmlWorldInfo, AmmlWorldText};

use crate::support::{ClientGame, GAME_CLIENT_A, GAME_CLIENT_B, Harness, InfoGame, LogFile};

const KIND_LOCAL: i16 = 0;
const KIND_NETWORK: i16 = 1;

// Installs the session and world info adapters over the stand-in client entries and the world info entries, for a
// connection info of `size` bytes, after one frame and one update of client A (the primary client, no world joined).
fn install(size: usize) -> Harness {
    let mut entries = ClientGame::entries(&[]);
    entries.extend(InfoGame::entries(size));
    let harness = Harness::start_with_entries(&entries);
    SessionAdapter::install(harness.state);
    WorldInfoAdapter::install(harness.state);
    ClientGame::frame();
    ClientGame::update(&GAME_CLIENT_A);
    harness
}

// `install`, then a joined world (epoch 1).
fn start_in_world(size: usize) -> Harness {
    let harness = install(size);
    ClientGame::set_player(0x5000);
    ClientGame::update(&GAME_CLIENT_A);
    assert_eq!(harness.state.events().gate().epoch().0, 1);
    harness
}

// The client table, through the services table as a mod reaches it.
fn table(harness: &Harness) -> AmmlClientTable {
    // SAFETY: the loader's tables live for the process.
    let services = unsafe { AmmlApi::services(harness.state.api()) }.unwrap();
    // SAFETY: as above.
    let table = unsafe { AmmlServices::client(services) }.unwrap();
    // SAFETY: as above.
    unsafe { AmmlClientTable::read(table) }.unwrap()
}

fn handle() -> *const c_void {
    ClientGame::client(&GAME_CLIENT_A).cast_const()
}

fn world_info(table: &AmmlClientTable) -> (AmmlStatus, AmmlWorldInfo) {
    let mut out = AmmlWorldInfo { flags: 0xDEAD, ..AmmlWorldInfo::default() };
    // SAFETY: the table is live and `out` is writable.
    let status = unsafe { (table.world_info.unwrap())(handle(), &mut out) };
    (status, out)
}

// The text `field` read into a buffer of `cap` bytes: the status, the length the call gave and the bytes copied.
fn world_text(table: &AmmlClientTable, field: AmmlWorldText, cap: usize) -> (AmmlStatus, usize, Vec<u8>) {
    let mut buffer = vec![0u8; cap];
    let mut len = usize::MAX;
    // SAFETY: the table is live, `buffer` is writable for `cap` bytes and `len` is writable.
    let status = unsafe { (table.world_text.unwrap())(handle(), field, buffer.as_mut_ptr(), cap, &mut len) };
    let copied = len.min(cap);
    buffer.truncate(if status == AmmlStatus::OK { copied } else { 0 });
    (status, len, buffer)
}

#[test]
fn world_info_outside_a_world_is_none() {
    let harness = install(InfoGame::SIZE_2663);
    let table = table(&harness);

    let (status, info) = world_info(&table);
    let (text_status, _, _) = world_text(&table, AmmlWorldText::LEVEL_NAME, 16);

    assert_eq!(status, AmmlStatus::REFUSED);
    assert_eq!(info.flags, 0xDEAD);
    assert_eq!(text_status, AmmlStatus::REFUSED);
}

#[test]
fn world_info_of_a_client_that_is_not_the_primary_one_is_refused() {
    let harness = start_in_world(InfoGame::SIZE_2663);
    let table = table(&harness);
    let mut out = AmmlWorldInfo::default();

    // SAFETY: the table is live and `out` is writable.
    let status = unsafe { (table.world_info.unwrap())(ClientGame::client(&GAME_CLIENT_B).cast_const(), &mut out) };

    assert_eq!(status, AmmlStatus::REFUSED);
}

#[test]
fn the_facts_of_the_world_are_read_from_the_game() {
    let harness = start_in_world(InfoGame::SIZE_2663);
    InfoGame::set_seed(0xFEED_0000_0000_1234);
    InfoGame::set_realm(true);
    InfoGame::set_commands(true);
    InfoGame::set_level_name(b"My World");
    InfoGame::set_connection(Some((KIND_LOCAL, b"", 0)));
    InfoGame::construct_storage("C:/worlds/My World");
    let table = table(&harness);

    let (status, info) = world_info(&table);

    assert_eq!(status, AmmlStatus::OK);
    let all = AmmlWorldInfo::HAS_SEED
        | AmmlWorldInfo::HAS_STORAGE
        | AmmlWorldInfo::HAS_CONNECTION
        | AmmlWorldInfo::HAS_REALM
        | AmmlWorldInfo::REALM
        | AmmlWorldInfo::HAS_COMMANDS
        | AmmlWorldInfo::COMMANDS
        | AmmlWorldInfo::HAS_LEVEL_NAME;
    assert_eq!(info.flags, all);
    assert_eq!(info.seed, 0xFEED_0000_0000_1234);
    assert_eq!(info.connection_kind, AmmlConnectionKind::LOCAL);
    assert_eq!(world_text(&table, AmmlWorldText::LEVEL_NAME, 64).2, b"My World");
    assert_eq!(world_text(&table, AmmlWorldText::STORAGE_FOLDER, 64).2, b"C:/worlds/My World");
    assert_eq!(world_text(&table, AmmlWorldText::HOST, 64), (AmmlStatus::OK, 0, Vec::new()));
}

#[test]
fn a_world_with_no_level_still_gives_the_realm_and_the_connection() {
    let harness = start_in_world(InfoGame::SIZE_2663);
    InfoGame::set_level_present(false);
    InfoGame::set_realm(false);
    InfoGame::set_connection(Some((KIND_NETWORK, b"play.example.net", 19132)));
    let table = table(&harness);

    let (status, info) = world_info(&table);

    assert_eq!(status, AmmlStatus::OK);
    assert_eq!(info.flags, AmmlWorldInfo::HAS_REALM | AmmlWorldInfo::HAS_CONNECTION);
    assert_eq!((info.connection_kind, info.port), (AmmlConnectionKind::REMOTE, 19132));
}

#[test]
fn storage_cleared_at_shutdown_done_gives_no_folder() {
    let harness = start_in_world(InfoGame::SIZE_2663);
    InfoGame::construct_storage("C:/worlds/My World");
    let table = table(&harness);
    assert_eq!(world_text(&table, AmmlWorldText::STORAGE_FOLDER, 64).2, b"C:/worlds/My World");

    InfoGame::shutdown_storage();
    InfoGame::destroy_storage();

    let (status, info) = world_info(&table);
    assert_eq!(status, AmmlStatus::OK);
    assert_eq!(info.flags & AmmlWorldInfo::HAS_STORAGE, 0);
    assert_eq!(world_text(&table, AmmlWorldText::STORAGE_FOLDER, 64).0, AmmlStatus::NOT_AVAILABLE);
    // Both stand-ins ran through the loader's detours and reached the game's own functions.
    assert_eq!(InfoGame::storage_calls(), 2);
}

#[test]
fn a_storage_made_after_another_one_shut_down_is_the_live_one() {
    let harness = start_in_world(InfoGame::SIZE_2663);
    InfoGame::construct_storage("C:/worlds/First");
    InfoGame::shutdown_storage();
    InfoGame::construct_storage("C:/worlds/Second");
    let table = table(&harness);

    assert_eq!(world_text(&table, AmmlWorldText::STORAGE_FOLDER, 64).2, b"C:/worlds/Second");
}

#[test]
fn long_host_string_reads_whole() {
    let harness = start_in_world(InfoGame::SIZE_2663);
    let host = "a-server-name-that-is-longer-than-fifteen-bytes.example.net";
    InfoGame::set_connection(Some((KIND_NETWORK, host.as_bytes(), 25565)));
    let table = table(&harness);

    let (status, info) = world_info(&table);
    let (_, len, short) = world_text(&table, AmmlWorldText::HOST, 8);
    let (whole_status, whole_len, whole) = world_text(&table, AmmlWorldText::HOST, 128);

    assert_eq!(status, AmmlStatus::OK);
    assert_ne!(info.flags & AmmlWorldInfo::HAS_CONNECTION, 0);
    assert_eq!((info.connection_kind, info.port), (AmmlConnectionKind::REMOTE, 25565));
    assert_eq!((len, short.as_slice()), (host.len(), &host.as_bytes()[..8]));
    assert_eq!((whole_status, whole_len), (AmmlStatus::OK, host.len()));
    assert_eq!(whole, host.as_bytes());
    // Each read ran the game's destructor on the value the getter built, once.
    let (destructors, getters) = InfoGame::calls();
    assert_eq!(destructors, getters);
}

#[test]
fn non_utf8_level_name_reads_none() {
    let harness = start_in_world(InfoGame::SIZE_2663);
    InfoGame::set_seed(7);
    InfoGame::set_level_name(&[b'W', 0xFF, 0xFE, b'x']);
    let table = table(&harness);

    let (status, info) = world_info(&table);

    assert_eq!(status, AmmlStatus::OK);
    assert_eq!(info.flags & AmmlWorldInfo::HAS_LEVEL_NAME, 0);
    assert_eq!((info.flags & AmmlWorldInfo::HAS_SEED, info.seed), (AmmlWorldInfo::HAS_SEED, 7));
    assert_eq!(world_text(&table, AmmlWorldText::LEVEL_NAME, 16).0, AmmlStatus::NOT_AVAILABLE);
}

#[test]
fn non_utf8_host_leaves_the_connection_out_and_keeps_the_other_facts() {
    let harness = start_in_world(InfoGame::SIZE_2663);
    InfoGame::set_seed(9);
    InfoGame::set_connection(Some((KIND_NETWORK, &[0xC3, 0x28, b'h'], 19132)));
    let table = table(&harness);

    let (status, info) = world_info(&table);

    assert_eq!(status, AmmlStatus::OK);
    assert_eq!(info.flags & AmmlWorldInfo::HAS_CONNECTION, 0);
    assert_ne!(info.flags & AmmlWorldInfo::HAS_SEED, 0);
    assert_eq!(world_text(&table, AmmlWorldText::HOST, 16).0, AmmlStatus::NOT_AVAILABLE);
    // The value the game built is destroyed also when its host is left out.
    assert_eq!(InfoGame::calls().0, InfoGame::calls().1);
}

#[test]
fn the_connection_of_a_308h_value_is_read_with_its_own_layout() {
    let harness = start_in_world(InfoGame::SIZE_2670);
    InfoGame::set_connection(Some((KIND_NETWORK, b"mc.example.org", 19133)));
    let table = table(&harness);

    let (status, info) = world_info(&table);

    assert_eq!(status, AmmlStatus::OK);
    assert_eq!((info.connection_kind, info.port), (AmmlConnectionKind::REMOTE, 19133));
    assert_eq!(world_text(&table, AmmlWorldText::HOST, 64).2, b"mc.example.org");
}

#[test]
fn a_size_with_no_known_layout_leaves_the_connection_out_and_calls_nothing() {
    let harness = start_in_world(0x300);
    InfoGame::set_seed(5);
    InfoGame::set_connection(Some((KIND_NETWORK, b"mc.example.org", 19133)));
    let table = table(&harness);

    let (status, info) = world_info(&table);

    assert_eq!(status, AmmlStatus::OK);
    assert_eq!(info.flags & AmmlWorldInfo::HAS_CONNECTION, 0);
    assert_ne!(info.flags & AmmlWorldInfo::HAS_SEED, 0);
    assert_eq!(InfoGame::calls(), (0, 0));
    let log = LogFile::wait_for(&harness.logs().join("loader.log"), "the connection of the world info is left out");
    assert!(log.contains("the connection of the world info is left out"));
}

#[test]
fn a_connection_the_game_does_not_hold_is_left_out() {
    let harness = start_in_world(InfoGame::SIZE_2663);
    InfoGame::set_connection(None);
    let table = table(&harness);

    let (status, info) = world_info(&table);

    assert_eq!(status, AmmlStatus::OK);
    assert_eq!(info.flags & AmmlWorldInfo::HAS_CONNECTION, 0);
    // No value was built, so no destructor ran.
    assert_eq!(InfoGame::calls().0, 0);
}

#[test]
fn an_unknown_text_field_is_invalid() {
    let harness = start_in_world(InfoGame::SIZE_2663);
    let table = table(&harness);

    assert_eq!(world_text(&table, AmmlWorldText(99), 8).0, AmmlStatus::INVALID_ARGUMENT);
}
