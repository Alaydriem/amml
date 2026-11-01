use super::TokenValue;

/// A colour written as `#RRGGBB`, `#RRGGBBAA` or `rgb(R G B / P%)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColorValue {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl ColorValue {
    /// The colour, or `None` for any other syntax (`color-mix()`, names, `hsl()`).
    pub fn parse(raw: &str) -> Option<Self> {
        match raw.strip_prefix('#') {
            Some(hex) => Self::hex(hex),
            None => Self::rgb(raw.strip_prefix("rgb(")?.strip_suffix(')')?),
        }
    }

    fn hex(hex: &str) -> Option<Self> {
        if !(hex.len() == 6 || hex.len() == 8) || !hex.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return None;
        }
        let channel = |at: usize| u8::from_str_radix(&hex[at..at + 2], 16).ok();
        let a = if hex.len() == 8 { channel(6)? } else { u8::MAX };
        Some(Self { r: channel(0)?, g: channel(2)?, b: channel(4)?, a })
    }

    fn rgb(inner: &str) -> Option<Self> {
        let (channels, alpha) = match inner.split_once('/') {
            Some((channels, alpha)) => (channels, Some(alpha.trim())),
            None => (inner, None),
        };
        let mut parts = channels.split_whitespace().map(str::parse::<u8>);
        let (r, g, b) = (parts.next()?.ok()?, parts.next()?.ok()?, parts.next()?.ok()?);
        if parts.next().is_some() {
            return None;
        }
        let a = match alpha {
            None => u8::MAX,
            Some(alpha) => {
                let percent: f64 = alpha.strip_suffix('%')?.trim().parse().ok()?;
                if !(0.0..=100.0).contains(&percent) {
                    return None;
                }
                // In range 0..=255 after the check above, so the cast cannot truncate.
                (percent / 100.0 * 255.0).round() as u8
            }
        };
        Some(Self { r, g, b, a })
    }
}

impl From<ColorValue> for TokenValue {
    fn from(color: ColorValue) -> Self {
        TokenValue::Color { r: color.r, g: color.g, b: color.b, a: color.a }
    }
}
