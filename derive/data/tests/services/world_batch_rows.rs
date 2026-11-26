use std::path::Path;

use amml_data::{AdapterCatalog, BuildDataStore, EntryId, NeedKind};

// The event rows of the levels, freecam and minimap ports (the union of their `amml_uses!` blocks).
const EVENTS: &[&str] = &[
    "client.tick",
    "render.world_rendered",
    "camera.setup",
    "camera.instruction_received",
    "input.key",
    "input.mouse",
    "world.left",
    "world.block_changed",
    "registry.provider_ready",
    "registry.provider_gone",
    "mods.loaded",
    "input.lease_suspended",
    "input.lease_resumed",
    "input.lease_lost",
    "overlay.screenshot_taken",
    "overlay.screenshot_written",
];

// The same union for events whose row version is above 1.
const EVENTS_AT: &[(&str, u32)] = &[("input.injected_released", 2)];

// The area rows of the three ports, with the table version each needs.
const AREAS: &[(&str, u32)] = &[
    ("input.lease", 1),
    ("client.read", 1),
    ("client.hud", 1),
    ("client.mouse", 1),
    ("client.world_info", 1),
    ("actor.read", 3),
    ("actor.health", 1),
    ("window.read", 1),
    ("window.cursor_clip", 1),
    ("overlay.screenshot", 1),
    ("world.sky_darken", 1),
    ("world.chunk_read", 1),
    ("world.map_color", 1),
    ("world.biomes", 1),
    ("world.biomes", 2),
    ("world.scan_surfaces", 1),
    ("world.shapes", 1),
    ("world.raycast", 1),
    ("world.actors_in", 1),
    ("world.actors_in", 2),
    ("render.world_canvas", 2),
    ("camera.pose", 1),
    ("camera.projection", 1),
    ("camera.free", 1),
    ("camera.active", 1),
];

fn check_rows(file: &str, kind: NeedKind, wanted: &[(&str, u32)]) {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data").join(file);
    let data = BuildDataStore::read(&path).unwrap();
    let rows = AdapterCatalog::availability(&|name| data.entries.contains_key(&EntryId(name.to_string())));
    for (id, version) in wanted {
        let found: Vec<_> =
            rows.iter().filter(|row| row.id == *id && row.kind == kind && row.version >= *version).collect();
        assert!(!found.is_empty(), "{file}: no catalog row for {id}");
        assert!(found.iter().any(|row| row.shape.is_some()), "{file}: {id} has no shape: {found:?}");
        let served = AdapterCatalog::served_version(kind, id).unwrap();
        assert!(served >= *version, "{file}: {id} is served at {served}, the contract needs {version}");
    }
}

fn check_build(file: &str) {
    let events: Vec<(&str, u32)> = EVENTS.iter().map(|id| (*id, 1)).chain(EVENTS_AT.iter().copied()).collect();
    check_rows(file, NeedKind::Event, &events);
    check_rows(file, NeedKind::Area, AREAS);
}

#[test]
fn every_world_contract_row_has_a_shape_on_26_63() {
    check_build("6aed1906-b127418e74d2f2f9.hcl");
}

#[test]
fn every_world_contract_row_has_a_shape_on_70_26() {
    check_build("6afa4e47-dadc375797a3d31a.hcl");
}
