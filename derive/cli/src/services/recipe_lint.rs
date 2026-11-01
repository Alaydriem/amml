use std::collections::BTreeSet;
use std::fmt::Write;

use amml_data::{Check, DataError, EntryKind, LiteralOk, PatternText, RecipeSet, Step};

use crate::services::{Deriver, FixedFields, LayoutGuard, PatternHit};
use crate::structs::{EntryLint, FixedByte, Image, LintFinding, LintReport};

pub struct RecipeLint<'a> {
    image: &'a Image,
    set: &'a RecipeSet,
    fixed: FixedFields<'a>,
}

impl<'a> RecipeLint<'a> {
    /// A `range` check on a `vslot` entry must allow at least this many slots.
    pub const MIN_RANGE_SLOTS: u64 = 8;

    pub fn new(image: &'a Image, set: &'a RecipeSet) -> Self {
        Self { image, set, fixed: FixedFields::new(image) }
    }

    /// Derives once to resolve `ref` steps, then runs every recipe the way `derive` does and decodes the
    /// instructions under every pattern match. Fails only on a dependency cycle.
    pub fn lint(&self) -> Result<LintReport, DataError> {
        let deriver = Deriver::new(self.image, self.set);
        let outcome = deriver.derive()?;
        let runner = deriver.runner();
        // every derived function value is a proven code start
        let code_starts: BTreeSet<u64> = outcome
            .data
            .entries
            .values()
            .filter(|resolved| resolved.kind == EntryKind::Function)
            .map(|resolved| resolved.value)
            .collect();
        let mut entries = Vec::new();
        for entry in self.set.entries() {
            // an entry another layout owns is not linted: its recipes are written for a build that has that layout
            if let Some(refusal) =
                LayoutGuard::refusal(entry, |id| outcome.data.entries.get(id).map(|resolved| resolved.value))
            {
                entries.push(EntryLint {
                    id: entry.id.clone(),
                    kind: entry.kind,
                    findings: Vec::new(),
                    info: Vec::new(),
                    not_for_build: Some(refusal.to_string()),
                });
                continue;
            }
            let mut findings = Vec::new();
            let mut info = Vec::new();
            for (recipe_index, recipe) in entry.recipes.iter().enumerate() {
                let mut hits = Vec::new();
                let ran = match runner.run_traced(recipe, entry.kind, &outcome.data.entries, &mut hits) {
                    Ok(_) => true,
                    Err(error) => {
                        findings.push(LintFinding::DidNotRun { recipe: recipe_index, error: error.to_string() });
                        false
                    }
                };
                let derived = outcome.data.entries.get(&entry.id).map(|resolved| resolved.value);
                let first = findings.len();
                for hit in &hits {
                    let code_start = Self::starts_code(entry.kind, &recipe.steps, hit, ran, derived)
                        || code_starts.contains(&u64::from(hit.rva));
                    findings.extend(self.lint_hit(recipe_index, &recipe.steps, hit, code_start));
                    info.extend(Self::unlinted_alternatives(recipe_index, &recipe.steps, hit));
                }
                info.extend(Self::unused_literal_ok(recipe_index, &recipe.steps, &findings[first..]));
                if entry.kind == EntryKind::Function {
                    for (step, def) in recipe.steps.iter().enumerate() {
                        if let Step::VtableSlot { slot } = def {
                            findings.push(LintFinding::LiteralSlot { recipe: recipe_index, step, slot: *slot });
                        }
                    }
                }
            }
            if entry.kind == EntryKind::VSlot {
                for check in &entry.checks {
                    if let Check::Range { min, max } = check
                        && Self::range_slots(*min, *max) < Self::MIN_RANGE_SLOTS
                    {
                        findings.push(LintFinding::NarrowRange { min: *min, max: *max });
                    }
                }
            }
            entries.push(EntryLint { id: entry.id.clone(), kind: entry.kind, findings, info, not_for_build: None });
        }
        Ok(LintReport { build: self.image.build(), entries })
    }

    // True when the match start is proven to be the first byte of code: the step has no offset and either the
    // entry's derived value (the result of all its recipes and checks) is the match start, or the next step
    // (`slot_of`, in a recipe that succeeded) found the match start in a vtable. Only then may a match outside
    // every `.pdata` range be decoded from its start. `lint` also accepts a match that starts at the derived value of
    // any function entry (a field read from the first bytes of a leaf).
    fn starts_code(kind: EntryKind, steps: &[Step], hit: &PatternHit, ran: bool, derived: Option<u64>) -> bool {
        let no_offset =
            matches!(steps.get(hit.step), Some(Step::Pattern { offset: 0, .. } | Step::PatternAny { offset: 0, .. }));
        let is_value = kind == EntryKind::Function && derived == Some(u64::from(hit.rva));
        let in_vtable = ran && matches!(steps.get(hit.step + 1), Some(Step::SlotOf { .. }));
        no_offset && (is_value || in_vtable)
    }

    // the findings of one step's single match, for every alternative that hit it
    fn lint_hit(&self, recipe: usize, steps: &[Step], hit: &PatternHit, code_start: bool) -> Vec<LintFinding> {
        let (texts, literal_ok, any): (Vec<&String>, &[LiteralOk], bool) = match steps.get(hit.step) {
            Some(Step::Pattern { bytes, literal_ok, .. }) => (vec![bytes], literal_ok, false),
            Some(Step::PatternAny { patterns, literal_ok, .. }) => (patterns.iter().collect(), literal_ok, true),
            _ => return Vec::new(),
        };
        let mut found = Vec::new();
        for alternative in &hit.alternatives {
            let Some(text) = texts.get(*alternative) else { continue };
            let Ok(bytes) = PatternText::parse(text) else { continue };
            let decoded = match self.fixed.cover(hit.rva, bytes.len(), code_start) {
                Ok(decoded) => decoded,
                Err(why) => {
                    found.push(LintFinding::NotDecodable { recipe, step: hit.step, rva: hit.rva, why });
                    continue;
                }
            };
            for instruction in &decoded {
                for (field, offset, size) in &instruction.fields {
                    for at in 0..*size {
                        let absolute = instruction.ip + (*offset + at) as u32;
                        let Some(byte) = absolute.checked_sub(hit.rva).map(|index| index as usize) else { continue };
                        let Some(Some(value)) = bytes.get(byte) else { continue };
                        found.push(LintFinding::Byte(FixedByte {
                            recipe,
                            step: hit.step,
                            alternative: any.then_some(*alternative),
                            byte,
                            value: *value,
                            rva: instruction.ip,
                            field: *field,
                            instruction: instruction.text.clone(),
                            kept: literal_ok
                                .iter()
                                .find(|item| item.index == byte && item.alternative == any.then_some(*alternative))
                                .map(|item| item.reason.clone()),
                        }));
                    }
                }
            }
        }
        found
    }

    // a `pattern_any` alternative that did not hit is never decoded
    fn unlinted_alternatives(recipe: usize, steps: &[Step], hit: &PatternHit) -> Vec<String> {
        let Some(Step::PatternAny { patterns, .. }) = steps.get(hit.step) else { return Vec::new() };
        (0..patterns.len())
            .filter(|alternative| !hit.alternatives.contains(alternative))
            .map(|alternative| {
                format!(
                    "alternative {alternative} not linted (no match on this build): recipe {recipe}, step {}",
                    hit.step
                )
            })
            .collect()
    }

    // `literal_ok` entries that kept no byte of this recipe: the pattern changed, or the step did not match
    fn unused_literal_ok(recipe: usize, steps: &[Step], findings: &[LintFinding]) -> Vec<String> {
        let mut unused = Vec::new();
        for (step, def) in steps.iter().enumerate() {
            let (Step::Pattern { literal_ok, .. } | Step::PatternAny { literal_ok, .. }) = def else { continue };
            for item in literal_ok {
                let used = findings.iter().any(|finding| {
                    matches!(finding, LintFinding::Byte(b) if b.kept.is_some() && b.step == step
                        && b.alternative == item.alternative && b.byte == item.index)
                });
                if !used {
                    let place = item.alternative.map(|a| format!("{a}.")).unwrap_or_default();
                    unused.push(format!(
                        "unused literal_ok: recipe {recipe}, step {step}, `{place}{}: {}` is in no immediate, displacement or branch field",
                        item.index, item.reason
                    ));
                }
            }
        }
        unused
    }

    fn range_slots(min: u64, max: u64) -> u64 {
        max.saturating_sub(min).saturating_add(1)
    }

    /// The Markdown report, grouped by entry; entries without findings are only counted.
    pub fn render(report: &LintReport) -> String {
        let mut out = String::new();
        let with_findings: Vec<_> =
            report.entries.iter().filter(|entry| !entry.findings.is_empty() || !entry.info.is_empty()).collect();
        let _ = writeln!(out, "# Recipe lint: build {}\n", report.build.file_stem());
        let _ = writeln!(
            out,
            "{} entries, {} with findings or kept bytes, {} open findings, {} kept bytes, {} not for this build.\n",
            report.entries.len(),
            with_findings.len(),
            report.open_findings(),
            report.kept_bytes(),
            report.skipped()
        );
        let _ = writeln!(
            out,
            "The decode runs linearly from the start of the function range that holds the match; data inside a range \
             could desync it. Nothing independent checks the boundaries, so a wrong decode would not be reported.\n"
        );
        for entry in report.entries.iter().filter(|entry| entry.not_for_build.is_some()) {
            let why = entry.not_for_build.as_deref().unwrap_or_default();
            let _ = writeln!(out, "- not for this build: {} ({why})", entry.id);
        }
        out.push('\n');
        for entry in with_findings {
            let _ = writeln!(out, "## {} ({})\n", entry.id, entry.kind.name());
            for finding in &entry.findings {
                let _ = writeln!(out, "- {}", Self::line(finding));
            }
            for note in &entry.info {
                let _ = writeln!(out, "- info: {note}");
            }
            out.push('\n');
        }
        out
    }

    fn line(finding: &LintFinding) -> String {
        match finding {
            LintFinding::Byte(b) => {
                let alternative = b.alternative.map(|a| format!(", alternative {a}")).unwrap_or_default();
                let place = format!(
                    "recipe {}, step {}{alternative}, byte {} (`{:02X}`): {} in `{}` at {:#x}",
                    b.recipe,
                    b.step,
                    b.byte,
                    b.value,
                    b.field.label(),
                    b.instruction,
                    b.rva
                );
                match &b.kept {
                    Some(reason) => format!("kept: {reason} ({place})"),
                    None => format!("fixed byte: {place}"),
                }
            }
            LintFinding::NotDecodable { recipe, step, rva, why } => {
                format!("not decodable: recipe {recipe}, step {step}, match at {rva:#x}: {why}")
            }
            LintFinding::DidNotRun { recipe, error } => format!("did not run: recipe {recipe}: {error}"),
            LintFinding::LiteralSlot { recipe, step, slot } => {
                format!("literal vtable slot: recipe {recipe}, step {step}, `vtable_slot` {slot} on a function entry")
            }
            LintFinding::NarrowRange { min, max } => format!(
                "narrow range: check `range` {min}..={max} allows {} slots, under {}",
                Self::range_slots(*min, *max),
                Self::MIN_RANGE_SLOTS
            ),
        }
    }
}
