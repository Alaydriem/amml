use std::path::Path;

use amml_data::{AdapterCatalog, BuildDataStore, EntryId, NeedKind};

// The event rows of the dejavu port (its `amml_uses!` block), all required.
const EVENTS: &[(&str, u32)] = &[
    ("frame.update", 1),
    ("client.tick", 1),
    ("world.joined", 1),
    ("world.left", 1),
    ("mods.loaded", 1),
    ("registry.provider_ready", 1),
    ("registry.provider_gone", 1),
    ("input.lease_suspended", 1),
    ("input.lease_resumed", 1),
    ("input.lease_lost", 1),
    ("network.packet_wire", 1),
    ("network.connection_started", 1),
    ("world.join_started", 2),
    ("world.server_join_starting", 2),
    ("client.join_server_result", 1),
    ("camera.setup", 1),
    ("camera.fov", 1),
    ("player.moved", 1),
    ("time.step_applied", 1),
    ("time.frame_end", 1),
    ("time.latency_barrier", 1),
    ("client.app_focus_lost", 1),
    ("client.app_focus_gained", 1),
    ("client.idle_screen_open", 1),
    ("world.storage_opened", 1),
    ("world.storage_closed", 1),
];

// The area rows of the dejavu port, with the table version each needs.
const AREAS: &[(&str, u32)] = &[
    ("client.read", 1),
    ("client.top_screen", 2),
    ("input.lease", 1),
    ("network.chunk_cache", 1),
    ("client.world_info", 1),
    ("client.join_server", 1),
    ("client.join_server_direct", 1),
    ("client.leave_world", 1),
    ("client.in_gameplay", 1),
    ("client.can_join", 1),
    ("camera.pose", 1),
    ("camera.free", 1),
    ("camera.hide_actor", 1),
    ("actor.read", 3),
    ("actor.look_write", 1),
    ("time.control", 1),
    ("client.focus_replay", 1),
    ("client.activity", 1),
    ("options.int", 1),
    ("world.shapes", 1),
    ("world.raycast", 1),
    ("world.actors_in", 1),
    ("window.read", 1),
    ("network.protocol", 1),
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
fn every_dejavu_contract_row_has_a_shape_on_26_63() {
    check_build("6aed1906-b127418e74d2f2f9.hcl");
}

#[test]
fn every_dejavu_contract_row_has_a_shape_on_70_26() {
    check_build("6afa4e47-dadc375797a3d31a.hcl");
}
