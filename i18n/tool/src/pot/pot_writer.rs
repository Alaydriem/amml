use std::fmt::Write as _;
use std::path::Path;

use amml_i18n::message::Message;

use crate::error::ToolError;

/// Writes a .pot template from a message enum.
///
/// The output depends only on the enum: entries follow `M::ALL` and there is no timestamp,
/// so running it twice gives the same file.
pub struct PotWriter;

impl PotWriter {
    // English plural rules; translators replace this header in their .po file.
    const PLURAL_FORMS: &'static str = "nplurals=2; plural=(n != 1);";

    pub fn render<M: Message>() -> String {
        let domain = M::DOMAIN.name();
        let mut out = String::new();
        let _ = writeln!(out, "# Messages of the {domain} domain, written by amml-i18n-tool. Do not edit.");
        out.push_str("msgid \"\"\nmsgstr \"\"\n");
        out.push_str("\"Content-Type: text/plain; charset=UTF-8\\n\"\n");
        out.push_str("\"Content-Transfer-Encoding: 8bit\\n\"\n");
        let _ = writeln!(out, "\"Plural-Forms: {}\\n\"", Self::PLURAL_FORMS);
        let _ = writeln!(out, "\"X-Domain: {domain}\\n\"");

        for message in M::ALL {
            out.push('\n');
            if let Some(note) = message.note() {
                for line in note.lines() {
                    let _ = writeln!(out, "#. {line}");
                }
            }
            let has_placeholder = message.msgid().contains('{') || message.plural().is_some_and(|p| p.contains('{'));
            if has_placeholder {
                out.push_str("#, python-brace-format\n");
            }
            if let Some(context) = message.context() {
                let _ = writeln!(out, "msgctxt \"{}\"", Self::escape(context));
            }
            let _ = writeln!(out, "msgid \"{}\"", Self::escape(message.msgid()));
            match message.plural() {
                Some(plural) => {
                    let _ = writeln!(out, "msgid_plural \"{}\"", Self::escape(plural));
                    out.push_str("msgstr[0] \"\"\nmsgstr[1] \"\"\n");
                }
                None => out.push_str("msgstr \"\"\n"),
            }
        }
        out
    }

    /// Writes the .pot to `path` when it changed. Returns whether it changed. With `check`,
    /// nothing is written: the result says whether the file on disk is stale.
    pub fn write<M: Message>(path: &Path, check: bool) -> Result<bool, ToolError> {
        let fresh = Self::render::<M>();
        let changed = std::fs::read_to_string(path).map_or(true, |current| !Self::same_text(&current, &fresh));
        if changed && !check {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|source| ToolError::Io { path: parent.to_path_buf(), source })?;
            }
            std::fs::write(path, fresh).map_err(|source| ToolError::Io { path: path.to_path_buf(), source })?;
        }
        Ok(changed)
    }

    /// Compares two catalogs as text. Git on Windows can check a .pot out with CRLF line
    /// ends, which is not a change.
    pub fn same_text(left: &str, right: &str) -> bool {
        left.replace("\r\n", "\n") == right.replace("\r\n", "\n")
    }

    pub fn escape(text: &str) -> String {
        let mut out = String::with_capacity(text.len());
        for letter in text.chars() {
            match letter {
                '\\' => out.push_str("\\\\"),
                '"' => out.push_str("\\\""),
                '\n' => out.push_str("\\n"),
                '\t' => out.push_str("\\t"),
                other => out.push(other),
            }
        }
        out
    }
}
