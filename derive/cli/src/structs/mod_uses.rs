use std::path::{Path, PathBuf};

use hcl::{Body, Expression};

use super::{AreaUse, EventUse, FeatureUse};
use crate::errors::CliError;

/// The `uses` block of one mod's `amml.hcl`, with the mod id and the DLL the manifest names.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModUses {
    pub mod_id: String,
    pub dll: Option<PathBuf>,
    pub features: Vec<FeatureUse>,
}

impl ModUses {
    /// File name of a mod manifest.
    pub const MANIFEST: &'static str = "amml.hcl";

    /// Reads `<dir>/amml.hcl`.
    pub fn read(dir: &Path) -> Result<Self, CliError> {
        let path = dir.join(Self::MANIFEST);
        let text = std::fs::read_to_string(&path)
            .map_err(|error| CliError::Manifest { path: path.clone(), message: error.to_string() })?;
        Self::parse(dir, &text).map_err(|message| CliError::Manifest { path, message })
    }

    /// Reads manifest `text` of the mod folder `dir`. A manifest with no `uses` block has no features.
    pub fn parse(dir: &Path, text: &str) -> Result<Self, String> {
        let body = hcl::parse(text).map_err(|error| error.to_string())?;
        let text_of = |key: &str| {
            body.attributes().find(|attribute| attribute.key() == key).and_then(|attribute| match attribute.expr() {
                Expression::String(value) => Some(value.clone()),
                _ => None,
            })
        };
        let namespace = text_of("namespace").ok_or("missing `namespace`")?;
        let name = text_of("name").ok_or("missing `name`")?;
        let mut features = Vec::new();
        for uses in body.blocks().filter(|block| block.identifier() == "uses") {
            for feature in uses.body().blocks().filter(|block| block.identifier() == "feature") {
                features.push(Self::feature(feature)?);
            }
        }
        Ok(Self {
            mod_id: format!("@{namespace}/{name}"),
            dll: text_of("entry").map(|entry| dir.join(entry)),
            features,
        })
    }

    fn feature(block: &hcl::Block) -> Result<FeatureUse, String> {
        let [label] = block.labels() else {
            return Err("a `feature` block needs exactly one label".to_string());
        };
        let mut events = Vec::new();
        for event in block.body().blocks().filter(|inner| inner.identifier() == "event") {
            let [event_name] = event.labels() else {
                return Err(format!("an `event` block of feature `{}` needs exactly one label", label.as_str()));
            };
            events.push(Self::event(event_name.as_str(), event.body())?);
        }
        let mut areas = Vec::new();
        for area in block.body().blocks().filter(|inner| inner.identifier() == "area") {
            let [area_name] = area.labels() else {
                return Err(format!("an `area` block of feature `{}` needs exactly one label", label.as_str()));
            };
            areas.push(Self::area(area_name.as_str(), area.body())?);
        }
        Ok(FeatureUse { name: label.as_str().to_string(), events, areas })
    }

    fn event(name: &str, body: &Body) -> Result<EventUse, String> {
        let (version, required) = Self::version_and_required("event", name, body)?;
        Ok(EventUse { name: name.to_string(), version, required })
    }

    fn area(name: &str, body: &Body) -> Result<AreaUse, String> {
        let (version, required) = Self::version_and_required("area", name, body)?;
        Ok(AreaUse { name: name.to_string(), version, required })
    }

    // The two attributes of an `event` or `area` block; `what` names the block kind in messages.
    fn version_and_required(what: &str, name: &str, body: &Body) -> Result<(u32, bool), String> {
        let mut version = None;
        let mut required = None;
        for attribute in body.attributes() {
            match (attribute.key(), attribute.expr()) {
                ("version", Expression::Number(number)) => {
                    version = number.as_u64().and_then(|value| u32::try_from(value).ok());
                }
                ("required", Expression::Bool(value)) => required = Some(*value),
                (key, _) => return Err(format!("{what} `{name}`: unknown or invalid attribute `{key}`")),
            }
        }
        Ok((
            version.ok_or_else(|| format!("{what} `{name}` needs `version = <number>`"))?,
            required.ok_or_else(|| format!("{what} `{name}` needs `required = true` or `false`"))?,
        ))
    }
}
