/// A pattern byte the author keeps fixed on purpose, from a `literal_ok` list. On `pattern` the entry reads
/// `"<byte index>: <reason>"`; on `pattern_any` it names the alternative: `"<alternative>.<byte index>: <reason>"`.
/// The deriver ignores it; `amml-derive lint` lists the byte as kept instead of as a finding.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LiteralOk {
    // the alternative of a `pattern_any`; `None` for `pattern`
    pub alternative: Option<usize>,
    // counts pattern tokens from 0
    pub index: usize,
    pub reason: String,
}

impl LiteralOk {
    /// Reads one entry; `any` is true for a `pattern_any`, where the alternative is required. The reason must not be empty.
    pub fn parse(text: &str, any: bool) -> Result<Self, String> {
        let form = if any { "<alternative>.<byte index>: <reason>" } else { "<byte index>: <reason>" };
        let bad = || format!("literal_ok entry `{text}` must look like `{form}`");
        let (place, reason) = text.split_once(':').ok_or_else(bad)?;
        let number = |part: &str| part.trim().parse::<usize>().map_err(|_| bad());
        let (alternative, index) = match (any, place.split_once('.')) {
            (true, Some((alternative, index))) => (Some(number(alternative)?), number(index)?),
            (false, None) => (None, number(place)?),
            _ => return Err(bad()),
        };
        let reason = reason.trim();
        if reason.is_empty() {
            return Err(bad());
        }
        Ok(Self { alternative, index, reason: reason.to_string() })
    }
}
