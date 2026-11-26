use amml_abi::{AmmlScreenKind, AmmlSlotAt};
use amml_runtime::CollectionNames;

#[test]
fn game_names_map_to_collections() {
    assert_eq!(CollectionNames::collection_of("container_items"), AmmlSlotAt::CONTAINER);
    assert_eq!(CollectionNames::collection_of("hotbar_items"), AmmlSlotAt::HOTBAR);
    assert_eq!(CollectionNames::collection_of("inventory_items"), AmmlSlotAt::INVENTORY);
}

#[test]
fn unknown_name_is_other() {
    assert_eq!(CollectionNames::collection_of("armor_items"), AmmlSlotAt::OTHER);
    assert_eq!(CollectionNames::collection_of(""), AmmlSlotAt::OTHER);
    // The cursor collection is read through its own name, not as a slot collection.
    assert_eq!(CollectionNames::collection_of(CollectionNames::CURSOR), AmmlSlotAt::OTHER);
}

#[test]
fn a_collection_has_one_game_name_and_other_has_none() {
    assert_eq!(CollectionNames::game_name(AmmlSlotAt::CONTAINER), Some("container_items"));
    assert_eq!(CollectionNames::game_name(AmmlSlotAt::HOTBAR), Some("hotbar_items"));
    assert_eq!(CollectionNames::game_name(AmmlSlotAt::INVENTORY), Some("inventory_items"));
    assert_eq!(CollectionNames::game_name(AmmlSlotAt::OTHER), None);
    assert_eq!(CollectionNames::game_name(99), None);
}

#[test]
fn slot_counts_follow_the_screen_kind() {
    let count = |kind| CollectionNames::slot_count(kind, AmmlSlotAt::CONTAINER);

    assert_eq!(count(AmmlScreenKind::SMALL_CHEST), Some(27));
    assert_eq!(count(AmmlScreenKind::BARREL), Some(27));
    assert_eq!(count(AmmlScreenKind::SHULKER_BOX), Some(27));
    assert_eq!(count(AmmlScreenKind::ENDER_CHEST), Some(27));
    assert_eq!(count(AmmlScreenKind::LARGE_CHEST), Some(54));
    assert_eq!(count(AmmlScreenKind::HOPPER), Some(5));
    assert_eq!(count(AmmlScreenKind::DISPENSER), Some(9));
    assert_eq!(count(AmmlScreenKind::DROPPER), Some(9));
    assert_eq!(count(AmmlScreenKind::PLAYER_INVENTORY), Some(0));
    assert_eq!(count(AmmlScreenKind::OTHER), None);
}

#[test]
fn the_player_collections_have_fixed_counts_and_other_has_none() {
    assert_eq!(CollectionNames::slot_count(AmmlScreenKind::SMALL_CHEST, AmmlSlotAt::HOTBAR), Some(9));
    assert_eq!(CollectionNames::slot_count(AmmlScreenKind::PLAYER_INVENTORY, AmmlSlotAt::INVENTORY), Some(27));
    assert_eq!(CollectionNames::slot_count(AmmlScreenKind::SMALL_CHEST, AmmlSlotAt::OTHER), None);
}
