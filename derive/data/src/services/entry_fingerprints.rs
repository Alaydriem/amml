use std::collections::{BTreeMap, BTreeSet};

use xxhash_rust::xxh3::xxh3_64;

use super::DependencyOrder;
use crate::errors::DataError;
use crate::structs::{Check, EntryId, Fingerprint, LiteralOk, RecipeSet, Step};

/// Computes the fingerprint of every entry of a recipe set.
///
/// The fingerprint is `xxh3_64` over this byte string, in order:
/// `FORMAT` (u32 LE), the deriver version (u32 LE), the entry id, the kind name, the recipe count (u32 LE) and every
/// recipe (its step count, then every step), the check count and every check, then the fingerprints of the entries it
/// references (u64 LE), sorted by id, each once. Strings are a u32 LE byte length and the UTF-8 bytes. Numbers are
/// little-endian at their full width (`i64` as two's complement). An `Option` is a 0 byte, or a 1 byte and the value.
/// Every step and check starts with a fixed variant tag byte. `notes`, `msvc_name` and `hooked` are not part of it;
/// `literal_ok` is, as a plain copy of the recipe. Entries hash in dependency order, so a change reaches every dependent.
pub struct EntryFingerprints;

impl EntryFingerprints {
    /// The encoding version; raise it when the byte string above changes.
    pub const FORMAT: u32 = 1;

    pub fn of(set: &RecipeSet, deriver_version: u32) -> Result<BTreeMap<EntryId, Fingerprint>, DataError> {
        let mut prints: BTreeMap<EntryId, Fingerprint> = BTreeMap::new();
        for id in DependencyOrder::of(set)? {
            let Some(entry) = set.get(&id) else { continue };
            let mut out = Vec::new();
            out.extend_from_slice(&Self::FORMAT.to_le_bytes());
            out.extend_from_slice(&deriver_version.to_le_bytes());
            Self::text(&mut out, &entry.id.0);
            Self::text(&mut out, entry.kind.name());
            Self::count(&mut out, entry.recipes.len());
            for recipe in &entry.recipes {
                Self::count(&mut out, recipe.steps.len());
                for step in &recipe.steps {
                    Self::step(&mut out, step);
                }
            }
            Self::count(&mut out, entry.checks.len());
            for check in &entry.checks {
                Self::check(&mut out, check);
            }
            let referenced: BTreeSet<&EntryId> = entry.references().collect();
            for target in referenced {
                if let Some(print) = prints.get(target) {
                    out.extend_from_slice(&print.0.to_le_bytes());
                }
            }
            prints.insert(id, Fingerprint(xxh3_64(&out)));
        }
        Ok(prints)
    }

    fn count(out: &mut Vec<u8>, value: usize) {
        out.extend_from_slice(&(value as u32).to_le_bytes());
    }

    fn text(out: &mut Vec<u8>, value: &str) {
        Self::count(out, value.len());
        out.extend_from_slice(value.as_bytes());
    }

    fn optional_u32(out: &mut Vec<u8>, value: Option<u32>) {
        match value {
            None => out.push(0),
            Some(number) => {
                out.push(1);
                out.extend_from_slice(&number.to_le_bytes());
            }
        }
    }

    fn literal_ok(out: &mut Vec<u8>, list: &[LiteralOk]) {
        Self::count(out, list.len());
        for item in list {
            Self::optional_u32(out, item.alternative.map(|alternative| alternative as u32));
            out.extend_from_slice(&(item.index as u64).to_le_bytes());
            Self::text(out, &item.reason);
        }
    }

    fn step(out: &mut Vec<u8>, step: &Step) {
        match step {
            Step::Pattern { bytes, offset, literal_ok } => {
                out.push(0);
                Self::text(out, bytes);
                out.extend_from_slice(&offset.to_le_bytes());
                Self::literal_ok(out, literal_ok);
            }
            Step::PatternAny { patterns, offset, literal_ok } => {
                out.push(1);
                Self::count(out, patterns.len());
                for pattern in patterns {
                    Self::text(out, pattern);
                }
                out.extend_from_slice(&offset.to_le_bytes());
                Self::literal_ok(out, literal_ok);
            }
            Step::FunctionStartOf => out.push(2),
            Step::VtableHolding => out.push(3),
            Step::StringXref { text, nth } => {
                out.push(4);
                Self::text(out, text);
                Self::optional_u32(out, *nth);
            }
            Step::RttiVtable { class, slot } => {
                out.push(5);
                Self::text(out, class);
                Self::optional_u32(out, *slot);
            }
            Step::Ref { entry } => {
                out.push(6);
                Self::text(out, &entry.0);
            }
            Step::Call { nth } => {
                out.push(7);
                out.extend_from_slice(&nth.to_le_bytes());
            }
            Step::Operand { index, kind } => {
                out.push(8);
                out.extend_from_slice(&index.to_le_bytes());
                Self::text(out, kind.name());
            }
            Step::SlotIn { class } => {
                out.push(9);
                Self::text(out, class);
            }
            Step::VtableSlot { slot } => {
                out.push(10);
                out.extend_from_slice(&slot.to_le_bytes());
            }
            Step::ReadPointer => out.push(11),
            Step::SlotOf { vtable } => {
                out.push(12);
                Self::text(out, &vtable.0);
            }
            Step::DisplacementToSlot => out.push(13),
            Step::Add { value } => {
                out.push(14);
                out.extend_from_slice(&value.to_le_bytes());
            }
            Step::MemberDisplacement { index, local_at } => {
                out.push(15);
                out.extend_from_slice(&index.to_le_bytes());
                out.extend_from_slice(&local_at.to_le_bytes());
            }
            Step::Alternative => out.push(16),
        }
    }

    fn check(out: &mut Vec<u8>, check: &Check) {
        match check {
            Check::FunctionStart => out.push(0),
            Check::LeafStart => out.push(1),
            Check::InVtableOf { class } => {
                out.push(2);
                Self::text(out, class);
            }
            Check::InVtable { vtable, min_slot, max_slot } => {
                out.push(3);
                Self::text(out, &vtable.0);
                out.extend_from_slice(&min_slot.to_le_bytes());
                out.extend_from_slice(&max_slot.to_le_bytes());
            }
            Check::Section { name } => {
                out.push(4);
                Self::text(out, name);
            }
            Check::Range { min, max } => {
                out.push(5);
                out.extend_from_slice(&min.to_le_bytes());
                out.extend_from_slice(&max.to_le_bytes());
            }
            Check::EntryIs { entry, value } => {
                out.push(6);
                Self::text(out, &entry.0);
                out.extend_from_slice(&value.to_le_bytes());
            }
        }
    }
}
