use amml_runtime::services::ProviderRegistry;
use amml_runtime::structs::ProvideError;

#[test]
fn require_gives_a_provider_with_the_same_major_and_a_new_enough_minor() {
    let registry = ProviderRegistry::new();
    registry.provide("amm.settings", 3, 2, 1, 10u32).unwrap();
    assert_eq!(registry.require("amm.settings", 3, 1), Some((10, 2)));
    assert_eq!(registry.require("amm.settings", 3, 3), None);
    assert_eq!(registry.require("amm.settings", 2, 0), None);
    assert_eq!(registry.require("amm.other", 3, 0), None);
}

#[test]
fn a_second_owner_cannot_take_a_name_and_major() {
    let registry = ProviderRegistry::new();
    registry.provide("amm.settings", 3, 2, 1, 10u32).unwrap();
    assert_eq!(registry.provide("amm.settings", 3, 5, 2, 20), Err(ProvideError::Taken { owner: 1 }));
    assert_eq!(registry.require("amm.settings", 3, 0), Some((10, 2)));
}

#[test]
fn another_major_of_the_same_name_can_be_provided_by_another_owner() {
    let registry = ProviderRegistry::new();
    registry.provide("amm.settings", 3, 2, 1, 10u32).unwrap();
    registry.provide("amm.settings", 4, 0, 2, 20).unwrap();
    assert_eq!(registry.require("amm.settings", 4, 0), Some((20, 0)));
    assert_eq!(registry.require("amm.settings", 3, 0), Some((10, 2)));
}

#[test]
fn the_same_owner_providing_again_replaces_its_table_and_minor() {
    let registry = ProviderRegistry::new();
    registry.provide("amm.settings", 3, 2, 1, 10u32).unwrap();
    registry.provide("amm.settings", 3, 4, 1, 11).unwrap();
    assert_eq!(registry.require("amm.settings", 3, 0), Some((11, 4)));
    assert_eq!(registry.drop_owner(1), vec![("amm.settings".to_string(), 3)]);
}

#[test]
fn drop_owner_removes_its_providers_and_reports_them() {
    let registry = ProviderRegistry::new();
    registry.provide("amm.settings", 3, 2, 1, 10u32).unwrap();
    registry.provide("amm.keys", 1, 0, 2, 20).unwrap();
    assert_eq!(registry.drop_owner(1), vec![("amm.settings".to_string(), 3)]);
    assert_eq!(registry.require("amm.settings", 3, 0), None);
    assert_eq!(registry.require("amm.keys", 1, 0), Some((20, 0)));
    assert!(registry.drop_owner(1).is_empty());
}
