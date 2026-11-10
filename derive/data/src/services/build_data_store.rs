use std::collections::BTreeMap;
use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};

use super::BlockReader;
use crate::errors::DataError;
use crate::structs::{BuildData, BuildId, EntryId, EntryKind, Fingerprint, Resolved};

pub struct BuildDataStore;

impl BuildDataStore {
    // The format this build writes.
    const FORMAT: u32 = 2;
    // The oldest format this build still reads.
    const OLDEST_FORMAT: u32 = 1;

    // Writes the stable text format by hand so the on-disk bytes never depend on a formatter version.
    // `resolved` blocks come first, then `failed` blocks, each run sorted by id.
    // The text goes to a temporary file in the same folder and is renamed over `path`, so a reader
    // or a crash never sees a partial file.
    pub fn write(path: &Path, data: &BuildData) -> Result<(), DataError> {
        let mut text = String::new();
        text.push_str(&format!("format = {}\n\n", Self::FORMAT));
        text.push_str("build {\n");
        text.push_str(&format!("  timestamp = \"{:#010x}\"\n", data.build.timestamp));
        text.push_str(&format!("  text_hash = \"{:#018x}\"\n", data.build.text_hash));
        text.push_str("}\n");
        for id in data.entries.keys().chain(data.failed.keys()) {
            if !EntryId::is_valid_text(&id.0) {
                return Err(DataError::Invalid {
                    entry: id.clone(),
                    message: "ids may only use A-Z a-z 0-9 _ . : - and cannot be empty".to_string(),
                    path: Some(path.to_path_buf()),
                });
            }
        }
        for (id, resolved) in &data.entries {
            text.push('\n');
            text.push_str(&format!("resolved \"{}\" {{\n", id.0));
            text.push_str(&format!("  kind        = \"{}\"\n", resolved.kind.name()));
            text.push_str(&format!("  value       = \"{:#x}\"\n", resolved.value));
            text.push_str(&format!("  recipes     = {}\n", resolved.recipes_agreeing));
            if let Some(fingerprint) = data.fingerprints.get(id) {
                text.push_str(&format!("  fingerprint = \"{fingerprint}\"\n"));
            }
            text.push_str("}\n");
        }
        for (id, reason) in &data.failed {
            text.push('\n');
            text.push_str(&format!("failed \"{}\" {{\n", id.0));
            if let Some(fingerprint) = data.fingerprints.get(id) {
                text.push_str(&format!("  fingerprint = \"{fingerprint}\"\n"));
            }
            text.push_str(&format!("  reason      = {}\n", Self::quote(reason)));
            text.push_str("}\n");
        }
        let io = |source| DataError::Io { path: path.to_path_buf(), source };
        let temp = Self::temp_path(path);
        let written = fs::File::create(&temp).and_then(|mut file| {
            file.write_all(text.as_bytes())?;
            file.sync_all()
        });
        if let Err(source) = written.and_then(|()| fs::rename(&temp, path)) {
            let _ = fs::remove_file(&temp);
            return Err(io(source));
        }
        Ok(())
    }

    // An HCL string literal: backslash, quote and control characters are escaped. A `$` or `%`
    // before `{` is doubled (`$${`, `%%{`) so HCL does not read a template start.
    fn quote(text: &str) -> String {
        let mut quoted = String::from("\"");
        let mut characters = text.chars().peekable();
        while let Some(character) = characters.next() {
            match character {
                '\\' => quoted.push_str("\\\\"),
                '"' => quoted.push_str("\\\""),
                '\n' => quoted.push_str("\\n"),
                '\r' => quoted.push_str("\\r"),
                '\t' => quoted.push_str("\\t"),
                '$' | '%' if characters.peek() == Some(&'{') => {
                    quoted.push(character);
                    quoted.push(character);
                }
                other if other.is_control() => quoted.push_str(&format!("\\u{:04X}", other as u32)),
                other => quoted.push(other),
            }
        }
        quoted.push('"');
        quoted
    }

    // `<name>.<process id>.tmp` beside `path`, so two processes never share a temporary file.
    fn temp_path(path: &Path) -> PathBuf {
        let mut name = path.file_name().map(|name| name.to_os_string()).unwrap_or_default();
        name.push(format!(".{}.tmp", std::process::id()));
        path.with_file_name(name)
    }

    pub fn read(path: &Path) -> Result<BuildData, DataError> {
        let text = fs::read_to_string(path).map_err(|source| DataError::Io { path: path.to_path_buf(), source })?;
        let body = hcl::parse(&text).map_err(|error| DataError::Parse {
            path: path.to_path_buf(),
            entry: None,
            message: error.to_string(),
        })?;
        let format = Self::check_format(path, &body)?;
        let mut build = None;
        let mut entries = BTreeMap::new();
        let mut fingerprints = BTreeMap::new();
        let mut failed = BTreeMap::new();
        for block in body.blocks() {
            match block.identifier() {
                "build" => {
                    let reader = BlockReader::new(path, None, "build", block.body());
                    if build.is_some() {
                        return Err(reader.fail("duplicate `build` block".to_string()));
                    }
                    if !block.labels().is_empty() {
                        return Err(reader.fail("build block takes no label".to_string()));
                    }
                    reader.only(&["timestamp", "text_hash"])?;
                    let timestamp = Self::hex(&reader, "timestamp")?;
                    let text_hash = Self::hex(&reader, "text_hash")?;
                    let timestamp = u32::try_from(timestamp)
                        .map_err(|_| reader.fail("attribute `timestamp` must fit in 32 bits".to_string()))?;
                    build = Some(BuildId::new(timestamp, text_hash));
                }
                "resolved" => {
                    let [label] = block.labels() else {
                        let reader = BlockReader::new(path, None, "resolved", block.body());
                        return Err(reader.fail("resolved block needs exactly one id label".to_string()));
                    };
                    let id = label.as_str();
                    let reader = BlockReader::new(path, Some(id), "resolved", block.body());
                    if !EntryId::is_valid_text(id) {
                        return Err(reader.fail(format!("`{id}` is not a valid id")));
                    }
                    if format >= 2 {
                        reader.only(&["kind", "value", "recipes", "fingerprint"])?;
                    } else {
                        reader.only(&["kind", "value", "recipes"])?;
                    }
                    let key = EntryId(id.to_string());
                    if entries.contains_key(&key) || failed.contains_key(&key) {
                        return Err(reader.fail(format!("duplicate resolved id `{id}`")));
                    }
                    let kind_name = reader.string("kind")?;
                    let kind = EntryKind::from_name(&kind_name)
                        .ok_or_else(|| reader.fail(format!("unknown kind `{kind_name}` in attribute `kind`")))?;
                    let resolved = Resolved {
                        kind,
                        value: Self::hex(&reader, "value")?,
                        recipes_agreeing: reader.u32("recipes")?,
                    };
                    if let Some(fingerprint) = Self::fingerprint(&reader)? {
                        fingerprints.insert(key.clone(), fingerprint);
                    }
                    entries.insert(key, resolved);
                }
                "failed" if format >= 2 => {
                    let [label] = block.labels() else {
                        let reader = BlockReader::new(path, None, "failed", block.body());
                        return Err(reader.fail("failed block needs exactly one id label".to_string()));
                    };
                    let id = label.as_str();
                    let reader = BlockReader::new(path, Some(id), "failed", block.body());
                    if !EntryId::is_valid_text(id) {
                        return Err(reader.fail(format!("`{id}` is not a valid id")));
                    }
                    reader.only(&["fingerprint", "reason"])?;
                    let key = EntryId(id.to_string());
                    if entries.contains_key(&key) || failed.contains_key(&key) {
                        return Err(reader.fail(format!("duplicate id `{id}`")));
                    }
                    let reason = reader.string("reason")?;
                    if let Some(fingerprint) = Self::fingerprint(&reader)? {
                        fingerprints.insert(key.clone(), fingerprint);
                    }
                    failed.insert(key, reason);
                }
                other => {
                    let reader = BlockReader::new(path, None, "build data", block.body());
                    return Err(reader.fail(format!("unknown block `{other}`")));
                }
            }
        }
        let build = build.ok_or_else(|| DataError::Parse {
            path: path.to_path_buf(),
            entry: None,
            message: "missing `build` block".to_string(),
        })?;
        Ok(BuildData { build, entries, fingerprints, failed })
    }

    // the file has to say which format it is, and this build has to know it
    fn check_format(path: &Path, body: &hcl::Body) -> Result<u32, DataError> {
        let reader = BlockReader::new(path, None, "build data", body);
        reader.only(&["format"])?;
        let format = reader
            .optional_u32("format")?
            .ok_or_else(|| reader.fail(format!("missing `format` attribute; expected `format = {}`", Self::FORMAT)))?;
        if !(Self::OLDEST_FORMAT..=Self::FORMAT).contains(&format) {
            return Err(reader.fail(format!(
                "unsupported data format {format}; this version reads format {} and format {}",
                Self::OLDEST_FORMAT,
                Self::FORMAT
            )));
        }
        Ok(format)
    }

    fn fingerprint(reader: &BlockReader<'_>) -> Result<Option<Fingerprint>, DataError> {
        match reader.optional_string("fingerprint")? {
            None => Ok(None),
            Some(text) => Fingerprint::parse(&text)
                .map(Some)
                .ok_or_else(|| reader.fail("attribute `fingerprint` must be 16 hex digits".to_string())),
        }
    }

    fn hex(reader: &BlockReader<'_>, name: &str) -> Result<u64, DataError> {
        let text = reader.string(name)?;
        text.strip_prefix("0x")
            .and_then(|digits| u64::from_str_radix(digits, 16).ok())
            .ok_or_else(|| reader.fail(format!("attribute `{name}` must be a 0x hex string")))
    }
}
