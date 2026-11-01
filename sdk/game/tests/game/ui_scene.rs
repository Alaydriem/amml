use amml_game::{EntryId, GameEntries, UiScene};

use crate::support::{FakeEntries, GameMemory};

fn entries(offset: u64) -> GameEntries {
    FakeEntries::build(&[
        (EntryId::UI_SCENE_SCREEN_VIEW, offset),
        (EntryId::SCREEN_VIEW_POINTER_LOCATION_PREVIOUS, 0x33c),
    ])
}

#[test]
fn screen_view_is_the_pointer_at_the_entry_offset() {
    for offset in [0x40, 0x58] {
        let entries = entries(offset);
        let mut view = GameMemory::new(0x480);
        view.put::<[f32; 2]>(0x33c, [12.0, 34.0]);
        let mut scene = GameMemory::new(0x100);
        scene.put::<usize>(offset as usize, view.ptr() as usize);
        // SAFETY: a zeroed buffer as large as the field the wrapper reads.
        let scene = unsafe { UiScene::from_raw(scene.ptr(), &entries) }.unwrap();

        let read = scene.screen_view().unwrap();

        assert_eq!(read.as_ptr(), view.ptr());
        assert_eq!(read.pointer(), Some([12.0, 34.0]));
    }
}

#[test]
fn a_null_view_a_null_scene_or_a_missing_entry_gives_none() {
    let on = entries(0x40);
    let mut scene = GameMemory::new(0x100);
    // SAFETY: a zeroed buffer: the view pointer is null.
    assert!(unsafe { UiScene::from_raw(scene.ptr(), &on) }.unwrap().screen_view().is_none());
    // SAFETY: a null pointer is allowed and gives None.
    assert!(unsafe { UiScene::from_raw(std::ptr::null_mut(), &on) }.is_none());

    let off = FakeEntries::build(&[]);
    let mut view = GameMemory::new(0x480);
    scene.put::<usize>(0x40, view.ptr() as usize);
    // SAFETY: as above; with the entry off the wrapper reads nothing.
    assert!(unsafe { UiScene::from_raw(scene.ptr(), &off) }.unwrap().screen_view().is_none());
}
