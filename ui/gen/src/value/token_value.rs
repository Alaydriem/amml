/// A parsed token value. An alias keeps the name of the token it points to.
#[derive(Debug, Clone, PartialEq)]
pub enum TokenValue {
    Color {
        r: u8,
        g: u8,
        b: u8,
        a: u8,
    },
    Px(f32),
    Em(f32),
    /// A percentage as a fraction: `45%` is `0.45`.
    Percent(f32),
    Number(f32),
    Family(String),
    Alias(String),
}
