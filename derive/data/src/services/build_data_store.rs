use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};

use super::BlockReader;
use crate::errors::DataError;
use crate::structs::{BuildData, BuildId, EntryId, EntryKind, Resolved};

pub struct BuildDataStore;

impl BuildDataStore {
    // The only data format this build writes and reads.
    const FORMAT: u32 = 1;

    // Writes the stable text format by hand so the on-disk bytes never depend on a formatter version.
    // The text goes to a temporary file in the same folder and is renamed over `path`, so a reader
    // or a crash never sees a partial file.
    pub fn write(path: &Path, data: &BuildData) -> Result<(), DataError> {
        let mut text = String::new();
        text.push_str(&format!("format = {}\n\n", Self::FORMAT));
        text.push_str("build {\n");
        text.push_str(&format!("  timestamp = \"{:#010x}\"\n", data.build.timestamp));
        text.push_str(&format!("  text_hash = \"{:#018x}\"\n", data.build.text_hash));
        text.push_str("}\n");
        for (id, resolved) in &data.entries {
            if !EntryId::is_valid_text(&id.0) {
                return Err(DataError::Invalid {
                    entry: id.clone(),
                    message: "ids may only use A-Z a-z 0-9 _ . : - and cannot be empty".to_string(),
                    path: Some(path.to_path_buf()),
                });
            }
            text.push('\n');
            text.push_str(&format!("resolved \"{}\" {{\n", id.0));
            text.push_str(&format!("  kind    = \"{}\"\n", resolved.kind.name()));
            text.push_str(&format!("  value   = \"{:#x}\"\n", resolved.value));
            text.push_str(&format!("  recipes = {}\n", resolved.recipes_agreeing));
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
        Self::check_format(path, &body)?;
        let mut build = None;
        let mut entries = std::collections::BTreeMap::new();
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
                    reader.only(&["kind", "value", "recipes"])?;
                    if entries.contains_key(&EntryId(id.to_string())) {
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
                    entries.insert(EntryId(id.to_string()), resolved);
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
        Ok(BuildData { build, entries })
    }

    // the file has to say which format it is, and this build has to know it
    fn check_format(path: &Path, body: &hcl::Body) -> Result<(), DataError> {
        let reader = BlockReader::new(path, None, "build data", body);
        reader.only(&["format"])?;
        let format = reader
            .optional_u32("format")?
            .ok_or_else(|| reader.fail(format!("missing `format` attribute; expected `format = {}`", Self::FORMAT)))?;
        if format != Self::FORMAT {
            return Err(
                reader.fail(format!("unsupported data format {format}; this version reads format {}", Self::FORMAT))
            );
        }
        Ok(())
    }

    fn hex(reader: &BlockReader<'_>, name: &str) -> Result<u64, DataError> {
        let text = reader.string(name)?;
        text.strip_prefix("0x")
            .and_then(|digits| u64::from_str_radix(digits, 16).ok())
            .ok_or_else(|| reader.fail(format!("attribute `{name}` must be a 0x hex string")))
    }
}
