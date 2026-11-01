use std::collections::BTreeMap;

use crate::error::GenError;
use crate::model::{Owner, TokenKind};
use crate::naming::TokenName;
use crate::resolve::ResolvedToken;
use crate::value::TokenValue;

use super::OutputTree;

/// Writes the file of one token: one unit struct and its trait impl.
pub struct TokenWriter;

impl TokenWriter {
    /// rustfmt's `max_width` in the amml workspace.
    pub const MAX_WIDTH: usize = 120;

    /// The file content for `token`. `tokens` holds every resolved token by name, for aliases.
    pub fn render(token: &ResolvedToken, tokens: &BTreeMap<&str, &ResolvedToken>) -> Result<String, GenError> {
        let kind = token.entry.kind;
        let name = &token.entry.name;
        let struct_name = Self::struct_name(name)?;
        let family_trait = TokenName::family_trait(kind);
        let value = Self::value(token, tokens)?;

        let mut out = String::new();
        out.push_str(OutputTree::HEADER);
        out.push('\n');
        out.push_str(&format!("use {}::{family_trait};\n\n", Self::trait_root(token.owner)));
        out.push_str(&Self::doc(name, token.entry.description.as_deref(), ""));
        out.push_str(&format!("pub struct {struct_name};\n\n"));
        out.push_str(&format!("impl {family_trait} for {struct_name} {{\n"));
        out.push_str(&format!("    const NAME: &'static str = {name:?};\n"));
        out.push_str(&Self::const_line("VALUE", TokenName::value_type(kind), &value));
        out.push_str("}\n");
        Ok(out)
    }

    /// The struct name of a token, or `BadName`.
    pub fn struct_name(css: &str) -> Result<String, GenError> {
        TokenName::split(css)
            .map(|(_, rest)| TokenName::pascal(&rest))
            .ok_or_else(|| GenError::BadName { token: css.to_owned() })
    }

    /// The module (file stem) of a token, or `BadName`.
    pub fn module_name(css: &str) -> Result<String, GenError> {
        TokenName::split(css)
            .map(|(_, rest)| TokenName::snake(&rest))
            .ok_or_else(|| GenError::BadName { token: css.to_owned() })
    }

    /// Where the token traits live, seen from a crate.
    pub fn trait_root(owner: Owner) -> &'static str {
        match owner {
            Owner::AmmlUi => "crate::tokens",
            Owner::StudioUi => "amml_ui::tokens",
        }
    }

    /// `/// Paper: <name>. <description>` doc lines, each indented by `indent`.
    pub fn doc(name: &str, description: Option<&str>, indent: &str) -> String {
        let mut lines = description.map(str::trim).filter(|text| !text.is_empty()).map(|text| text.lines());
        let first = lines.as_mut().and_then(Iterator::next).map(str::trim_end);
        let mut out = match first {
            Some(text) => format!("{indent}/// Paper: {name}. {text}\n"),
            None => format!("{indent}/// Paper: {name}.\n"),
        };
        for line in lines.into_iter().flatten() {
            let line = line.trim_end();
            if line.is_empty() {
                out.push_str(&format!("{indent}///\n"));
            } else {
                out.push_str(&format!("{indent}/// {line}\n"));
            }
        }
        out
    }

    /// `    const <name>: <ty> = <expr>;`, broken after `=` the way rustfmt breaks it when too wide.
    fn const_line(name: &str, ty: &str, expr: &str) -> String {
        let line = format!("    const {name}: {ty} = {expr};\n");
        if line.len() - 1 <= Self::MAX_WIDTH { line } else { format!("    const {name}: {ty} =\n        {expr};\n") }
    }

    fn value(token: &ResolvedToken, tokens: &BTreeMap<&str, &ResolvedToken>) -> Result<String, GenError> {
        let kind = token.entry.kind;
        let bad = || GenError::BadValue { token: token.entry.name.clone(), raw: token.entry.value.clone() };
        Ok(match (&token.value, kind) {
            (TokenValue::Alias(target_name), _) => {
                let target = tokens.get(target_name.as_str()).ok_or_else(|| GenError::MissingAlias {
                    token: token.entry.name.clone(),
                    target: target_name.clone(),
                })?;
                let target_struct = Self::struct_name(target_name)?;
                let path = if target.owner == token.owner {
                    format!("super::{}::{target_struct}", Self::module_name(target_name)?)
                } else {
                    format!("amml_ui::tokens::generated::{}::{target_struct}", TokenName::family(kind))
                };
                format!("<{path} as {}>::VALUE", TokenName::family_trait(kind))
            }
            (TokenValue::Color { r, g, b, a }, TokenKind::Color) => {
                format!("egui::Color32::from_rgba_unmultiplied_const(0x{r:02X}, 0x{g:02X}, 0x{b:02X}, 0x{a:02X})")
            }
            (TokenValue::Family(family), TokenKind::FontFamily) => format!("{family:?}"),
            (TokenValue::Number(weight), TokenKind::FontWeight) => {
                // The parser accepts only whole weights from 1 to 1000, so the cast is exact.
                format!("{}", *weight as u16)
            }
            (TokenValue::Em(value), TokenKind::LetterSpacing) | (TokenValue::Percent(value), TokenKind::Opacity) => {
                format!("{value:?}")
            }
            (
                TokenValue::Px(value),
                TokenKind::FontSize
                | TokenKind::Spacing
                | TokenKind::Radius
                | TokenKind::Container
                | TokenKind::Breakpoint
                | TokenKind::LineHeight,
            ) => format!("{value:?}"),
            _ => return Err(bad()),
        })
    }
}
