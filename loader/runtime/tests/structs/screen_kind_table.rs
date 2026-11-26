use amml_abi::AmmlScreenKind;
use amml_runtime::ScreenKindTable;

#[test]
fn small_chest_root_name_is_small_chest() {
    assert_eq!(ScreenKindTable::kind_of("small_chest_screen"), AmmlScreenKind::SMALL_CHEST);
}

#[test]
fn large_chest_root_name_is_large_chest() {
    assert_eq!(ScreenKindTable::kind_of("large_chest_screen"), AmmlScreenKind::LARGE_CHEST);
}

#[test]
fn barrel_root_name_is_barrel() {
    assert_eq!(ScreenKindTable::kind_of("barrel_screen"), AmmlScreenKind::BARREL);
}

#[test]
fn shulker_box_root_name_is_shulker_box() {
    assert_eq!(ScreenKindTable::kind_of("shulker_box_screen"), AmmlScreenKind::SHULKER_BOX);
}

#[test]
fn ender_chest_root_name_is_ender_chest() {
    assert_eq!(ScreenKindTable::kind_of("ender_chest_screen"), AmmlScreenKind::ENDER_CHEST);
}

#[test]
fn hopper_root_name_is_hopper() {
    assert_eq!(ScreenKindTable::kind_of("hopper_screen"), AmmlScreenKind::HOPPER);
}

#[test]
fn dispenser_root_name_is_dispenser() {
    assert_eq!(ScreenKindTable::kind_of("dispenser_screen"), AmmlScreenKind::DISPENSER);
}

#[test]
fn dropper_root_name_is_dropper() {
    assert_eq!(ScreenKindTable::kind_of("dropper_screen"), AmmlScreenKind::DROPPER);
}

#[test]
fn both_inventory_root_names_are_the_player_inventory() {
    assert_eq!(ScreenKindTable::kind_of("inventory_screen"), AmmlScreenKind::PLAYER_INVENTORY);
    assert_eq!(ScreenKindTable::kind_of("inventory_screen_pocket"), AmmlScreenKind::PLAYER_INVENTORY);
}

#[test]
fn unknown_root_is_other() {
    assert_eq!(ScreenKindTable::kind_of("crafting_screen"), AmmlScreenKind::OTHER);
    assert_eq!(ScreenKindTable::kind_of(""), AmmlScreenKind::OTHER);
}
