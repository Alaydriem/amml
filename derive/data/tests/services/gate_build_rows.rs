use std::path::Path;

use amml_data::{AdapterCatalog, BuildDataStore, EntryId, NeedKind};

// The event and area rows of the three mod ports (peek, hotbar-manager, heads), with the table version each needs.
const EVENTS: &[&str] = &[
    "ui.screen_render",
    "ui.container_slot_hover",
    "ui.container_slot_unhover",
    "ui.screen_closing",
    "ui.screen_changed",
    "input.ui_scroll",
    "input.ui_pointer",
    "input.key",
    "ui.item_hovertext",
    "client.tick",
    "inventory.request_done",
    "network.player_list",
    "network.player_skin",
    "ui.locator_bar_icon",
    "ui.chat_message",
    "ui.textures_reloaded",
    "world.left",
    "registry.provider_ready",
    "registry.provider_gone",
    "mods.loaded",
];

const AREAS: &[(&str, u32)] = &[
    ("ui.draw", 2),
    ("ui.screen", 2),
    ("ui.text", 1),
    ("ui.textures", 1),
    ("ui.inline_icons", 1),
    ("inventory.read", 4),
    ("inventory.request", 1),
    ("client.read", 1),
    ("client.top_screen", 2),
    ("network.skin", 1),
    ("network.skin_animated", 1),
    ("network.skin_geometry", 1),
    ("network.skin_details", 1),
];

fn check_build(file: &str) {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data").join(file);
    let data = BuildDataStore::read(&path).unwrap();
    let rows = AdapterCatalog::availability(&|name| data.entries.contains_key(&EntryId(name.to_string())));
    for id in EVENTS {
        let found: Vec<_> = rows.iter().filter(|row| row.id == *id && row.kind == NeedKind::Event).collect();
        assert!(!found.is_empty(), "{file}: no catalog row for event {id}");
        assert!(found.iter().any(|row| row.shape.is_some()), "{file}: event {id} has no shape: {found:?}");
    }
    for (id, version) in AREAS {
        let found: Vec<_> = rows.iter().filter(|row| row.id == *id && row.kind == NeedKind::Area).collect();
        assert!(!found.is_empty(), "{file}: no catalog row for area {id}");
        assert!(found.iter().any(|row| row.shape.is_some()), "{file}: area {id} has no shape: {found:?}");
        let served = AdapterCatalog::served_version(NeedKind::Area, id).unwrap();
        assert!(served >= *version, "{file}: area {id} is served at {served}, the contract needs {version}");
    }
}

#[test]
fn every_contract_row_has_a_shape_on_26_63() {
    check_build("6aed1906-b127418e74d2f2f9.hcl");
}

#[test]
fn every_contract_row_has_a_shape_on_70_26() {
    check_build("6afa4e47-dadc375797a3d31a.hcl");
}
