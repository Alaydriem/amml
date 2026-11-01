use amml_game::{EntryId, ScreenView};

use crate::support::{FakeEntries, GameMemory};

fn entries() -> amml_game::GameEntries {
    FakeEntries::build(&[(EntryId::SCREEN_VIEW_POINTER_LOCATION_PREVIOUS, 0x33c)])
}

#[test]
fn pointer_size_and_controller_are_read_at_the_26_60_offsets() {
    let entries = entries();
    let mut memory = GameMemory::new(0x480);
    // ScreenView::mSize 0x18 and mPointerLocationPrevious 0x33c are two f32; mController is a shared_ptr at 0x40
    memory.put::<[f32; 2]>(0x18, [854.0, 480.0]);
    memory.put::<[f32; 2]>(0x33c, [120.5, 64.25]);
    memory.put::<usize>(0x40, 0x1234_5000);
    memory.put::<usize>(0x48, 0x77);
    // SAFETY: a zeroed buffer as large as the fields the wrapper reads.
    let view = unsafe { ScreenView::from_raw(memory.ptr(), &entries) }.unwrap();

    assert_eq!(view.pointer(), Some([120.5, 64.25]));
    assert_eq!(view.size(), Some([854.0, 480.0]));
    assert_eq!(view.controller().map(|controller| controller.address()), Some(0x1234_5000));
}

#[test]
fn a_null_view_gives_none() {
    let entries = entries();

    // SAFETY: a null pointer is allowed and gives None.
    assert!(unsafe { ScreenView::from_raw(std::ptr::null_mut(), &entries) }.is_none());
}

#[test]
fn a_view_without_a_controller_gives_none() {
    let entries = entries();
    let mut memory = GameMemory::new(0x480);
    // SAFETY: a zeroed buffer as large as the fields the wrapper reads.
    let view = unsafe { ScreenView::from_raw(memory.ptr(), &entries) }.unwrap();

    assert!(view.controller().is_none());
}

#[test]
fn with_the_pointer_entry_missing_every_read_gives_none() {
    let entries = FakeEntries::build(&[]);
    let mut memory = GameMemory::new(0x480);
    memory.put::<[f32; 2]>(0x18, [854.0, 480.0]);
    memory.put::<usize>(0x40, 0x1234_5000);
    // SAFETY: a zeroed buffer as large as the fields the wrapper would read.
    let view = unsafe { ScreenView::from_raw(memory.ptr(), &entries) }.unwrap();

    assert!(view.pointer().is_none());
    assert!(view.size().is_none());
    assert!(view.controller().is_none());
}
