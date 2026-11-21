use amml_abi::{AmmlScreenKind, AmmlUiProfile};
use amml_runtime::PanelNames;

#[test]
fn container_kinds_anchor_to_the_root_panel() {
    for kind in [
        AmmlScreenKind::SMALL_CHEST,
        AmmlScreenKind::LARGE_CHEST,
        AmmlScreenKind::BARREL,
        AmmlScreenKind::SHULKER_BOX,
        AmmlScreenKind::ENDER_CHEST,
        AmmlScreenKind::HOPPER,
        AmmlScreenKind::DISPENSER,
        AmmlScreenKind::DROPPER,
    ] {
        assert_eq!(PanelNames::path(kind, AmmlUiProfile::CLASSIC), Some(&["root_panel"][..]), "{kind:?}");
    }
}

#[test]
fn the_player_inventory_anchors_to_its_own_panel() {
    assert_eq!(
        PanelNames::path(AmmlScreenKind::PLAYER_INVENTORY, AmmlUiProfile::CLASSIC),
        Some(&["content_stack_panel", "player_inventory"][..])
    );
}

#[test]
fn other_screens_and_the_pocket_profile_have_no_panel() {
    assert_eq!(PanelNames::path(AmmlScreenKind::OTHER, AmmlUiProfile::CLASSIC), None);
    assert_eq!(PanelNames::path(AmmlScreenKind::SMALL_CHEST, AmmlUiProfile::POCKET), None);
}

#[test]
fn the_walk_starts_at_the_content_control() {
    assert_eq!(PanelNames::CONTENT, "root_screen_panel");
}
