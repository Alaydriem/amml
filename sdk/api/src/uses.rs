use std::fmt::Write;

use amml_abi::AmmlUseRow;

use crate::UseRow;

/// What a mod says it uses, by feature: the events and areas `amml_uses!` builds rows for. The loader
/// reads them from the info block; `Events::listen` reads them to know whether a listen is required.
#[derive(Clone, Copy, Debug)]
pub struct Uses {
    rows: &'static [UseRow],
}

// SAFETY: the rows are immutable static data and their string views point to static strings
// (`UseRow` can only be built from `&'static str`).
#[allow(unsafe_code)]
unsafe impl Send for Uses {}

// SAFETY: as for `Send`; shared reads need no synchronization.
#[allow(unsafe_code)]
unsafe impl Sync for Uses {}

impl Uses {
    /// A mod that names nothing: every listen is optional.
    pub const NONE: Self = Self { rows: &[] };

    pub const fn new(rows: &'static [UseRow]) -> Self {
        Self { rows }
    }

    /// One row for event `name` at `version` in `feature`. Used by `amml_uses!`.
    pub const fn event_row(feature: &'static str, name: &'static str, version: u32, required: bool) -> UseRow {
        UseRow::event(feature, name, version, required)
    }

    /// One row for area `name` at `version` in `feature`. Used by `amml_uses!`.
    pub const fn area_row(feature: &'static str, name: &'static str, version: u32, required: bool) -> UseRow {
        UseRow::area(feature, name, version, required)
    }

    /// The rows as the ABI lays them out.
    pub const fn rows(&self) -> &'static [AmmlUseRow] {
        UseRow::as_abi(self.rows)
    }

    /// The `uses` block of `amml.hcl` for these rows, for the compat check: one `feature` block per run
    /// of rows with the same feature, in row order, holding an `event "<id>"` or `area "<id>"` block per
    /// row with `version` and `required` on their own lines (the HCL reader does not accept commas in a
    /// one-line block). Two-space indent; the text ends in a newline. Quotes, backslashes, control
    /// characters and template starts (`${`, `%{`) in a name are escaped.
    pub fn to_hcl(&self) -> String {
        let mut text = String::from("uses {\n");
        let mut open: Option<&str> = None;
        for row in self.rows {
            if open != Some(row.feature()) {
                if open.is_some() {
                    text.push_str("  }\n");
                }
                let _ = writeln!(text, "  feature \"{}\" {{", Self::quoted(row.feature()));
                open = Some(row.feature());
            }
            let kind = if row.is_area() { "area" } else { "event" };
            let _ = writeln!(text, "    {kind} \"{}\" {{", Self::quoted(row.name()));
            let _ = writeln!(text, "      version  = {}", row.version());
            let _ = writeln!(text, "      required = {}", row.required());
            text.push_str("    }\n");
        }
        if open.is_some() {
            text.push_str("  }\n");
        }
        text.push_str("}\n");
        text
    }

    /// `manifest`, the text of an `amml.hcl`, with its `uses` block replaced by [`Self::to_hcl`]. A
    /// block is a line that is exactly `uses {` up to the next line that is exactly `}`. Further `uses`
    /// blocks are removed. A manifest without one gets the block before its `listing {` line, or at its
    /// end. A manifest with any CRLF line end is written with CRLF throughout, so a second run on the
    /// result gives the same text.
    pub fn write_into(&self, manifest: &str) -> String {
        let crlf = manifest.contains("\r\n");
        let text = manifest.replace("\r\n", "\n");
        let mut rest = String::with_capacity(text.len());
        let mut at: Option<usize> = None;
        let mut cursor = 0;
        while let Some(start) = Self::line_start(&text, "uses {\n", cursor) {
            rest.push_str(&text[cursor..start]);
            at.get_or_insert(rest.len());
            cursor = text[start..].find("\n}\n").map_or(text.len(), |end| start + end + 3);
        }
        rest.push_str(&text[cursor..]);
        let at = at.or_else(|| Self::line_start(&rest, "listing {", 0)).unwrap_or(rest.len());
        let (head, tail) = rest.split_at(at);
        let lead = if head.is_empty() || head.ends_with('\n') { "" } else { "\n" };
        let trail = if tail.is_empty() || tail.starts_with('\n') { "" } else { "\n" };
        let written = format!("{head}{lead}{}{trail}{tail}", self.to_hcl());
        if crlf { written.replace('\n', "\r\n") } else { written }
    }

    // The first offset at or after `from` where `line` starts at the start of a line of `text`.
    fn line_start(text: &str, line: &str, from: usize) -> Option<usize> {
        let mut from = from;
        while let Some(found) = text.get(from..).and_then(|rest| rest.find(line)) {
            let start = from + found;
            if start == 0 || text.as_bytes()[start - 1] == b'\n' {
                return Some(start);
            }
            from = start + line.len();
        }
        None
    }

    // `name` as the inside of an HCL string literal.
    fn quoted(name: &str) -> String {
        let mut out = String::with_capacity(name.len());
        let mut chars = name.chars().peekable();
        while let Some(c) = chars.next() {
            match c {
                '"' => out.push_str("\\\""),
                '\\' => out.push_str("\\\\"),
                '\n' => out.push_str("\\n"),
                '\r' => out.push_str("\\r"),
                '\t' => out.push_str("\\t"),
                '$' | '%' if chars.peek() == Some(&'{') => {
                    out.push(c);
                    out.push(c);
                }
                c if c.is_control() => {
                    let _ = write!(out, "\\u{:04X}", u32::from(c));
                }
                c => out.push(c),
            }
        }
        out
    }

    /// Whether any feature names `event` as required.
    pub fn requires(&self, event: &str) -> bool {
        self.rows().iter().any(|row| {
            // SAFETY: every row comes from `UseRow::event`, whose views point to static strings.
            #[allow(unsafe_code)]
            let name = unsafe { row.name.as_str() };
            row.kind == AmmlUseRow::KIND_EVENT && row.required != 0 && name == Ok(event)
        })
    }
}
