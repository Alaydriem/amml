use std::fs;
use std::path::Path;

use amml_data::{BuildId, DataError, EntryId};
use hcl::{Block, Body, Expression};

use crate::structs::{EntrySignature, Reg, ReturnKind, Signature, SignatureFile, SignatureStatus};

pub struct SignatureStore;

impl SignatureStore {
    // The only signature format this build writes and reads.
    const FORMAT: u64 = 1;

    /// The stable text form, written by hand so the bytes never depend on a formatter version.
    pub fn render(file: &SignatureFile) -> String {
        let mut text = format!("format = {}\n\nbuild {{\n", Self::FORMAT);
        text.push_str(&format!("  timestamp = \"{:#010x}\"\n", file.build.timestamp));
        text.push_str(&format!("  text_hash = \"{:#018x}\"\n", file.build.text_hash));
        text.push_str("}\n");
        for entry in &file.entries {
            let signature = &entry.signature;
            text.push_str(&format!("\nsignature \"{}\" {{\n", entry.id.0));
            if let Some(rva) = entry.rva {
                text.push_str(&format!("  rva        = \"{rva:#x}\"\n"));
            }
            match &signature.status {
                SignatureStatus::Known => {
                    text.push_str(&format!("  args_int   = [{}]\n", Self::regs(&signature.args_int)));
                    text.push_str(&format!("  args_xmm   = [{}]\n", Self::regs(&signature.args_xmm)));
                    let slots: Vec<String> = signature.stack_args.iter().map(|slot| format!("\"{slot:#x}\"")).collect();
                    text.push_str(&format!("  stack_args = [{}]\n", slots.join(", ")));
                    text.push_str(&format!("  returns    = \"{}\"\n", signature.returns.name()));
                    text.push_str("  status     = \"known\"\n");
                }
                SignatureStatus::Unknown(reason) => {
                    text.push_str("  status     = \"unknown\"\n");
                    text.push_str(&format!("  reason     = \"{}\"\n", Self::escape(reason)));
                }
            }
            text.push_str("}\n");
        }
        text
    }

    pub fn write(path: &Path, file: &SignatureFile) -> Result<(), std::io::Error> {
        if let Some(parent) = path.parent().filter(|parent| !parent.as_os_str().is_empty()) {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, Self::render(file))
    }

    pub fn read(path: &Path) -> Result<SignatureFile, DataError> {
        let text = fs::read_to_string(path).map_err(|source| DataError::Io { path: path.to_path_buf(), source })?;
        let fail = |entry: Option<&str>, message: String| DataError::Parse {
            path: path.to_path_buf(),
            entry: entry.map(str::to_string),
            message,
        };
        let body = hcl::parse(&text).map_err(|error| fail(None, error.to_string()))?;
        Self::only(&body, &["format"]).map_err(|message| fail(None, message))?;
        match Self::attribute(&body, "format") {
            Some(Expression::Number(number)) if number.as_u64() == Some(Self::FORMAT) => {}
            _ => return Err(fail(None, format!("expected `format = {}`", Self::FORMAT))),
        }
        let mut build = None;
        let mut entries = Vec::new();
        for block in body.blocks() {
            match (block.identifier(), block.labels()) {
                ("build", []) => {
                    if build.is_some() {
                        return Err(fail(None, "duplicate `build` block".to_string()));
                    }
                    Self::only(block.body(), &["timestamp", "text_hash"]).map_err(|message| fail(None, message))?;
                    let timestamp = Self::hex(block.body(), "timestamp").and_then(|value| u32::try_from(value).ok());
                    let text_hash = Self::hex(block.body(), "text_hash");
                    match (timestamp, text_hash) {
                        (Some(timestamp), Some(text_hash)) => build = Some(BuildId::new(timestamp, text_hash)),
                        _ => return Err(fail(None, "build needs hex `timestamp` and `text_hash`".to_string())),
                    }
                }
                ("signature", [label]) => {
                    let id = label.as_str();
                    if entries.iter().any(|entry: &EntrySignature| entry.id.0 == id) {
                        return Err(fail(Some(id), format!("duplicate signature `{id}`")));
                    }
                    let entry = Self::entry(id, block).map_err(|message| fail(Some(id), message))?;
                    entries.push(entry);
                }
                (other, _) => return Err(fail(None, format!("unexpected block `{other}`"))),
            }
        }
        let build = build.ok_or_else(|| fail(None, "missing `build` block".to_string()))?;
        Ok(SignatureFile::new(build, entries))
    }

    fn entry(id: &str, block: &Block) -> Result<EntrySignature, String> {
        let body = block.body();
        let rva = match Self::attribute(body, "rva") {
            None => None,
            Some(_) => Some(
                Self::hex(body, "rva").and_then(|value| u32::try_from(value).ok()).ok_or("`rva` is not a hex rva")?,
            ),
        };
        let status = Self::string(body, "status");
        let allowed: &[&str] = match status.as_deref() {
            Some("unknown") => &["rva", "status", "reason"],
            _ => &["rva", "args_int", "args_xmm", "stack_args", "returns", "status"],
        };
        Self::only(body, allowed)?;
        let signature = match status.as_deref() {
            Some("unknown") => Signature::unknown(Self::string(body, "reason").unwrap_or_default()),
            Some("known") => {
                let returns = Self::string(body, "returns")
                    .as_deref()
                    .and_then(ReturnKind::from_name)
                    .ok_or("`returns` must be int, float or none")?;
                let mut stack_args = Vec::new();
                for slot in Self::strings(body, "stack_args")? {
                    let value = slot
                        .strip_prefix("0x")
                        .and_then(|digits| u32::from_str_radix(digits, 16).ok())
                        .ok_or_else(|| format!("stack slot `{slot}` is not hex"))?;
                    stack_args.push(value);
                }
                Signature::new(
                    Self::parse_regs(body, "args_int")?,
                    Self::parse_regs(body, "args_xmm")?,
                    stack_args,
                    returns,
                )
            }
            _ => return Err("`status` must be known or unknown".to_string()),
        };
        Ok(EntrySignature::new(EntryId(id.to_string()), rva, signature))
    }

    // the first attribute whose name is not in `allowed`
    fn only(body: &Body, allowed: &[&str]) -> Result<(), String> {
        match body.attributes().find(|attribute| !allowed.contains(&attribute.key())) {
            Some(attribute) => Err(format!("unknown attribute `{}`", attribute.key())),
            None => Ok(()),
        }
    }

    fn regs(regs: &[Reg]) -> String {
        regs.iter().map(|reg| format!("\"{}:{}\"", reg.name(), reg.bits)).collect::<Vec<_>>().join(", ")
    }

    fn parse_regs(body: &Body, name: &str) -> Result<Vec<Reg>, String> {
        let mut regs = Vec::new();
        for item in Self::strings(body, name)? {
            let parsed = item
                .split_once(':')
                .and_then(|(register, bits)| Some(Reg::new(Reg::from_name(register)?, bits.parse().ok()?)));
            regs.push(parsed.ok_or_else(|| format!("`{item}` in `{name}` is not `<register>:<bits>`"))?);
        }
        Ok(regs)
    }

    fn attribute<'b>(body: &'b Body, name: &str) -> Option<&'b Expression> {
        body.attributes().find(|attribute| attribute.key() == name).map(|attribute| attribute.expr())
    }

    fn string(body: &Body, name: &str) -> Option<String> {
        match Self::attribute(body, name) {
            Some(Expression::String(text)) => Some(text.clone()),
            _ => None,
        }
    }

    fn strings(body: &Body, name: &str) -> Result<Vec<String>, String> {
        match Self::attribute(body, name) {
            Some(Expression::Array(items)) => items
                .iter()
                .map(|item| match item {
                    Expression::String(text) => Ok(text.clone()),
                    _ => Err(format!("`{name}` must be a list of strings")),
                })
                .collect(),
            _ => Err(format!("`{name}` must be a list of strings")),
        }
    }

    fn hex(body: &Body, name: &str) -> Option<u64> {
        Self::string(body, name)?.strip_prefix("0x").and_then(|digits| u64::from_str_radix(digits, 16).ok())
    }

    fn escape(text: &str) -> String {
        text.replace('\\', "\\\\").replace('"', "\\\"").replace("${", "$${").replace("%{", "%%{")
    }
}
