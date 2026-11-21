#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};

use amml_data::{BuildData, BuildId, EntryId, EntryKind, Resolved};
use amml_derive::{CompatChecker, ModUses};

fn data_with(entries: &[&str]) -> BuildData {
    let mut data = BuildData::new(BuildId::new(1, 2));
    for name in entries {
        data.entries.insert(
            EntryId((*name).to_string()),
            Resolved { kind: EntryKind::Function, value: 0x1000, recipes_agreeing: 1 },
        );
    }
    data
}

fn manifest(required: bool) -> ModUses {
    let text = format!(
        "namespace = \"alay\"\nname = \"probe\"\nentry = \"probe.dll\"\nuses {{\n  feature \"core\" {{\n    event \"client.tick\" {{\n      version = 1\n      required = {required}\n    }}\n  }}\n}}\n"
    );
    ModUses::parse(Path::new("mods/@alay/probe"), &text).unwrap()
}

#[test]
fn a_mod_with_every_required_event_available_passes() {
    let report = CompatChecker::check(&data_with(&["ClientInstance.update"]), &[manifest(true)], &[]);
    assert_eq!(report.rows.len(), 1);
    assert!(report.rows[0].available);
    assert_eq!(report.rows[0].shape.as_deref(), Some("default"));
    assert_eq!(report.exit_code(), 0);
    assert!(!report.render().contains("will not enable"));
}

#[test]
fn a_missing_required_event_marks_the_mod() {
    let report = CompatChecker::check(&data_with(&[]), &[manifest(true)], &[]);
    assert!(!report.rows[0].available);
    assert_eq!(report.rows[0].missing, vec!["ClientInstance.update"]);
    assert_eq!(report.exit_code(), 1);
    assert!(report.render().contains("will not enable"));
}

#[test]
fn an_optional_missing_event_only_warns() {
    let report = CompatChecker::check(&data_with(&[]), &[manifest(false)], &[]);
    assert!(!report.rows[0].available);
    assert_eq!(report.exit_code(), 0);
    assert!(!report.render().contains("will not enable"));
}

#[test]
fn an_event_the_catalog_does_not_know_is_not_available() {
    let text = "namespace = \"a\"\nname = \"b\"\nuses {\n  feature \"x\" {\n    event \"nope.nothing\" {\n      version = 1\n      required = true\n    }\n  }\n}\n";
    let uses = ModUses::parse(Path::new("."), text).unwrap();
    let report = CompatChecker::check(&data_with(&[]), &[uses], &[]);
    assert!(!report.rows[0].available);
    assert_eq!(report.exit_code(), 1);
}

#[test]
fn a_forbidden_import_is_listed() {
    let dll = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/pe/key_reader.dll");
    let report = CompatChecker::check(&data_with(&[]), &[], &[dll]);
    assert_eq!(report.warnings.len(), 1);
    assert_eq!(report.warnings[0].function, "GetAsyncKeyState");
    assert!(report.render().contains("GetAsyncKeyState"));
    assert_eq!(report.exit_code(), 0);
}

#[test]
fn a_dll_without_forbidden_imports_has_no_warning() {
    let dll = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/pe/small.dll");
    let report = CompatChecker::check(&data_with(&[]), &[], &[dll]);
    assert!(report.warnings.is_empty());
}

#[test]
fn the_imports_of_the_rust_standard_library_have_no_warning() {
    let dll = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/pe/loader_imports.dll");
    let report = CompatChecker::check(&data_with(&[]), &[], &[dll]);
    assert!(report.warnings.is_empty(), "{:?}", report.warnings);
}

#[test]
fn a_required_area_that_is_not_available_marks_the_mod() {
    let text = "namespace = \"a\"\nname = \"b\"\nuses {\n  feature \"keys\" {\n    area \"input.key_inject\" {\n      version = 1\n      required = true\n    }\n  }\n}\n";
    let uses = ModUses::parse(Path::new("."), text).unwrap();
    let report = CompatChecker::check(&data_with(&[]), &[uses], &[]);
    assert!(!report.rows[0].available);
    assert_eq!(report.rows[0].missing[0], "HIDControllerGameCoreDesktop.onKeyDown");
    assert_eq!(report.exit_code(), 1);
    assert!(report.render().contains("will not enable"));
}

// Every entry the `actor.read` area needs; `Actor.isAlive` first.
const ACTOR_READ_ENTRIES: [&str; 9] = [
    "Actor.isAlive",
    "ClientInstance.getLocalPlayer",
    "Actor.stateVectorComponent",
    "StateVectorComponent.mPos",
    "Actor.aabbShapeComponent",
    "Actor.actorRotationComponent",
    "ActorRotationComponent.mRot",
    "Actor.getDimension",
    "Dimension.getDimensionId",
];

fn actor_read_at(version: u32) -> ModUses {
    let text = format!(
        "namespace = \"a\"\nname = \"b\"\nuses {{\n  feature \"death\" {{\n    area \"actor.read\" {{\n      version = {version}\n      required = true\n    }}\n  }}\n}}\n"
    );
    ModUses::parse(Path::new("."), &text).unwrap()
}

#[test]
fn an_area_version_above_the_catalog_needs_a_newer_loader() {
    let data = data_with(&ACTOR_READ_ENTRIES);
    let report = CompatChecker::check(&data, &[actor_read_at(3)], &[]);
    assert!(report.rows[0].loader_too_old && !report.rows[0].available);
    assert_eq!(report.rows[0].missing, ["loader too old: it serves area `actor.read` up to version 2"]);
    assert_eq!(report.exit_code(), 1);
    assert!(report.render().contains("  feature death: loader too old\n"), "{}", report.render());
}

#[test]
fn an_area_at_the_catalog_version_is_available_when_its_entries_resolve() {
    let data = data_with(&ACTOR_READ_ENTRIES);
    let report = CompatChecker::check(&data, &[actor_read_at(2)], &[]);
    assert!(report.rows[0].available && !report.rows[0].loader_too_old);
    assert!(!report.render().contains("loader too old"));
}

#[test]
fn a_missing_entry_is_not_a_loader_too_old() {
    let report = CompatChecker::check(&data_with(&ACTOR_READ_ENTRIES[1..]), &[actor_read_at(2)], &[]);
    assert!(!report.rows[0].available && !report.rows[0].loader_too_old);
    assert_eq!(report.rows[0].missing, ["Actor.isAlive"]);
}

#[test]
fn an_unknown_event_needs_a_newer_loader() {
    let text = "namespace = \"a\"\nname = \"b\"\nuses {\n  feature \"x\" {\n    event \"nope.nothing\" {\n      version = 1\n      required = true\n    }\n  }\n}\n";
    let uses = ModUses::parse(Path::new("."), text).unwrap();
    let report = CompatChecker::check(&data_with(&[]), &[uses], &[]);
    assert!(report.rows[0].loader_too_old);
    assert_eq!(report.rows[0].missing, ["loader too old: it has no event `nope.nothing`"]);
}
