use core::ffi::c_void;

use amml_abi::{AmmlClientFlag, AmmlClientTable, AmmlScreen, AmmlStatus, AmmlStr};

unsafe extern "C" fn current(_: *mut *const c_void) -> AmmlStatus {
    AmmlStatus::OK
}
unsafe extern "C" fn local_player(_: *const c_void, _: *mut *const c_void) -> AmmlStatus {
    AmmlStatus::OK
}
unsafe extern "C" fn flag(_: *const c_void, _: AmmlClientFlag, _: *mut u8) -> AmmlStatus {
    AmmlStatus::OK
}
unsafe extern "C" fn top_screen(_: *const c_void, _: *mut AmmlScreen) -> AmmlStatus {
    AmmlStatus::OK
}
unsafe extern "C" fn set_flag(_: AmmlStr, _: *const c_void, _: AmmlClientFlag, _: u8) -> AmmlStatus {
    AmmlStatus::OK
}
unsafe extern "C" fn open_pause(_: *const c_void) -> AmmlStatus {
    AmmlStatus::OK
}
unsafe extern "C" fn in_gameplay(_: *const c_void, _: *mut u8) -> AmmlStatus {
    AmmlStatus::OK
}
unsafe extern "C" fn replay(_: AmmlStr, _: *const c_void) -> AmmlStatus {
    AmmlStatus::OK
}
unsafe extern "C" fn join(_: AmmlStr, _: *const c_void, _: AmmlStr, _: u16, _: AmmlStr, _: *mut u64) -> AmmlStatus {
    AmmlStatus::OK
}

fn table() -> AmmlClientTable {
    AmmlClientTable {
        size: AmmlClientTable::SIZE_V8,
        version: 8,
        current,
        local_player,
        flag,
        top_screen,
        set_flag,
        open_pause_screen: open_pause,
        leave_world: None,
        can_join: None,
        start_local_world: None,
        join_server: None,
        open_menu: None,
        send_command: None,
        select_hotbar_slot: None,
        note_activity: None,
        world_info: None,
        world_text: None,
        replay_app_focus_lost: Some(replay),
        in_gameplay: Some(in_gameplay),
        join_server_direct: Some(join),
        title_screen: Some(in_gameplay),
        go_to_title: Some(open_pause),
    }
}

#[test]
fn client_table_v4_reads_none_for_replay() {
    let mut table = table();
    table.size = AmmlClientTable::SIZE_V4;
    // SAFETY: the table lives for the call; the read covers only its first `size` bytes (the v4 size).
    let read = unsafe { AmmlClientTable::read(&raw const table) }.unwrap();
    assert!(read.replay_app_focus_lost.is_none());
}

#[test]
fn client_table_v5_reads_none_for_in_gameplay() {
    let mut table = table();
    table.size = AmmlClientTable::SIZE_V5;
    // SAFETY: the table lives for the call; the read covers only its first `size` bytes (the v5 size).
    let read = unsafe { AmmlClientTable::read(&raw const table) }.unwrap();
    assert!(read.replay_app_focus_lost.is_some());
    assert!(read.in_gameplay.is_none());
}

#[test]
fn client_table_v6_reads_its_in_gameplay_function() {
    let mut table = table();
    table.size = AmmlClientTable::SIZE_V6;
    // SAFETY: the table lives for the call.
    let read = unsafe { AmmlClientTable::read(&raw const table) }.unwrap();
    assert!(read.in_gameplay.is_some());
    assert_eq!(AmmlClientTable::SIZE_V6, 152);
}

#[test]
fn client_table_v5_reads_its_replay_function() {
    let table = table();
    // SAFETY: the table lives for the call.
    let read = unsafe { AmmlClientTable::read(&raw const table) }.unwrap();
    assert!(read.replay_app_focus_lost.is_some());
    assert_eq!(AmmlClientTable::SIZE_V5, 144);
}

#[test]
fn client_table_v6_reads_none_for_join_server_direct() {
    let mut table = table();
    table.size = AmmlClientTable::SIZE_V6;
    // SAFETY: the table lives for the call; the read covers only its first `size` bytes (the v6 size).
    let read = unsafe { AmmlClientTable::read(&raw const table) }.unwrap();
    assert!(read.in_gameplay.is_some());
    assert!(read.join_server_direct.is_none());
}

#[test]
fn client_table_v7_reads_its_join_server_direct_function() {
    let mut table = table();
    table.size = AmmlClientTable::SIZE_V7;
    // SAFETY: the table lives for the call.
    let read = unsafe { AmmlClientTable::read(&raw const table) }.unwrap();
    assert!(read.join_server_direct.is_some());
    assert_eq!(AmmlClientTable::SIZE_V7, 160);
}

#[test]
fn client_table_v7_reads_none_for_the_title_functions() {
    let mut table = table();
    table.size = AmmlClientTable::SIZE_V7;
    // SAFETY: the table lives for the call; the read covers only its first `size` bytes (the v7 size).
    let read = unsafe { AmmlClientTable::read(&raw const table) }.unwrap();
    assert!(read.join_server_direct.is_some());
    assert!(read.title_screen.is_none() && read.go_to_title.is_none());
}

#[test]
fn client_table_v8_reads_its_title_functions() {
    let table = table();
    // SAFETY: the table lives for the call.
    let read = unsafe { AmmlClientTable::read(&raw const table) }.unwrap();
    assert!(read.title_screen.is_some() && read.go_to_title.is_some());
    assert_eq!(AmmlClientTable::SIZE_V8, 176);
}
