use crate::{LauncherModEntry, ModSource};
use serde_json::{Map, Value};
use std::collections::BTreeMap;

// The `launcher` object of `version.json`: the AMML version and the mods the launcher manages.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LauncherBlock {
    pub amml: Option<String>,
    pub mods: BTreeMap<String, LauncherModEntry>,
}

impl LauncherBlock {
    pub fn new(amml: Option<String>, mods: BTreeMap<String, LauncherModEntry>) -> LauncherBlock {
        LauncherBlock { amml, mods }
    }

    pub fn to_value(&self) -> Value {
        let mut object = Map::new();
        if let Some(amml) = &self.amml {
            object.insert("amml".to_string(), Value::String(amml.clone()));
        }
        let mut mods = Map::new();
        for (id, entry) in &self.mods {
            let mut item = Map::new();
            if let Some(version) = &entry.version {
                item.insert("version".to_string(), Value::String(version.clone()));
            }
            item.insert("source".to_string(), Value::String(entry.source.as_text()));
            if let Some(core) = entry.core {
                item.insert("core".to_string(), Value::Bool(core));
            }
            mods.insert(id.clone(), Value::Object(item));
        }
        object.insert("mods".to_string(), Value::Object(mods));
        Value::Object(object)
    }

    // a value that is not an object gives `None`; entries of the wrong shape are left out
    pub fn from_value(value: &Value) -> Option<LauncherBlock> {
        let object = value.as_object()?;
        let amml = object.get("amml").and_then(Value::as_str).map(str::to_string);
        let mut mods = BTreeMap::new();
        if let Some(listed) = object.get("mods").and_then(Value::as_object) {
            for (id, item) in listed {
                let Some(item) = item.as_object() else {
                    continue;
                };
                let version = item.get("version").and_then(Value::as_str).map(str::to_string);
                let source =
                    item.get("source").and_then(Value::as_str).map(ModSource::parse).unwrap_or(ModSource::Local);
                let core = item.get("core").and_then(Value::as_bool);
                mods.insert(id.clone(), LauncherModEntry { version, source, core });
            }
        }
        Some(LauncherBlock { amml, mods })
    }
}
