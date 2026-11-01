use amml_game::{EntryId, EntryNeed};
use amml_testmod::PeekEntries;

#[test]
fn wanted_covers_each_group_under_its_own_feature() {
    let wanted = PeekEntries::wanted();
    let feature_of = |id: EntryId| wanted.iter().find(|(entry, _)| *entry == id).map(|(_, need)| *need);
    assert_eq!(feature_of(EntryId::SCREEN_VIEW_RENDER), Some(EntryNeed::Optional { feature: "game.ui_render" }));
    assert_eq!(
        feature_of(EntryId::SCREEN_CONTROLLER_DTOR),
        Some(EntryNeed::Optional { feature: "game.container_hover" })
    );
    assert_eq!(
        feature_of(PeekEntries::FONT_HANDLE_SIZE),
        Some(EntryNeed::Optional { feature: PeekEntries::FONT_FEATURE })
    );
}

#[test]
fn wanted_names_no_entry_twice() {
    let wanted = PeekEntries::wanted();
    let mut names: Vec<&str> = wanted.iter().map(|(id, _)| id.as_str()).collect();
    names.sort_unstable();
    let count = names.len();
    names.dedup();
    assert_eq!(names.len(), count);
}
