use crate::{Catalogue, CatalogueEntry, Channel, GameVersion, GdkError};
use serde_json::Value;

pub struct CatalogueReader;

impl CatalogueReader {
    pub const URL: &'static str =
        "https://github.com/MinecraftBedrockArchiver/GdkLinks/raw/refs/heads/master/urls.json";

    const LIMIT_BYTES: u64 = 256 * 1024 * 1024;

    pub fn fetch(agent: &ureq::Agent) -> Result<Catalogue, GdkError> {
        Self::fetch_from(agent, Self::URL)
    }

    pub fn fetch_from(agent: &ureq::Agent, url: &str) -> Result<Catalogue, GdkError> {
        let mut response = agent.get(url).call().map_err(|error| GdkError::Download(error.to_string()))?;
        if !response.status().is_success() {
            return Err(GdkError::Download(format!("the catalogue answered {}", response.status().as_u16())));
        }
        let text = response
            .body_mut()
            .with_config()
            .limit(Self::LIMIT_BYTES)
            .read_to_string()
            .map_err(|error| GdkError::Download(error.to_string()))?;
        Self::parse(&text)
    }

    pub fn parse(json: &str) -> Result<Catalogue, GdkError> {
        let root: Value = serde_json::from_str(json).map_err(|_| GdkError::BadCatalogue("release"))?;
        let release = Self::section(&root, "release", Channel::Release)?;
        let preview = Self::section(&root, "preview", Channel::Preview)?;
        Ok(Catalogue::new(release, preview))
    }

    fn section(root: &Value, key: &'static str, channel: Channel) -> Result<Vec<CatalogueEntry>, GdkError> {
        let map = root.get(key).and_then(Value::as_object).ok_or(GdkError::BadCatalogue(key))?;
        let mut entries = Vec::new();
        for (name, value) in map {
            // a key that is not a version is not an install target
            let Ok(version) = GameVersion::parse(name) else { continue };
            let urls: Vec<String> = match value {
                Value::Array(items) => items.iter().filter_map(|u| u.as_str().map(str::to_string)).collect(),
                Value::String(url) => vec![url.clone()],
                _ => continue,
            };
            if urls.is_empty() {
                continue;
            }
            entries.push(CatalogueEntry { version, channel, urls });
        }
        Ok(entries)
    }
}
