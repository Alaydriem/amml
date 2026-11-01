use amml_game::{ClientInstance, EntryId};

use crate::support::{FakeActor, FakeClient, FakeEntries, FakeOptions};

#[test]
fn a_local_player_that_is_null_gives_none() {
    let entries = FakeClient::entries(None);
    let mut client = FakeClient::new(std::ptr::null_mut(), std::ptr::null_mut(), false);
    // SAFETY: a live fake client with the slots the entries name.
    let client = unsafe { ClientInstance::from_raw(client.ptr(), &entries) }.unwrap();

    assert!(client.local_player().is_none());
}

#[test]
fn a_local_player_that_is_set_gives_the_same_actor_pointer() {
    let entries = FakeClient::entries(None);
    let mut actor =
        FakeActor::new(std::ptr::null_mut(), std::ptr::null_mut(), std::ptr::null_mut(), std::ptr::null_mut());
    let actor_ptr = actor.ptr();
    let mut client = FakeClient::new(actor_ptr, std::ptr::null_mut(), true);
    // SAFETY: a live fake client with the slots the entries name.
    let client = unsafe { ClientInstance::from_raw(client.ptr(), &entries) }.unwrap();

    assert_eq!(client.local_player().map(|player| player.as_ptr()), Some(actor_ptr));
}

#[test]
fn the_three_flag_reads_follow_the_game_and_are_false_when_their_entry_is_off() {
    let all = FakeClient::entries(None);
    let mut fake = FakeClient::new(std::ptr::null_mut(), std::ptr::null_mut(), true);
    let ptr = fake.ptr();
    // SAFETY: a live fake client with the slots the entries name.
    let client = unsafe { ClientInstance::from_raw(ptr, &all) }.unwrap();
    assert!(client.is_world_active());
    assert!(client.is_showing_menu());
    assert!(!client.mouse_grabbed());

    let none = FakeEntries::build(&[]);
    // SAFETY: no entry is present, so the vtable is never read.
    let client = unsafe { ClientInstance::from_raw(ptr, &none) }.unwrap();
    assert!(!client.is_world_active());
    assert!(!client.is_showing_menu());
    assert!(!client.mouse_grabbed());
}

#[test]
fn hide_hud_reads_through_the_options_object() {
    let all = FakeClient::entries(None);
    for hidden in [true, false] {
        let mut options = FakeOptions::new(hidden);
        let mut fake = FakeClient::new(std::ptr::null_mut(), options.ptr(), true);
        // SAFETY: a live fake client with the slots the entries name.
        let client = unsafe { ClientInstance::from_raw(fake.ptr(), &all) }.unwrap();

        assert_eq!(client.hide_hud(), Some(hidden));
    }
}

#[test]
fn hide_hud_is_none_when_either_entry_is_missing_or_the_options_pointer_is_null() {
    let mut options = FakeOptions::new(true);
    let mut fake = FakeClient::new(std::ptr::null_mut(), options.ptr(), true);
    for missing in [EntryId::CLIENT_INSTANCE_GET_OPTIONS, EntryId::I_OPTION_REGISTRY_GET_HIDE_HUD] {
        let entries = FakeClient::entries(Some(missing));
        // SAFETY: a live fake client; the missing entry stops the read before its slot is used.
        let client = unsafe { ClientInstance::from_raw(fake.ptr(), &entries) }.unwrap();
        assert_eq!(client.hide_hud(), None);
    }

    let entries = FakeClient::entries(None);
    let mut no_options = FakeClient::new(std::ptr::null_mut(), std::ptr::null_mut(), true);
    // SAFETY: a live fake client with the slots the entries name.
    let client = unsafe { ClientInstance::from_raw(no_options.ptr(), &entries) }.unwrap();
    assert_eq!(client.hide_hud(), None);
}

#[test]
fn a_null_client_pointer_gives_none() {
    let entries = FakeClient::entries(None);

    // SAFETY: null is allowed.
    assert!(unsafe { ClientInstance::from_raw(std::ptr::null_mut(), &entries) }.is_none());
}

extern "C" fn get_minecraft_game(_: *mut std::ffi::c_void) -> *mut std::ffi::c_void {
    0x6A3E as *mut std::ffi::c_void
}

#[test]
fn minecraft_game_calls_the_slot_and_is_none_when_the_entry_is_off() {
    let on = FakeEntries::build(&[(EntryId::CLIENT_INSTANCE_GET_MINECRAFT_GAME, 78)]);
    let mut fake = crate::support::FakeVirtual::new(0x20, &[(78, get_minecraft_game as *const () as usize)]);
    let ptr = fake.ptr();
    // SAFETY: a live fake whose vtable holds the slot.
    let client = unsafe { ClientInstance::from_raw(ptr, &on) }.unwrap();
    assert_eq!(client.minecraft_game().map(|game| game as usize), Some(0x6A3E));

    let off = FakeEntries::build(&[]);
    // SAFETY: the vtable is never read because the entry is off.
    let client = unsafe { ClientInstance::from_raw(ptr, &off) }.unwrap();
    assert!(client.minecraft_game().is_none());
}
