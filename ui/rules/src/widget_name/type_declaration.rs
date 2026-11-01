/// A `struct` or `enum` declaration read from one source line.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeDeclaration {
    pub name: String,
    /// Declared `pub`, with no `(crate)` or `(super)` limit.
    pub public: bool,
}

impl TypeDeclaration {
    /// The declaration on `line`, or None for any other line (a `use` line, a comment, a field).
    pub fn read(line: &str) -> Option<Self> {
        let item = line.trim_start();
        if item.starts_with("//") {
            return None;
        }
        let (public, item) = if let Some(rest) = item.strip_prefix("pub ") {
            (true, rest)
        } else if let Some(rest) = item.strip_prefix("pub(crate) ").or_else(|| item.strip_prefix("pub(super) ")) {
            (false, rest)
        } else {
            (false, item)
        };
        let rest = item.strip_prefix("struct ").or_else(|| item.strip_prefix("enum "))?;
        let name: String =
            rest.chars().take_while(|character| character.is_ascii_alphanumeric() || *character == '_').collect();
        if name.is_empty() { None } else { Some(Self { name, public }) }
    }
}
