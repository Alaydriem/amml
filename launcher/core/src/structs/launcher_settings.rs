use crate::{LauncherPaths, RegistryRef};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LauncherSettings {
    // the official registry first, then the player's own, in the order the player added them
    pub registries: Vec<RegistryRef>,
    pub preview: bool,
    pub data_dir: Option<PathBuf>,
}

impl LauncherSettings {
    pub const OFFICIAL_REGISTRY: &'static str = "https://alaydriem.github.io/amml-registry/";

    pub fn new(registries: Vec<RegistryRef>, preview: bool, data_dir: Option<PathBuf>) -> LauncherSettings {
        LauncherSettings { registries, preview, data_dir }
    }

    pub fn official() -> RegistryRef {
        RegistryRef::new(Self::OFFICIAL_REGISTRY.to_string())
    }

    // the official registry first and never repeated; any other duplicate is dropped
    pub fn normalised(registries: Vec<RegistryRef>) -> Vec<RegistryRef> {
        let mut result = vec![Self::official()];
        for registry in registries {
            if !result.iter().any(|kept| kept.base == registry.base) {
                result.push(registry);
            }
        }
        result
    }

    pub fn data_dir(&self, paths: &LauncherPaths) -> PathBuf {
        self.data_dir.clone().unwrap_or_else(|| paths.default_data_dir())
    }
}
