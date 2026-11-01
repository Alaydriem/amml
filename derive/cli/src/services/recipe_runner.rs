use std::collections::BTreeMap;

use amml_data::{EntryId, EntryKind, OperandKind, Recipe, Resolved, Step};

use crate::errors::StepError;
use crate::services::{Disasm, PatternScanner, RttiIndex, StringXref};
use crate::structs::{Cursor, Image, Pattern, StepTrace};

/// Where one `pattern` / `pattern_any` step matched, before its `offset`. A `pattern_any` lists every alternative
/// that hit this position.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PatternHit {
    pub step: usize,
    pub rva: u32,
    // indices into `patterns` (always `[0]` for `pattern`)
    pub alternatives: Vec<usize>,
}

pub struct RecipeRunner<'a> {
    image: &'a Image,
    rtti: &'a RttiIndex,
    xrefs: &'a StringXref,
}

impl<'a> RecipeRunner<'a> {
    // slots `slot_of` reads before it gives up
    const SLOT_SCAN_LIMIT: u32 = 1024;

    pub fn new(image: &'a Image, rtti: &'a RttiIndex, xrefs: &'a StringXref) -> Self {
        Self { image, rtti, xrefs }
    }

    /// Runs the steps in order; the last cursor is the result and its type must fit `kind`.
    pub fn run(
        &self,
        recipe: &Recipe,
        kind: EntryKind,
        resolved: &BTreeMap<EntryId, Resolved>,
    ) -> Result<u64, StepError> {
        self.run_traced(recipe, kind, resolved, &mut Vec::new())
    }

    /// Like `run`, and appends one `PatternHit` to `hits` for every pattern step that found its single match,
    /// including the hits of a recipe that fails later. The result is the same as `run`.
    pub fn run_traced(
        &self,
        recipe: &Recipe,
        kind: EntryKind,
        resolved: &BTreeMap<EntryId, Resolved>,
        hits: &mut Vec<PatternHit>,
    ) -> Result<u64, StepError> {
        self.execute(recipe, kind, resolved, hits, None)
    }

    /// Like `run`, and appends one `StepTrace` to `steps` for every step it reaches, the failing one included.
    /// A pattern step records how many matches it found in its search range, also when it fails.
    pub fn run_explained(
        &self,
        recipe: &Recipe,
        kind: EntryKind,
        resolved: &BTreeMap<EntryId, Resolved>,
        steps: &mut Vec<StepTrace>,
    ) -> Result<u64, StepError> {
        self.execute(recipe, kind, resolved, &mut Vec::new(), Some(steps))
    }

    fn execute(
        &self,
        recipe: &Recipe,
        kind: EntryKind,
        resolved: &BTreeMap<EntryId, Resolved>,
        hits: &mut Vec<PatternHit>,
        mut steps: Option<&mut Vec<StepTrace>>,
    ) -> Result<u64, StepError> {
        let mut cursor: Option<Cursor> = None;
        for (index, step) in recipe.steps.iter().enumerate() {
            let mut matches = None;
            let outcome = self.step(index, step, cursor, resolved, hits, &mut matches);
            if let Some(steps) = steps.as_deref_mut() {
                steps.push(StepTrace {
                    index,
                    step: StepTrace::describe(step),
                    cursor: outcome.as_ref().ok().map(Cursor::value),
                    matches,
                    error: outcome.as_ref().err().map(ToString::to_string),
                });
            }
            cursor = Some(outcome?);
        }
        let last = Self::need(cursor)?;
        if !last.fits(kind) {
            return Err(StepError::WrongResult { kind, got: last.label().to_string() });
        }
        Ok(last.value())
    }

    fn step(
        &self,
        index: usize,
        step: &Step,
        cursor: Option<Cursor>,
        resolved: &BTreeMap<EntryId, Resolved>,
        hits: &mut Vec<PatternHit>,
        matches: &mut Option<usize>,
    ) -> Result<Cursor, StepError> {
        match step {
            Step::Pattern { bytes, offset, .. } => {
                self.single_match(index, std::slice::from_ref(bytes), cursor, *offset, hits, matches)
            }
            Step::PatternAny { patterns, offset, .. } => {
                self.single_match(index, patterns, cursor, *offset, hits, matches)
            }
            Step::FunctionStartOf => {
                let at = Self::need(cursor)?.rva()?;
                let (primary, _) = self.image.functions().containing(at).ok_or(StepError::NotInFunction(at))?;
                Ok(Cursor::Rva(primary))
            }
            Step::StringXref { text, nth } => {
                let functions = self.xrefs.functions_using(self.image, text)?;
                match nth {
                    None => match functions.as_slice() {
                        [only] => Ok(Cursor::Rva(*only)),
                        [] => Err(StepError::NoMatch),
                        many => Err(StepError::ManyMatches(many.len())),
                    },
                    Some(nth) => {
                        functions.get(*nth as usize).map(|rva| Cursor::Rva(*rva)).ok_or(StepError::XrefIndex {
                            nth: *nth,
                            found: u32::try_from(functions.len()).unwrap_or(u32::MAX),
                        })
                    }
                }
            }
            Step::RttiVtable { class, slot } => match slot {
                Some(slot) => self.rtti.slot_target(self.image, class, *slot).map(Cursor::Rva),
                None => self.rtti.vtable(class).map(Cursor::Rva),
            },
            Step::Ref { entry } => match resolved.get(entry) {
                Some(r) => Cursor::of_kind(r.kind, r.value),
                None => Err(StepError::DependencyFailed(entry.clone())),
            },
            Step::Call { nth } => Disasm::nth_call(self.image, Self::need(cursor)?.rva()?, *nth).map(Cursor::Rva),
            Step::Operand { index, kind } => {
                let value = Disasm::operand(self.image, Self::need(cursor)?.rva()?, *index, *kind)?;
                match kind {
                    OperandKind::Displacement | OperandKind::Immediate | OperandKind::ScaleFactor => {
                        Ok(Cursor::Number(value))
                    }
                    OperandKind::RipTarget | OperandKind::BranchTarget => Cursor::rva_of(value).map(Cursor::Rva),
                }
            }
            Step::SlotIn { class } => {
                self.rtti.slot_of(self.image, class, Self::need(cursor)?.rva()?).map(Cursor::Slot)
            }
            Step::VtableSlot { slot } => self.vtable_slot(Self::need(cursor)?.rva()?, *slot).map(Cursor::Rva),
            Step::ReadPointer => self.read_pointer(Self::need(cursor)?.rva()?).map(Cursor::Rva),
            Step::SlotOf { vtable } => {
                let function = Self::need(cursor)?.rva()?;
                let table = match resolved.get(vtable) {
                    Some(r) => Cursor::rva_of(r.value)?,
                    None => return Err(StepError::DependencyFailed(vtable.clone())),
                };
                self.slot_of(table, function, vtable).map(Cursor::Slot)
            }
            Step::DisplacementToSlot => match Self::need(cursor)? {
                Cursor::Number(n) if n % 8 == 0 => Cursor::slot_of(n / 8).map(Cursor::Slot),
                Cursor::Number(n) => Err(StepError::NotSlotAligned(n)),
                other => Err(StepError::NotAnRva(other.value())),
            },
            Step::Add { value } => Self::need(cursor)?.shifted(*value),
            Step::MemberDisplacement { index, local_at } => {
                let rva = Self::need(cursor)?.rva()?;
                Disasm::member_displacement(self.image, rva, *index, *local_at).map(Cursor::Number)
            }
            Step::Alternative => {
                let hit = hits.last().filter(|hit| hit.step + 1 == index).ok_or(StepError::NoAlternative)?;
                match hit.alternatives.as_slice() {
                    [only] => Ok(Cursor::Number(*only as u64)),
                    many => Err(StepError::AmbiguousAlternative(many.to_vec())),
                }
            }
        }
    }

    // the address of slot `slot`, after checking slots `0..=slot` all point into executable code
    fn vtable_slot(&self, vtable: u32, slot: u32) -> Result<u32, StepError> {
        let mut last = vtable;
        for index in 0..=slot {
            let overflow = StepError::Overflow { value: u64::from(vtable), delta: i64::from(index) * 8 };
            let rva = index.checked_mul(8).and_then(|d| vtable.checked_add(d)).ok_or(overflow)?;
            self.read_pointer(rva)?;
            last = rva;
        }
        Ok(last)
    }

    // the first slot of the vtable at `vtable` that holds `function`; every slot read must point into code
    fn slot_of(&self, vtable: u32, function: u32, name: &EntryId) -> Result<u32, StepError> {
        for index in 0..Self::SLOT_SCAN_LIMIT {
            let overflow = StepError::Overflow { value: u64::from(vtable), delta: i64::from(index) * 8 };
            let rva = index.checked_mul(8).and_then(|d| vtable.checked_add(d)).ok_or(overflow)?;
            if self.read_pointer(rva)? == function {
                return Ok(index);
            }
        }
        Err(StepError::NotInVtable { class: name.0.clone(), rva: function })
    }

    // the RVA the qword at `rva` points to, which must lie in an executable section
    fn read_pointer(&self, rva: u32) -> Result<u32, StepError> {
        self.image.code_pointer(rva).ok_or(StepError::NotCode { rva })
    }

    // the one distinct position where any of `patterns` matches, plus `offset`; without a cursor the scan covers
    // every executable section, with one it covers the function holding the cursor
    fn single_match(
        &self,
        step: usize,
        patterns: &[String],
        cursor: Option<Cursor>,
        offset: i64,
        hits: &mut Vec<PatternHit>,
        matches: &mut Option<usize>,
    ) -> Result<Cursor, StepError> {
        let mut positions = Vec::new();
        let mut by_alternative = Vec::new();
        for text in patterns {
            let pattern = Pattern::parse(text).map_err(|e| StepError::BadPattern(e.to_string()))?;
            let found = match cursor {
                None => PatternScanner::find_all(self.image, &pattern),
                Some(at) => self.matches_in_function(&pattern, at.rva()?)?,
            };
            positions.extend(found.iter().copied());
            by_alternative.push(found);
        }
        // two alternatives that hit the same position count once
        positions.sort_unstable();
        positions.dedup();
        *matches = Some(positions.len());
        let [only] = positions.as_slice() else {
            return Err(match positions.len() {
                0 => StepError::NoMatch,
                n => StepError::ManyMatches(n),
            });
        };
        let alternatives = (0..by_alternative.len()).filter(|i| by_alternative[*i].contains(only)).collect();
        hits.push(PatternHit { step, rva: *only, alternatives });
        Cursor::Rva(*only).shifted(offset)
    }

    // matches across every range of the function holding `rva`
    fn matches_in_function(&self, pattern: &Pattern, rva: u32) -> Result<Vec<u32>, StepError> {
        let (primary, _) = self.image.functions().containing(rva).ok_or(StepError::NotInFunction(rva))?;
        let mut matches = Vec::new();
        for range in self.image.functions().ranges_of(primary) {
            matches.extend(PatternScanner::find_in(self.image, pattern, range));
        }
        Ok(matches)
    }

    // a validated recipe starts with a source step, so a missing cursor means a malformed recipe
    fn need(cursor: Option<Cursor>) -> Result<Cursor, StepError> {
        cursor.ok_or(StepError::NoMatch)
    }
}
