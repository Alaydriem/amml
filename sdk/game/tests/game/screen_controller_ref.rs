use std::ffi::c_void;

use amml_game::{EntryId, ScreenControllerRef};

use crate::support::{FakeEntries, FakeVirtual};

const IS_CONTAINER_SLOT: u64 = 38;

extern "C" fn container_screen(_: *mut c_void) -> u8 {
    1
}

extern "C" fn hud_screen(_: *mut c_void) -> u8 {
    0
}

#[test]
fn is_container_screen_calls_the_slot_and_gives_the_answer() {
    let entries = FakeEntries::build(&[(EntryId::SCREEN_CONTROLLER_IS_CONTAINER_SCREEN, IS_CONTAINER_SLOT)]);
    let mut container = FakeVirtual::new(0x20, &[(IS_CONTAINER_SLOT, container_screen as *const () as usize)]);
    let mut hud = FakeVirtual::new(0x20, &[(IS_CONTAINER_SLOT, hud_screen as *const () as usize)]);

    // SAFETY: live fakes whose vtables hold the slot.
    let container = unsafe { ScreenControllerRef::from_raw(container.ptr(), &entries) }.unwrap();
    // SAFETY: as above.
    let hud = unsafe { ScreenControllerRef::from_raw(hud.ptr(), &entries) }.unwrap();

    assert_eq!(container.is_container_screen(), Some(true));
    assert_eq!(hud.is_container_screen(), Some(false));
    assert!(hud.container().is_none());
    assert!(container.container().is_some());
}

#[test]
fn is_container_screen_is_none_when_the_entry_is_off() {
    let entries = FakeEntries::build(&[]);
    let mut container = FakeVirtual::new(0x20, &[(IS_CONTAINER_SLOT, container_screen as *const () as usize)]);
    let ptr = container.ptr();

    // SAFETY: a live fake; the vtable is never read because the entry is off.
    let controller = unsafe { ScreenControllerRef::from_raw(ptr, &entries) }.unwrap();

    assert_eq!(controller.is_container_screen(), None);
    assert_eq!(controller.address(), ptr as usize);
}
