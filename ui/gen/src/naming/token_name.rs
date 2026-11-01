use crate::model::TokenKind;

/// Turns a Paper token name into its family and Rust names.
pub struct TokenName;

impl TokenName {
    // `--font-weight-` comes before `--font-`, so a weight is never read as a family.
    const PREFIXES: [(&'static str, TokenKind); 11] = [
        ("--color-", TokenKind::Color),
        ("--text-", TokenKind::FontSize),
        ("--font-weight-", TokenKind::FontWeight),
        ("--font-", TokenKind::FontFamily),
        ("--tracking-", TokenKind::LetterSpacing),
        ("--spacing-", TokenKind::Spacing),
        ("--radius-", TokenKind::Radius),
        ("--opacity-", TokenKind::Opacity),
        ("--container-", TokenKind::Container),
        ("--breakpoint-", TokenKind::Breakpoint),
        ("--leading-", TokenKind::LineHeight),
    ];

    const KEYWORDS: [&'static str; 51] = [
        "abstract", "as", "async", "await", "become", "box", "break", "const", "continue", "crate", "do", "dyn",
        "else", "enum", "extern", "false", "final", "fn", "for", "gen", "if", "impl", "in", "let", "loop", "macro",
        "match", "mod", "move", "mut", "override", "priv", "pub", "ref", "return", "self", "static", "struct", "super",
        "trait", "true", "try", "type", "typeof", "unsafe", "unsized", "use", "virtual", "where", "while", "yield",
    ];

    /// The family and the rest of the name: `--color-accent-hover` gives `(Color, "accent-hover")`.
    /// `None` for an unknown prefix or an empty rest.
    pub fn split(css: &str) -> Option<(TokenKind, String)> {
        Self::PREFIXES.iter().find_map(|(prefix, kind)| {
            css.strip_prefix(prefix).filter(|rest| !rest.is_empty()).map(|rest| (*kind, rest.to_owned()))
        })
    }

    /// The struct name: `accent-hover` gives `AccentHover`, `scene-1` gives `Scene1`.
    pub fn pascal(rest: &str) -> String {
        rest.split('-')
            .map(|part| {
                let mut chars = part.chars();
                chars.next().map(|first| first.to_ascii_uppercase().to_string() + chars.as_str()).unwrap_or_default()
            })
            .collect()
    }

    /// The file and module name: `accent-hover` gives `accent_hover`.
    pub fn snake(rest: &str) -> String {
        rest.replace('-', "_")
    }

    /// True when `rest` makes a valid struct and module name: lowercase ASCII words of letters and
    /// digits joined by single `-`, starting with a letter, and not a Rust keyword.
    pub fn is_identifier(rest: &str) -> bool {
        let starts_with_letter = rest.bytes().next().is_some_and(|byte| byte.is_ascii_lowercase());
        let words_ok = rest.split('-').all(|word| {
            !word.is_empty() && word.bytes().all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
        });
        starts_with_letter && words_ok && !Self::KEYWORDS.contains(&Self::snake(rest).as_str())
    }

    /// The module of a family and the stem of its trait file: `FontSize` gives `text_size`.
    pub fn family(kind: TokenKind) -> &'static str {
        match kind {
            TokenKind::Color => "color",
            TokenKind::FontSize => "text_size",
            TokenKind::FontFamily => "font_family",
            TokenKind::FontWeight => "font_weight",
            TokenKind::LetterSpacing => "tracking",
            TokenKind::Spacing => "spacing",
            TokenKind::Radius => "radius",
            TokenKind::Opacity => "opacity",
            TokenKind::Container => "container",
            TokenKind::Breakpoint => "breakpoint",
            TokenKind::LineHeight => "line_height",
        }
    }

    /// The amml-ui trait that a family's structs implement.
    pub fn family_trait(kind: TokenKind) -> &'static str {
        match kind {
            TokenKind::Color => "ColorToken",
            TokenKind::FontSize => "TextSizeToken",
            TokenKind::FontFamily => "FontFamilyToken",
            TokenKind::FontWeight => "FontWeightToken",
            TokenKind::LetterSpacing => "TrackingToken",
            TokenKind::Spacing => "SpacingToken",
            TokenKind::Radius => "RadiusToken",
            TokenKind::Opacity => "OpacityToken",
            TokenKind::Container => "ContainerToken",
            TokenKind::Breakpoint => "BreakpointToken",
            TokenKind::LineHeight => "LineHeightToken",
        }
    }

    /// The Rust type of a family's `VALUE`.
    pub fn value_type(kind: TokenKind) -> &'static str {
        match kind {
            TokenKind::Color => "egui::Color32",
            TokenKind::FontFamily => "&'static str",
            TokenKind::FontWeight => "u16",
            _ => "f32",
        }
    }
}
