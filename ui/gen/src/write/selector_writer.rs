use std::collections::BTreeMap;

use crate::error::GenError;
use crate::model::{Owner, SelectorSpec, TokenKind};
use crate::resolve::ResolvedToken;

use super::{OutputTree, TokenWriter};

/// Writes a selector enum: code that picks a colour token from data matches on its variants.
pub struct SelectorWriter;

impl SelectorWriter {
    /// The file stem of a selector: `SceneColor` gives `scene_color`.
    pub fn module_name(name: &str) -> String {
        let mut out = String::with_capacity(name.len() + 4);
        for (at, ch) in name.chars().enumerate() {
            if ch.is_ascii_uppercase() && at > 0 {
                out.push('_');
            }
            out.push(ch.to_ascii_lowercase());
        }
        out
    }

    /// The file content for `spec`. Each variant must name a colour token that the selector's crate
    /// can use: its own, or an amml-ui token.
    pub fn render(spec: &SelectorSpec, tokens: &BTreeMap<&str, &ResolvedToken>) -> Result<String, GenError> {
        let name = &spec.name;
        let mut arms = Vec::with_capacity(spec.variants.len());
        for (variant, token_name) in &spec.variants {
            let bad = || GenError::BadSelector { selector: name.clone(), token: token_name.clone() };
            let token = tokens.get(token_name.as_str()).ok_or_else(bad)?;
            let usable = token.owner == spec.crate_owner || token.owner == Owner::AmmlUi;
            if token.entry.kind != TokenKind::Color || !usable {
                return Err(bad());
            }
            let struct_name = TokenWriter::struct_name(token_name)?;
            let path = if token.owner == spec.crate_owner {
                format!("super::super::color::{struct_name}")
            } else {
                format!("amml_ui::tokens::generated::color::{struct_name}")
            };
            arms.push((variant, token_name, path));
        }

        let mut out = format!("{}\n", OutputTree::HEADER);
        out.push_str(&format!("use {}::ColorToken;\n\n", TokenWriter::trait_root(spec.crate_owner)));
        out.push_str("/// Picks one of these colour tokens at run time, from data.\n");
        out.push_str("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]\n");
        out.push_str(&format!("pub enum {name} {{\n"));
        for (variant, token_name, _) in &arms {
            out.push_str(&format!("    /// Paper: {token_name}.\n    {variant},\n"));
        }
        out.push_str("}\n\n");
        out.push_str(&format!("impl {name} {{\n"));
        out.push_str(&Self::all_line(&arms.iter().map(|(variant, _, _)| variant.as_str()).collect::<Vec<_>>()));
        out.push('\n');
        out.push_str("    pub fn color(self) -> egui::Color32 {\n        match self {\n");
        for (variant, _, path) in &arms {
            out.push_str(&format!("            Self::{variant} => <{path} as ColorToken>::VALUE,\n"));
        }
        out.push_str("        }\n    }\n}\n");
        Ok(out)
    }

    fn all_line(variants: &[&str]) -> String {
        let items: Vec<String> = variants.iter().map(|variant| format!("Self::{variant}")).collect();
        let line = format!("    pub const ALL: [Self; {}] = [{}];\n", items.len(), items.join(", "));
        if line.len() - 1 <= TokenWriter::MAX_WIDTH {
            return line;
        }
        // rustfmt next breaks after `=` and keeps the array on one line, indented 8.
        let array = format!("        [{}];", items.join(", "));
        if array.len() <= TokenWriter::MAX_WIDTH {
            return format!("    pub const ALL: [Self; {}] =\n{array}\n", items.len());
        }
        let mut out = format!("    pub const ALL: [Self; {}] = [\n", items.len());
        for item in &items {
            out.push_str(&format!("        {item},\n"));
        }
        out.push_str("    ];\n");
        out
    }
}
