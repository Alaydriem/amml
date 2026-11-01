use std::collections::HashMap;

use amml_game::{EntryId, EntryNeed, EntryResolver, GameEntries, GameError};
use amml_sdk::SdkError;
use amml_sdk::amml_abi::AmmlStatus;

/// A resolver that knows a fixed set of ids; `broken` ids fail with a status other than missing.
struct FakeResolver {
    known: HashMap<&'static str, u64>,
    broken: Vec<&'static str>,
}

impl FakeResolver {
    fn new(known: &[(&'static str, u64)]) -> Self {
        Self { known: known.iter().copied().collect(), broken: Vec::new() }
    }
}

impl EntryResolver for FakeResolver {
    fn resolve(&self, id: &str) -> Result<u64, SdkError> {
        if self.broken.contains(&id) {
            return Err(SdkError::Status(AmmlStatus::FAILED));
        }
        self.known.get(id).copied().ok_or_else(|| SdkError::MissingEntry(id.to_string()))
    }
}

const REQUIRED: EntryId = EntryId::CLIENT_INSTANCE_UPDATE;
const MAP: EntryId = EntryId::ACTOR_LEVEL;
const SKY: EntryId = EntryId::LEVEL_CHUNK_SUB_CHUNKS;
const GLOW: EntryId = EntryId::CAMERA_FORWARD;

fn wanted() -> Vec<(EntryId, EntryNeed)> {
    vec![
        (REQUIRED, EntryNeed::Required),
        (MAP, EntryNeed::Optional { feature: "map" }),
        (SKY, EntryNeed::Optional { feature: "map" }),
        (GLOW, EntryNeed::Optional { feature: "glow" }),
    ]
}

#[test]
fn a_resolver_with_only_the_required_ids_gives_entries_and_lists_each_off_feature_once() {
    let resolver = FakeResolver::new(&[("ClientInstance.update", 0x10)]);

    let entries = GameEntries::resolve(&resolver, &wanted()).unwrap();

    assert_eq!(entries.get(REQUIRED), Some(0x10));
    assert_eq!(entries.get(MAP), None);
    let off = entries.off_features();
    assert_eq!(off.iter().map(|(feature, _)| *feature).collect::<Vec<_>>(), vec!["map", "glow"]);
    assert!(off[0].1.contains("Actor.mLevel"));
}

#[test]
fn an_optional_entry_that_resolves_keeps_its_feature_on() {
    let resolver =
        FakeResolver::new(&[("ClientInstance.update", 0x10), ("Actor.mLevel", 0x20), ("Camera.mForward", 0x30)]);

    let entries = GameEntries::resolve(&resolver, &wanted()).unwrap();

    assert_eq!(entries.get(MAP), Some(0x20));
    assert_eq!(entries.get(GLOW), Some(0x30));
    let off = entries.off_features();
    assert_eq!(off.len(), 1);
    assert_eq!(off[0].0, "map");
}

#[test]
fn a_missing_required_id_names_that_id() {
    let resolver = FakeResolver::new(&[("Actor.mLevel", 0x20)]);

    let error = GameEntries::resolve(&resolver, &wanted()).unwrap_err();

    assert!(matches!(&error, GameError::MissingEntry { id } if id == "ClientInstance.update"));
}

#[test]
fn an_optional_entry_that_fails_for_another_reason_turns_only_its_feature_off_and_keeps_the_cause() {
    let mut resolver = FakeResolver::new(&[("ClientInstance.update", 0x10), ("Camera.mForward", 0x30)]);
    resolver.broken.push("Actor.mLevel");

    let entries = GameEntries::resolve(&resolver, &wanted()).unwrap();

    assert_eq!(entries.get(GLOW), Some(0x30));
    let off = entries.off_features();
    assert_eq!(off.len(), 1);
    assert_eq!(off[0].0, "map");
    assert!(off[0].1.contains("Actor.mLevel"));
    assert!(off[0].1.contains(&SdkError::Status(AmmlStatus::FAILED).to_string()));
}

#[test]
fn a_required_entry_that_fails_for_another_reason_is_an_error() {
    let mut resolver = FakeResolver::new(&[]);
    resolver.broken.push("ClientInstance.update");

    let error = GameEntries::resolve(&resolver, &wanted()).unwrap_err();

    assert!(matches!(&error, GameError::Resolve { id, .. } if id == "ClientInstance.update"));
}
