use amml_data::Step;

/// What one step of a recipe did: the value it produced, how often a pattern step matched, and why it failed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StepTrace {
    pub index: usize,
    pub step: String,
    pub cursor: Option<u64>,
    pub matches: Option<usize>,
    pub error: Option<String>,
}

impl StepTrace {
    /// The step as one line of text: its name and parameters.
    pub fn describe(step: &Step) -> String {
        let shift = |offset: &i64| if *offset == 0 { String::new() } else { format!(" offset {offset}") };
        match step {
            Step::Pattern { bytes, offset, .. } => format!("pattern \"{bytes}\"{}", shift(offset)),
            Step::PatternAny { patterns, offset, .. } => {
                let quoted: Vec<String> = patterns.iter().map(|text| format!("\"{text}\"")).collect();
                format!("pattern_any {}{}", quoted.join(" | "), shift(offset))
            }
            Step::StringXref { text, nth } => match nth {
                Some(nth) => format!("string_xref \"{text}\" nth {nth}"),
                None => format!("string_xref \"{text}\""),
            },
            Step::RttiVtable { class, slot } => match slot {
                Some(slot) => format!("rtti_vtable {class} slot {slot}"),
                None => format!("rtti_vtable {class}"),
            },
            Step::Ref { entry } => format!("ref {entry}"),
            Step::Call { nth } => format!("call {nth}"),
            Step::Operand { index, kind } => format!("operand {index} {}", kind.name()),
            Step::SlotIn { class } => format!("slot_in {class}"),
            Step::VtableSlot { slot } => format!("vtable_slot {slot}"),
            Step::SlotOf { vtable } => format!("slot_of {vtable}"),
            Step::Add { value } => format!("add {value}"),
            Step::MemberDisplacement { index, local_at } => format!("member_displacement {index} local_at {local_at}"),
            Step::FunctionStartOf
            | Step::VtableHolding
            | Step::ReadPointer
            | Step::DisplacementToSlot
            | Step::Alternative => step.name().to_string(),
        }
    }

    /// The report line: `recipe <i> step <j> <step>  cursor=<hex|->  matches=<n|->  <error>`.
    pub fn line(&self, recipe: usize) -> String {
        let cursor = self.cursor.map_or_else(|| "-".to_string(), |value| format!("{value:#x}"));
        let matches = self.matches.map_or_else(|| "-".to_string(), |count| count.to_string());
        let mut line = format!("recipe {recipe} step {} {}  cursor={cursor}  matches={matches}", self.index, self.step);
        if let Some(error) = &self.error {
            line.push_str("  ");
            line.push_str(error);
        }
        line
    }
}
