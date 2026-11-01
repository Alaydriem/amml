use crate::error::GenError;
use crate::model::{TokenEntry, TokenKind};

use super::{ColorValue, TokenValue};

/// Parses a raw Paper value into the one form its token type accepts.
pub struct ValueParser;

impl ValueParser {
    /// The value of a token of type `kind`. Any token may be an alias, `var(--name)`.
    ///
    /// The error names no token; use [`ValueParser::parse_entry`] to name it.
    pub fn parse(kind: TokenKind, raw: &str) -> Result<TokenValue, GenError> {
        let raw = raw.trim();
        let bad = || GenError::BadValue { token: String::new(), raw: raw.to_owned() };
        if let Some(target) = raw.strip_prefix("var(").and_then(|rest| rest.strip_suffix(')')) {
            let target = target.trim();
            return match target.strip_prefix("--") {
                Some(rest) if !rest.is_empty() && !target.contains(char::is_whitespace) => {
                    Ok(TokenValue::Alias(target.to_owned()))
                }
                _ => Err(bad()),
            };
        }
        let value = match kind {
            TokenKind::Color => ColorValue::parse(raw).map(TokenValue::from),
            TokenKind::FontFamily => Self::family(raw),
            TokenKind::FontWeight => Self::number(raw)
                .filter(|weight| weight.fract() == 0.0 && (1.0..=1000.0).contains(weight))
                .map(TokenValue::Number),
            TokenKind::LetterSpacing => Self::number(raw.strip_suffix("em").unwrap_or("")).map(TokenValue::Em),
            TokenKind::Opacity => {
                Self::number(raw.strip_suffix('%').unwrap_or("")).map(|percent| TokenValue::Percent(percent / 100.0))
            }
            TokenKind::FontSize
            | TokenKind::Spacing
            | TokenKind::Radius
            | TokenKind::Container
            | TokenKind::Breakpoint
            | TokenKind::LineHeight => Self::number(raw.strip_suffix("px").unwrap_or("")).map(TokenValue::Px),
        };
        value.ok_or_else(bad)
    }

    /// The value of `entry`. An error names the token.
    pub fn parse_entry(entry: &TokenEntry) -> Result<TokenValue, GenError> {
        Self::parse(entry.kind, &entry.value).map_err(|error| match error {
            GenError::BadValue { raw, .. } => GenError::BadValue { token: entry.name.clone(), raw },
            other => other,
        })
    }

    fn number(text: &str) -> Option<f32> {
        // Parsed as f64 and then rounded once, so `0.45` and `45 / 100` give the same f32.
        let parsed: f64 = text.trim().parse().ok()?;
        parsed.is_finite().then_some(parsed as f32)
    }

    fn family(raw: &str) -> Option<TokenValue> {
        let bare = raw.trim_matches(|c| c == '"' || c == '\'');
        let plain = !bare.is_empty() && !bare.contains(['(', ')', ';', ',', '"', '\\']);
        plain.then(|| TokenValue::Family(bare.to_owned()))
    }
}
