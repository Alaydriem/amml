use std::collections::HashMap;

use amml_game::{EntryId, EntryResolver, GameEntries, GameError, MaterialLayout};
use amml_sdk::SdkError;

/// A resolver that knows a fixed set of ids.
struct Known(HashMap<&'static str, u64>);

impl EntryResolver for Known {
    fn resolve(&self, id: &str) -> Result<u64, SdkError> {
        self.0.get(id).copied().ok_or_else(|| SdkError::MissingEntry(id.to_string()))
    }
}

fn resolve(values: &[(EntryId, u64)]) -> GameEntries {
    let known = Known(values.iter().map(|(id, value)| (id.as_str(), *value)).collect());
    match GameEntries::resolve(&known, &MaterialLayout::wanted()) {
        Ok(entries) => entries,
        Err(error) => panic!("{error}"),
    }
}

#[test]
fn the_layout_entry_selects_the_mirror_by_its_value() {
    // the values of `BlockType.mMaterialInline` in the 26.51 and 26.60 derive data
    let pointer = resolve(&[(EntryId::BLOCK_TYPE_MATERIAL_INLINE, 0)]);
    let value = resolve(&[(EntryId::BLOCK_TYPE_MATERIAL_INLINE, 1)]);
    assert_eq!(MaterialLayout::of(&pointer).unwrap(), MaterialLayout::Pointer);
    assert_eq!(MaterialLayout::of(&value).unwrap(), MaterialLayout::Value);
    assert!(pointer.off_features().is_empty());
}

#[test]
fn a_value_with_no_mirror_or_a_missing_entry_is_an_error() {
    let unknown = resolve(&[(EntryId::BLOCK_TYPE_MATERIAL_INLINE, 2)]);
    let missing = resolve(&[]);
    assert_eq!(missing.off_features().first().map(|(feature, _)| *feature), Some(MaterialLayout::FEATURE));
    for entries in [unknown, missing] {
        assert!(matches!(MaterialLayout::of(&entries), Err(GameError::EntrySet { .. })));
    }
}
