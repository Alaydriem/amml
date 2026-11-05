use std::fs;
use std::path::{Path, PathBuf};

use hcl::structure::Block;

use super::BlockReader;
use crate::errors::DataError;
use crate::structs::{Check, Entry, EntryId, EntryKind, LiteralOk, OperandKind, PatternText, Recipe, RecipeSet, Step};

pub struct RecipeLoader;

impl RecipeLoader {
    // Reads every `.hcl` file under `dir` in sorted path order and validates the whole set.
    pub fn load_dir(dir: &Path) -> Result<RecipeSet, DataError> {
        let mut files = Vec::new();
        Self::collect(dir, &mut files)?;
        files.sort();
        let mut entries = Vec::new();
        let mut sources = Vec::new();
        for path in &files {
            let text = fs::read_to_string(path).map_err(|source| DataError::Io { path: path.clone(), source })?;
            let body = hcl::parse(&text).map_err(|error| DataError::Parse {
                path: path.clone(),
                entry: None,
                message: error.to_string(),
            })?;
            for block in body.blocks() {
                if block.identifier() != "entry" {
                    return Err(DataError::Parse {
                        path: path.clone(),
                        entry: None,
                        message: format!("unknown top-level block `{}`", block.identifier()),
                    });
                }
                entries.push(Self::entry(path, block)?);
                sources.push(path.clone());
            }
        }
        RecipeSet::with_sources(entries, sources).map(|set| set.with_root(dir.to_path_buf()))
    }

    fn collect(dir: &Path, files: &mut Vec<PathBuf>) -> Result<(), DataError> {
        let io = |source| DataError::Io { path: dir.to_path_buf(), source };
        for item in fs::read_dir(dir).map_err(io)? {
            let path = item.map_err(io)?.path();
            if path.is_dir() {
                Self::collect(&path, files)?;
            } else if path.extension().is_some_and(|extension| extension.eq_ignore_ascii_case("hcl")) {
                files.push(path);
            }
        }
        Ok(())
    }

    fn entry(path: &Path, block: &Block) -> Result<Entry, DataError> {
        let id = match block.labels() {
            [label] => label.as_str().to_string(),
            _ => {
                return Err(DataError::Parse {
                    path: path.to_path_buf(),
                    entry: None,
                    message: "entry block needs exactly one label".to_string(),
                });
            }
        };
        let reader = BlockReader::new(path, Some(&id), "entry", block.body());
        reader.only(&["kind", "msvc_name", "notes", "hooked"])?;
        let kind_name = reader.string("kind")?;
        let kind = EntryKind::from_name(&kind_name)
            .ok_or_else(|| reader.fail(format!("unknown kind `{kind_name}` in attribute `kind`")))?;
        let hooked = reader.optional_bool("hooked")?.unwrap_or(false);
        if hooked && kind != EntryKind::Function {
            return Err(reader.fail(format!("attribute `hooked` is only valid on function entries, not {kind_name}")));
        }
        let mut recipes = Vec::new();
        let mut checks = Vec::new();
        for child in block.body().blocks() {
            match child.identifier() {
                "recipe" => recipes.push(Self::recipe(path, &id, child)?),
                "check" => checks.push(Self::check(path, &id, child)?),
                other => return Err(reader.fail(format!("unknown block `{other}`"))),
            }
        }
        Ok(Entry {
            id: EntryId(id.clone()),
            kind,
            msvc_name: reader.optional_string("msvc_name")?,
            recipes,
            checks,
            notes: reader.optional_string("notes")?.unwrap_or_default(),
            hooked,
        })
    }

    fn recipe(path: &Path, id: &str, block: &Block) -> Result<Recipe, DataError> {
        let reader = BlockReader::new(path, Some(id), "recipe", block.body());
        if !block.labels().is_empty() {
            return Err(reader.fail("recipe block takes no label".to_string()));
        }
        reader.only(&[])?;
        let mut steps = Vec::new();
        for child in block.body().blocks() {
            if child.identifier() != "step" {
                return Err(reader.fail(format!("unknown block `{}`", child.identifier())));
            }
            steps.push(Self::step(path, id, child)?);
        }
        let after_pattern_any = |index: usize| index > 0 && matches!(steps[index - 1], Step::PatternAny { .. });
        if let Some(index) = (0..steps.len()).find(|&i| matches!(steps[i], Step::Alternative) && !after_pattern_any(i))
        {
            return Err(reader.fail(format!("step {index} `alternative` must come right after a `pattern_any` step")));
        }
        Ok(Recipe { steps })
    }

    fn step(path: &Path, id: &str, block: &Block) -> Result<Step, DataError> {
        let [label] = block.labels() else {
            let reader = BlockReader::new(path, Some(id), "step", block.body());
            return Err(reader.fail("step block needs exactly one type label".to_string()));
        };
        let name = label.as_str();
        let reader = BlockReader::new(path, Some(id), &format!("step \"{name}\""), block.body());
        let allowed: &[&str] = match name {
            "pattern" => &["bytes", "offset", "literal_ok"],
            "pattern_any" => &["patterns", "offset", "literal_ok"],
            "function_start_of" => &[],
            "vtable_holding" => &[],
            "string_xref" => &["text", "nth"],
            "rtti_vtable" => &["class", "slot"],
            "ref" => &["entry"],
            "call" => &["nth"],
            "operand" => &["index", "kind"],
            "slot_in" => &["class"],
            "vtable_slot" => &["slot"],
            "displacement_to_slot" => &[],
            "read_pointer" => &[],
            "slot_of" => &["vtable"],
            "add" => &["value"],
            "member_displacement" => &["index", "local_at"],
            "alternative" => &[],
            other => return Err(reader.fail(format!("unknown step type `{other}`"))),
        };
        reader.only(allowed)?;
        match name {
            "pattern" => {
                let bytes = reader.string("bytes")?;
                PatternText::parse(&bytes).map_err(|error| reader.fail(error.to_string()))?;
                let literal_ok = Self::literal_ok(&reader, std::slice::from_ref(&bytes), false)?;
                Ok(Step::Pattern { bytes, offset: reader.optional_i64("offset")?.unwrap_or(0), literal_ok })
            }
            "pattern_any" => {
                let patterns = reader.string_list("patterns")?;
                if patterns.len() < 2 {
                    return Err(reader.fail("attribute `patterns` needs at least 2 alternatives".to_string()));
                }
                for pattern in &patterns {
                    PatternText::parse(pattern).map_err(|error| reader.fail(error.to_string()))?;
                }
                let literal_ok = Self::literal_ok(&reader, &patterns, true)?;
                Ok(Step::PatternAny { patterns, offset: reader.optional_i64("offset")?.unwrap_or(0), literal_ok })
            }
            "function_start_of" => Ok(Step::FunctionStartOf),
            "vtable_holding" => Ok(Step::VtableHolding),
            "string_xref" => {
                let text = reader.string("text")?;
                if text.is_empty() || text.contains(char::from(0)) {
                    return Err(reader.fail("attribute `text` must be non-empty and contain no NUL".to_string()));
                }
                Ok(Step::StringXref { text, nth: reader.optional_u32("nth")? })
            }
            "rtti_vtable" => {
                Ok(Step::RttiVtable { class: reader.string("class")?, slot: reader.optional_u32("slot")? })
            }
            "ref" => Ok(Step::Ref { entry: EntryId(reader.string("entry")?) }),
            "call" => Ok(Step::Call { nth: reader.u32("nth")? }),
            "operand" => {
                let kind_name = reader.string("kind")?;
                let kind = OperandKind::from_name(&kind_name)
                    .ok_or_else(|| reader.fail(format!("unknown operand kind `{kind_name}`")))?;
                Ok(Step::Operand { index: reader.u32("index")?, kind })
            }
            "slot_in" => Ok(Step::SlotIn { class: reader.string("class")? }),
            "vtable_slot" => Ok(Step::VtableSlot { slot: reader.u32("slot")? }),
            "displacement_to_slot" => Ok(Step::DisplacementToSlot),
            "read_pointer" => Ok(Step::ReadPointer),
            "slot_of" => Ok(Step::SlotOf { vtable: EntryId(reader.string("vtable")?) }),
            "add" => Ok(Step::Add { value: reader.i64("value")? }),
            "member_displacement" => {
                Ok(Step::MemberDisplacement { index: reader.u32("index")?, local_at: reader.i64("local_at")? })
            }
            "alternative" => Ok(Step::Alternative),
            other => Err(reader.fail(format!("unknown step type `{other}`"))),
        }
    }

    // The `literal_ok` list of a pattern step. Each entry must name (on `pattern_any`) an alternative, lie inside that
    // pattern, point at a fixed byte, and not repeat an earlier entry.
    fn literal_ok(reader: &BlockReader, patterns: &[String], any: bool) -> Result<Vec<LiteralOk>, DataError> {
        let mut parsed = Vec::new();
        for pattern in patterns {
            parsed.push(PatternText::parse(pattern).map_err(|error| reader.fail(error.to_string()))?);
        }
        let mut kept: Vec<LiteralOk> = Vec::new();
        for text in reader.string_list("literal_ok")? {
            let item = LiteralOk::parse(&text, any).map_err(|message| reader.fail(message))?;
            let alternative = item.alternative.unwrap_or(0);
            let Some(bytes) = parsed.get(alternative) else {
                return Err(reader.fail(format!("literal_ok names alternative {alternative}, which does not exist")));
            };
            match bytes.get(item.index) {
                None => {
                    return Err(reader.fail(format!("literal_ok index {} is past the end of the pattern", item.index)));
                }
                Some(None) => return Err(reader.fail(format!("literal_ok index {} points at a wildcard", item.index))),
                Some(Some(_)) => {}
            }
            if kept.iter().any(|other| other.alternative == item.alternative && other.index == item.index) {
                return Err(reader.fail(format!("literal_ok lists index {} twice", item.index)));
            }
            kept.push(item);
        }
        Ok(kept)
    }

    fn check(path: &Path, id: &str, block: &Block) -> Result<Check, DataError> {
        let [label] = block.labels() else {
            let reader = BlockReader::new(path, Some(id), "check", block.body());
            return Err(reader.fail("check block needs exactly one type label".to_string()));
        };
        let name = label.as_str();
        let reader = BlockReader::new(path, Some(id), &format!("check \"{name}\""), block.body());
        let allowed: &[&str] = match name {
            "function_start" | "leaf_start" => &[],
            "in_vtable_of" => &["class"],
            "in_vtable" => &["vtable", "min_slot", "max_slot"],
            "range" => &["min", "max"],
            "section" => &["name"],
            "entry_is" => &["entry", "value"],
            other => return Err(reader.fail(format!("unknown check type `{other}`"))),
        };
        reader.only(allowed)?;
        match name {
            "function_start" => Ok(Check::FunctionStart),
            "leaf_start" => Ok(Check::LeafStart),
            "in_vtable_of" => Ok(Check::InVtableOf { class: reader.string("class")? }),
            "in_vtable" => {
                let (min_slot, max_slot) = (reader.u32("min_slot")?, reader.u32("max_slot")?);
                if min_slot > max_slot {
                    return Err(reader.fail(format!("min_slot {min_slot} is above max_slot {max_slot}")));
                }
                Ok(Check::InVtable { vtable: EntryId(reader.string("vtable")?), min_slot, max_slot })
            }
            "section" => Ok(Check::Section { name: reader.string("name")? }),
            "entry_is" => Ok(Check::EntryIs { entry: EntryId(reader.string("entry")?), value: reader.u64("value")? }),
            "range" => {
                let (min, max) = (reader.u64("min")?, reader.u64("max")?);
                if min > max {
                    return Err(reader.fail(format!("min {min:#x} is above max {max:#x}")));
                }
                Ok(Check::Range { min, max })
            }
            other => Err(reader.fail(format!("unknown check type `{other}`"))),
        }
    }
}
