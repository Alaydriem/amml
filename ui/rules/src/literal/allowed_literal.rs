use super::LiteralHit;

/// One design literal that a crate keeps on purpose, with the reason. It names the line by its text, not its number,
/// so an edit above it does not move it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AllowedLiteral {
    /// The end of the file path, with `/` between folders (for example "views/bar_controls.rs").
    pub file_suffix: &'static str,
    /// A part of the line text.
    pub text: &'static str,
    pub reason: &'static str,
}

impl AllowedLiteral {
    pub fn covers(&self, hit: &LiteralHit) -> bool {
        let file = hit.file.to_string_lossy().replace('\\', "/");
        file.ends_with(self.file_suffix) && hit.text.contains(self.text)
    }
}
