use std::path::Path;

use amml_data::{AdapterCatalog, BuildDataStore, EntryId, NeedKind};

// The event rows of the stratum port (its `amml_uses!` block), all required.
const EVENTS: &[(&str, u32)] = &[
    ("client.tick", 1),
    ("world.joined", 1),
    ("world.left", 1),
    ("world.dimension_changing", 1),
    ("registry.provider_ready", 1),
    ("registry.provider_gone", 1),
    ("mods.loaded", 1),
    ("input.lease_suspended", 1),
    ("input.lease_resumed", 1),
    ("input.lease_lost", 1),
    ("render.world_rendered", 1),
    ("world.block_changed", 1),
    ("render.block_entity_pass", 1),
    ("player.use_pressed", 1),
    ("player.build_block", 1),
    ("inventory.request_done", 1),
    ("input.mouse", 1),
    ("window.focus", 1),
    ("client.app_focus_lost", 1),
];

// The area rows of the stratum port, with the table version each needs.
const AREAS: &[(&str, u32)] = &[
    ("input.lease", 1),
    ("client.read", 1),
    ("client.world_info", 1),
    ("actor.read", 4),
    ("options.view_distance", 1),
    ("render.world_canvas", 3),
    ("render.block_mesher", 1),
    ("world.block_states", 1),
    ("render.water_tint", 1),
    ("render.block_entities", 1),
    ("render.chest_pair", 1),
    ("render.skull_pose", 1),
    ("world.blocks", 1),
    ("player.build", 1),
    ("player.pick_range", 1),
    ("player.hit_result", 1),
    ("hud.select_hotbar", 1),
    ("inventory.read", 3),
    ("inventory.request", 2),
    ("client.send_command", 1),
    ("actor.game_type", 1),
    ("actor.permission", 1),
    ("world.kinds_in_box", 1),
];

fn check_rows(file: &str, kind: NeedKind, wanted: &[(&str, u32)]) {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data").join(file);
    let data = BuildDataStore::read(&path).unwrap();
    let rows = AdapterCatalog::availability(&|name| data.entries.contains_key(&EntryId(name.to_string())));
    for (id, version) in wanted {
        let found: Vec<_> = rows.iter().filter(|row| row.id == *id && row.kind == kind).collect();
        assert!(!found.is_empty(), "{file}: no catalog row for {id}");
        assert!(found.iter().any(|row| row.shape.is_some()), "{file}: {id} has no shape: {found:?}");
        let served = AdapterCatalog::served_version(kind, id).unwrap();
        assert!(served >= *version, "{file}: {id} is served at {served}, the contract needs {version}");
    }
}

fn check_build(file: &str) {
    check_rows(file, NeedKind::Event, EVENTS);
    check_rows(file, NeedKind::Area, AREAS);
}

#[test]
fn every_stratum_contract_row_has_a_shape_on_26_63() {
    check_build("6aed1906-b127418e74d2f2f9.hcl");
}

#[test]
fn every_stratum_contract_row_has_a_shape_on_70_26() {
    check_build("6afa4e47-dadc375797a3d31a.hcl");
}
