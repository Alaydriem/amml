use crate::errors::DataError;

/// The token grammar of `pattern` text: two hex digits per byte, `?` or `??` for a wildcard.
pub struct PatternText;

impl PatternText {
    /// One entry per token, `None` for a wildcard. Fails on a bad token and on a pattern without a fixed byte.
    pub fn parse(text: &str) -> Result<Vec<Option<u8>>, DataError> {
        let mut bytes = Vec::new();
        for token in text.split_whitespace() {
            let bad = || DataError::BadPatternToken { token: token.to_string() };
            if token == "?" || token == "??" {
                bytes.push(None);
            } else if token.len() == 2 && token.bytes().all(|b| b.is_ascii_hexdigit()) {
                bytes.push(Some(u8::from_str_radix(token, 16).map_err(|_| bad())?));
            } else {
                return Err(bad());
            }
        }
        if bytes.iter().all(Option::is_none) {
            return Err(DataError::BadPatternToken { token: text.trim().to_string() });
        }
        Ok(bytes)
    }
}
