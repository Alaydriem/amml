use serde::Deserialize;

/// The Paper token type. Each kind is one token family and one trait in amml-ui.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TokenKind {
    Color,
    FontSize,
    FontFamily,
    FontWeight,
    LetterSpacing,
    Spacing,
    Radius,
    Opacity,
    Container,
    Breakpoint,
    LineHeight,
}
