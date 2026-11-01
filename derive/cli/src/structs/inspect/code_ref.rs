use super::CodeRefKind;

/// One place that refers to an address: the instruction or data word at `site`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CodeRef {
    pub site: u32,
    /// The primary start of the function that holds `site`; `None` for a data pointer.
    pub function: Option<u32>,
    pub kind: CodeRefKind,
    pub text: String,
}

impl CodeRef {
    /// `0x<site>  <kind>  in 0x<function>  <text>`
    pub fn line(&self) -> String {
        let function = self.function.map_or_else(|| "none".to_string(), |start| format!("{start:#x}"));
        format!("{:#x}  {}  in {function}  {}", self.site, self.kind.label(), self.text)
    }
}
